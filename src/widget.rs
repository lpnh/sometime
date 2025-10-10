use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerSurface,
    shm::{Shm, slot::SlotPool},
};
use wayland_client::protocol::wl_keyboard;

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
