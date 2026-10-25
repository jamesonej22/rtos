# Weather

An ESP32-S3 Wi-Fi application that periodically requests current weather from the OpenWeather API over HTTPS and prints the response over the serial connection. The response is deserialized into a `WeatherResponse` every 10 minutes.

## Setup

Install Rust with `rustup`, then install the ESP Rust toolchain and flasher:

```sh
cargo install espup --locked
espup install
source ~/export-esp.sh
cargo install espflash --locked
```

The `espup` environment must be sourced in each shell before building. Connect an ESP32-S3 board over USB.

Create a `.env` file in this directory with the Wi-Fi and OpenWeather settings:

```text
SSID="your-wifi-name"
PASSWORD="your-wifi-password"
OPENWEATHER_API_KEY="your-api-key"
LATITUDE="your-latitude"
LONGITUDE="your-longitude"
```

Do not commit `.env` or share your API key.

## Build and Upload

From this directory, with the ESP toolchain environment sourced:

```sh
cargo build
cargo run
```

`cargo run` flashes the ESP32-S3 and opens a serial monitor. The current HTTPS client encrypts traffic but **does not verify the server certificate**; use it for development only.
