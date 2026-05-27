use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

pub struct AppWindow {
    pub window: Window,
    pub size: PhysicalSize<u32>,
}

impl AppWindow {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, winit::error::OsError> {
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_title("Toy RNN Visualizer")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        )?;
        let size = window.inner_size();

        Ok(Self { window, size })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }
}
