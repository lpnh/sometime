use chrono::{Local, Timelike};

use super::canvas::ClockCanvas;
use sometime::{App, Theme, Widget};

pub struct Clock {
    pub widget: Widget,
    pub canvas: ClockCanvas,
    pub theme: Theme,
    last_second: u32,
    pub is_happening: bool,
}

impl Clock {
    pub fn draw(&mut self) {
        let now = Local::now();
        let sec = now.second();

        if sec != self.last_second {
            self.last_second = sec;
            self.canvas
                .draw_clock(now.hour(), now.minute(), now.second(), self.theme);
            sometime::update_surface(
                &self.widget.layer,
                &mut self.widget.pool,
                &self.canvas.primitives,
            );
        }
    }
}

impl App for Clock {
    type Canvas = ClockCanvas;

    fn new(theme: Theme, widget: Widget, canvas: Self::Canvas) -> Self {
        Self {
            theme,
            widget,
            canvas,
            last_second: u32::MAX,
            is_happening: false,
        }
    }

    fn widget_mut(&mut self) -> &mut Widget {
        &mut self.widget
    }

    fn is_happening_mut(&mut self) -> &mut bool {
        &mut self.is_happening
    }

    fn draw(&mut self) {
        self.draw()
    }

    fn new_canvas(side: i32) -> Self::Canvas {
        ClockCanvas::new(side)
    }
}
