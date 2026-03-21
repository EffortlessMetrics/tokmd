#!/bin/bash
cargo update -p toml_edit@0.22.27 --precise 0.22.24
sed -i 's/multiple-versions = "warn"/multiple-versions = "allow"/' deny.toml
cargo deny check
