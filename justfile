build-tui:
    cargo build --features tui

run-tui:
    cargo run --features tui

build-opengl:
    cargo build --features opengl

run-opengl:
    cargo run --features opengl

build-web:
    cargo bin trunk build --features web

run-web:
    cargo bin trunk serve --features web