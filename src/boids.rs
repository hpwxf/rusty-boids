use std::{error, fmt, process};

use gl;
use glutin::{
    self, Api, ContextBuilder, ContextError, CreationError, EventsLoop, GlContext, GlProfile,
    GlRequest, GlWindow, VirtualKeyCode, WindowBuilder,
};

use crate::fps::{FpsCache, FpsCounter};
use crate::glx;
use crate::render::{Renderer, RendererConfig};
use crate::system::{FlockingConfig, FlockingSystem};

const TITLE: &'static str = "rusty-boids";
const CACHE_FPS_MS: u64 = 500;

#[derive(Debug)]
pub enum SimulatorError {
    GlCreation(CreationError),
    GlContext(ContextError),
    Window(String),
}

impl fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimulatorError::GlCreation(ref err) => write!(f, "GL creation error, {}", err),
            SimulatorError::GlContext(ref err) => write!(f, "GL context error, {}", err),
            SimulatorError::Window(ref err) => write!(f, "Window error, {}", err),
        }
    }
}

impl error::Error for SimulatorError {
    fn description(&self) -> &str {
        match *self {
            SimulatorError::GlCreation(ref err) => err.description(),
            SimulatorError::GlContext(ref err) => err.description(),
            SimulatorError::Window(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SimulatorError::GlCreation(ref err) => Some(err),
            SimulatorError::GlContext(ref err) => Some(err),
            SimulatorError::Window(..) => None,
        }
    }
}

impl From<CreationError> for SimulatorError {
    fn from(err: CreationError) -> SimulatorError {
        SimulatorError::GlCreation(err)
    }
}

impl From<ContextError> for SimulatorError {
    fn from(err: ContextError) -> SimulatorError {
        SimulatorError::GlContext(err)
    }
}

impl SimulatorError {
    pub fn exit(&self) -> ! {
        println!("{}", self);
        process::exit(1);
    }
}

pub struct SimulationConfig {
    pub boid_count: u32,
    pub window_size: WindowSize,
    pub debug: bool,
    pub max_speed: f32,
    pub max_force: f32,
    pub mouse_weight: f32,
    pub sep_weight: f32,
    pub ali_weight: f32,
    pub coh_weight: f32,
    pub sep_radius: f32,
    pub ali_radius: f32,
    pub coh_radius: f32,
    pub boid_size: f32,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            boid_count: 1000,
            window_size: WindowSize::Dimensions((800, 800)),
            debug: false,
            max_speed: 2.5,
            max_force: 0.4,
            mouse_weight: 600.,
            sep_radius: 6.,
            ali_radius: 11.5,
            coh_radius: 11.5,
            sep_weight: 1.5,
            ali_weight: 1.0,
            coh_weight: 1.0,
            boid_size: 3.0,
        }
    }
}

fn build_configs(
    sim_config: &SimulationConfig,
    window: &GlWindow,
) -> Result<(FlockingConfig, RendererConfig), SimulatorError> {
    let hidpi = window.hidpi_factor();
    let (width, height) = window
        .get_inner_size()
        .map(|(w, h)| (hidpi * w as f32, hidpi * h as f32))
        .ok_or(SimulatorError::Window(
            "Tried to get size of closed window".to_string(),
        ))?;

    Ok((
        FlockingConfig {
            boid_count: sim_config.boid_count,
            width: width,
            height: height,
            max_speed: sim_config.max_speed,
            max_force: sim_config.max_force,
            mouse_weight: sim_config.mouse_weight,
            sep_weight: sim_config.sep_weight,
            ali_weight: sim_config.ali_weight,
            coh_weight: sim_config.coh_weight,
            sep_radius: sim_config.sep_radius,
            ali_radius: sim_config.ali_radius,
            coh_radius: sim_config.coh_radius,
        },
        RendererConfig {
            width: width,
            height: height,
            boid_size: sim_config.boid_size,
            max_speed: sim_config.max_speed,
        },
    ))
}

pub enum WindowSize {
    Fullscreen,
    Dimensions((u32, u32)),
}

