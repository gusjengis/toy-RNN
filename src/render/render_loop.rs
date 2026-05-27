use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

use crate::network::Network;

use super::{pipeline::GpuState, window::AppWindow};

const DEFAULT_FRAMES_PER_ANIMATION_STEP: usize = 10;
const MIN_FRAMES_PER_ANIMATION_STEP: usize = 1;

struct NetworkViewer {
    network: Network,
    inputs: Vec<f32>,
    character_labels: Vec<char>,
    animation_step: usize,
    frames_per_animation_step: usize,
    frames_until_next_step: usize,
    window: Option<AppWindow>,
    gpu: Option<GpuState>,
    is_panning: bool,
    last_cursor_position: Option<PhysicalPosition<f64>>,
}

impl NetworkViewer {
    fn new(network: Network, inputs: Vec<f32>, character_labels: Vec<char>) -> Self {
        Self {
            network,
            inputs,
            character_labels,
            animation_step: 0,
            frames_per_animation_step: DEFAULT_FRAMES_PER_ANIMATION_STEP,
            frames_until_next_step: 0,
            window: None,
            gpu: None,
            is_panning: false,
            last_cursor_position: None,
        }
    }

    fn advance_animation(&mut self) {
        if self.animation_step > self.network.layer_count() {
            return;
        }

        if self.animation_step == 0 {
            self.network.clear_outputs();
        } else {
            self.network
                .compute_layer(self.animation_step - 1, &self.inputs);
        }

        self.animation_step += 1;
    }

    fn tick_animation(&mut self) {
        if self.animation_step > self.network.layer_count() {
            return;
        }

        if self.frames_until_next_step == 0 {
            self.advance_animation();
            self.frames_until_next_step = self.frames_per_animation_step.saturating_sub(1);
        } else {
            self.frames_until_next_step -= 1;
        }
    }

    fn increase_animation_delay(&mut self) {
        self.frames_per_animation_step += 1;
        eprintln!(
            "Animation delay: {} frames per step",
            self.frames_per_animation_step
        );
    }

    fn decrease_animation_delay(&mut self) {
        self.frames_per_animation_step = self
            .frames_per_animation_step
            .saturating_sub(1)
            .max(MIN_FRAMES_PER_ANIMATION_STEP);
        self.frames_until_next_step = self
            .frames_until_next_step
            .min(self.frames_per_animation_step.saturating_sub(1));
        eprintln!(
            "Animation delay: {} frames per step",
            self.frames_per_animation_step
        );
    }

    fn reset_with_random_input(&mut self) {
        if self.inputs.is_empty() {
            return;
        }

        self.inputs.fill(0.0);
        let input_index = rand::random_range(0..self.inputs.len());
        self.inputs[input_index] = 1.0;
        self.network.clear_outputs();
        self.animation_step = 0;
        self.frames_until_next_step = 0;
    }
}

impl ApplicationHandler for NetworkViewer {
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
            &self.network,
            &self.inputs,
            &self.character_labels,
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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Space | KeyCode::KeyR) => {
                            window.window.request_redraw();
                            self.reset_with_random_input();
                        }
                        PhysicalKey::Code(KeyCode::Equal | KeyCode::NumpadAdd) => {
                            self.increase_animation_delay();
                        }
                        PhysicalKey::Code(KeyCode::Minus | KeyCode::NumpadSubtract) => {
                            self.decrease_animation_delay();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let window_size = window.size;
                self.tick_animation();

                let Some(gpu) = self.gpu.as_mut() else {
                    return;
                };
                gpu.refresh_network(&self.network, &self.inputs, &self.character_labels);

                match gpu.render() {
                    Ok(()) => {}
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => gpu.resize(window_size),
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
    network: Network,
    inputs: Vec<f32>,
    character_labels: Vec<char>,
) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = NetworkViewer::new(network, inputs, character_labels);
    event_loop.run_app(&mut app)
}
