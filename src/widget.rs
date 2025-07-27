use chrono::Timelike;
use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::{Shm, slot::SlotPool},
};
use std::f32::consts::PI;
use wayland_client::{
    QueueHandle,
    protocol::{wl_keyboard, wl_shm},
};

pub struct Widget {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub exit: bool,
    pub first_configure: bool,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub keyboard_focus: bool,
    pub visible: bool,
}

impl Widget {
    pub fn draw_clock(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // Get current time
        let now = chrono::Local::now();
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = (width.min(height) as f32 / 2.0) - 10.0;

        // Convert canvas to a Vec for easier manipulation
        let mut pixel_data: Vec<u8> = canvas.to_vec();

        // Clear canvas with transparent background
        for pixel in pixel_data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // BGRA format
        }

        // Draw clock face
        Self::draw_circle(
            &mut pixel_data,
            width,
            height,
            center_x,
            center_y,
            radius,
            [0xFF, 0xFF, 0xFF, 0xFF],
        );
        Self::draw_circle(
            &mut pixel_data,
            width,
            height,
            center_x,
            center_y,
            radius - 2.0,
            [0x20, 0x20, 0x20, 0xFF],
        );

        // Draw hour markers
        for hour in 0..12 {
            let angle = (hour as f32 * PI / 6.0) - PI / 2.0;
            let inner_radius = radius - 10.0;
            let outer_radius = radius - 5.0;

            let x1 = center_x + inner_radius * angle.cos();
            let y1 = center_y + inner_radius * angle.sin();
            let x2 = center_x + outer_radius * angle.cos();
            let y2 = center_y + outer_radius * angle.sin();

            Self::draw_line(
                &mut pixel_data,
                width,
                height,
                x1,
                y1,
                x2,
                y2,
                [0xFF, 0xFF, 0xFF, 0xFF],
            );
        }

        // Hour hand
        let hour_angle =
            ((now.hour() % 12) as f32 + now.minute() as f32 / 60.0) * PI / 6.0 - PI / 2.0;
        let hour_x = center_x + (radius * 0.5) * hour_angle.cos();
        let hour_y = center_y + (radius * 0.5) * hour_angle.sin();
        Self::draw_thick_line(
            &mut pixel_data,
            width,
            height,
            center_x,
            center_y,
            hour_x,
            hour_y,
            [0xFF, 0xFF, 0xFF, 0xFF],
            3,
        );

        // Minute hand
        let minute_angle = now.minute() as f32 * PI / 30.0 - PI / 2.0;
        let minute_x = center_x + (radius * 0.8) * minute_angle.cos();
        let minute_y = center_y + (radius * 0.8) * minute_angle.sin();
        Self::draw_thick_line(
            &mut pixel_data,
            width,
            height,
            center_x,
            center_y,
            minute_x,
            minute_y,
            [0xFF, 0xFF, 0xFF, 0xFF],
            2,
        );

        // Center dot
        Self::draw_circle(
            &mut pixel_data,
            width,
            height,
            center_x,
            center_y,
            5.0,
            [0xFF, 0xFF, 0xFF, 0xFF],
        );

        // Copy back to canvas
        canvas.copy_from_slice(&pixel_data);

        // Damage and present
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }

    // Helper drawing functions
    fn draw_circle(
        canvas: &mut [u8],
        width: u32,
        height: u32,
        cx: f32,
        cy: f32,
        radius: f32,
        color: [u8; 4],
    ) {
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= radius {
                    let index = ((y * width + x) * 4) as usize;
                    if index + 3 < canvas.len() {
                        canvas[index..index + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    fn draw_line(
        canvas: &mut [u8],
        width: u32,
        height: u32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [u8; 4],
    ) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let steps = dx.max(dy) as i32;

        if steps == 0 {
            return;
        }

        let x_inc = (x2 - x1) / steps as f32;
        let y_inc = (y2 - y1) / steps as f32;

        for i in 0..=steps {
            let x = (x1 + i as f32 * x_inc) as u32;
            let y = (y1 + i as f32 * y_inc) as u32;

            if x < width && y < height {
                let index = ((y * width + x) * 4) as usize;
                if index + 3 < canvas.len() {
                    canvas[index..index + 4].copy_from_slice(&color);
                }
            }
        }
    }

    fn draw_thick_line(
        canvas: &mut [u8],
        width: u32,
        height: u32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [u8; 4],
        thickness: i32,
    ) {
        for dx in -thickness / 2..=thickness / 2 {
            for dy in -thickness / 2..=thickness / 2 {
                Self::draw_line(
                    canvas,
                    width,
                    height,
                    x1 + dx as f32,
                    y1 + dy as f32,
                    x2 + dx as f32,
                    y2 + dy as f32,
                    color,
                );
            }
        }
    }
}
