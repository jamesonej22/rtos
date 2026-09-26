# Temperature Logger

An Arduino Uno R3 firmware project that samples an analog temperature sensor on A0 and streams readings over USB serial. Samples are taken every 10 seconds and reported in degrees Fahrenheit.

The conversion assumes a TMP36-style sensor: 500 mV at 0 degrees Celsius and a change of 10 mV per degree Celsius. The ADC uses the Uno's internal 1.1 V reference, so make sure the sensor output stays within that range.

## Build and Upload

Connect the Uno, then run these commands from this directory:

```sh
cargo build
cargo run
```

`cargo run` uploads the firmware and opens the serial console at 115200 baud. The firmware prints a CSV header followed by one row every 10 seconds:

```text
time_s,temp_f
10,71.09677
```

## Collect and Plot Data

The collector requires the repository's Python dependencies (`pyserial` and `matplotlib`), installed by `uv sync` from the repository root. Close the firmware serial console, then from the repository root run:

```sh
uv run python temperature/collect.py
```

It reads `/dev/ttyACM0` at 115200 baud for 12 minutes, saves the readings to `temperature.csv`, and saves/displays a plot as `temperature_plot.png`. Both output files are written to the current working directory. To use another serial device, change `SERIAL_PORT` in `collect.py`.
