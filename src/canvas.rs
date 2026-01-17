use chrono::{Datelike, Duration, NaiveDate};
use cosmic_text::{Align, Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use std::f32::consts::PI;

use crate::{canvas_primitives::CanvasPrimitives, theme::Bgra, theme::Theme};

pub struct Canvas {
    pub primitives: CanvasPrimitives,
    clock_cache: Vec<u8>,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Canvas {
    pub fn new(side: i32) -> Self {
        Self {
            primitives: CanvasPrimitives::new(side),
            clock_cache: Vec::new(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn init(&mut self, theme: Theme) {
        self.draw_face(theme);
        self.clock_cache = self.primitives.pixel_data.clone();
    }

    fn draw_face(&mut self, theme: Theme) {
        let radius = self.primitives.radius;
        let side = self.primitives.side;
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

                self.primitives.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_clock(&mut self, hour: u32, minute: u32, second: u32, theme: Theme) {
        self.primitives
            .pixel_data
            .copy_from_slice(&self.clock_cache);
        self.draw_hour_hand(hour, minute, theme.primary);
        self.draw_minute_hand(minute, theme.primary);
        self.draw_second_hand(second, theme.secondary);
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
        let radius = self.primitives.radius;
        let side = self.primitives.side;
        let end_x = radius + (radius * distance) * angle.cos();
        let end_y = radius + (radius * distance) * angle.sin();

        let dx = end_x - radius;
        let dy = end_y - radius;
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

        let mut x = radius + x_inc * skip_steps as f32;
        let mut y = radius + y_inc * skip_steps as f32;

        for _ in skip_steps..=steps {
            for dy_offset in -search_radius..=search_radius {
                for dx_offset in -search_radius..=search_radius {
                    let px = (x + dx_offset as f32).round() as i32;
                    let py = (y + dy_offset as f32).round() as i32;

                    if px >= 0 && px < side && py >= 0 && py < side {
                        let squared_dist =
                            CanvasPrimitives::squared_distance(x, y, px as f32, py as f32);

                        // Fade out at the edges
                        let alpha = if squared_dist <= inner_radius_sq {
                            1.0
                        } else if squared_dist <= outer_radius_sq {
                            1.0 - (squared_dist - inner_radius_sq)
                                / (outer_radius_sq - inner_radius_sq)
                        } else {
                            continue;
                        };

                        CanvasPrimitives::alpha_blending(
                            &mut self.primitives.pixel_data,
                            CanvasPrimitives::pixel_idx(side, px, py),
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

    pub fn draw_calendar(&mut self, year: i32, month: u32, today: u32, theme: Theme) {
        // Calculate grid dimensions
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
        let start_weekday = first_of_month.weekday().num_days_from_sunday() as i32;
        let days_in_month = Self::days_in_month(year, month);
        let rows_needed = (start_weekday + days_in_month + 6) / 7;

        let side = self.primitives.side;

        // Base spacing
        let padding = (side as f32 / 32.0).ceil() as i32;
        let frame_thickness = 2;

        // Grid layout with 7 columns
        let cell_width = ((side - 2 * padding) / 7) as f32;
        let cell_height = (cell_width * 0.7).ceil() as i32;

        // Font sizes
        let month_font_size = (cell_width * 0.5).ceil();
        let weekday_font_size = (cell_width * 0.4).ceil();
        let day_font_size = (cell_width * 0.5).ceil();

        let month_header = first_of_month.format("%B %Y").to_string();
        let month_height = month_font_size.ceil() as i32;

        // Calendar dimensions
        let total_width = cell_width as i32 * 7 + 2 * padding;
        let total_height = 3 * padding + month_height + cell_height + rows_needed * cell_height;

        // Center on canvas
        let rect_x = (side - total_width) / 2;
        let rect_y = (side - total_height) / 2;

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
        self.draw_text(
            &month_header,
            rect_x + padding,
            content_y,
            month_font_size,
            total_width as f32 - 2.0 * padding as f32,
            theme.primary,
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
                theme.secondary,
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
                    theme.secondary,
                );
                self.draw_text(
                    &day_str,
                    text_x,
                    text_y - 1,
                    font_size,
                    cell_width,
                    theme.highlight,
                );
                self.draw_text(
                    &day_str,
                    text_x - 1,
                    text_y - 2,
                    font_size,
                    cell_width,
                    theme.highlight,
                );
            } else {
                self.draw_text(
                    &day_str,
                    text_x,
                    text_y,
                    font_size,
                    cell_width,
                    theme.primary,
                );
            }
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
        self.primitives.fill_rect(x, y, width, height, theme.frame);

        // Draw background
        let inner_x = x + frame_thickness;
        let inner_y = y + frame_thickness;
        let inner_w = width - 2 * frame_thickness;
        let inner_h = height - 2 * frame_thickness;
        self.primitives
            .fill_rect(inner_x, inner_y, inner_w, inner_h, theme.background);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: f32, width: f32, color: Bgra) {
        let buffer = self.create_drawing_buffer(text, font_size, width);
        // Convert BGRA to RGBA
        let text_color = Color::rgba(color.r(), color.g(), color.b(), color.a());

        // Capture needed fields to avoid borrow issues
        let side = self.primitives.side;
        let pixel_data = &mut self.primitives.pixel_data;

        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            text_color,
            |gx, gy, _w, _h, glyph_color| {
                let px = x + gx;
                let py = y + gy;

                if px >= 0 && px < side && py >= 0 && py < side {
                    CanvasPrimitives::alpha_blending(
                        pixel_data,
                        CanvasPrimitives::pixel_idx(side, px, py),
                        color,
                        glyph_color.a(),
                    );
                }
            },
        );
    }

    fn create_drawing_buffer(&mut self, text: &str, font_size: f32, width: f32) -> Buffer {
        let side = self.primitives.side;
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(width), Some(side as f32));
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
}
