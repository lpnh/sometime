use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, Layer, LayerShell, LayerSurface},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_keyboard::WlKeyboard};

use crate::{SIDE, Sometime};

pub struct Wayland {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub pool: SlotPool,
    pub shm: Shm,
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub layer: Option<LayerSurface>,
    pub keyboard: Option<WlKeyboard>,
    pub exit: bool,
}

impl Wayland {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Sometime>) -> anyhow::Result<Self> {
        let shm = Shm::bind(globals, qh)?;

        Ok(Self {
            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),
            pool: SlotPool::new((SIDE * SIDE * 4) as usize, &shm)?,
            shm,
            compositor: CompositorState::bind(globals, qh)?,
            layer_shell: LayerShell::bind(globals, qh)?,
            layer: None,
            keyboard: None,
            exit: false,
        })
    }

    pub fn destroy_layer(&mut self) {
        if let Some(layer) = self.layer.take() {
            layer.wl_surface().destroy();
        }
    }

    pub fn create_layer_surface(&mut self, qh: &QueueHandle<Sometime>, namespace: &str) {
        let surface = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Overlay,
            Some(namespace),
            None,
        );
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_size(SIDE as u32, SIDE as u32);
        layer.commit();
        self.layer = Some(layer);
    }
}
