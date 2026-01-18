use smithay_client_toolkit::{
    compositor::CompositorState,
    reexports::{
        calloop::{
            EventLoop, Interest, Mode, PostAction, generic::Generic, timer::TimeoutAction,
            timer::Timer,
        },
        calloop_wayland_source::WaylandSource,
    },
    shell::wlr_layer::LayerShell,
    shm::{Shm, slot::SlotPool},
};
use std::{
    env,
    fs::{self, File, OpenOptions},
    os::unix::io::AsRawFd,
    path::PathBuf,
    process,
    sync::mpsc,
    time::Duration,
};
use wayland_client::{Connection, globals};

use sometime::{Canvas, SIDE, Sometime, View, ipc::IpcServer, widget::Widget};

fn main() {
    let Some(_lock) = try_singleton_lock() else {
        process::exit(1);
    };

    let exit_on_release = env::args()
        .next_back()
        .is_some_and(|arg| arg == "--exit-on-release");

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, event_queue) = globals::registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");
    let pool = SlotPool::new((SIDE * SIDE * 4) as usize, &shm).expect("Failed to create pool");
    let compositor = CompositorState::bind(&globals, &qh).expect("WlCompositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");

    let widget = Widget::new(&globals, &qh, shm, pool, compositor, layer_shell);

    let mut app = Sometime::new(widget, Canvas::new(SIDE), exit_on_release);
    app.widget.create_layer_surface(&qh, "sometime");

    let mut event_loop: EventLoop<Sometime> =
        EventLoop::try_new().expect("Failed to initialize event loop");

    let loop_handle = event_loop.handle();

    WaylandSource::new(conn, event_queue)
        .insert(loop_handle.clone())
        .unwrap();

    let timer = Timer::from_duration(calc_next_tick());
    loop_handle
        .insert_source(timer, |_deadline, _timer_handle, data| {
            data.is_happening = true;
            TimeoutAction::ToDuration(calc_next_tick())
        })
        .expect("Failed to insert timer");

    let ipc_server = IpcServer::new().expect("Failed to create IPC server");
    let (sender, receiver) = mpsc::channel();

    let event_source = Generic::new(
        ipc_server.listener().try_clone().unwrap(),
        Interest::READ,
        Mode::Level,
    );

    loop_handle
        .insert_source(event_source, move |readiness, listener, _app| {
            if readiness.readable
                && let Ok((stream, _)) = listener.accept()
                && let Some(command) = IpcServer::handle_client(stream)
            {
                sender.send(command).expect("Failed to send command");
            }
            Ok(PostAction::Continue)
        })
        .expect("Failed to insert socket source");

    loop {
        event_loop
            .dispatch(Some(Duration::from_millis(100)), &mut app)
            .unwrap();

        while let Ok(view) = receiver.try_recv() {
            app.view = if app.view == view { View::Hidden } else { view };
            app.is_happening = true;
        }

        if app.is_happening {
            app.is_happening = false;
            app.draw(&qh);
        }

        if app.widget.exit {
            break;
        }
    }
}

struct SingletonLock {
    _file: File, // holds the flock :3
    path: PathBuf,
}

impl Drop for SingletonLock {
    fn drop(&mut self) {
        fs::remove_file(&self.path).expect("Failed to remove lock file");
    }
}

fn try_singleton_lock() -> Option<SingletonLock> {
    let xdg_runtime = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    let path = PathBuf::from(xdg_runtime).join("sometime.lock");

    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .ok()?;

    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

    if ret != 0 {
        eprintln!("sometime-daemon is already running");
        return None;
    }

    Some(SingletonLock { _file: file, path })
}

fn calc_next_tick() -> Duration {
    use chrono::Local;
    let now = Local::now();
    let ms_in_current_sec = now.timestamp_subsec_millis();
    Duration::from_millis((1000 - ms_in_current_sec) as u64)
}
