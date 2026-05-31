use crate::{mnist::MnistDataset, network::Network};

struct Training {
    training_data: MnistDataset,
    test_data: MnistDataset,
}
pub fn compute_loss(network: Network) {}

// need to manage training and compute,
// related state needs to persist between render calls,
// render calls will call exposed functions to trigger steps of compute/training frame by frame for
// animation
//
// training data
// current location in training data, to persist progress through training between render calls
// test data and location
// network
// current layer (feed forward)
// current layer (feed backward)
// result/output (probability distribution)
// loss (cross entropy)
// learning rate
// state machine
