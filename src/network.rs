use crate::neuron::{ActivationFunction, Neuron};

pub struct Network {
    hidden_layers: Vec<Vec<Neuron>>,
    output_layer: Vec<Neuron>,
}

impl Network {
    pub fn new(input_size: usize, hidden_layer_sizes: Vec<usize>, output_size: usize) -> Self {
        let mut hidden_layers = Vec::new();
        let mut input_size = input_size;
        for i in 0..hidden_layer_sizes.len() {
            if i > 0 {
                input_size = hidden_layer_sizes[i - 1];
            }
            hidden_layers.push(
                (0..hidden_layer_sizes[i])
                    .map(|_| Neuron::new(ActivationFunction::ReLU, input_size))
                    .collect(),
            );
        }
        Self {
            hidden_layers,
            output_layer: (0..output_size)
                .map(|_| Neuron::new(ActivationFunction::Raw, *hidden_layer_sizes.last().unwrap()))
                .collect(),
        }
    }

    pub fn feed_forward(&mut self, inputs: &[f32]) -> Vec<f32> {
        let mut inputs = inputs.to_vec();
        for layer in self.hidden_layers.iter_mut() {
            // compute the output of each neuron in the current layer
            for neuron in layer.iter_mut() {
                neuron.compute_output(inputs.as_slice());
            }
            // set inputs to be the outputs of the current layer
            inputs = layer.iter().map(|n| n.output).collect();
        }
        // compute the output of the output layer
        for neuron in self.output_layer.iter_mut() {
            neuron.compute_output(inputs.as_slice());
        }

        // return the outputs of the output layer
        return self.output_layer.iter().map(|n| n.output).collect();
    }

    pub fn layer_count(&self) -> usize {
        self.hidden_layers.len() + 1
    }

    pub fn clear_outputs(&mut self) {
        for layer in self.hidden_layers.iter_mut() {
            for neuron in layer.iter_mut() {
                neuron.output = 0.0;
            }
        }

        for neuron in self.output_layer.iter_mut() {
            neuron.output = 0.0;
        }
    }

    pub fn compute_layer(&mut self, layer_index: usize, inputs: &[f32]) {
        if layer_index < self.hidden_layers.len() {
            let layer_inputs = if layer_index == 0 {
                inputs.to_vec()
            } else {
                self.hidden_layers[layer_index - 1]
                    .iter()
                    .map(|neuron| neuron.output)
                    .collect()
            };

            for neuron in self.hidden_layers[layer_index].iter_mut() {
                neuron.compute_output(&layer_inputs);
            }
            return;
        }

        if layer_index == self.hidden_layers.len() {
            let layer_inputs = self
                .hidden_layers
                .last()
                .map(|layer| layer.iter().map(|neuron| neuron.output).collect::<Vec<_>>())
                .unwrap_or_else(|| inputs.to_vec());

            for neuron in self.output_layer.iter_mut() {
                neuron.compute_output(&layer_inputs);
            }
        }
    }

    pub fn neuron_layers(&self) -> impl Iterator<Item = &[Neuron]> {
        self.hidden_layers
            .iter()
            .map(Vec::as_slice)
            .chain(std::iter::once(self.output_layer.as_slice()))
    }
}
