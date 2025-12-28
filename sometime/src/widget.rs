use smithay_client_toolkit::{
    globals::GlobalData,
    output::{OutputData, OutputState},
    reexports::protocols::xdg::xdg_output::zv1::client::{
        zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1::ZxdgOutputV1,
    },
    registry::RegistryState,
    seat::{SeatData, SeatState},
    shell::wlr_layer::LayerSurface,
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{
    Dispatch, QueueHandle,
    globals::GlobalList,
    protocol::{wl_keyboard::WlKeyboard, wl_output::WlOutput, wl_seat::WlSeat},
};

pub struct Widget {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub exit: bool,
    pub pool: SlotPool,
    pub layer: LayerSurface,
    pub keyboard: Option<WlKeyboard>,
}

impl Widget {
    pub fn new<T>(
        globals: &GlobalList,
        qh: &QueueHandle<T>,
        shm: Shm,
        pool: SlotPool,
        layer: LayerSurface,
    ) -> Self
    where
        T: Dispatch<WlOutput, OutputData> + 'static,
        T: Dispatch<WlSeat, SeatData> + 'static,
        T: Dispatch<ZxdgOutputManagerV1, GlobalData> + 'static,
        T: Dispatch<ZxdgOutputV1, OutputData> + 'static,
    {
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
