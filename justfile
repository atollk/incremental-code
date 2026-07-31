check-tui:
    cargo check --workspace --features tui

check-ratzilla:
    cargo check --workspace --features ratzilla --target wasm32-unknown-unknown

check-all:
    cargo check --workspace --features tui
    cargo check --workspace --features opengl
    cargo check --workspace --features ratzilla --target wasm32-unknown-unknown

clippy:
    cargo clippy --workspace --fix --features tui -- -D warnings

clippy-all:
    cargo clippy --workspace --fix --features tui -- -D warnings
    cargo clippy --workspace --fix --features opengl --allow-dirty -- -D warnings
    cargo clippy --workspace --fix --features ratzilla --target wasm32-unknown-unknown --allow-dirty -- -D warnings

test:
    cargo test --workspace --features tui

profile-opengl:
    #!/usr/bin/env bash
    set -x
    cargo build --profile profiling --features opengl
    BIN_PATH="./target/profiling/incremental-code"
    if [[ $(uname -a) == *"Linux"* ]]; then
      echo '-1' | sudo tee /proc/sys/kernel/perf_event_paranoid
    fi
    cargo bin samply record "$BIN_PATH"

bench_test:
    #!/usr/bin/env bash
    set -x
    export TEST_NAME=stage_e_has_print_len
    BUILD_OUTPUT=$(cargo test --profile profiling --features tui --no-run 2>&1)
    BIN_PATH=$(echo "$BUILD_OUTPUT" | sed -n 's/.*(\(.*\))/\1/p' | tail -n1)
    if [[ $(uname -a) == *"Linux"* ]]; then
      echo '-1' | sudo tee /proc/sys/kernel/perf_event_paranoid
    fi
    cargo bin samply record "$BIN_PATH" "$TEST_NAME"

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