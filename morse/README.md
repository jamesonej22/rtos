# Morse Code LED

An Arduino Uno Rust application for Project 1: it reads a typed line over serial and displays letters and digits as Morse code on the built-in LED.

The program uses a simple round-robin loop: wait for a line, transmit it, and repeat. Morse timing uses a 132 ms unit: dot = 1 unit, dash = 3, gaps within a character = 1, between characters = 3, and between words = 7.

## Build and Upload

From this directory:

```sh
cargo build
cargo run
```

Type a line in the serial console and press Enter. Input is limited to 64 bytes; unsupported punctuation is ignored.
