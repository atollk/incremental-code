pub mod backend;
pub mod input;
pub mod events;

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

#[cfg(any(feature = "egui-desktop", feature = "egui-web"))]
pub mod egui;
#[cfg(any(feature = "egui-desktop", feature = "egui-web"))]
pub use egui::{BackendType, BACKEND_INSTANCE};

