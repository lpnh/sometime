use chrono::{Datelike, Duration, NaiveDate};
use cosmic_text::{
    Align, Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache,
    fontdb::{Database, Source},
};
use std::{f32::consts::PI, sync::Arc};

use super::theme::{Bgra, Theme};

pub struct Canvas {
    pub side: i32,
    radius: f32,
    pub pixel_data: Vec<u8>,
    pub clock_bg_cache: Vec<u8>,
    pub calendar_bg_cache: Vec<u8>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    theme: Theme,
}

impl Canvas {
    pub fn new(side: i32) -> Self {
        let theme = Theme::default();

        let radius = (side / 2) as f32;
        let clock_bg_cache = Self::draw_clock_bg(side, radius, &theme);
        let calendar_bg_cache = Self::draw_calendar_bg(side, &theme);

        let font = Arc::new(include_bytes!("../fonts/Inter-Regular.ttf"));
        let mut font_db = Database::new();
        font_db.load_font_source(Source::Binary(font));

        Self {
            side,
            radius,
            pixel_data: Self::new_buffer(side),
            clock_bg_cache,
            calendar_bg_cache,
            font_system: FontSystem::new_with_locale_and_db("en-US".into(), font_db),
            swash_cache: SwashCache::new(),
            theme,
        }
    }

    pub fn draw_clock_hands(&mut self, hour: u32, minute: u32, second: u32) {
        self.draw_hour_hand(hour, minute, self.theme.primary);
        self.draw_minute_hand(minute, self.theme.primary);
        self.draw_second_hand(second, self.theme.secondary);
    }

