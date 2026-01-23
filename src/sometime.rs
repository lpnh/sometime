use chrono::{Datelike, Local, Timelike};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::protocol::wl_shm::Format;

use crate::{View, canvas::Canvas, theme::Theme, widget::Widget};

pub struct Sometime {
    pub widget: Widget,
    pub canvas: Canvas,
    pub theme: Theme,
    last_second: u32,
    last_calendar_day: u32,
    pub view: View,
    pub wake_up: bool,
    pub initialization_done: bool,
    pub exit_on_release: bool,
    last_view: Option<View>,
    // TODO: we definitely need a proper state machine
}

impl Sometime {
    pub fn new(widget: Widget, canvas: Canvas, exit_on_release: bool) -> Self {
        Self {
            widget,
            canvas,
            theme: Theme::default(),
            last_second: u32::MAX,
            last_calendar_day: u32::MAX,
            view: View::Hidden,
            wake_up: false,
            initialization_done: false,
            exit_on_release,
            last_view: None,
        }
    }

    pub fn draw(&mut self, qh: &wayland_client::QueueHandle<Self>) {
        match self.view {
            View::Clock => {
                if self.widget.layer.is_none() {
                    self.widget.create_layer_surface(qh, "sometime");
                    self.initialization_done = false;
                } else if self.initialization_done {
                    self.draw_clock();
                }
            }
            View::Calendar => {
                if self.widget.layer.is_none() {
                    self.widget.create_layer_surface(qh, "sometime");
                    self.initialization_done = false;
                } else if self.initialization_done {
                    self.draw_calendar();
                }
            }
            View::Hidden => {
                // TODO: implement redraw state?
                self.last_second = u32::MAX;
                self.last_calendar_day = u32::MAX;
                self.canvas.clear();
                self.widget.destroy_layer();
            }
        }
    }

    pub fn draw_clock(&mut self) {
        let now = Local::now();
        let sec = now.second();

        if sec != self.last_second {
            self.last_second = sec;

            self.canvas
                .pixel_data
                .copy_from_slice(&self.canvas.clock_bg_cache);
            self.last_view = Some(View::Clock);

            self.canvas
                .draw_clock_hands(now.hour(), now.minute(), now.second(), self.theme);
            self.update_surface();
        }
    }

    pub fn draw_calendar(&mut self) {
        let now = Local::now();
        let day = now.day();

        if day != self.last_calendar_day {
            self.last_calendar_day = day;

            if self.last_view != Some(View::Calendar) {
                self.canvas
                    .pixel_data
                    .copy_from_slice(&self.canvas.calendar_bg_cache);
                self.last_view = Some(View::Calendar);
            }

            self.canvas
                .draw_calendar_fonts(now.year(), now.month(), now.day(), self.theme);
            self.update_surface();
        }
    }

    fn update_surface(&mut self) {
        let Some(layer) = self.widget.layer.as_ref() else {
            return;
        };
        let side = self.canvas.side;
        let stride = side * 4;

        let (buffer, surface) = self
            .widget
            .pool
            .create_buffer(side, side, stride, Format::Argb8888)
            .expect("create buffer");

        surface.copy_from_slice(&self.canvas.pixel_data);

        let wl_surface = layer.wl_surface();
        wl_surface.damage_buffer(0, 0, side, side);
        buffer.attach_to(wl_surface).expect("buffer attach");
        layer.commit();
    }
}
