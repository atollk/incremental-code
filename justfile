build-tui:
    cargo build --features tui

run-tui:
    cargo run --features tui

build-opengl:
    cargo build --features opengl

run-opengl:
    cargo run --features opengl

build-egui-desktop:
    cargo build --features egui-desktop

run-egui-desktop:
    cargo run --features egui-desktop

build-egui-web:
    cargo bin trunk build --features egui-web

run-egui-web:
    cargo bin trunk serve --features egui-web