# Propeller RPM Logger

An Arduino Uno R3 firmware project that estimates propeller RPM from an IR beam-break sensor. Connect the sensor output to D2; D9 indicates whether the beam is blocked, and D7 provides a debug pulse for an oscilloscope. Set `INTERRUPTIONS_PER_REVOLUTION` in `src/main.rs` to match the number of beam interruptions per revolution.

## Build and Upload

Connect the Uno, then run these commands from this directory:

```sh
cargo build
cargo run
```

`cargo run` uploads the firmware and opens the serial console at 115200 baud. The firmware prints a CSV header and a row when the sensor state changes:

```text
t_ms,state,count,rpm
1000,BLOCKED,1,0
```

## Collect and Plot Data

The collector requires the repository's Python dependencies (`pyserial` and `matplotlib`), installed by `uv sync` from the repository root. Close the firmware serial console, then from the repository root run:

```sh
uv run python propeller/collect.py
```

Click **Start collection** and **Stop collection** to control which RPM samples are recorded. The plot updates live; **Quit & save** writes the collected data to `propeller.csv` and the plot to `propeller_plot.png` in the current working directory. To use another serial device, change `SERIAL_PORT` in `collect.py`.
