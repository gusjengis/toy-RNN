mod network;
mod neuron;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

fn main() {
    // training
    // load data
    // feed data to rnn
    // compute loss
    // backprop loss
    // repeat until total loss is low enough
    //
    // running
    // load weights
    // accept input
    // feed input to rnn
    // collect output
    // loop feeding and collecting for as many characters as I want to predict
    let file_path = "data/tinystories_sample.txt";

    // Reads the entire file into a String
    let content = fs::read_to_string(file_path).expect("Should have been able to read the file");

    let mut character_set: HashSet<char, _> = HashSet::new();
    for c in content.chars() {
        character_set.insert(c);
    }

    let mut character_vector = character_set.iter().collect::<Vec<_>>();
    character_vector.sort();
    println!("Character set: {:?}", character_vector);
    println!("One-Hot length: {:?}", character_vector.len());

    // set up map from character to one-hot input vector
    let mut one_hot_map: HashMap<char, Vec<f32>> = HashMap::new();
    for (i, c) in character_vector.iter().enumerate() {
        let character = *c;
        one_hot_map.insert(*character, vec![0.0; character_vector.len()]);
        one_hot_map.get_mut(c).unwrap()[i] = 1.0;
    }

    let mut predicted_char = 'a';
    // let mut prediction = String::from(predicted_char);

    let mut network = network::Network::new(vec![10, 10, 10, 10], character_vector.len());
    network.print_diagram(character_vector.len());
    println!("\n\n\n\n\n\n\n\n");
    for character in character_vector.iter() {
        let input = one_hot_map.get(&character).unwrap().as_slice();
        // println!("Input: {:?}", input);
        let mut output = network.feed_forward(input);
        // normalize output
        // let sum = output.iter().sum::<f32>();
        // output.iter_mut().for_each(|x| *x /= sum);

        // println!("Output: {:?}", output);

        // find max output
        let mut max = f32::MIN;
        let mut max_index = 0;
        for (i, output) in output.iter().enumerate() {
            if *output > max {
                max = *output;
                max_index = i;
            }
        }

        // println!(" max: {:?}", max);
        predicted_char = *character_vector[max_index];
        // prediction.push(predicted_char);
        println!("{} -> ({:?})", character, predicted_char);
    }
    // println!("Prediction: {:?}", prediction);
    // for character in character_vector.iter() {
    //     let input = one_hot_map.get(character).unwrap().as_slice();
    //     let output = network.feed_forward(input);
}
