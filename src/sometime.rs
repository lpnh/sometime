use chrono::{Datelike, Local, Timelike};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::protocol::wl_shm::Format::Argb8888;

use super::{canvas::Canvas, theme::Theme, widget::Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Clock,
    Calendar,
}

pub struct Sometime {
    pub widget: Widget,
    pub canvas: Canvas,
    pub theme: Theme,
    view: View,
    last_second: u32,
    last_calendar_day: u32,
    pub is_happening: bool,
}

impl Sometime {
    pub fn new(theme: Theme, widget: Widget, canvas: Canvas) -> Self {
        Self {
            theme,
            widget,
            canvas,
            view: View::Clock,
            last_second: u32::MAX,
            last_calendar_day: u32::MAX,
            is_happening: false,
        }
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Clock => View::Calendar,
            View::Calendar => View::Clock,
        };
        // Force immediate redraw
        self.last_second = u32::MAX;
        self.last_calendar_day = u32::MAX;
    }

    pub fn draw(&mut self) {
        let now = Local::now();

        match self.view {
            View::Clock => {
                let sec = now.second();
                if sec != self.last_second {
                    self.last_second = sec;
                    self.canvas.draw_clock(
                        now.hour(),
                        now.minute(),
                        now.second(),
                        self.theme,
                    );
                }
            }
            View::Calendar => {
                let day = now.day();
                if day != self.last_calendar_day {
                    self.last_calendar_day = day;
                    self.canvas.clear();
                    self.canvas.draw_calendar(
                        now.year(),
                        now.month(),
                        now.day(),
                        self.theme,
                    );
                }
            }
        }

        self.update_surface();
    }

    fn update_surface(&mut self) {
        let data = self.canvas.get_data();
        let side = self.canvas.side;
        let stride = side * 4;

        let (buffer, surface) = self
            .widget
            .pool
            .create_buffer(side, side, stride, Argb8888)
            .expect("create buffer");

        surface.copy_from_slice(data);

        let wl_surface = self.widget.layer.wl_surface();
        wl_surface.damage_buffer(0, 0, side, side);
        buffer.attach_to(wl_surface).expect("buffer attach");
        self.widget.layer.commit();
    }
}
