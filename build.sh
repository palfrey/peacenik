#!/bin/bash
set -eux -o pipefail

cargo fmt --version || rustup component add rustfmt
cargo fmt -- --check
cargo build
cargo test