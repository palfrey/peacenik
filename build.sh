#!/bin/bash
set -eux -o pipefail

cargo fmt -- --check
cargo build
cargo test