pub fn run_simulation(config: SimulationConfig) -> Result<(), SimulatorError> {
    let mut events_loop = EventsLoop::new();
    let window = build_window(&events_loop, &config.window_size)?;
    gl_init(&window)?;
    if config.debug {
        print_debug_info(&window);
    }
    let (flock_conf, render_conf) = build_configs(&config, &window)?;
    let mut simulation = FlockingSystem::new(flock_conf);
    simulation.randomise();
    let renderer = Renderer::new(render_conf);
    renderer.init_pipeline();
    let mut fps_counter = FpsCounter::new();
    let mut fps_cacher = FpsCache::new(CACHE_FPS_MS);
    let mut running = true;
    while running {
        simulation.update();
        events_loop.poll_events(|e| match process_event(e) {
            Some(ControlEvent::Stop) => running = false,
            Some(ControlEvent::Key(k)) => handle_key(&mut simulation, k),
            Some(ControlEvent::MouseMove(x, y)) => simulation.set_mouse(x, y),
            Some(ControlEvent::MousePress) => simulation.enable_mouse_attraction(),
            Some(ControlEvent::MouseRelease) => simulation.enable_mouse_repulsion(),
            _ => (),
        });
        renderer.render(&simulation.boids());
        window.swap_buffers()?;
        fps_counter.tick();
        fps_cacher.poll(&fps_counter, |new_fps| {
            let title = format!("{} - {:02} fps", TITLE, new_fps);
            window.set_title(&title);
        });
    }
    Ok(())
}

fn handle_key(simulation: &mut FlockingSystem, key: VirtualKeyCode) {
    match key {
        VirtualKeyCode::R => simulation.randomise(),
        VirtualKeyCode::F => simulation.zeroise(),
        VirtualKeyCode::C => simulation.centralise(),
        _ => (),
    }
}

enum ControlEvent {
    Stop,
    Key(VirtualKeyCode),
    MouseMove(f32, f32),
    MousePress,
    MouseRelease,
}

fn process_event(event: glutin::Event) -> Option<ControlEvent> {
    match event {
        glutin::Event::WindowEvent { event: e, .. } => process_window_event(e),
        _ => None,
    }
}

fn process_window_event(event: glutin::WindowEvent) -> Option<ControlEvent> {
    use glutin::{ElementState, KeyboardInput, WindowEvent};
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(k),
                    ..
                },
            ..
        } => process_keypress(k),

        WindowEvent::CursorMoved {
            position: (x, y), ..
        } => Some(ControlEvent::MouseMove(x as f32, y as f32)),

        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            ..
        } => Some(ControlEvent::MousePress),

        WindowEvent::MouseInput {
            state: ElementState::Released,
            ..
        } => Some(ControlEvent::MouseRelease),

        WindowEvent::Closed => Some(ControlEvent::Stop),
        _ => None,
    }
}

fn process_keypress(key: VirtualKeyCode) -> Option<ControlEvent> {
    match key {
        VirtualKeyCode::Escape | VirtualKeyCode::Q => Some(ControlEvent::Stop),
        _ => Some(ControlEvent::Key(key)),
    }
}

fn build_window(
    events_loop: &EventsLoop,
    window_size: &WindowSize,
) -> Result<GlWindow, SimulatorError> {
    let window_builder = WindowBuilder::new().with_title(TITLE);
    let window_builder = match window_size {
        &WindowSize::Fullscreen => {
            let screen = Some(events_loop.get_primary_monitor());
            window_builder.with_fullscreen(screen)
        }
        &WindowSize::Dimensions((width, height)) => window_builder.with_dimensions(width, height),
    };

    let context_builder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true);

    Ok(GlWindow::new(window_builder, context_builder, events_loop)?)
}

fn gl_init(window: &GlWindow) -> Result<(), SimulatorError> {
    unsafe {
        window.make_current()?;
    }
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    Ok(())
}

fn print_debug_info(window: &GlWindow) {
    println!("Vendor: {}", glx::get_gl_str(gl::VENDOR));
    println!("Renderer: {}", glx::get_gl_str(gl::RENDERER));
    println!("Version: {}", glx::get_gl_str(gl::VERSION));
    println!(
        "GLSL version: {}",
        glx::get_gl_str(gl::SHADING_LANGUAGE_VERSION)
    );
    println!("Extensions: {}", glx::get_gl_extensions().join(","));
    println!("Hidpi factor: {}", window.hidpi_factor());
}
