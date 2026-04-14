#[cfg(feature = "opengl")]
mod beamterm_native;
#[cfg(feature = "tui")]
mod crossterm;
#[cfg(any(feature = "egui-web", feature = "egui-desktop"))]
mod egui;

const FEATURE_COUNT: usize =
    cfg!(feature = "opengl") as usize +
        cfg!(feature = "tui")    as usize +
        cfg!(feature = "egui-web")    as usize +
        cfg!(feature = "egui-desktop")    as usize;

const _: () = {
    assert!(FEATURE_COUNT == 1, "Exactly one feature must be enabled");
};

fn main() {
    #[cfg(feature = "opengl")]
    beamterm_native::main().unwrap();

    #[cfg(feature = "tui")]
    crossterm::main().unwrap();

    #[cfg(feature = "egui-web")]
    egui::main_web();

    #[cfg(feature = "egui-desktop")]
    egui::main_desktop().unwrap();
}