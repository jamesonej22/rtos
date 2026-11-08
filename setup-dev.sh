#!/usr/bin/env bash

set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$ROOT"

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

if ! command_exists curl; then
    echo "curl is required to install rustup and uv."
    echo "Install it with your system package manager, then run this script again."
    exit 1
fi

if ! command_exists rustup; then
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

PATH_LINE='export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"'
for profile in "$HOME/.profile" "$HOME/.bashrc"; do
    touch "$profile"
    if ! grep -Fqx "$PATH_LINE" "$profile"; then
        printf '\n# Added by RTOS setup-dev.sh\n%s\n' "$PATH_LINE" >> "$profile"
    fi
done

export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

if ! command_exists avr-gcc; then
    echo "avr-gcc is required for the morse firmware build."
    echo "On Fedora, install it with: sudo dnf install avr-gcc avr-libc"
    echo "On Raspberry Pi OS, install it with: sudo apt install gcc-avr avr-libc"
    exit 1
fi

echo "Installing the repository Rust toolchain..."
rustup toolchain install nightly-2026-09-18 --profile minimal \
    --component clippy --component rust-src --component rustfmt \
    --target thumbv7em-none-eabihf
rustup toolchain install stable --profile minimal

if ! command_exists taplo; then
    echo "Installing taplo-cli..."
    cargo +stable install taplo-cli --locked
else
    echo "taplo already installed."
fi

if ! command_exists uv; then
    echo "Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
else
    echo "uv already installed."
fi

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" == "x86_64" ]]; then
    if ! command_exists espup; then
        echo "Installing espup..."
        cargo +stable install espup --locked
    else
        echo "espup already installed."
    fi

    if [[ ! -f "$HOME/export-esp.sh" ]]; then
        echo "Installing the ESP Rust toolchain..."
        espup install
    else
        echo "ESP Rust toolchain export already exists."
    fi

    if ! command_exists espflash; then
        echo "Installing espflash..."
        cargo +stable install espflash --locked
    else
        echo "espflash already installed."
    fi
else
    echo "Skipping ESP toolchain setup on this ARM Linux host; use an x86_64 Linux host for the weather firmware."
fi

echo "Creating the shared Python environment..."
uv sync

echo "Enabling the repository pre-push hook..."
git config core.hooksPath .githooks

cat <<'EOF'

Development environment ready.

Python collector:
  uv run python propeller/collect.py
  uv run python temperature/collect.py

Rust checks:
  cargo fmt --all
  cargo build --workspace

ESP32-S3 weather project:
  source ~/export-esp.sh
  cd weather
  cargo build
EOF
