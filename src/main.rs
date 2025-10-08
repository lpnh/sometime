mod canvas;
use canvas::Canvas;
mod registry;
mod sometime;
use sometime::Sometime;
mod theme;
use theme::Theme;
mod widget;
use widget::Widget;

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{Connection, globals::registry_queue_init};

const SIDE: i32 = 512;

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("sometime"), None);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(SIDE as u32, SIDE as u32);
    layer.commit();

    let pool = SlotPool::new((SIDE * SIDE * 4) as usize, &shm).expect("Failed to create pool");

    let widget = Widget {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        pool,
        side: SIDE,
        layer,
        keyboard: None,
        pointer: None,
    };
    let canvas = Canvas::new(SIDE, Theme::default());
    let mut sometime = Sometime::new(widget, canvas);

    loop {
        event_queue.blocking_dispatch(&mut sometime).unwrap();

        if sometime.widget.exit {
            println!("Exiting sometime");
            break;
        }
    }
}
