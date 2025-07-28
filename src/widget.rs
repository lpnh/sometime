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
    protocol::{wl_keyboard, wl_shm},
};

use super::{canvas::Canvas, theme::Theme};

pub struct Widget {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub exit: bool,
    pub first_configure: bool,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub keyboard_focus: bool,
    pub visible: bool,
}

impl Widget {
    pub fn draw_clock_with_theme<T: Theme>(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, surface) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // Get current time
        let now = chrono::Local::now();
        let radius = (width.min(height) as f32 / 2.0) - 10.0;

        // Create our canvas helper - no borrowing of self here!
        let mut canvas = Canvas::new(width, height);

        // Clock face
        canvas.draw_circle(radius, T::FRAME);
        canvas.draw_circle(radius - 2.0, T::FACE);

        // Center dot
        canvas.draw_circle(4.0, T::HANDS);

        // Hands
        canvas.draw_hour_hand(now.hour(), now.minute(), radius, T::HANDS);
        canvas.draw_minute_hand(now.minute(), radius, T::HANDS);
        canvas.draw_second_hand(now.second(), radius, T::HANDS);

        // Copy back to the surface
        surface.copy_from_slice(canvas.get_data());

        // Damage and present
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}
