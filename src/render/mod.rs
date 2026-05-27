pub mod pipeline;
pub mod render_loop;
mod text;
pub mod window;

use crate::network::Network;

pub fn run(
    network: &Network,
    inputs: &[f32],
    character_labels: &[char],
) -> Result<(), winit::error::EventLoopError> {
    render_loop::run(network, inputs, character_labels)
}
