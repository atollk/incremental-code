#[cfg(feature = "beamterm-native")]
mod beamterm_native;
mod crossterm;

enum Backend {
    Crossterm,
    BeamtermNative,
    BeamtermWeb,
}

fn main() {
    let backend = Backend::BeamtermNative;
    match backend {
        Backend::Crossterm => {
            crossterm::main().unwrap();
        }
        Backend::BeamtermNative => {
            beamterm_native::main().unwrap();
        }
        Backend::BeamtermWeb => {}
    }
}