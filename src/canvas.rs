use chrono::{Datelike, Duration, NaiveDate};
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use std::f32::consts::PI;

use super::theme::{Bgra, Theme};

pub struct Canvas {
    pub side: i32,
    radius: f32,
    pixel_data: Vec<u8>,
    clock_cache: Vec<u8>,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Canvas {
    pub fn new(side: i32) -> Self {
        let pixel_data = vec![0u8; (side * side * 4) as usize];

        Self {
            side,
            radius: (side / 2) as f32,
            pixel_data,
            clock_cache: Vec::new(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn init(&mut self, theme: Theme) {
        self.draw_face(theme);
        self.clock_cache = self.pixel_data.clone();
    }

    fn draw_face(&mut self, theme: Theme) {
        let radius_sq = self.radius * self.radius;
        let bg_radius_sq = (self.radius - 2.0) * (self.radius - 2.0);

        for y in 0..self.side {
            for x in 0..self.side {
                let dx = x as f32 - self.radius;
                let dy = y as f32 - self.radius;
                let center_dist_sq = dx * dx + dy * dy;

                // Center dot
                let color = if center_dist_sq < 16.0 {
                    theme.primary
                // Background
                } else if center_dist_sq <= bg_radius_sq {
                    theme.background
                // Frame
                } else if center_dist_sq <= radius_sq {
                    theme.frame
                } else {
                    continue; // Outside circle
                };

                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_clock(&mut self, hour: u32, minute: u32, second: u32, theme: Theme) {
        self.pixel_data.copy_from_slice(&self.clock_cache);
        self.draw_hour_hand(hour, minute, theme.primary);
        self.draw_minute_hand(minute, theme.primary);
        self.draw_second_hand(second, theme.secondary);
    }

    fn draw_hour_hand(&mut self, hour: u32, minute: u32, color: Bgra) {
        let angle = ((hour % 12) as f32 + minute as f32 / 60.0) * PI / 6.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.5, angle, 3, color);
    }

    fn draw_minute_hand(&mut self, minute: u32, color: Bgra) {
        let angle = minute as f32 * PI / 30.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.8, angle, 2, color);
    }

    fn draw_second_hand(&mut self, second: u32, color: Bgra) {
        let angle = second as f32 * PI / 30.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.9, angle, 1, color);
    }

    fn draw_thick_line_from_center(
        &mut self,
        distance: f32,
        angle: f32,
        thickness: i32,
        color: Bgra,
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
        let search_radius = (half_thickness + 2.0).ceil() as i32;
        let inner_radius = half_thickness - 1.0;
        let outer_radius = half_thickness + 1.0;
        let inner_radius_sq = inner_radius * inner_radius;
        let outer_radius_sq = outer_radius * outer_radius;

        for _ in 0..=steps {
            for dy_offset in -search_radius..=search_radius {
                for dx_offset in -search_radius..=search_radius {
                    let px = (x + dx_offset as f32).round() as i32;
                    let py = (y + dy_offset as f32).round() as i32;

                    if px >= 0 && px < self.side && py >= 0 && py < self.side {
                        let squared_dist = Self::squared_distance(x, y, px as f32, py as f32);

                        // Fade out at the edges
                        let alpha = if squared_dist <= inner_radius_sq {
                            1.0
                        } else if squared_dist <= outer_radius_sq {
                            1.0 - (squared_dist - inner_radius_sq)
                                / (outer_radius_sq - inner_radius_sq)
                        } else {
                            continue;
                        };

                        Self::alpha_blending(
                            &mut self.pixel_data,
                            Self::pixel_idx(self.side, px, py),
                            color,
                            (alpha * 255.0) as u8,
                        );
                    }
                }
            }
            x += x_inc;
            y += y_inc;
        }
    }

    pub fn clear(&mut self) {
        self.pixel_data.fill(0);
    }

    pub fn get_data(&self) -> &[u8] {
        &self.pixel_data
    }

    #[inline]
    fn pixel_idx(side: i32, x: i32, y: i32) -> usize {
        ((y * side + x) * 4) as usize
    }

    #[inline]
    fn squared_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        dx * dx + dy * dy
    }

    pub fn draw_calendar(&mut self, year: i32, month: u32, today: u32, theme: Theme) {
        let primary_color = theme.primary;
        let secondary_color = theme.secondary;

        // Calculate grid dimensions
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
        let start_weekday = first_of_month.weekday().num_days_from_sunday() as i32;
        let days_in_month = Self::days_in_month(year, month);
        let rows_needed = (start_weekday + days_in_month + 6) / 7;

        // Base spacing
        let padding = (self.side as f32 / 32.0).ceil() as i32;
        let frame_thickness = 2;

        // Grid layout with 7 columns
        let cell_width = (self.side - 2 * padding) / 7;
        let cell_height = (cell_width as f32 * 0.7).ceil() as i32;

        // Font sizes
        let month_font_size = (cell_width as f32 * 0.5).ceil();
        let weekday_font_size = (cell_width as f32 * 0.4).ceil();
        let day_font_size = (cell_width as f32 * 0.5).ceil();

        let month_header = first_of_month.format("%B %Y").to_string();
        let month_height = self.measure_text_height(month_font_size).ceil() as i32;

        // Calendar dimensions
        let total_width = cell_width * 7 + 2 * padding;
        let total_height = 3 * padding + month_height + cell_height + rows_needed * cell_height;

        // Center on canvas
        let rect_x = (self.side - total_width) / 2;
        let rect_y = (self.side - total_height) / 2;

        // Draw background frame
        self.draw_calendar_bg(
            rect_x,
            rect_y,
            total_width,
            total_height,
            frame_thickness,
            theme,
        );

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
        let day_header_height = self.measure_text_height(weekday_font_size).ceil() as i32;
        for (i, day_name) in weekdays.iter().enumerate() {
            let day_w = self.measure_text_width(day_name, weekday_font_size);
            let day_x =
                rect_x + padding + i as i32 * cell_width + (cell_width - day_w.ceil() as i32) / 2;
            let day_y = content_y + (cell_height - day_header_height) / 2;
            self.draw_text(day_name, day_x, day_y, weekday_font_size, secondary_color);
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

            if today == day as u32 {
                // Draw background rectangle for today
                let margin = 4;
                let cell_x = rect_x + padding + col * cell_width + margin;
                let cell_y = content_y + row * cell_height;
                let cell_w = cell_width - 2 * margin;

                self.fill_rect(cell_x, cell_y, cell_w, cell_height, theme.frame);
            }
            self.draw_text(&day_str, text_x, text_y, day_font_size, primary_color);
        }
    }

    fn draw_calendar_bg(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        frame_thickness: i32,
        theme: Theme,
    ) {
        // Draw frame
        self.fill_rect(x, y, width, height, theme.frame);

        // Draw background
        let inner_x = x + frame_thickness;
        let inner_y = y + frame_thickness;
        let inner_w = width - 2 * frame_thickness;
        let inner_h = height - 2 * frame_thickness;
        self.fill_rect(inner_x, inner_y, inner_w, inner_h, theme.background);
    }

    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Bgra) {
        for py in y..(y + height).min(self.side) {
            for px in x..(x + width).min(self.side) {
                if px >= 0 && py >= 0 && px < self.side && py < self.side {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: f32, color: Bgra) {
        let buffer = self.create_drawing_buffer(text, font_size);
        // Convert BGRA to RGBA
        let text_color = Color::rgba(color.r(), color.g(), color.b(), color.a());

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
                    Self::alpha_blending(
                        pixel_data,
                        Self::pixel_idx(side, px, py),
                        color,
                        glyph_color.a(),
                    );
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
        let height = self.measure_text_height(font_size);
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

        buffer.layout_runs().next().map_or(0.0, |run| run.line_w)
    }

    fn measure_text_height(&self, font_size: f32) -> f32 {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        metrics.line_height
    }

    fn days_in_month(year: i32, month: u32) -> i32 {
        let (ny, nm) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let next_month = NaiveDate::from_ymd_opt(ny, nm, 1).expect("days_in_month: invalid date");
        (next_month - Duration::days(1)).day() as i32
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Bgra) {
        let index = Self::pixel_idx(self.side, x, y);
        if index + 3 < self.pixel_data.len() {
            self.pixel_data[index..index + 4].copy_from_slice(color.as_ref());
        }
    }

    fn alpha_blending(pxl_data: &mut [u8], idx: usize, color: Bgra, alpha: u8) {
        if idx + 3 >= pxl_data.len() {
            return;
        }

        let inv_alpha = 255 - alpha;

        pxl_data[idx] = Self::blend_color(color.b(), alpha, pxl_data[idx], inv_alpha);
        pxl_data[idx + 1] = Self::blend_color(color.g(), alpha, pxl_data[idx + 1], inv_alpha);
        pxl_data[idx + 2] = Self::blend_color(color.r(), alpha, pxl_data[idx + 2], inv_alpha);
    }

    #[inline]
    fn blend_color(src: u8, alpha: u8, dst: u8, inv_alpha: u8) -> u8 {
        ((src as u16 * alpha as u16 + dst as u16 * inv_alpha as u16) >> 8) as u8
    }
}
