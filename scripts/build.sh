#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
if [[ "${1:-}" == "windows" ]]; then
    rustup target add x86_64-pc-windows-gnu
    cargo build --release --locked --target x86_64-pc-windows-gnu
else
    cargo build --release --locked
fi
