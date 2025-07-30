use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_i32(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    // Create a point at a distance and angle from the current point
    pub fn with_radius_and_angle(self, distance: f32, angle: f32) -> Self {
        Self {
            x: self.x + distance * angle.cos(),
            y: self.y + distance * angle.sin(),
        }
    }

    // Distance from this point to another
    pub fn distance_to(self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    // Check if point is within canvas bounds
    pub fn is_valid(self, side: i32) -> bool {
        self.x >= 0.0 && self.x < side as f32 && self.y >= 0.0 && self.y < side as f32
    }

    // Convert to pixel coordinates
    pub fn as_coords(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    // Offset by integer amounts
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx as f32,
            y: self.y + dy as f32,
        }
    }
}

pub struct Angle;

// Angle in radians starting from the top (12 o'clock)
impl Angle {
    pub fn hour(hour: u32, minute: u32) -> f32 {
        ((hour % 12) as f32 + minute as f32 / 60.0) * PI / 6.0 - PI / 2.0
    }

    pub fn minute(minute: u32) -> f32 {
        minute as f32 * PI / 30.0 - PI / 2.0
    }

    pub fn second(second: u32) -> f32 {
        second as f32 * PI / 30.0 - PI / 2.0
    }
}
