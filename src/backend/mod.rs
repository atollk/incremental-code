#![allow(clippy::all, unused_imports)]

pub mod backend;
pub mod events;
pub mod input;

const FEATURE_COUNT: usize = cfg!(feature = "opengl") as usize
    + cfg!(feature = "ratzilla") as usize
    + cfg!(feature = "tui") as usize;

const _: () = {
    assert!(FEATURE_COUNT == 1, "Exactly one feature must be enabled");
};

#[cfg(feature = "tui")]
mod crossterm;
#[cfg(feature = "tui")]
use crossterm::BACKEND_INSTANCE;
#[cfg(feature = "tui")]
pub use crossterm::{BackendType, StorageType};
#[cfg(any(feature = "tui", feature = "opengl"))]
mod store_native;

#[cfg(feature = "opengl")]
mod beamterm_native;
#[cfg(feature = "opengl")]
use beamterm_native::BACKEND_INSTANCE;
#[cfg(feature = "opengl")]
pub use beamterm_native::{BackendType, StorageType};

#[cfg(feature = "ratzilla")]
mod beamterm_web;
#[cfg(feature = "ratzilla")]
use beamterm_web::BACKEND_INSTANCE;
#[cfg(feature = "ratzilla")]
pub use beamterm_web::{BackendType, StorageType};

#[cfg(feature = "ratzilla")]
mod store_web;

pub fn with_backend<T>(
    f: impl FnOnce(&dyn backend::BackendSuite<BackendType, StorageType>) -> T,
) -> T {
    use std::ops::Deref;
    let lock = BACKEND_INSTANCE.read().unwrap();
    f(lock.deref())
}
