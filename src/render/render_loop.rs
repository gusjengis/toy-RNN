use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

use crate::network::Network;

use super::{pipeline::GpuState, window::AppWindow};

struct NetworkViewer<'network> {
    network: &'network Network,
    inputs: &'network [f32],
    character_labels: &'network [char],
    window: Option<AppWindow>,
    gpu: Option<GpuState>,
    is_panning: bool,
    last_cursor_position: Option<PhysicalPosition<f64>>,
}

impl<'network> NetworkViewer<'network> {
    fn new(
        network: &'network Network,
        inputs: &'network [f32],
        character_labels: &'network [char],
    ) -> Self {
        Self {
            network,
            inputs,
            character_labels,
            window: None,
            gpu: None,
            is_panning: false,
            last_cursor_position: None,
        }
    }
}

impl ApplicationHandler for NetworkViewer<'_> {
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

        let gpu = match pollster::block_on(GpuState::new(
            &window.window,
            window.size,
            self.network,
            self.inputs,
            self.character_labels,
        )) {
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
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.is_panning = state == ElementState::Pressed;
                if !self.is_panning {
                    self.last_cursor_position = None;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_panning {
                    if let (Some(last_position), Some(gpu)) =
                        (self.last_cursor_position, self.gpu.as_mut())
                    {
                        gpu.pan_by_screen_delta(
                            (position.x - last_position.x) as f32,
                            (position.y - last_position.y) as f32,
                        );
                    }
                    window.window.request_redraw();
                }
                self.last_cursor_position = Some(position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.01,
                };

                if let Some(gpu) = self.gpu.as_mut() {
                    let screen_position = self
                        .last_cursor_position
                        .map(|position| [position.x as f32, position.y as f32])
                        .unwrap_or([
                            window.size.width as f32 * 0.5,
                            window.size.height as f32 * 0.5,
                        ]);
                    gpu.zoom_at_screen_position(scroll_delta, screen_position);
                    window.window.request_redraw();
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

pub fn run(
    network: &Network,
    inputs: &[f32],
    character_labels: &[char],
) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = NetworkViewer::new(network, inputs, character_labels);
    event_loop.run_app(&mut app)
}
