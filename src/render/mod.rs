pub mod pipeline;
pub mod render_loop;
mod text;
pub mod window;

use crate::{mnist::MnistDataset, network::Network};

pub fn run(network: Network, inputs: MnistDataset) -> Result<(), winit::error::EventLoopError> {
    render_loop::run(network, inputs)
}
