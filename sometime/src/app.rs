use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, SurfaceData},
    globals::GlobalData,
    output::OutputData,
    reexports::{
        calloop::{
            EventLoop,
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
        protocols::xdg::xdg_output::zv1::client::{
            zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1::ZxdgOutputV1,
        },
        protocols_wlr::layer_shell::v1::client::{
            zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        },
    },
    seat::SeatData,
    shell::{
        WaylandSurface,
        wlr_layer::{
            KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurfaceData,
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, Dispatch,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{
        wl_compositor::WlCompositor, wl_output::WlOutput, wl_registry::WlRegistry, wl_seat::WlSeat,
        wl_shm::WlShm, wl_surface::WlSurface,
    },
};

use crate::{SIDE, Theme, Widget, calc_next_tick};

pub trait App: Sized {
    type Canvas;
    fn draw(&mut self);
    fn is_happening_mut(&mut self) -> &mut bool;
    fn new(theme: Theme, widget: Widget, canvas: Self::Canvas) -> Self;
    fn new_canvas(side: i32) -> Self::Canvas;
    fn widget_mut(&mut self) -> &mut Widget;
}

pub fn run_app<T>()
where
    T: App + 'static,
    T: CompositorHandler,
    T: Dispatch<WlCompositor, GlobalData>,
    T: Dispatch<WlOutput, OutputData>,
    T: Dispatch<WlRegistry, GlobalListContents>,
    T: Dispatch<WlSeat, SeatData>,
    T: Dispatch<WlShm, GlobalData>,
    T: Dispatch<WlSurface, SurfaceData>,
    T: Dispatch<ZwlrLayerShellV1, GlobalData>,
    T: Dispatch<ZwlrLayerSurfaceV1, LayerSurfaceData>,
    T: Dispatch<ZxdgOutputManagerV1, GlobalData>,
    T: Dispatch<ZxdgOutputV1, OutputData>,
    T: LayerShellHandler,
    T: ShmHandler,
{
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

    let exit_on_release = std::env::args().any(|arg| arg == "--exit-on-release");

    let mut app = T::new(
        Theme::default(),
        Widget::new(&globals, &qh, shm, pool, layer, exit_on_release),
        T::new_canvas(SIDE),
    );

    let mut event_loop: EventLoop<T> =
        EventLoop::try_new().expect("Failed to initialize event loop");
    let loop_handle = event_loop.handle();

    WaylandSource::new(conn, event_queue)
        .insert(loop_handle.clone())
        .unwrap();

    let timer = Timer::from_duration(calc_next_tick());
    loop_handle
        .insert_source(timer, move |_deadline, _timer_handle, data| {
            *data.is_happening_mut() = true;
            TimeoutAction::ToDuration(calc_next_tick())
        })
        .expect("Failed to insert timer");

    loop {
        event_loop.dispatch(None, &mut app).unwrap();

        if *app.is_happening_mut() {
            *app.is_happening_mut() = false;
            app.draw();
        }

        if app.widget_mut().exit {
            println!("Exiting sometime");
            break;
        }
    }
}
