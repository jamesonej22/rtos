#!/usr/bin/env bash

set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$ROOT"

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

if ! command_exists rustup; then
    echo "rustup is required. Install it from https://rustup.rs/ and run this script again."
    exit 1
fi

echo "Installing the repository Rust toolchain..."
rustup toolchain install nightly --profile minimal --component rust-src --target thumbv7em-none-eabihf
rustup toolchain install stable --profile minimal

if ! command_exists taplo; then
    echo "Installing taplo-cli..."
    cargo +stable install taplo-cli --locked
else
    echo "taplo already installed."
fi

if ! command_exists uv; then
    echo "uv is required. Install it from https://docs.astral.sh/uv/getting-started/installation/ and run this script again."
    exit 1
fi

echo "Creating the shared Python environment..."
uv sync

echo "Enabling the repository pre-push hook..."
git config core.hooksPath .githooks

cat <<'EOF'

Development environment ready.

Python collector:
  uv run python temperature/collect.py

Rust checks:
  cargo fmt --all
  cargo build --workspace
EOF