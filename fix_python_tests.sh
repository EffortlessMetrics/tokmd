#!/bin/bash
export RUSTFLAGS="-C debuginfo=0"
cargo test -p tokmd-python
