# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

A `justfile` provides common tasks (requires `just`):

```bash
just run-tui           # Run TUI variant (most common during development)
just build-all         # Build all backends in parallel
```

Direct cargo commands always require a backend feature flag:

```bash
cargo check --features tui                  # Fast type-check
cargo build --features tui                  # Build
cargo test --features tui                   # Run all tests
cargo test --features tui test_name         # Run a single test
cargo clippy --features tui -- -D warnings  # Lint (CI mode)
cargo fmt --all -- --check                  # Format check (CI mode)
```

Web builds use trunk targeting `wasm32-unknown-unknown` (see justfile for exact commands).

## Architecture

This is a Cargo workspace with two crates:
- **Root crate `incremental-code`**: The game/UI layer
- **`language/`**: Interpreter for "NotPython", a Python-like scripting language

### Multi-Backend System

The app supports multiple rendering backends selected at compile time via mutually exclusive feature flags: `tui` (crossterm), `opengl` (beamterm native), `ratzilla` (beamterm wasm), `egui-desktop`, `egui-web`. Only one backend may be active per build.

`BackendSuite` (`src/backend/`) abstracts over backends. `BasicTerminalApp<A>` (`src/basic_terminal_app.rs`) wraps any `App` impl and drives the frame loop.

### Scene System

`src/game_scenes/` implements a scene-switching pattern. `SceneGame` owns the active `Box<dyn Scene>` and calls `frame()` each tick, receiving a `SceneSwitch` return value (stay, exit, or transition). Time delta is tracked per-frame for animations.

Current scenes: `HomeTerminalScene` (command-prompt UI) and `CodeEditorScene` (full code editor).

### Global Game State

`src/game_state.rs` holds a `LazyLock<Mutex<GameState>>`. All access goes through `with_game_state(|state| { ... })`. The state stores the program source code and compiled output.

### Widget Layer

`src/widgets/` contains reusable UI components:
- **Terminal widget**: Shell-like command runner with scrollable history and pluggable `RunningCommand` impls
- **Code editor widget**: Rich editor backed by `ropey`; supports mouse, text selection, undo/redo, and syntax highlighting

Syntax highlighting themes live in `src/widgets/code_editor/not_python_logos.rs` (NotPython), `python_logos.rs`, and `rust_logos.rs`. `not_python_logos.rs` owns `not_python_language()` and `not_python_default_theme()` — these depend on widget types and cannot live in the `language` crate.

### Language Crate (`language/`)

Three-stage pipeline:
1. **Lexer** (`language/src/lexer.rs`): `logos`-based tokenizer → `NotPythonLangToken`
2. **Parser** (`language/src/parser.rs`): `chumsky` combinator parser → `NotPythonProgram` AST
3. **Compiler** (`language/src/compile.rs`): AST interpreter → `CompiledProgram`

Public API: `parse_program()`, `compile()`, `NotPythonLangToken`.

The language crate is a separate workspace member specifically to cache chumsky/Logos LLVM codegen: touching only game-layer files does not recompile the parser/lexer.
