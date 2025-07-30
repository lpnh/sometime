use chrono::Timelike;
use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{
    QueueHandle,
    protocol::{wl_keyboard, wl_pointer, wl_shm},
};

use super::{canvas::Canvas, theme::Theme};

pub struct Widget {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub exit: bool,
    pub pool: SlotPool,
    pub side: i32,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub keyboard_focus: bool,
    pub pointer: Option<wl_pointer::WlPointer>,
}

impl Widget {
    pub fn draw_clock_with_theme<T: Theme>(&mut self, qh: &QueueHandle<Self>) {
        let side = self.side;
        let stride = side * 4;

        let (buffer, surface) = self
            .pool
            .create_buffer(side, side, stride, wl_shm::Format::Argb8888)
            .expect("create buffer");

        // Get current time
        let now = chrono::Local::now();

        let mut canvas: Canvas<T> = Canvas::new(side);

        // Clock face with center dot
        canvas.draw_face();

        // Hands
        canvas.draw_hour_hand(now.hour(), now.minute());
        canvas.draw_minute_hand(now.minute());
        canvas.draw_second_hand(now.second());

        // Copy back to the surface
        surface.copy_from_slice(canvas.get_data());

        // Damage and present
        self.layer.wl_surface().damage_buffer(0, 0, side, side);
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}
