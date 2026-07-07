mod canvas;
mod command;
pub mod flock;
pub mod ipc;
mod lifecycle;
mod registry;
mod theme;
mod wayland;

pub use canvas::Canvas;
pub use command::Command;
pub use lifecycle::{Action, Event, State, View};
pub use theme::{Bgra, Theme};
pub use wayland::Wayland;

use chrono::{Datelike, Local, Timelike};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{QueueHandle, protocol::wl_shm::Format};

pub const SIDE: i32 = 448;

pub struct Sometime {
    pub wl: Wayland,
    canvas: Canvas,
    pub state: State,
    pub last_second: u32,
    pub last_day: u32,
    pub is_happening: bool,
    pub(crate) exit_on_release: bool,
}

impl Sometime {
    pub fn new(wl: Wayland, exit_on_release: bool) -> Self {
        Self {
            wl,
            canvas: Canvas::new(SIDE),
            state: State::Sleep,
            last_second: u32::MAX,
            last_day: u32::MAX,
            is_happening: false,
            exit_on_release,
        }
    }

    pub fn handle(&mut self, event: Event, qh: &QueueHandle<Self>) {
        let (state, action) = self.state.and_then(event);

        self.state = state;

        match action {
            Action::CreateLayer => self.wl.create_layer(qh, "sometime"),
            Action::Draw => self.draw(),
            Action::DestroyLayer => self.wl.destroy_layer(),
            Action::Vanish => self.wl.exit = true,
            Action::Ignore => {}
        }
    }

    pub fn draw(&mut self) {
        let State::Awake(view) = self.state else {
            return;
        };

        let now = Local::now();

        match view {
            View::Clock => {
                self.canvas
                    .pixel_data
                    .copy_from_slice(&self.canvas.clock_bg_cache);
                self.canvas
                    .draw_clock_hands(now.hour(), now.minute(), now.second());

                self.last_second = now.second();
            }
            View::Calendar => {
                self.canvas
                    .pixel_data
                    .copy_from_slice(&self.canvas.calendar_bg_cache);
                self.canvas
                    .draw_calendar_fonts(now.year(), now.month(), now.day());

                self.last_day = now.day();
            }
        }

        self.update_surface();
    }

    fn update_surface(&mut self) {
        let side = self.canvas.side;
        let stride = side * 4;

        if let Some(layer) = self.wl.layer.as_ref()
            && let Ok((buffer, surface)) =
                self.wl
                    .pool
                    .create_buffer(side, side, stride, Format::Argb8888)
        {
            surface.copy_from_slice(&self.canvas.pixel_data);

            let wl_surface = layer.wl_surface();
            wl_surface.damage_buffer(0, 0, side, side);
            buffer.attach_to(wl_surface).ok();
            layer.commit();
        }
    }
}
