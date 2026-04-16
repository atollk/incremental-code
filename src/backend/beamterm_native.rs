use crate::backend::backend::{BackendSuite, TerminalApp};
use beamterm_core::{
    Drawable, FontAtlasData, GlState, GlslVersion, RenderContext, StaticFontAtlas, TerminalGrid,
};
use glutin::surface::GlSurface;
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, Version,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{Surface, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use ratbeam::BeamtermBackend;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};
use winit::{
    dpi::LogicalSize,
    window::{Window, WindowAttributes},
};
use winit::event::KeyEvent;
use crate::backend::events::{Event, IntoEvent};

pub static BACKEND_INSTANCE: LazyLock<Mutex<BeamtermCoreBackendSuite>> =
    LazyLock::new(|| Mutex::new(BeamtermCoreBackendSuite {}));

pub type BackendType = BeamtermBackend;

pub struct BeamtermCoreBackendSuite {}

impl BeamtermCoreBackendSuite {
    fn get_window_state(event_loop: &ActiveEventLoop) -> WindowState {
        let builder = GlWindowBuilder::new(event_loop, "ratbeam demo", (1280, 800));
        let (win, gl_raw) = builder.build();
        let gl = Rc::new(gl_raw);
        let gl_state = GlState::new(&gl);
        WindowState { win, gl, gl_state }
    }
}

impl BackendSuite<BackendType> for BeamtermCoreBackendSuite {
    fn run(&mut self, terminal_app: impl TerminalApp<BackendType>) -> anyhow::Result<()> {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let mut app = BeamtermCoreApplicationHandler {
            terminal_app,
            window_state: None,
             events: Vec::new(),
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

struct BeamtermCoreApplicationHandler<A: TerminalApp<BackendType>> {
    terminal_app: A,
    window_state: Option<WindowState>,
    events: Vec<Event>,
}

struct WindowState {
    win: GlWindow,
    gl: Rc<glow::Context>,
    gl_state: GlState,
}

impl WindowState {
    fn create_backend(&self) -> BackendType {
        let physical_size = self.win.window.inner_size();
        let pixel_ratio = self.win.window.scale_factor();
        let atlas_data = FontAtlasData::default();
        let atlas = StaticFontAtlas::load(&self.gl, atlas_data).expect("failed to load font atlas");
        let grid = TerminalGrid::new(
            &self.gl,
            atlas.into(),
            physical_size.into(),
            pixel_ratio as f32,
            &GlslVersion::Gl330,
        )
        .expect("failed to create terminal grid");
        BackendType::new(grid, self.gl.clone())
    }
}

impl<A: TerminalApp<BackendType>> ApplicationHandler for BeamtermCoreApplicationHandler<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_state = BeamtermCoreBackendSuite::get_window_state(event_loop);
        self.window_state = Some(window_state);
        let backend = self.window_state.as_ref().unwrap().create_backend();
        self.terminal_app.init(backend).unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.window_state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(event) = event.into_event() {
                    self.events.push(event);
                }
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    state.win.resize_surface(new_size);
                    let _ = self.terminal_app.backend_mut().grid_mut().resize(
                        &state.gl,
                        (new_size.width as i32, new_size.height as i32),
                        state.win.pixel_ratio(),
                    );
                    state.win.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let exit = self.terminal_app.frame(&self.events).expect("failed to draw");
                if exit {
                    event_loop.exit();
                }
                self.events.clear();

                // GL render
                let (w, h) = self.terminal_app.backend().grid().canvas_size();
                state.gl_state.viewport(&state.gl, 0, 0, w, h);
                state.gl_state.clear_color(&state.gl, 0.0, 0.0, 0.0, 1.0);

                unsafe {
                    use glow::HasContext;
                    state.gl.clear(glow::COLOR_BUFFER_BIT);
                }

                let mut ctx = RenderContext {
                    gl: &state.gl,
                    state: &mut state.gl_state,
                };
                let grid = self.terminal_app.backend().grid();
                grid.prepare(&mut ctx).expect("failed to prepare grid");
                grid.draw(&mut ctx);
                grid.cleanup(&mut ctx);

                state.win.swap_buffers();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.window_state.as_ref() {
            state.win.window.request_redraw();
        }
    }
}

impl IntoEvent for KeyEvent {
    fn into_event(self) -> Option<Event> {
        use crate::backend::input::{
            KeyCode, KeyEvent as IKeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
        };
        use winit::event::ElementState;
        use winit::keyboard::{Key, NamedKey};

        let code = match self.logical_key.as_ref() {
            Key::Named(named) => match named {
                NamedKey::Backspace => KeyCode::Backspace,
                NamedKey::Enter => KeyCode::Enter,
                NamedKey::ArrowLeft => KeyCode::Left,
                NamedKey::ArrowRight => KeyCode::Right,
                NamedKey::ArrowUp => KeyCode::Up,
                NamedKey::ArrowDown => KeyCode::Down,
                NamedKey::Home => KeyCode::Home,
                NamedKey::End => KeyCode::End,
                NamedKey::PageUp => KeyCode::PageUp,
                NamedKey::PageDown => KeyCode::PageDown,
                NamedKey::Tab => KeyCode::Tab,
                NamedKey::Delete => KeyCode::Delete,
                NamedKey::Insert => KeyCode::Insert,
                NamedKey::F1 => KeyCode::F(1),
                NamedKey::F2 => KeyCode::F(2),
                NamedKey::F3 => KeyCode::F(3),
                NamedKey::F4 => KeyCode::F(4),
                NamedKey::F5 => KeyCode::F(5),
                NamedKey::F6 => KeyCode::F(6),
                NamedKey::F7 => KeyCode::F(7),
                NamedKey::F8 => KeyCode::F(8),
                NamedKey::F9 => KeyCode::F(9),
                NamedKey::F10 => KeyCode::F(10),
                NamedKey::F11 => KeyCode::F(11),
                NamedKey::F12 => KeyCode::F(12),
                NamedKey::Escape => KeyCode::Esc,
                NamedKey::CapsLock => KeyCode::CapsLock,
                NamedKey::ScrollLock => KeyCode::ScrollLock,
                NamedKey::NumLock => KeyCode::NumLock,
                NamedKey::PrintScreen => KeyCode::PrintScreen,
                NamedKey::Pause => KeyCode::Pause,
                NamedKey::ContextMenu => KeyCode::Menu,
                _ => return None,
            },
            Key::Character(s) => {
                let mut chars = s.chars();
                let c = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }
                KeyCode::Char(c)
            }
            _ => return None,
        };

