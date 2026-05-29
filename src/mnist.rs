use std::{fs::File, io::Read, path::Path};

pub const IMAGE_WIDTH: usize = 28;
pub const IMAGE_HEIGHT: usize = 28;
pub const PIXEL_COUNT: usize = IMAGE_WIDTH * IMAGE_HEIGHT;
pub const CLASS_COUNT: usize = 10;

const MNIST_BIN_MAGIC: &[u8; 8] = b"MNISTBIN";
const MNIST_BIN_VERSION: u32 = 1;

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

pub fn load_train() -> MnistDataset {
    load_mnist_bin("data/mnist/train.mnistbin")
}

pub fn load_test() -> MnistDataset {
    load_mnist_bin("data/mnist/test.mnistbin")
}

pub fn load_mnist_bin(path: impl AsRef<Path>) -> MnistDataset {
    let mut file = File::open(path).expect("should be able to open MNIST binary data");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .expect("should be able to read MNIST binary data");

    let header_size = MNIST_BIN_MAGIC.len() + 16;
    assert!(
        bytes.len() >= header_size,
        "MNIST binary data should contain a complete header"
    );
    assert_eq!(
        &bytes[..MNIST_BIN_MAGIC.len()],
        MNIST_BIN_MAGIC,
        "MNIST binary data should have the expected magic bytes"
    );

    let version = read_u32_le(&bytes, 8);
    let count = read_u32_le(&bytes, 12) as usize;
    let width = read_u32_le(&bytes, 16) as usize;
    let height = read_u32_le(&bytes, 20) as usize;

    assert_eq!(
        version, MNIST_BIN_VERSION,
        "unsupported MNIST binary version"
    );
    assert_eq!(width, IMAGE_WIDTH, "unexpected MNIST image width");
    assert_eq!(height, IMAGE_HEIGHT, "unexpected MNIST image height");

    let labels_start = header_size;
    let pixels_start = labels_start + count;
    let expected_size = pixels_start + count * PIXEL_COUNT;
    assert_eq!(
        bytes.len(),
        expected_size,
        "MNIST binary data should match the header dimensions"
    );

    let labels = &bytes[labels_start..pixels_start];
    let pixels = &bytes[pixels_start..];
    let mut numbers = Vec::with_capacity(count);

    for (label, pixel_bytes) in labels.iter().zip(pixels.chunks_exact(PIXEL_COUNT)) {
        let pixels = pixel_bytes
            .iter()
            .map(|pixel| f32::from(*pixel) / 255.0)
            .collect();

        numbers.push(MnistNumber {
            label: *label,
            pixels,
        });
    }

    MnistDataset { numbers }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("MNIST binary header field should be four bytes"),
    )
}
