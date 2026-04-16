use crate::backend::backend::BackendSuite;
use beamterm_core::{FontAtlasData, GlState, GlslVersion, StaticFontAtlas, TerminalGrid};
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
use tachyonfx::Interpolation::*;
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

pub struct BeamtermCoreBackendSuite {}

impl BeamtermCoreBackendSuite {
    fn get_backend(event_loop: &ActiveEventLoop) -> BeamtermBackend {
        let builder = GlWindowBuilder::new(event_loop, "ratbeam demo", (1280, 800));
        let physical_size = builder.physical_size();
        let pixel_ratio = builder.pixel_ratio();
        let (win, gl_raw) = builder.build();
        let gl = Rc::new(gl_raw);
        let gl_state = GlState::new(&gl);

        let atlas_data = FontAtlasData::default();
        let atlas = StaticFontAtlas::load(&gl, atlas_data).expect("failed to load font atlas");

        let grid = TerminalGrid::new(
            &gl,
            atlas.into(),
            physical_size,
            pixel_ratio,
            &GlslVersion::Gl330,
        )
        .expect("failed to create terminal grid");

        let backend = BeamtermBackend::new(grid, gl.clone());
        backend
    }
}

impl BackendSuite<BeamtermBackend> for BeamtermCoreBackendSuite {
    fn run(&mut self, runner: impl FnMut(BeamtermBackend)) -> anyhow::Result<()> {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let mut app = BeamtermCoreApplicationHandler {
            backend: self,
            runner,
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

pub static BACKEND_INSTANCE: LazyLock<Mutex<BeamtermCoreBackendSuite>> =
    LazyLock::new(|| Mutex::new(BeamtermCoreBackendSuite {}));

struct BeamtermCoreApplicationHandler<'a, F: FnMut(BeamtermBackend)> {
    backend: &'a mut BeamtermCoreBackendSuite,
    runner: F,
}

impl<F: FnMut(BeamtermBackend)> ApplicationHandler for BeamtermCoreApplicationHandler<'_, F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        (self.runner)(BeamtermCoreBackendSuite::get_backend(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        todo!()
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

    fn physical_size(&self) -> (i32, i32) {
        let s = self.window.inner_size();
        (s.width as i32, s.height as i32)
    }

    fn pixel_ratio(&self) -> f32 {
        self.window.scale_factor() as f32
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
