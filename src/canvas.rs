use chrono::Datelike;
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use std::f32::consts::PI;

use super::theme::Theme;

pub struct Canvas {
    pub side: i32,
    pixel_data: Vec<u8>,
    radius: f32,
    theme: Theme,
    face_cache: Vec<u8>,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Canvas {
    pub fn new(side: i32, theme: Theme) -> Self {
        let pixel_data = vec![0u8; (side * side * 4) as usize];

        Self {
            pixel_data,
            side,
            radius: (side / 2) as f32,
            theme,
            face_cache: Vec::new(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn draw_face(&mut self) {
        for y in 0..self.side {
            for x in 0..self.side {
                let distance = Self::distance_to(self, (x, y));

                // Center dot
                if distance < 4.0 {
                    let index = ((y * self.side + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4]
                            .copy_from_slice(self.theme.primary.as_ref());
                    }
                // Background
                } else if distance <= self.radius - 2.0 {
                    let index = ((y * self.side + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4]
                            .copy_from_slice(self.theme.background.as_ref());
                    }
                // Frame
                } else if distance <= self.radius {
                    let index = ((y * self.side + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4]
                            .copy_from_slice(self.theme.frame.as_ref());
                    }
                }
            }
        }
    }

    pub fn draw_hour_hand(&mut self, hour: u32, minute: u32) {
        let angle = Self::hour_angle(hour, minute);
        let color = self.theme.primary;
        self.draw_thick_line_from_center(0.5, angle, 3, color.as_ref());
    }

    pub fn draw_minute_hand(&mut self, minute: u32) {
        let angle = Self::minute_angle(minute);
        let color = self.theme.primary;
        self.draw_thick_line_from_center(0.8, angle, 2, color.as_ref());
    }

    pub fn draw_second_hand(&mut self, second: u32) {
        let angle = Self::second_angle(second);
        let color = self.theme.secondary;
        self.draw_thick_line_from_center(0.9, angle, 1, color.as_ref());
    }

    pub fn draw_thick_line_from_center(
        &mut self,
        distance: f32,
        angle: f32,
        thickness: i32,
        color: &[u8],
    ) {
        let end_x = self.radius + (self.radius * distance) * angle.cos();
        let end_y = self.radius + (self.radius * distance) * angle.sin();

        let dx = end_x - self.radius;
        let dy = end_y - self.radius;
        let steps = dx.abs().max(dy.abs()) as i32;

        if steps == 0 {
            return;
        }

        let x_inc = dx / steps as f32;
        let y_inc = dy / steps as f32;

        let mut x = self.radius;
        let mut y = self.radius;

        let half_thickness = thickness as f32 / 2.0;

        for _ in 0..=steps {
            let search_radius = (half_thickness + 2.0).ceil() as i32;
            for dy_offset in -search_radius..=search_radius {
                for dx_offset in -search_radius..=search_radius {
                    let px = (x + dx_offset as f32).round() as i32;
                    let py = (y + dy_offset as f32).round() as i32;

                    if px >= 0 && px < self.side && py >= 0 && py < self.side {
                        let dist = ((px as f32 - x).powi(2) + (py as f32 - y).powi(2)).sqrt();

                        // Fade out at the edges
                        let alpha = if dist <= half_thickness - 1.0 {
                            1.0
                        } else if dist <= half_thickness + 1.0 {
                            1.0 - (dist - (half_thickness - 1.0)) / 2.0
                        } else {
                            continue;
                        };

                        let index = ((py * self.side + px) * 4) as usize;
                        blend_pixel(
                            &mut self.pixel_data,
                            index,
                            [color[0], color[1], color[2], 255],
                            alpha,
                        );
                    }
                }
            }
            x += x_inc;
            y += y_inc;
        }
    }

    pub fn cache_face(&mut self) {
        self.draw_face();
        self.face_cache = self.pixel_data.clone();
    }

    pub fn restore_face(&mut self) {
        debug_assert_eq!(self.pixel_data.len(), self.face_cache.len());
        self.pixel_data.copy_from_slice(&self.face_cache);
    }

    pub fn clear(&mut self) {
        self.pixel_data.fill(0);
    }

    pub fn get_data(&self) -> &[u8] {
        &self.pixel_data
    }

    fn distance_to(&self, other: (i32, i32)) -> f32 {
        let dx = self.radius - other.0 as f32;
        let dy = self.radius - other.1 as f32;
        (dx * dx + dy * dy).sqrt()
    }

    // Functions to handle clock angles
    fn hour_angle(hour: u32, minute: u32) -> f32 {
        ((hour % 12) as f32 + minute as f32 / 60.0) * PI / 6.0 - PI / 2.0
    }

    fn minute_angle(minute: u32) -> f32 {
        minute as f32 * PI / 30.0 - PI / 2.0
    }

    fn second_angle(second: u32) -> f32 {
        second as f32 * PI / 30.0 - PI / 2.0
    }

    pub fn draw_calendar_view(&mut self, year: i32, month: u32, today: u32) {
        let primary_color = self
            .theme
            .primary
            .as_ref()
            .try_into()
            .expect("invalid color");
        let secondary_color = self
            .theme
            .secondary
            .as_ref()
            .try_into()
            .expect("invalid color");

        // Calculate grid dimensions
        let first_of_month =
            chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date: first of month");
        let start_weekday = first_of_month.weekday().num_days_from_sunday() as i32;
        let days_in_month = Self::days_in_month(year, month);
        let rows_needed = (start_weekday + days_in_month + 6) / 7;

        // Base spacing
        let padding = (self.side as f32 / 32.0).ceil() as i32;
        let frame_thickness = 2;

        // Grid layout with 7 columns
        let available_width = self.side - 2 * padding;
        let cell_width = available_width / 7;
        let grid_width = cell_width * 7;
        let cell_height = (cell_width as f32 * 0.7).ceil() as i32;

        // Font sizes
        let month_font_size = (cell_width as f32 * 0.5).ceil();
        let weekday_font_size = (cell_width as f32 * 0.4).ceil();
        let day_font_size = (cell_width as f32 * 0.5).ceil();

        let month_header = first_of_month.format("%B %Y").to_string();
        let month_height = Self::measure_text_height(month_font_size).ceil() as i32;

        // Calendar grid + left/right padding
        let total_width = grid_width + 2 * padding;
        // (top/bottom padding + spacing) + month header + weekday headers + calendar grid
        let total_height = 3 * padding + month_height + cell_height + rows_needed * cell_height;

        // Center on canvas
        let rect_x = (self.side - total_width) / 2;
        let rect_y = (self.side - total_height) / 2;

        // Draw background frame
        self.draw_calendar_bg(rect_x, rect_y, total_width, total_height, frame_thickness);

        // Draw content relative to top-left, with padding
        let mut content_y = rect_y + padding;

        // Month name
        let month_w = self.measure_text_width(&month_header, month_font_size);
        let month_x = rect_x + (total_width - month_w.ceil() as i32) / 2;
        self.draw_text(
            &month_header,
            month_x,
            content_y,
            month_font_size,
            primary_color,
        );
        content_y += month_height + padding;

        // Weekday headers
        let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let day_header_height = Self::measure_text_height(weekday_font_size).ceil() as i32;
        for (i, day) in weekdays.iter().enumerate() {
            let day_w = self.measure_text_width(day, weekday_font_size);
            let day_x =
                rect_x + padding + i as i32 * cell_width + (cell_width - day_w.ceil() as i32) / 2;
            let day_y = content_y + (cell_height - day_header_height) / 2;
            self.draw_text(day, day_x, day_y, weekday_font_size, secondary_color);
        }
        content_y += cell_height;

        // Calendar grid
        for day in 1..=days_in_month {
            let day_pos = start_weekday + day - 1;
            let row = day_pos / 7;
            let col = day_pos % 7;

            let day_str = day.to_string();
            let (text_w, text_h) = self.measure_text(&day_str, day_font_size);

            let text_x =
                rect_x + padding + col * cell_width + (cell_width - text_w.ceil() as i32) / 2;
            let text_y = content_y + row * cell_height + (cell_height - text_h.ceil() as i32) / 2;

            let is_today = day == today as i32;
            if is_today {
                // Draw background rectangle for today
                let margin = 4;
                let cell_x = rect_x + padding + col * cell_width + margin;
                let cell_y = content_y + row * cell_height;

                for py in cell_y..(cell_y + cell_height).min(self.side) {
                    for px in cell_x..(cell_x + cell_width - 2 * margin).min(self.side) {
                        if px >= 0 && py >= 0 && px < self.side && py < self.side {
                            let index = ((py * self.side + px) * 4) as usize;
                            if index + 3 < self.pixel_data.len() {
                                self.pixel_data[index..index + 4]
                                    .copy_from_slice(self.theme.frame.as_ref());
                            }
                        }
                    }
                }
            }
            self.draw_text(&day_str, text_x, text_y, day_font_size, primary_color);
        }
    }

    pub fn draw_calendar_bg(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        frame_thickness: i32,
    ) {
        for py in y..(y + height).min(self.side) {
            for px in x..(x + width).min(self.side) {
                if px < 0 || py < 0 || px >= self.side || py >= self.side {
                    continue;
                }

                let index = ((py * self.side + px) * 4) as usize;
                if index + 3 >= self.pixel_data.len() {
                    continue;
                }

                let is_frame = px < x + frame_thickness
                    || px >= x + width - frame_thickness
                    || py < y + frame_thickness
                    || py >= y + height - frame_thickness;

                let color = if is_frame {
                    self.theme.frame.as_ref()
                } else {
                    self.theme.background.as_ref()
                };

                self.pixel_data[index..index + 4].copy_from_slice(color);
            }
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: f32, color: [u8; 4]) {
        let buffer = self.create_drawing_buffer(text, font_size);

        // Convert BGRA to RGBA
        let text_color = Color::rgba(color[2], color[1], color[0], color[3]);

        // Capture needed fields to avoid borrow issues
        let side = self.side;
        let pixel_data = &mut self.pixel_data;

        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            text_color,
            |gx, gy, _w, _h, glyph_color| {
                let px = x + gx;
                let py = y + gy;

                if px >= 0 && px < side && py >= 0 && py < side {
                    let index = ((py * side + px) * 4) as usize;
                    let alpha = glyph_color.a() as f32 / 255.0;
                    let color_bgra = [
                        glyph_color.b(),
                        glyph_color.g(),
                        glyph_color.r(),
                        glyph_color.a(),
                    ];
                    blend_pixel(pixel_data, index, color_bgra, alpha);
                }
            },
        );
    }

    fn create_drawing_buffer(&mut self, text: &str, font_size: f32) -> Buffer {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(
            &mut self.font_system,
            Some(self.side as f32),
            Some(self.side as f32),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new(),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        let width = self.measure_text_width(text, font_size);
        let height = Self::measure_text_height(font_size);
        (width, height)
    }

    fn measure_text_width(&mut self, text: &str, font_size: f32) -> f32 {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new(),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        buffer
            .layout_runs()
            .next()
            .map(|run| run.line_w)
            .unwrap_or(0.0)
    }

    fn measure_text_height(font_size: f32) -> f32 {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        metrics.line_height
    }

    fn days_in_month(year: i32, month: u32) -> i32 {
        let (ny, nm) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let next_month = chrono::NaiveDate::from_ymd_opt(ny, nm, 1)
            .expect("days_in_month: invalid next month date");
        let last = next_month - chrono::Duration::days(1);
        last.day() as i32
    }
}

fn blend_pixel(pixel_data: &mut [u8], index: usize, color: [u8; 4], alpha: f32) {
    if index + 3 >= pixel_data.len() {
        return;
    }
    let inv_alpha = 1.0 - alpha;
    pixel_data[index] = (color[0] as f32 * alpha + pixel_data[index] as f32 * inv_alpha) as u8;
    pixel_data[index + 1] =
        (color[1] as f32 * alpha + pixel_data[index + 1] as f32 * inv_alpha) as u8;
    pixel_data[index + 2] =
        (color[2] as f32 * alpha + pixel_data[index + 2] as f32 * inv_alpha) as u8;
}
