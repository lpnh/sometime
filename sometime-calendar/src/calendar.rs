use chrono::{Datelike, Local};

use super::canvas::CalendarCanvas;
use sometime_core::{App, Theme, Widget};

pub struct Calendar {
    pub widget: Widget,
    pub canvas: CalendarCanvas,
    pub theme: Theme,
    last_calendar_day: u32,
    pub is_happening: bool,
}

impl Calendar {
    pub fn draw(&mut self) {
        let now = Local::now();
        let day = now.day();

        if day != self.last_calendar_day {
            self.last_calendar_day = day;
            self.canvas.primitives.clear();
            self.canvas
                .draw_calendar(now.year(), now.month(), now.day(), self.theme);
            sometime_core::update_surface(
                &self.widget.layer,
                &mut self.widget.pool,
                &self.canvas.primitives,
            );
        }
    }
}

impl App for Calendar {
    type Canvas = CalendarCanvas;

    fn new(theme: Theme, widget: Widget, canvas: Self::Canvas) -> Self {
        Self {
            theme,
            widget,
            canvas,
            last_calendar_day: u32::MAX,
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
        CalendarCanvas::new(side)
    }
}
