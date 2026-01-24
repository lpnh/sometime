mod canvas;
pub mod ipc;
mod registry;
mod theme;
pub mod widget;

pub use canvas::Canvas;
pub use theme::{Bgra, Theme};
pub use widget::Widget;

use chrono::{Datelike, Local, Timelike};
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::protocol::wl_shm::Format;

pub const SIDE: i32 = 448;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Sleep,
    Init(View),
    Awake(View),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Clock,
    Calendar,
}

pub struct Sometime {
    pub widget: Widget,
    canvas: Canvas,
    pub state: State,
    should_redraw: bool,
    theme: Theme,
    pub exit_on_release: bool,
    pub last_second: u32,
    pub last_day: u32,
}

impl Sometime {
    pub fn new(widget: Widget, canvas: Canvas, exit_on_release: bool) -> Self {
        Self {
            widget,
            canvas,
            state: State::Sleep,
            should_redraw: false,
            theme: Theme::default(),
            last_second: u32::MAX,
            last_day: u32::MAX,
            exit_on_release,
        }
    }

    pub fn sleep(&mut self) {
        self.state = State::Sleep;
        self.canvas.clear();
        self.widget.destroy_layer();
    }

    pub fn init(&mut self, view: View, qh: &wayland_client::QueueHandle<Self>) {
        self.state = State::Init(view);
        self.widget.create_layer_surface(qh, "sometime");
    }

    pub fn wake_up(&mut self, view: View) {
        self.state = State::Awake(view);
        self.canvas.init(self.theme);
        self.draw();
    }

    pub fn request_redraw(&mut self, view: View) {
        let now = Local::now();
        self.should_redraw = match view {
            View::Clock => now.second() != self.last_second,
            View::Calendar => now.day() != self.last_day,
        };
    }

    pub fn consume_redraw(&mut self) {
        if std::mem::take(&mut self.should_redraw) {
            self.draw();
        }
    }

    fn draw(&mut self) {
        if let State::Awake(view) = self.state {
            let now = Local::now();
            match view {
                View::Clock => {
                    self.canvas
                        .pixel_data
                        .copy_from_slice(&self.canvas.clock_bg_cache);
                    self.canvas.draw_clock_hands(
                        now.hour(),
                        now.minute(),
                        now.second(),
                        self.theme,
                    );

                    self.last_second = now.second();
                }
                View::Calendar => {
                    self.canvas
                        .pixel_data
                        .copy_from_slice(&self.canvas.calendar_bg_cache);
                    self.canvas
                        .draw_calendar_fonts(now.year(), now.month(), now.day(), self.theme);

                    self.last_day = now.day()
                }
            }

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