    fn draw_hour_hand(&mut self, hour: u32, minute: u32, color: Bgra) {
        let angle = ((hour % 12) as f32 + minute as f32 / 60.0) * PI / 6.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.5, angle, 3.0, color);
    }

    fn draw_minute_hand(&mut self, minute: u32, color: Bgra) {
        let angle = minute as f32 * PI / 30.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.8, angle, 2.0, color);
    }

    fn draw_second_hand(&mut self, second: u32, color: Bgra) {
        let angle = second as f32 * PI / 30.0 - PI / 2.0;
        self.draw_thick_line_from_center(0.9, angle, 0.7, color);
    }

    fn draw_thick_line_from_center(
        &mut self,
        distance: f32,
        angle: f32,
        thickness: f32,
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

        let half_thickness = thickness / 2.0;
        let search_radius = (half_thickness + 2.0).ceil() as i32;
        let inner_radius = half_thickness - 1.0;
        let outer_radius = half_thickness + 1.0;
        let inner_radius_sq = inner_radius * inner_radius;
        let outer_radius_sq = outer_radius * outer_radius;

        let center_gap_radius = 4;
        let skip_steps = center_gap_radius + search_radius;

        let mut x = self.radius + x_inc * skip_steps as f32;
        let mut y = self.radius + y_inc * skip_steps as f32;

        for _ in skip_steps..=steps {
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

    fn draw_clock_bg(side: i32, radius: f32, theme: &Theme) -> Vec<u8> {
        let mut buffer = Self::new_buffer(side);

        let radius_sq = radius * radius;
        let bg_radius_sq = (radius - 2.0) * (radius - 2.0);

        for y in 0..side {
            for x in 0..side {
                let dx = x as f32 - radius;
                let dy = y as f32 - radius;
                let center_dist_sq = dx * dx + dy * dy;

                // Center dot
                let color = if center_dist_sq < 12.0 {
                    theme.highlight
                // Background
                } else if center_dist_sq <= bg_radius_sq {
                    theme.background
                // Frame
                } else if center_dist_sq <= radius_sq {
                    theme.frame
                } else {
                    continue; // Outside circle
                };

                Self::set_pixel(&mut buffer, side, x, y, color);
            }
        }
        buffer
    }

    pub fn clear(&mut self) {
        self.pixel_data.fill(0);
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

    pub fn draw_calendar_fonts(&mut self, year: i32, month: u32, today: u32) {
        // Calculate grid dimensions
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
        let start_weekday = first_of_month.weekday().num_days_from_sunday() as i32;
        let days_in_month = Self::days_in_month(year, month);
        let rows_needed = (start_weekday + days_in_month + 6) / 7;

        // Grid layout with 7 columns
        let (padding, cell_width, cell_height, month_height) = Self::calendar_layout(self.side);

        let weekday_font_size = (cell_width * 0.4).ceil();
        let day_font_size = (cell_width * 0.5).ceil();

        let month_header = first_of_month.format("%B %Y").to_string();

        // Calendar dimensions
        let total_width = cell_width as i32 * 7 + 2 * padding;
        let total_height = 3 * padding + month_height + cell_height + rows_needed * cell_height;

        // Center on canvas
        let rect_x = (self.side - total_width) / 2;
        let rect_y = (self.side - total_height) / 2;

        // Draw content relative to top-left, with padding
        let mut content_y = rect_y + padding;

        // Month name
        self.draw_text(
            &month_header,
            rect_x + padding,
            content_y,
            month_height as f32,
            total_width as f32 - 2.0 * padding as f32,
            self.theme.primary,
        );
        content_y += month_height + padding;

        // Weekday headers
        let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let day_header_height = weekday_font_size.ceil() as i32;
        for (i, day_name) in weekdays.iter().enumerate() {
            let day_x = rect_x + padding + i as i32 * cell_width as i32;
            let day_y = content_y + (cell_height - day_header_height) / 2;
            self.draw_text(
                day_name,
                day_x,
                day_y,
                weekday_font_size,
                cell_width,
                self.theme.secondary,
            );
        }
        content_y += cell_height;

        // Calendar grid
        for day in 1..=days_in_month {
            let day_pos = start_weekday + day - 1;
            let row = day_pos / 7;
            let col = day_pos % 7;

            let day_str = day.to_string();
            let is_today = today == day as u32;
            let font_size = if is_today {
                day_font_size + 6.0
            } else {
                day_font_size
            };

            let text_x = rect_x + padding + col * cell_width as i32;
            let text_y =
                content_y + row * cell_height + (cell_height - font_size.ceil() as i32) / 2;

            if is_today {
                // Bold + shadow effect
                self.draw_text(
                    &day_str,
                    text_x + 1,
                    text_y,
                    font_size,
                    cell_width,
                    self.theme.secondary,
                );
                self.draw_text(
                    &day_str,
                    text_x,
                    text_y - 1,
                    font_size,
                    cell_width,
                    self.theme.highlight,
                );
                self.draw_text(
                    &day_str,
                    text_x - 1,
                    text_y - 2,
                    font_size,
                    cell_width,
                    self.theme.highlight,
                );
            } else {
                self.draw_text(
                    &day_str,
                    text_x,
                    text_y,
                    font_size,
                    cell_width,
                    self.theme.primary,
                );
            }
        }
    }

    fn draw_calendar_bg(side: i32, theme: &Theme) -> Vec<u8> {
        let mut buffer = Self::new_buffer(side);

        // Grid layout with 7 columns
        let (padding, cell_width, cell_height, month_height) = Self::calendar_layout(side);
        let frame_thickness = 2;

        // Calendar dimensions
        let max_rows_needed = 6;
        let width = cell_width as i32 * 7 + 2 * padding;
        let height = 3 * padding + month_height + cell_height + max_rows_needed * cell_height;

        // Center on canvas
        let x = (side - width) / 2;
        let y = (side - height) / 2;

        // Draw frame
        Self::fill_rect(&mut buffer, side, x, y, width, height, theme.frame);

        // Draw background
        Self::fill_rect(
            &mut buffer,
            side,
            x + frame_thickness,
            y + frame_thickness,
            width - 2 * frame_thickness,
            height - 2 * frame_thickness,
            theme.background,
        );
        buffer
    }

    fn fill_rect(buffer: &mut [u8], side: i32, x: i32, y: i32, w: i32, h: i32, color: Bgra) {
        for py in y..(y + h).min(side) {
            for px in x..(x + w).min(side) {
                if px >= 0 && py >= 0 && px < side && py < side {
                    Self::set_pixel(buffer, side, px, py, color);
                }
            }
        }
    }

    fn calendar_layout(side: i32) -> (i32, f32, i32, i32) {
        let padding = (side as f32 / 32.0).ceil() as i32;
        let width = ((side - 2 * padding) / 7) as f32;
        let height = (width * 0.7).ceil() as i32;
        let month_height = (width * 0.5).ceil() as i32;

        (padding, width, height, month_height)
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: f32, width: f32, color: Bgra) {
        let buffer = self.create_drawing_buffer(text, font_size, width);
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

    fn create_drawing_buffer(&mut self, text: &str, font_size: f32, width: f32) -> Buffer {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(width), Some(self.side as f32));
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new(),
            Shaping::Advanced,
            Some(Align::Center),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
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

    fn set_pixel(buffer: &mut [u8], side: i32, x: i32, y: i32, color: Bgra) {
        let index = Self::pixel_idx(side, x, y);
        if index + 3 < buffer.len() {
            buffer[index..index + 4].copy_from_slice(color.as_ref());
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
    fn new_buffer(side: i32) -> Vec<u8> {
        vec![0u8; (side * side * 4) as usize]
    }

    #[inline]
    fn blend_color(src: u8, alpha: u8, dst: u8, inv_alpha: u8) -> u8 {
        ((src as u16 * alpha as u16 + dst as u16 * inv_alpha as u16) >> 8) as u8
    }
}
