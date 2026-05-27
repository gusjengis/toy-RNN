pub struct Neuron {
    weights: Vec<f32>,
    pub bias: f32,
    activation_function: ActivationFunction,
    pub output: f32,
}

impl Neuron {
    pub fn new(activation_function: ActivationFunction, input_size: usize) -> Self {
        let weights = (0..input_size)
            .map(|_| rand::random_range(-1.0..1.0))
            .collect();
        Self {
            weights,
            bias: rand::random_range(-1.0..1.0),
            activation_function,
            output: 0.0,
        }
    }

    pub fn compute_output(&mut self, inputs: &[f32]) {
        let mut acc = self.bias;
        for (i, input) in inputs.iter().enumerate() {
            acc += input * self.weights[i];
        }
        self.output = self.activation_function(acc);
    }

    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    fn activation_function(&self, x: f32) -> f32 {
        match self.activation_function {
            ActivationFunction::ReLU => ReLU(x),
            ActivationFunction::Raw => Raw(x),
        }
    }
}

pub enum ActivationFunction {
    ReLU,
    Raw,
}

fn ReLU(x: f32) -> f32 {
    if x > 0.0 {
        return x;
    }
    return 0.0;
}

fn Raw(x: f32) -> f32 {
    return x;
}
