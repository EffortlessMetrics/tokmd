#!/bin/bash
set -e

# Build release
cargo build --release --quiet

# Run tokmd run on the repo itself
echo "Running tokmd run..."
time ./target/release/tokmd run . --output-dir .runs/bench > /dev/null

echo "Done."
