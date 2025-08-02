use super::theme::Theme;
use std::f32::consts::PI;
use std::marker::PhantomData;

pub struct Canvas<T: Theme> {
    pixel_data: Vec<u8>,
    side: i32,
    radius: f32,
    _theme: PhantomData<T>,
}

impl<T: Theme> Canvas<T> {
    pub fn new(side: i32) -> Self {
        let pixel_data = vec![0u8; (side * side * 4) as usize];

        Self {
            pixel_data,
            side,
            radius: (side / 2) as f32,
            _theme: PhantomData,
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
                        self.pixel_data[index..index + 4].copy_from_slice(T::HANDS.as_ref());
                    }
                // Face
                } else if distance <= self.radius - 2.0 {
                    let index = ((y * self.side + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4].copy_from_slice(T::FACE.as_ref());
                    }
                // Frame
                } else if distance <= self.radius {
                    let index = ((y * self.side + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4].copy_from_slice(T::FRAME.as_ref());
                    }
                }
            }
        }
    }

    pub fn draw_hour_hand(&mut self, hour: u32, minute: u32) {
        let angle = Self::hour_angle(hour, minute);
        self.draw_thick_line_from_center(0.5, angle, 3);
    }

    pub fn draw_minute_hand(&mut self, minute: u32) {
        let angle = Self::minute_angle(minute);
        self.draw_thick_line_from_center(0.8, angle, 2);
    }

    pub fn draw_second_hand(&mut self, second: u32) {
        let angle = Self::second_angle(second);
        self.draw_thick_line_from_center(0.9, angle, 1);
    }

    pub fn draw_thick_line_from_center(&mut self, distance: f32, angle: f32, thickness: i32) {
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

        let half_thickness = thickness / 2;

        for _ in 0..=steps {
            for brush_dx in -half_thickness..=half_thickness {
                for brush_dy in -half_thickness..=half_thickness {
                    let px = (x + brush_dx as f32).round() as i32;
                    let py = (y + brush_dy as f32).round() as i32;

                    if px >= 0 && px < self.side && py >= 0 && py < self.side {
                        let index = ((py * self.side + px) * 4) as usize;
                        if index + 3 < self.pixel_data.len() {
                            self.pixel_data[index..index + 4].copy_from_slice(T::HANDS.as_ref());
                        }
                    }
                }
            }
            x += x_inc;
            y += y_inc;
        }
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
}
