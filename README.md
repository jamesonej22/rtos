# EN.605.715 RTOS Class Code

Rust firmware and tools for the Johns Hopkins University real-time operating
systems class (EN.605.715). The repository is a Cargo workspace containing
projects for the Arduino Uno R3, Arduino Uno R4 WiFi, and ESP32-S3.

## Projects

- [`morse`](morse/): Morse code LED application for the Arduino Uno R3.
- [`morse-r4`](morse-r4/): Morse code LED application for the Arduino Uno R4 WiFi.
- [`arduino-uno-r4-hal`](arduino-uno-r4-hal/): Class-modified HAL for the Uno R4 Minima and WiFi boards, used by `morse-r4`.
- [`propeller`](propeller/): Uno R3 firmware to measure propeller RPM with an IR beam-break sensor, plus a live data collector and plotter.
- [`temperature`](temperature/): Uno R3 temperature logger, plus a serial data collector and plotter.
- [`weather`](weather/): ESP32-S3 Wi-Fi weather display with an OLED and local DHT11 sensor.

See each project README for hardware connections, project-specific
prerequisites, build and upload commands, and data collection instructions.

## Development Setup

For the Uno R3 projects, install `rustup`, `uv`, and the AVR GNU toolchain,
clone this repository, then run:

On Fedora, install the AVR toolchain with:

```sh
sudo dnf install avr-gcc avr-libc
```

```sh
./setup-dev.sh
```

The script installs the pinned nightly Rust toolchain and the Uno R4 target,
installs `taplo`, `espup`, and `espflash`, sets up the ESP Rust toolchain,
creates the shared root Python environment, and enables the pre-push hook. The
Python environment includes `pyserial` and `matplotlib`, used by the data
collectors:

```sh
uv run python propeller/collect.py
uv run python temperature/collect.py
```

Connect the relevant board and close its firmware serial monitor before
starting a collector. The collectors default to `/dev/ttyACM0`; see each
project's README for its collection behavior and how to change the serial
device.

For the ESP32-S3 weather project, source `~/export-esp.sh` in each build shell.
Create `weather/.env` with the required Wi-Fi and OpenWeather settings. See
[weather/README.md](weather/README.md) for details.

## Pre-push Checks

The setup script enables the repository's pre-push hook. To enable it manually,
run:

```sh
git config core.hooksPath .githooks
```

The hook checks Rust, TOML, and Python formatting, runs Python linting, checks
and lints the Uno R3 temperature project, runs Clippy on the Uno R4 HAL and
Morse projects, and builds the Uno R3 Morse and Uno R4 Morse projects. It
requires `taplo` and `uv`. The propeller and weather projects are not currently
included in the hook.
