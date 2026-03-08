#!/bin/bash
set -e
sed -i 's/let mut s = String::new();/let mut s = String::with_capacity(1024);/' crates/tokmd-cockpit/src/render.rs
