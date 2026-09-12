# EN.605.715 RTOS Class Code

Rust code for the Johns Hopkins University real-time operating systems class (EN.605.715).

## Projects

- [`morse`](morse/): Morse code application for the Arduino Uno R3.
- [`morse-r4`](morse-r4/): Morse code application for the Arduino Uno R4 WiFi.
- [`arduino-uno-r4-hal`](arduino-uno-r4-hal/): Modified Arduino Uno R4 hardware abstraction layer used by `morse-r4`.

See each project README for build, flash, and serial console instructions.

## Pre-push Checks

Enable the repository's pre-push hook once after cloning:

```sh
git config core.hooksPath .githooks
```

The hook checks Rust and TOML formatting, runs Clippy, and builds both firmware
projects for their respective targets. It requires `taplo` to be installed.
