use chrono::Local;
use smithay_client_toolkit::reexports::{
    calloop::{
        EventLoop, Interest, Mode, PostAction, generic::Generic, timer::TimeoutAction, timer::Timer,
    },
    calloop_wayland_source::WaylandSource,
};
use std::time::{Duration, Instant};
use wayland_client::{Connection, globals};

use sometime::{Command, Sometime, State, Wayland, flock, ipc};

fn main() -> anyhow::Result<()> {
    let _lock = flock::try_acquire_daemon_lock()?;

    let exit_on_release = std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == "--exit-on-release");

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = globals::registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let wl = Wayland::new(&globals, &qh)?;
    let mut app = Sometime::new(wl, exit_on_release);

    let mut event_loop = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();

    WaylandSource::new(conn, event_queue).insert(loop_handle.clone())?;

    let timer = Timer::from_duration(next_tick());
    loop_handle.insert_source(timer, is_happening).ok();

    let ipc_listener = ipc::setup_listener()?;
    let event_source = Generic::new(ipc_listener, Interest::READ, Mode::Level);
    loop_handle.insert_source(event_source, move |readiness, listener, app| {
        if readiness.readable
            && let Ok((stream, _)) = listener.accept()
            && let Ok(cmd) = ipc::recv_cmd(stream)
        {
            match cmd {
                Command::Dismiss => app.wl.exit = true,
                Command::Clock | Command::Calendar => match app.state {
                    State::Sleep => app.init(cmd.into(), &qh),
                    State::Awake(view) if cmd == view => app.sleep(), // toggle
                    _ => {} // ensure sleep -> init -> awake -> sleep lifecycle
                },
            }
        }
        Ok(PostAction::Continue)
    })?;

    loop {
        event_loop.dispatch(None, &mut app)?;

        app.consume_redraw();

        if app.is_happening {
            let timer = Timer::from_duration(next_tick());
            loop_handle.insert_source(timer, is_happening).ok();
            app.is_happening = false;
        }

        if app.wl.exit {
            break;
        }
    }

    ipc::unlink_socket()?;

    Ok(())
}

fn is_happening(_: Instant, _: &mut (), app: &mut Sometime) -> TimeoutAction {
    if let State::Awake(view) = app.state {
        app.request_redraw(view);
        TimeoutAction::ToDuration(next_tick())
    } else {
        TimeoutAction::Drop
    }
}

fn next_tick() -> Duration {
    let ms_since_last_sec = Local::now().timestamp_subsec_millis();
    Duration::from_millis((1000 - ms_since_last_sec) as u64)
}
