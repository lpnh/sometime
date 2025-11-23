#[macro_export]
macro_rules! impl_registry_handlers {
    ($type:ty) => {
        use smithay_client_toolkit::{
            compositor::CompositorHandler,
            delegate_compositor, delegate_keyboard, delegate_layer, delegate_output,
            delegate_registry, delegate_seat, delegate_shm,
            output::{OutputHandler, OutputState},
            registry::{ProvidesRegistryState, RegistryState},
            registry_handlers,
            seat::{
                Capability, SeatHandler, SeatState,
                keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
            },
            shell::{
                WaylandSurface,
                wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
            },
            shm::{Shm, ShmHandler},
        };
        use wayland_client::{
            Connection, QueueHandle,
            protocol::{wl_keyboard, wl_output, wl_seat, wl_surface},
        };

        impl CompositorHandler for $type {
            fn scale_factor_changed(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_surface::WlSurface,
                _: i32,
            ) {
            }
            fn transform_changed(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_surface::WlSurface,
                _: wl_output::Transform,
            ) {
            }
            fn frame(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_surface::WlSurface,
                _: u32,
            ) {
            }
            fn surface_enter(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_surface::WlSurface,
                _: &wl_output::WlOutput,
            ) {
            }
            fn surface_leave(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_surface::WlSurface,
                _: &wl_output::WlOutput,
            ) {
            }
        }

        impl OutputHandler for $type {
            fn output_state(&mut self) -> &mut OutputState {
                &mut self.widget.output_state
            }
            fn new_output(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: wl_output::WlOutput,
            ) {
            }
            fn update_output(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: wl_output::WlOutput,
            ) {
            }
            fn output_destroyed(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: wl_output::WlOutput,
            ) {
            }
        }

        impl LayerShellHandler for $type {
            fn closed(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _layer: &LayerSurface,
            ) {
                self.widget.exit = true;
            }

            fn configure(
                &mut self,
                _conn: &Connection,
                _qh: &QueueHandle<Self>,
                _layer: &LayerSurface,
                _configure: LayerSurfaceConfigure,
                _serial: u32,
            ) {
                self.canvas.init(self.theme);
                self.draw();
            }
        }

        impl SeatHandler for $type {
            fn seat_state(&mut self) -> &mut SeatState {
                &mut self.widget.seat_state
            }

            fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

            fn new_capability(
                &mut self,
                _conn: &Connection,
                qh: &QueueHandle<Self>,
                seat: wl_seat::WlSeat,
                capability: Capability,
            ) {
                if capability == Capability::Keyboard && self.widget.keyboard.is_none() {
                    let keyboard = self
                        .widget
                        .seat_state
                        .get_keyboard(qh, &seat, None)
                        .expect("Failed to create keyboard");
                    self.widget.keyboard = Some(keyboard);
                }
            }

            fn remove_capability(
                &mut self,
                _conn: &Connection,
                _: &QueueHandle<Self>,
                _: wl_seat::WlSeat,
                capability: Capability,
            ) {
                if capability == Capability::Keyboard && self.widget.keyboard.is_some() {
                    self.widget.keyboard.take().unwrap().release();
                }
            }

            fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
        }

        impl KeyboardHandler for $type {
            fn enter(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                _: &wl_surface::WlSurface,
                _: u32,
                _: &[u32],
                _: &[Keysym],
            ) {
            }

            fn leave(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                surface: &wl_surface::WlSurface,
                _: u32,
            ) {
                if self.widget.layer.wl_surface() == surface {
                    self.widget.exit = true;
                }
            }

            fn press_key(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                _: u32,
                event: KeyEvent,
            ) {
                let pressed_key = event.keysym;

                // Exit on `esc` or `q`
                if pressed_key == Keysym::Escape || pressed_key == Keysym::q {
                    self.widget.exit = true;
                }
            }

            fn repeat_key(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                _: u32,
                _: KeyEvent,
            ) {
            }

            fn release_key(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                _: u32,
                _: KeyEvent,
            ) {
            }

            fn update_modifiers(
                &mut self,
                _: &Connection,
                _: &QueueHandle<Self>,
                _: &wl_keyboard::WlKeyboard,
                _: u32,
                _: Modifiers,
                _: RawModifiers,
                _: u32,
            ) {
            }
        }

        impl ShmHandler for $type {
            fn shm_state(&mut self) -> &mut Shm {
                &mut self.widget.shm
            }
        }

        delegate_compositor!($type);
        delegate_output!($type);
        delegate_shm!($type);
        delegate_seat!($type);
        delegate_keyboard!($type);
        delegate_layer!($type);
        delegate_registry!($type);

        impl ProvidesRegistryState for $type {
            fn registry(&mut self) -> &mut RegistryState {
                &mut self.widget.registry_state
            }
            registry_handlers![OutputState, SeatState];
        }
    };
}
