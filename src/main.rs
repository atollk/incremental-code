#[cfg(feature = "opengl")]
mod beamterm_native;
#[cfg(feature = "tui")]
mod crossterm;
#[cfg(feature = "web")]
mod egui;

const FEATURE_COUNT: usize =
    cfg!(feature = "opengl") as usize +
        cfg!(feature = "tui")    as usize +
        cfg!(feature = "web")    as usize;

const _: () = {
    assert!(FEATURE_COUNT <= 1, "Only one backend feature may be enabled at a time");
    assert!(FEATURE_COUNT >= 1, "At least one backend feature must be enabled");
};

fn main() {
    #[cfg(feature = "opengl")]
    beamterm_native::main().unwrap();

    #[cfg(feature = "tui")]
    crossterm::main().unwrap();

    #[cfg(feature = "web")]
    egui::main().unwrap();
}