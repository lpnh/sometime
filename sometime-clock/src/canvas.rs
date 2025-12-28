use std::f32::consts::PI;

use sometime::{Bgra, CanvasPrimitives, Theme};

pub struct ClockCanvas {
    pub primitives: CanvasPrimitives,
    clock_cache: Vec<u8>,
}

impl ClockCanvas {
    pub fn new(side: i32) -> Self {
        Self {
            primitives: CanvasPrimitives::new(side),
            clock_cache: Vec::new(),
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
}
