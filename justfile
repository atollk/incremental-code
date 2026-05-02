check:
    cargo check --features tui
    cargo check --features opengl
    cargo check --features ratzilla --target wasm32-unknown-unknown

clippy:
    cargo clippy --fix --features tui

clippy-all:
    cargo clippy --fix --features tui
    cargo clippy --fix --features opengl --allow-dirty
    cargo clippy --fix --features ratzilla --target wasm32-unknown-unknown --allow-dirty

test:
    cargo test --features tui

[parallel]
build-all: build-tui build-opengl build-ratzilla build-egui-desktop build-egui-web

build-tui:
    cargo build --features tui

run-tui:
    cargo run --features tui

build-opengl:
    cargo build --features opengl

run-opengl:
    cargo run --features opengl

build-ratzilla:
    cargo bin trunk build --features ratzilla

run-ratzilla:
    cargo bin trunk serve --features ratzilla

build-egui-desktop:
    cargo build --features egui-desktop

run-egui-desktop:
    cargo run --features egui-desktop

build-egui-web:
    cargo bin trunk build --features egui-web

run-egui-web:
    cargo bin trunk serve --features egui-web