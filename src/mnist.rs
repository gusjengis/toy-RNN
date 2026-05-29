use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub const IMAGE_WIDTH: usize = 28;
pub const IMAGE_HEIGHT: usize = 28;
pub const PIXEL_COUNT: usize = IMAGE_WIDTH * IMAGE_HEIGHT;
pub const CLASS_COUNT: usize = 10;

#[derive(Debug, Clone)]
pub struct MnistNumber {
    pub label: u8,
    pub pixels: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct MnistDataset {
    pub numbers: Vec<MnistNumber>,
}

impl MnistDataset {
    pub fn len(&self) -> usize {
        self.numbers.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MnistNumber> {
        self.numbers.iter()
    }
}

pub fn load_train_csv() -> MnistDataset {
    load_mnist_csv("data/csv/mnist_train_normalized.csv")
}

pub fn load_test_csv() -> MnistDataset {
    load_mnist_csv("data/csv/mnist_test_normalized.csv")
}

pub fn load_mnist_csv(path: impl AsRef<Path>) -> MnistDataset {
    let file = File::open(path).expect("should be able to open MNIST CSV");
    let mut lines = BufReader::new(file).lines();

    if let Some(header) = lines.next() {
        header.expect("should be able to read MNIST CSV header");
    }

    let mut examples = Vec::new();
    for line in lines {
        let line = line.expect("should be able to read MNIST CSV line");

        if line.trim().is_empty() {
            continue;
        }

        examples.push(parse_mnist_row(&line));
    }

    MnistDataset { numbers: examples }
}

fn parse_mnist_row(line: &str) -> MnistNumber {
    let mut values = line.split(',');
    let label = values
        .next()
        .expect("MNIST row should contain a label")
        .parse::<u8>()
        .expect("MNIST label should be a digit");

    let mut pixels = Vec::with_capacity(PIXEL_COUNT);
    for value in values {
        pixels.push(value.parse::<f32>().expect("MNIST pixel should be a float"));
    }

    MnistNumber { label, pixels }
}
