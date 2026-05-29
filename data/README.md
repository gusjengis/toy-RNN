# MNIST Processed Data

Generated from the original MNIST IDX gzip files. Source code was not modified.

## Contents

- `mnist/train.mnistbin`: 60,000 training rows in compact binary form.
- `mnist/test.mnistbin`: 10,000 test rows in compact binary form.

## Binary Format

Each `.mnistbin` file stores raw 28x28 grayscale images. Pixel values are bytes from 0 to 255 and are normalized to `0.0..1.0` by the Rust loader.

Layout:

- 8 bytes: ASCII magic `MNISTBIN`
- 4 bytes: little-endian `u32` version, currently `1`
- 4 bytes: little-endian `u32` row count
- 4 bytes: little-endian `u32` image width, currently `28`
- 4 bytes: little-endian `u32` image height, currently `28`
- `count` bytes: labels
- `count * 784` bytes: flattened image pixels in row-major order
