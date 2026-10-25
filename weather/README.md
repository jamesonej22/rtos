# Weather

An ESP32-S3 Wi-Fi application that requests current weather from the OpenWeather API over HTTPS, deserializes it into a `WeatherResponse`, and displays outdoor and local conditions on a 128x64 I2C OLED. Weather refreshes every 10 minutes; outdoor conditions and DHT11 readings are also printed to the serial monitor.

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

`cargo run` flashes the ESP32-S3 and opens a serial monitor. The OLED uses GPIO8 for SDA and GPIO9 for SCL. The DHT11 data line uses GPIO7 and must be pulled up to 3.3 V (many modules include a pull-up).

The display alternates between outdoor weather (including its weather icon) and local DHT11 readings every five seconds. The DHT11 is sampled about every 10 seconds; its temperature is shown in Fahrenheit. Sensor readings are logged once per minute.

The HTTPS client encrypts traffic but **does not verify the server certificate**; use it for development only.
