#!/bin/bash
set -eux -o pipefail

cargo install --force rustfmt --vers 0.6

export PATH=$PATH:~/.cargo/bin &&
cargo fmt -- --write-mode=diff
cargo build
cargo test