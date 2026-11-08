# EN.605.715 RTOS Class Code

Rust firmware and tools for the Johns Hopkins University real-time operating
systems class (EN.605.715). The repository is a Cargo workspace containing
projects for the Arduino Uno R3, Arduino Uno R4 WiFi, ESP32-S3, and a GPS
telemetry dashboard.

## Projects

- [`morse`](morse/): Morse code LED application for the Arduino Uno R3.
- [`morse-r4`](morse-r4/): Morse code LED application for the Arduino Uno R4 WiFi.
- [`arduino-uno-r4-hal`](arduino-uno-r4-hal/): Class-modified HAL for the Uno R4 Minima and WiFi boards, used by `morse-r4`.
- [`gps-dashboard`](gps-dashboard/): Stable-Rust GPS telemetry dashboard that reads from GPSD and serves a local web interface.
- [`propeller`](propeller/): Uno R3 firmware to measure propeller RPM with an IR beam-break sensor, plus a live data collector and plotter.
- [`temperature`](temperature/): Uno R3 temperature logger, plus a serial data collector and plotter.
- [`weather`](weather/): ESP32-S3 Wi-Fi weather display with an OLED and local DHT11 sensor.

See each project README for hardware connections, project-specific
prerequisites, build and upload commands, and data collection instructions.
The GPS dashboard uses the stable Rust toolchain; the embedded projects use
their own toolchain requirements.

## Development Setup

Install the system prerequisites for your platform, clone this repository,
then run `./setup-dev.sh`. The script installs Rust with `rustup`, `taplo`,
`uv`, and the repository's Rust toolchains.

On Raspberry Pi OS (64-bit recommended), install the build tools and AVR
toolchain with:

```sh
sudo apt update
sudo apt install build-essential curl gcc-avr avr-libc git libudev-dev pkg-config
```

On Fedora, install the AVR toolchain with:

```sh
sudo dnf install avr-gcc avr-libc
```

Then, from the repository root, run:

```sh
./setup-dev.sh
```

The script adds Cargo and `uv` to `PATH` in `~/.profile` and `~/.bashrc`,
installs the pinned nightly Rust toolchain and Uno R4 target, creates the shared
Python environment, and enables the pre-push hook. On supported hosts it also
installs `espup` and `espflash` and sets up the ESP Rust toolchain. ESP toolchain
setup is skipped on ARM Linux hosts such as Raspberry Pi; use an x86_64 Linux
host to build the weather firmware. The Python environment includes `pyserial`
and `matplotlib`, used by the data collectors:

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

The hook checks Rust and TOML formatting, formats and lints Python, and runs
build and Clippy checks for the GPS dashboard, Uno R3 and Uno R4 firmware,
temperature logger, and propeller RPM logger. Weather firmware checks also run
when the ESP toolchain export and weather credentials are available; otherwise
the hook reports that they were skipped. The hook requires `taplo` and `uv`.
