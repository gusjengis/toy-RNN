#!/usr/bin/env python3
"""Create normalized MNIST CSV files with pixel values scaled to 0..1."""

from __future__ import annotations

import csv
from pathlib import Path


CSV_DIR = Path(__file__).resolve().parents[1] / "data" / "processed" / "csv"
SOURCES = {
    "mnist_train.csv": "mnist_train_normalized.csv",
    "mnist_test.csv": "mnist_test_normalized.csv",
}


def normalize_csv(source: Path, destination: Path) -> int:
    row_count = 0
    with source.open(newline="") as source_file, destination.open("w", newline="") as dest_file:
        reader = csv.reader(source_file)
        writer = csv.writer(dest_file)

        header = next(reader)
        writer.writerow(header)

        for row in reader:
            label = row[0]
            pixels = [f"{int(pixel) / 255.0:.8f}" for pixel in row[1:]]
            writer.writerow([label, *pixels])
            row_count += 1

    return row_count


def main() -> None:
    for source_name, dest_name in SOURCES.items():
        source = CSV_DIR / source_name
        destination = CSV_DIR / dest_name
        rows = normalize_csv(source, destination)
        print(f"wrote {destination.relative_to(CSV_DIR.parents[2])} ({rows} rows)")


if __name__ == "__main__":
    main()
