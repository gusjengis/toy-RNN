use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

use super::{pipeline::GpuState, window::AppWindow};

#[derive(Default)]
struct TriangleApp {
    window: Option<AppWindow>,
    gpu: Option<GpuState>,
}

impl ApplicationHandler for TriangleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match AppWindow::new(event_loop) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        let gpu = match pollster::block_on(GpuState::new(&window.window, window.size)) {
            Ok(gpu) => gpu,
            Err(error) => {
                eprintln!("Failed to initialize GPU renderer: {error}");
                event_loop.exit();
                return;
            }
        };

        self.gpu = Some(gpu);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_mut() else {
            return;
        };
        if window_id != window.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                window.resize(size);
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(gpu) = self.gpu.as_mut() else {
                    return;
                };

                match gpu.render() {
                    Ok(()) => {}
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => gpu.resize(window.size),
                    Err(SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(error) => eprintln!("Render error: {error}"),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.window.request_redraw();
        }
    }
}

pub fn run() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = TriangleApp::default();
    event_loop.run_app(&mut app)
}
