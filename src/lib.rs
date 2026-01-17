mod canvas;
mod canvas_primitives;
pub mod ipc;
mod registry;
mod sometime;
mod theme;
pub mod widget;

pub use canvas::Canvas;
pub use canvas_primitives::CanvasPrimitives;
pub use sometime::Sometime;
pub use theme::{Bgra, Theme};
pub use widget::Widget;

pub const SIDE: i32 = 448;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Hidden,
    Clock,
    Calendar,
}
