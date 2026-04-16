pub mod backend;

const FEATURE_COUNT: usize =
    cfg!(feature = "opengl") as usize +
        cfg!(feature = "tui")    as usize +
        cfg!(feature = "egui-web")    as usize +
        cfg!(feature = "egui-desktop")    as usize;

const _: () = {
    assert!(FEATURE_COUNT == 1, "Exactly one feature must be enabled");
};

#[cfg(feature = "tui")]
mod crossterm;
#[cfg(feature = "tui")]
pub use crossterm::{BackendType, BACKEND_INSTANCE};

#[cfg(feature = "opengl")]
mod beamterm_native;
#[cfg(feature = "opengl")]
pub use beamterm_native::{BackendType, BACKEND_INSTANCE};

#[cfg(feature = "egui-desktop")]
mod egui_desktop;
#[cfg(feature = "egui-desktop")]
pub use egui_desktop::{BackendType, BACKEND_INSTANCE};

#[cfg(feature = "egui-web")]
mod egui_web;
#[cfg(feature = "egui-web")]
pub use egui_web::{BackendType, BACKEND_INSTANCE};
