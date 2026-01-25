use chrono::Local;
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
use std::{env, time::Duration};
use wayland_client::{Connection, globals};

use sometime::{Canvas, SIDE, Sometime, State, flock, ipc, widget::Widget};

fn main() -> anyhow::Result<()> {
    let _lock = flock::try_acquire_daemon_lock()?;

    let exit_on_release = env::args()
        .nth(1)
        .is_some_and(|arg| arg == "--exit-on-release");

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = globals::registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let shm = Shm::bind(&globals, &qh)?;
    let pool = SlotPool::new((SIDE * SIDE * 4) as usize, &shm)?;
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;

    let widget = Widget::new(&globals, &qh, shm, pool, compositor, layer_shell);

    let mut app = Sometime::new(widget, Canvas::new(SIDE), exit_on_release);

    let mut event_loop = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();

    WaylandSource::new(conn, event_queue).insert(loop_handle.clone())?;

    let timer = Timer::from_duration(next_tick());
    loop_handle
        .insert_source(timer, |_deadline, _timer_handle, app| {
            if let State::Awake(view) = app.state {
                app.request_redraw(view);
            }
            TimeoutAction::ToDuration(next_tick())
        })
        .ok();

    let ipc_listener = ipc::setup_ipc_listener()?;
    let event_source = Generic::new(ipc_listener, Interest::READ, Mode::Level);
    loop_handle.insert_source(event_source, move |readiness, listener, app| {
        if readiness.readable
            && let Ok((stream, _)) = listener.accept()
            && let Some(new_view) = ipc::handle_client(stream)
        {
            // handle `sometime <clock|calendar>` command
            match app.state {
                State::Sleep => app.init(new_view, &qh),
                State::Awake(view) if view == new_view => app.sleep(), // toggle
                _ => {} // ensure sleep -> init -> awake -> sleep lifecycle
            }
        }
        Ok(PostAction::Continue)
    })?;

    loop {
        event_loop.dispatch(None, &mut app)?;

        app.consume_redraw();

        if app.widget.exit {
            break;
        }
    }

    ipc::cleanup_socket();

    Ok(())
}

fn next_tick() -> Duration {
    let ms_since_last_sec = Local::now().timestamp_subsec_millis();
    Duration::from_millis((1000 - ms_since_last_sec) as u64)
}
