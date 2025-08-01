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
        let mut pixel_data = vec![0u8; (side * side * 4) as usize];

        // Clear canvas with transparent background
        for pixel in pixel_data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }

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
        let center = (self.radius, self.radius);
        let end = Self::with_radius_and_angle(self, center, distance, angle);

        for dx in -thickness / 2..=thickness / 2 {
            for dy in -thickness / 2..=thickness / 2 {
                let (start_x, start_y) = Self::offset(center, dx, dy);
                let (end_x, end_y) = Self::offset(end, dx, dy);
                self.draw_line(start_x, start_y, end_x, end_y);
            }
        }
    }

    pub fn draw_line(&mut self, start_x: f32, start_y: f32, end_x: f32, end_y: f32) {
        let dx = (end_x - start_x).abs();
        let dy = (end_y - start_y).abs();
        let steps = dx.max(dy) as i32;

        let x_inc = (end_x - start_x) / steps as f32;
        let y_inc = (end_y - start_y) / steps as f32;

        for i in 0..=steps {
            let (x, y) = (start_x + i as f32 * x_inc, start_y + i as f32 * y_inc);
            let index = ((y as i32 * self.side + x as i32) * 4) as usize;
            if index + 3 < self.pixel_data.len() {
                self.pixel_data[index..index + 4].copy_from_slice(T::HANDS.as_ref());
            }
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.pixel_data
    }

    // Functions to handle distances
    fn with_radius_and_angle(&self, one: (f32, f32), distance: f32, angle: f32) -> (f32, f32) {
        let x = one.0 + (self.radius * distance) * angle.cos();
        let y = one.1 + (self.radius * distance) * angle.sin();
        (x, y)
    }
    fn distance_to(&self, other: (i32, i32)) -> f32 {
        let dx = self.radius - other.0 as f32;
        let dy = self.radius - other.1 as f32;
        (dx * dx + dy * dy).sqrt()
    }
    fn offset(s: (f32, f32), dx: i32, dy: i32) -> (f32, f32) {
        (s.0 + dx as f32, s.1 + dy as f32)
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
