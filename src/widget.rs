use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerSurface,
    shm::{Shm, slot::SlotPool},
};
use wayland_client::protocol::wl_keyboard;
use wayland_client::{QueueHandle, globals::GlobalList};

use super::Sometime;

pub struct Widget {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub exit: bool,
    pub pool: SlotPool,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
}

impl Widget {
    pub fn new(
        globals: &GlobalList,
        qh: &QueueHandle<Sometime>,
        shm: Shm,
        pool: SlotPool,
        layer: LayerSurface,
    ) -> Self {
        Self {
            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),
            shm,
            exit: false,
            pool,
            layer,
            keyboard: None,
        }
    }
}
