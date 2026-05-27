mod network;
mod neuron;
mod render;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::network::Network;

fn main() {
    if let Err(error) = render::run() {
        eprintln!("Renderer exited with an error: {error}");
    }
}

#[allow(dead_code)]
fn run_network_demo() {
    let data = get_data("data/tinystories_sample.txt");
    let character_set = get_character_set(&data);
    let mut character_vector = character_set.iter().collect::<Vec<_>>();
    character_vector.sort();
    // set up map from character to one-hot input vector
    let one_hot_length = character_vector.len();
    let one_hot_map = get_one_hot_map(&character_vector);
    // initialize network
    let mut network = network::Network::new(one_hot_length, vec![10, 10, 10, 10], one_hot_length);
    // print diagram
    // network.print_diagram(character_vector.len());
    println!("\n");
    for character in character_vector.iter() {
        let input = one_hot_map.get(&character).unwrap().as_slice();
        // println!("Input: {:?}", input);
        let mut output = network.feed_forward(input);
        // convert output to a probability distribution
        let output = softmax(&output);

        // find max output
        let mut max = f32::MIN;
        let mut max_index = 0;
        for (i, output) in output.iter().enumerate() {
            if *output > max {
                max = *output;
                max_index = i;
            }
        }

        // this is wrong, don't have a target, need to set up training first
        let loss = one_hot_cross_entropy(output[max_index]);

        let predicted_char = *character_vector[max_index];
        println!("{} -> ({:?})", character, predicted_char);
    }
    // println!("Prediction: {:?}", prediction);
    // for character in character_vector.iter() {
    //     let input = one_hot_map.get(character).unwrap().as_slice();
    //     let output = network.feed_forward(input);
}

fn get_character_set(data: &str) -> HashSet<char> {
    let mut character_set: HashSet<char, _> = HashSet::new();
    for c in data.chars() {
        character_set.insert(c);
    }
    return character_set;
}

fn get_data(file_path: &str) -> String {
    fs::read_to_string(file_path).expect("Should have been able to read the file")
}

fn get_one_hot_map(character_vector: &Vec<&char>) -> HashMap<char, Vec<f32>> {
    let mut one_hot_map: HashMap<char, Vec<f32>> = HashMap::new();
    for (i, c) in character_vector.iter().enumerate() {
        let character = *c;
        one_hot_map.insert(*character, vec![0.0; character_vector.len()]);
        one_hot_map.get_mut(c).unwrap()[i] = 1.0;
    }
    return one_hot_map;
}

fn training(
    data: String,
    network: &mut Network,
    character_vector: &[char],
    character_set: &HashSet<char>,
    one_hot_map: &HashMap<char, Vec<f32>>,
) {
}
fn softmax(x: &[f32]) -> Vec<f32> {
    let mut exp_x = Vec::new();
    for i in x.iter() {
        exp_x.push(f32::exp(*i));
    }
    let sum = exp_x.iter().sum::<f32>();
    exp_x.iter_mut().for_each(|x| *x /= sum);
    return exp_x;
}

fn one_hot_cross_entropy(target_output: f32) -> f32 {
    -1.0 * target_output.ln()
}

fn cross_entropy(output: &[f32], target: &[f32]) -> f32 {
    let mut loss = 0.0;
    for (output, target) in output.iter().zip(target.iter()) {
        loss += -target * output.ln();
    }
    return loss;
}
