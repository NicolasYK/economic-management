#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
