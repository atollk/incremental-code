pub mod backend;

#[cfg(feature = "tui")]
mod crossterm;

#[cfg(feature = "tui")]
pub use crossterm::BACKEND_INSTANCE;

#[cfg(feature = "opengl")]
mod beamterm_native;

#[cfg(feature = "opengl")]
pub use beamterm_native::BACKEND_INSTANCE;

#[cfg(feature = "egui-desktop")]
mod egui_desktop;

#[cfg(feature = "egui-desktop")]
pub use egui_desktop::BACKEND_INSTANCE;
