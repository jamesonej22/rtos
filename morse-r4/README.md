# Morse R4

Morse code application for the Arduino Uno R4 WiFi. It reads a line over the
serial connection and displays supported letters and digits on the built-in LED
in Morse code.

## Run

From this directory, or from the repository root with the package flag:

```sh
cargo run -p morse-r4 --target thumbv7em-none-eabihf
```

This builds the firmware and flashes it with `probe-rs`. The board must be
connected over USB, with its CMSIS-DAP debug probe available. The serial
connection uses `115200` baud.

After flashing, type a line in a serial terminal and press Enter. Input is
limited to 64 bytes; unsupported punctuation is ignored.

To build without flashing:

```sh
cargo build -p morse-r4 --target thumbv7em-none-eabihf
```
