pub mod pipeline;
pub mod render_loop;
pub mod window;

use crate::network::Network;

pub fn run(network: &Network, inputs: &[f32]) -> Result<(), winit::error::EventLoopError> {
    render_loop::run(network, inputs)
}
