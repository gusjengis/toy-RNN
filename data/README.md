# MNIST Processed Data

Generated from the original MNIST IDX gzip files. Source code was not modified.

## Contents

- `idx/`: decompressed original IDX byte files.
- `csv/mnist_train.csv`: 60,000 rows, one label plus 784 pixel columns per row.
- `csv/mnist_test.csv`: 10,000 rows, one label plus 784 pixel columns per row.
- `csv/mnist_train_normalized.csv`: same training rows with pixel values divided by 255.
- `csv/mnist_test_normalized.csv`: same test rows with pixel values divided by 255.
- `images/train/<digit>/`: training images as PNG files grouped by label.
- `images/test/<digit>/`: test images as PNG files grouped by label.
- `previews/`: contact-sheet PNGs showing the first 100 examples for each digit/split.
- `train_manifest.csv` and `test_manifest.csv`: index, label, and relative PNG path.

## Image Format

Each PNG is a 28x28 grayscale image. Pixel values are 0-255, where 0 is black and 255 is white.

## CSV Format

Each CSV row has this shape:

`label,pixel_0,pixel_1,...,pixel_783`

The 784 pixels are the flattened 28x28 image in row-major order.

The normalized CSVs use the same column layout, but each pixel is a floating-point value from 0.0 to 1.0.
