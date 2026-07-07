use chrono::{Datelike, Local, Timelike};
use smithay_client_toolkit::reexports::{
    calloop::{
        EventLoop, Interest, Mode, PostAction, generic::Generic, timer::TimeoutAction, timer::Timer,
    },
    calloop_wayland_source::WaylandSource,
};
use std::time::Duration;
use wayland_client::{Connection, globals};

use sometime::{Sometime, State, View, Wayland, flock, ipc};

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

    let ipc_listener = ipc::setup_listener()?;
    let event_source = Generic::new(ipc_listener, Interest::READ, Mode::Level);
    loop_handle.insert_source(event_source, move |readiness, listener, app| {
        if readiness.readable
            && let Ok((stream, _)) = listener.accept()
            && let Ok(cmd) = ipc::recv_cmd(stream)
        {
            app.handle(cmd.into(), &qh);
        }
        Ok(PostAction::Continue)
    })?;

    loop {
        event_loop.dispatch(None, &mut app)?;

        if matches!(app.state, State::Awake(_)) && !app.is_happening {
            let timer = Timer::from_duration(next_tick());

            loop_handle
                .insert_source(timer, |_, _, app| {
                    let State::Awake(view) = app.state else {
                        app.is_happening = false;
                        return TimeoutAction::Drop;
                    };

                    let now = Local::now();

                    if match view {
                        View::Clock => app.last_second != now.second(),
                        View::Calendar => app.last_day != now.day(),
                    } {
                        app.draw();
                    }

                    TimeoutAction::ToDuration(next_tick())
                })
                .map_err(|e| e.error)?;

            app.is_happening = true;
        }

        if app.wl.exit {
            break;
        }
    }

    ipc::unlink_socket()?;

    Ok(())
}

fn next_tick() -> Duration {
    let ms_since_last_sec = Local::now().timestamp_subsec_millis();
    Duration::from_millis((1000 - ms_since_last_sec) as u64)
}
