# EN.605.715 RTOS Class Code

Rust code for the Johns Hopkins University real-time operating systems class (EN.605.715).

## Projects

- [`morse`](morse/): Morse code application for the Arduino Uno R3.
- [`morse-r4`](morse-r4/): Morse code application for the Arduino Uno R4 WiFi.
- [`arduino-uno-r4-hal`](arduino-uno-r4-hal/): Modified Arduino Uno R4 hardware abstraction layer used by `morse-r4`.

See each project README for build, flash, and serial console instructions.

## Development Setup

On a new laptop, install `rustup` and `uv`, clone this repository, and run:

```sh
./setup-dev.sh
```

The script installs the pinned nightly Rust toolchain and target, installs
`taplo` for repository checks, creates the shared root Python environment, and
enables the pre-push hook. The Python environment contains `pyserial` for the
temperature collector:

```sh
uv run python temperature/collect.py
```

The collector expects an Arduino serial device at `/dev/ttyACM0`. Change the
device path in `temperature/collect.py` when the laptop exposes a different
serial device.

## Pre-push Checks

The setup script enables the repository's pre-push hook. To enable it manually:

```sh
git config core.hooksPath .githooks
```

The hook checks Rust and TOML formatting, runs Clippy, and builds both firmware
projects for their respective targets. It requires `taplo` to be installed.
