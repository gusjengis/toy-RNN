pub mod pipeline;
pub mod render_loop;
pub mod window;

pub fn run() -> Result<(), winit::error::EventLoopError> {
    render_loop::run()
}
