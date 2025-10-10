use chrono::{DateTime, Datelike, Local, Timelike};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::{QueueHandle, protocol::wl_shm::Format::Argb8888};

use super::{canvas::Canvas, widget::Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Clock,
    Calendar,
}

pub struct Sometime {
    pub widget: Widget,
    canvas: Canvas,
    view: View,
    last_second: u32,
    last_calendar_day: u32,
}

impl Sometime {
    pub fn new(widget: Widget, canvas: Canvas) -> Self {
        Self {
            widget,
            canvas,
            view: View::Clock,
            last_second: u32::MAX,
            last_calendar_day: u32::MAX,
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

    pub fn init(&mut self) {
        self.canvas.cache_face();
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let now = Local::now();

        match self.view {
            View::Clock => {
                let sec = now.second();
                if sec != self.last_second {
                    self.last_second = sec;
                    self.draw_clock_view(qh, now);
                } else {
                    self.widget
                        .layer
                        .wl_surface()
                        .frame(qh, self.widget.layer.wl_surface().clone());
                    self.widget.layer.commit();
                }
            }
            View::Calendar => {
                let day = now.day();
                if day != self.last_calendar_day {
                    self.last_calendar_day = day;
                    self.draw_calendar(qh, now);
                } else {
                    self.widget
                        .layer
                        .wl_surface()
                        .frame(qh, self.widget.layer.wl_surface().clone());
                    self.widget.layer.commit();
                }
            }
        }
    }

    fn draw_clock_view(&mut self, qh: &QueueHandle<Self>, now: DateTime<Local>) {
        self.canvas.restore_face();
        self.canvas.draw_hour_hand(now.hour(), now.minute());
        self.canvas.draw_minute_hand(now.minute());
        self.canvas.draw_second_hand(now.second());
        self.update_surface(qh);
    }

    fn draw_calendar(&mut self, qh: &QueueHandle<Self>, now: DateTime<Local>) {
        self.canvas.clear();
        self.canvas
            .draw_calendar_view(now.year(), now.month(), now.day());
        self.update_surface(qh);
    }

    fn update_surface(&mut self, qh: &QueueHandle<Self>) {
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
        wl_surface.frame(qh, wl_surface.clone());
        buffer.attach_to(wl_surface).expect("buffer attach");
        self.widget.layer.commit();
    }
}
