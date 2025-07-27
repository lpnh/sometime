mod registry;
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

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);

    // Create a layer surface in the center
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay, // Above windows
        Some("niri_clock"),
        None,
    );

    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(400, 400);

    layer.commit();

    let pool = SlotPool::new(200 * 200 * 4, &shm).expect("Failed to create pool");

    let mut clock_widget = Widget {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: 200,
        height: 200,
        layer,
        keyboard: None,
        keyboard_focus: false,
        visible: true,
    };

    // TODO: Improve this
    // Setup timer for clock updates
    std::thread::spawn({
        let _conn = conn.clone();
        move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    loop {
        event_queue.blocking_dispatch(&mut clock_widget).unwrap();

        if clock_widget.exit {
            println!("Exiting clock widget");
            break;
        }
    }
}
