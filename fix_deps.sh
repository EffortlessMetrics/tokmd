#!/bin/bash
cargo update -p rustls-webpki
cargo update -p winnow
cargo update -p windows_x86_64_msvc
cargo deny check
