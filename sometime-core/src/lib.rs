pub mod app;
pub mod canvas_primitives;
pub mod registry;
pub mod theme;
pub mod widget;

use chrono::Local;
use smithay_client_toolkit::{
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::slot::SlotPool,
};
use std::time::Duration;
use wayland_client::protocol::wl_shm::Format::Argb8888;

pub use app::{App, run_app};
pub use canvas_primitives::CanvasPrimitives;
pub use theme::{Bgra, Theme};
pub use widget::Widget;

pub const SIDE: i32 = 448;

pub fn calc_next_tick() -> Duration {
    let now = Local::now();
    let ms_in_current_sec = now.timestamp_subsec_millis();
    Duration::from_millis((1000 - ms_in_current_sec) as u64)
}

pub fn update_surface(layer: &LayerSurface, pool: &mut SlotPool, primitives: &CanvasPrimitives) {
    let data = primitives.get_data();
    let side = primitives.side;
    let stride = side * 4;

    let (buffer, surface) = pool
        .create_buffer(side, side, stride, Argb8888)
        .expect("create buffer");

    surface.copy_from_slice(data);

    let wl_surface = layer.wl_surface();
    wl_surface.damage_buffer(0, 0, side, side);
    buffer.attach_to(wl_surface).expect("buffer attach");
    layer.commit();
}
