#![allow(clippy::all, unused_imports)]

pub mod backend;
pub mod events;
pub mod input;

const FEATURE_COUNT: usize = cfg!(feature = "opengl") as usize
    + cfg!(feature = "ratzilla") as usize
    + cfg!(feature = "tui") as usize
    + cfg!(feature = "egui-web") as usize
    + cfg!(feature = "egui-desktop") as usize;

const _: () = {
    assert!(FEATURE_COUNT == 1, "Exactly one feature must be enabled");
};

#[cfg(feature = "tui")]
mod crossterm;
#[cfg(feature = "tui")]
pub use crossterm::{BACKEND_INSTANCE, BackendType};

#[cfg(feature = "opengl")]
mod beamterm_native;
#[cfg(feature = "opengl")]
pub use beamterm_native::{BACKEND_INSTANCE, BackendType};

#[cfg(feature = "ratzilla")]
mod beamterm_web;
#[cfg(feature = "ratzilla")]
pub use beamterm_web::{BACKEND_INSTANCE, BackendType};

#[cfg(any(feature = "egui-desktop", feature = "egui-web"))]
pub mod egui;
#[cfg(any(feature = "egui-desktop", feature = "egui-web"))]
pub use egui::{BACKEND_INSTANCE, BackendType};
