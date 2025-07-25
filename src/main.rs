use std::{f32::consts::PI, num::NonZeroU32};

use chrono::Timelike;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
};

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);

    // Create a layer surface in the center
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay, // Above windows
        Some("niri_clock"),
        None,
    );

    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(400, 400);

    layer.commit();

    let pool = SlotPool::new(200 * 200 * 4, &shm).expect("Failed to create pool");

    let mut clock_widget = ClockWidget {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: 200,
        height: 200,
        layer,
        keyboard: None,
        keyboard_focus: false,
        visible: true,
    };

    // TODO: Improve this
    // Setup timer for clock updates
    std::thread::spawn({
        let _conn = conn.clone();
        move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    loop {
        event_queue.blocking_dispatch(&mut clock_widget).unwrap();

        if clock_widget.exit {
            println!("Exiting clock widget");
            break;
        }
    }
}

struct ClockWidget {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    visible: bool,
}

impl CompositorHandler for ClockWidget {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        if self.visible {
            self.draw_clock(qh);
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for ClockWidget {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for ClockWidget {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(200, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(200, NonZeroU32::get);

        if self.first_configure {
            self.first_configure = false;
            self.draw_clock(qh);
        }
    }
}

impl SeatHandler for ClockWidget {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for ClockWidget {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        match event.keysym {
            Keysym::Escape | Keysym::q => {
                self.exit = true;
            }
            _ => {}
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _layout: u32,
    ) {
    }
}

impl ShmHandler for ClockWidget {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ClockWidget {
    fn draw_clock(&mut self, qh: &QueueHandle<Self>) {
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

// Delegate implementations
delegate_compositor!(ClockWidget);
delegate_output!(ClockWidget);
delegate_shm!(ClockWidget);
delegate_seat!(ClockWidget);
delegate_keyboard!(ClockWidget);
delegate_layer!(ClockWidget);
delegate_registry!(ClockWidget);

impl ProvidesRegistryState for ClockWidget {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