        let kind = match (self.state, self.repeat) {
            (ElementState::Pressed, true) => KeyEventKind::Repeat,
            (ElementState::Pressed, false) => KeyEventKind::Press,
            (ElementState::Released, _) => KeyEventKind::Release,
        };

        Some(Event::KeyEvent(IKeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        }))
    }
}

// ── GL window boilerplate ───────────────────────────────────────────

struct GlWindowBuilder {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
}

struct GlWindow {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
}

impl GlWindowBuilder {
    fn new(event_loop: &ActiveEventLoop, title: &str, size: (u32, u32)) -> Self {
        let window_attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(size.0, size.1));

        let config_template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs))
            .build(event_loop, config_template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .expect("failed to build display");

        let window = window.expect("failed to create window");
        let gl_display = gl_config.display();

        let context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(
                window
                    .window_handle()
                    .expect("failed to get window handle")
                    .into(),
            ));

        let not_current_context = unsafe { gl_display.create_context(&gl_config, &context_attrs) }
            .expect("failed to create GL context");

        let inner = window.inner_size();
        let surface_attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(
                window
                    .window_handle()
                    .expect("failed to get window handle")
                    .into(),
                NonZeroU32::new(inner.width).unwrap(),
                NonZeroU32::new(inner.height).unwrap(),
            );

        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attrs) }
            .expect("failed to create GL surface");

        let gl_context = not_current_context
            .make_current(&gl_surface)
            .expect("failed to make GL context current");

        let _ = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name| gl_display.get_proc_address(name))
        };

        Self {
            window,
            gl_context,
            gl_surface,
            gl,
        }
    }

    /// Splits into a GlWindow (for surface ops) and the glow context (for wrapping in Rc).
    fn build(self) -> (GlWindow, glow::Context) {
        let win = GlWindow {
            window: self.window,
            gl_context: self.gl_context,
            gl_surface: self.gl_surface,
        };
        (win, self.gl)
    }
}

impl GlWindow {
    fn pixel_ratio(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    fn resize_surface(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gl_surface.resize(
            &self.gl_context,
            NonZeroU32::new(new_size.width).unwrap(),
            NonZeroU32::new(new_size.height).unwrap(),
        );
    }

    fn swap_buffers(&self) {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .expect("failed to swap buffers");
    }
}
