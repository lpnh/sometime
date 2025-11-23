use super::theme::Bgra;

pub struct CanvasPrimitives {
    pub side: i32,
    pub radius: f32,
    pub pixel_data: Vec<u8>,
}

impl CanvasPrimitives {
    pub fn new(side: i32) -> Self {
        let pixel_data = vec![0u8; (side * side * 4) as usize];
        Self {
            side,
            radius: (side / 2) as f32,
            pixel_data,
        }
    }

    pub fn clear(&mut self) {
        self.pixel_data.fill(0);
    }

    pub fn get_data(&self) -> &[u8] {
        &self.pixel_data
    }

    #[inline]
    pub fn pixel_idx(side: i32, x: i32, y: i32) -> usize {
        ((y * side + x) * 4) as usize
    }

    #[inline]
    pub fn squared_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        dx * dx + dy * dy
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Bgra) {
        let index = Self::pixel_idx(self.side, x, y);
        if index + 3 < self.pixel_data.len() {
            self.pixel_data[index..index + 4].copy_from_slice(color.as_ref());
        }
    }

    pub fn alpha_blending(pxl_data: &mut [u8], idx: usize, color: Bgra, alpha: u8) {
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

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Bgra) {
        for py in y..(y + height).min(self.side) {
            for px in x..(x + width).min(self.side) {
                if px >= 0 && py >= 0 && px < self.side && py < self.side {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }
}
