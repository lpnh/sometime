use chrono::{Local, Timelike};
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
    pub last_second: u32,
    pub face_cache: Vec<u8>,
}

impl Widget {
    pub fn init<T: Theme>(&mut self) {
        let mut canvas: Canvas<T> = Canvas::new(self.side);
        canvas.draw_face();
        self.face_cache = canvas.get_data().to_vec();
    }

    pub fn draw<T: Theme>(&mut self, qh: &QueueHandle<Self>) {
        let now = Local::now();
        let sec = now.second();

        if sec != self.last_second {
            self.last_second = sec;
            self.draw_clock_with_theme::<T>(qh, now.hour(), now.minute(), sec);
        } else {
            self.layer
                .wl_surface()
                .frame(qh, self.layer.wl_surface().clone());
            self.layer.commit();
        }
    }

    fn draw_clock_with_theme<T: Theme>(
        &mut self,
        qh: &QueueHandle<Self>,
        hour: u32,
        minute: u32,
        second: u32,
    ) {
        let side = self.side;
        let stride = side * 4;

        let (buffer, surface) = self
            .pool
            .create_buffer(side, side, stride, wl_shm::Format::Argb8888)
            .expect("create buffer");

        let mut canvas: Canvas<T> = Canvas::new(side);

        canvas.copy_from_raw(&self.face_cache);

        canvas.draw_hour_hand(hour, minute);
        canvas.draw_minute_hand(minute);
        canvas.draw_second_hand(second);

        surface.copy_from_slice(canvas.get_data());

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
