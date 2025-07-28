use super::{
    geometry::{Angle, Point},
    theme::Bgra,
};

pub struct Canvas {
    pixel_data: Vec<u8>,
    width: u32,
    height: u32,
    center: Point,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let mut pixel_data = vec![0u8; (width * height * 4) as usize];

        // Clear canvas with transparent background
        for pixel in pixel_data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }

        let center = Point::from_u32(width / 2, height / 2);

        Self {
            pixel_data,
            width,
            height,
            center,
        }
    }

    pub fn draw_circle(&mut self, radius: f32, color: Bgra) {
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::from_u32(x, y);
                let distance = self.center.distance_to(&point);

                if distance <= radius {
                    let index = ((y * self.width + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4].copy_from_slice(color.as_ref());
                    }
                }
            }
        }
    }

    pub fn draw_marker(&mut self, angle: f32, radius: f32, size: f32, color: Bgra) {
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::from_u32(x, y);
                let distance = self
                    .center
                    .with_radius_and_angle(radius, angle)
                    .distance_to(&point);

                if distance <= size {
                    let index = ((y * self.width + x) * 4) as usize;
                    if index + 3 < self.pixel_data.len() {
                        self.pixel_data[index..index + 4].copy_from_slice(color.as_ref());
                    }
                }
            }
        }
    }

    pub fn draw_hour_hand(&mut self, hour: u32, minute: u32, radius: f32, color: Bgra) {
        let hour_angle = Angle::hour(hour, minute);
        self.draw_thick_line_from_center(radius * 0.5, hour_angle, color, 3);
    }

    pub fn draw_minute_hand(&mut self, minute: u32, radius: f32, color: Bgra) {
        let minute_angle = Angle::minute(minute);
        self.draw_thick_line_from_center(radius * 0.8, minute_angle, color, 2);
    }

    pub fn draw_second_hand(&mut self, second: u32, radius: f32, color: Bgra) {
        let second_angle = Angle::second(second);
        self.draw_thick_line_from_center(radius * 0.9, second_angle, color, 1);
    }

    pub fn draw_thick_line_from_center(
        &mut self,
        radius: f32,
        angle: f32,
        color: Bgra,
        thickness: i32,
    ) {
        for dx in -thickness / 2..=thickness / 2 {
            for dy in -thickness / 2..=thickness / 2 {
                let offset_start = self.center.offset(dx, dy);
                let offset_end = self
                    .center
                    .with_radius_and_angle(radius, angle)
                    .offset(dx, dy);
                self.draw_line(offset_start, offset_end, color);
            }
        }
    }

    pub fn draw_line(&mut self, start: Point, end: Point, color: Bgra) {
        let dx = (end.x - start.x).abs();
        let dy = (end.y - start.y).abs();
        let steps = dx.max(dy) as i32;

        if steps == 0 {
            return;
        }

        let x_inc = (end.x - start.x) / steps as f32;
        let y_inc = (end.y - start.y) / steps as f32;

        for i in 0..=steps {
            let point = Point::new(start.x + i as f32 * x_inc, start.y + i as f32 * y_inc);

            if point.is_valid(self.width, self.height) {
                let (x, y) = point.as_coords();
                let index = ((y * self.width + x) * 4) as usize;
                if index + 3 < self.pixel_data.len() {
                    self.pixel_data[index..index + 4].copy_from_slice(color.as_ref());
                }
            }
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.pixel_data
    }
}
