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

    pub fn print_diagram(&self, input_size: usize) {
        println!("Network");
        println!("=======");
        println!();

        let mut columns: Vec<(String, Vec<String>)> = Vec::new();
        columns.push((
            "Input Layer".to_string(),
            (0..input_size).map(|index| format!("x{}", index)).collect(),
        ));

        for (layer_index, layer) in self.hidden_layers.iter().enumerate() {
            columns.push((
                format!("Hidden Layer {}", layer_index),
                layer
                    .iter()
                    .enumerate()
                    .map(|(neuron_index, neuron)| {
                        format!("n{} out={:.4}", neuron_index, neuron.output)
                    })
                    .collect(),
            ));
        }

        columns.push((
            "Output Layer".to_string(),
            self.output_layer
                .iter()
                .enumerate()
                .map(|(neuron_index, neuron)| format!("n{} out={:.4}", neuron_index, neuron.output))
                .collect(),
        ));

        let column_width = columns
            .iter()
            .flat_map(|(header, rows)| {
                rows.iter()
                    .map(|row| row.len())
                    .chain(std::iter::once(header.len()))
            })
            .max()
            .unwrap_or(0)
            + 4;
        let row_count = columns
            .iter()
            .map(|(_, rows)| rows.len())
            .max()
            .unwrap_or(0);

        for (header, _) in columns.iter() {
            print!("{:<width$}", header, width = column_width);
        }
        println!();

        for (header, _) in columns.iter() {
            print!("{:-<width$}", "", width = header.len());
            print!("{:width$}", "", width = column_width - header.len());
        }
        println!();

        for row_index in 0..row_count {
            for (_, rows) in columns.iter() {
                let row = rows.get(row_index).map(String::as_str).unwrap_or("");
                print!("{:<width$}", row, width = column_width);
            }
            println!();
        }
    }
}
