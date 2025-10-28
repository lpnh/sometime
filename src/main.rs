mod canvas;
use canvas::Canvas;
mod registry;
mod sometime;
use sometime::Sometime;
mod theme;
use theme::Theme;
mod widget;
use widget::Widget;

use chrono::Local;
use smithay_client_toolkit::{
    compositor::CompositorState,
    reexports::{
        calloop::{
            EventLoop,
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
    },
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{Connection, globals::registry_queue_init};

const SIDE: i32 = 448;

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let surface = compositor.create_surface(&qh);
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("sometime"), None);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(SIDE as u32, SIDE as u32);
    layer.commit();

    let pool = SlotPool::new((SIDE * SIDE * 4) as usize, &shm).expect("Failed to create pool");

    let mut sometime = Sometime::new(
        Theme::default(),
        Widget::new(&globals, &qh, shm, pool, layer),
        Canvas::new(SIDE),
    );

    let mut event_loop: EventLoop<Sometime> =
        EventLoop::try_new().expect("Failed to initialize event loop");
    let loop_handle = event_loop.handle();

    WaylandSource::new(conn, event_queue)
        .insert(loop_handle.clone())
        .unwrap();

    let timer = Timer::from_duration(calc_next_tick());
    loop_handle
        .insert_source(timer, move |_deadline, _timer_handle, data| {
            data.is_happening = true;
            TimeoutAction::ToDuration(calc_next_tick())
        })
        .expect("Failed to insert timer");

    loop {
        event_loop.dispatch(None, &mut sometime).unwrap();

        if sometime.is_happening {
            sometime.is_happening = false;
            sometime.draw();
        }

        if sometime.widget.exit {
            println!("Exiting sometime");
            break;
        }
    }
}

fn calc_next_tick() -> std::time::Duration {
    let now = Local::now();
    let ms_in_current_sec = now.timestamp_subsec_millis();
    std::time::Duration::from_millis((1000 - ms_in_current_sec) as u64)
}
