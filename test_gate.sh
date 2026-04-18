#!/bin/bash
cargo xtask gate > gate.log 2>&1
echo "Gate finished with $?"
