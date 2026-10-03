"""Live RPM collector for the propeller serial output."""

import csv
import time

import matplotlib.pyplot as plt
import serial
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import Button

SERIAL_PORT = "/dev/ttyACM0"
BAUD_RATE = 115200
POLL_INTERVAL_MS = 100
TIMESTAMP_PERIOD_MS = (1 << 32) // 2_000 + 1

CSV_FILE = "propeller.csv"
PLOT_FILE = "propeller_plot.png"


def parse_line(raw_line):
    parts = raw_line.decode("utf-8", errors="ignore").strip().split(",")
    if len(parts) != 4:
        return None

    try:
        time_ms = int(parts[0])
        state = parts[1]
        count = int(parts[2])
        rpm = int(parts[3])
    except ValueError:
        return None

    if state not in ("BLOCKED", "CLEAR"):
        return None
    return time_ms, state, count, rpm


def main():
    print(f"Opening {SERIAL_PORT} at {BAUD_RATE} baud...")
    ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=0)
    time.sleep(2)
    ser.reset_input_buffer()

    rows = []
    pending = bytearray()
    last_sample_ms = None
    elapsed_s = 0.0
    collecting = False
    latest_reading = "Waiting for serial data..."

    fig, ax = plt.subplots(figsize=(10, 6))
    fig.subplots_adjust(bottom=0.23)
    (rpm_line,) = ax.plot([], [], marker="o", markersize=4)
    ax.set_xlabel("Elapsed time from first sample (s)")
    ax.set_ylabel("RPM")
    ax.set_title("Propeller RPM")
    ax.set_ylim(bottom=0)
    ax.grid(True)
    status = fig.text(0.12, 0.16, latest_reading)

    start_axis = fig.add_axes((0.12, 0.06, 0.22, 0.07))
    quit_axis = fig.add_axes((0.39, 0.06, 0.22, 0.07))
    collection_button = Button(start_axis, "Start collection")
    quit_button = Button(quit_axis, "Quit & save")

    def toggle_collection(_event):
        nonlocal collecting
        collecting = not collecting
        collection_button.label.set_text(
            "Stop collection" if collecting else "Start collection"
        )
        status.set_text(
            f"Collecting | {latest_reading}"
            if collecting
            else f"Not collecting | {latest_reading}"
        )
        fig.canvas.draw_idle()

    def quit_program(_event):
        plt.close(fig)

    def update(_frame):
        nonlocal elapsed_s, last_sample_ms, latest_reading
        available = ser.in_waiting
        if available:
            pending.extend(ser.read(available))

        while b"\n" in pending:
            raw_line, _, remainder = pending.partition(b"\n")
            pending[:] = remainder
            reading = parse_line(raw_line)
            if reading is None:
                continue

            time_ms, state, count, rpm = reading
            latest_reading = f"{state} | count {count} | {rpm} RPM"
            if collecting and state == "BLOCKED":
                if last_sample_ms is not None:
                    elapsed_ms = (time_ms - last_sample_ms) % TIMESTAMP_PERIOD_MS
                    elapsed_s += elapsed_ms / 1_000
                last_sample_ms = time_ms
                rows.append((elapsed_s, time_ms, count, rpm))

        prefix = "Collecting" if collecting else "Not collecting"
        status.set_text(f"{prefix} | {latest_reading}")

        if rows:
            rpm_values = [row[3] for row in rows]
            rpm_line.set_data(
                [row[0] for row in rows],
                rpm_values,
            )
            ax.relim()
            ax.autoscale_view(scalex=True, scaley=False)
            ax.set_ylim(0, max(1, max(rpm_values) * 1.1))
        return rpm_line, status

    collection_button.on_clicked(toggle_collection)
    quit_button.on_clicked(quit_program)
    animation = FuncAnimation(
        fig,
        update,
        interval=POLL_INTERVAL_MS,
        blit=False,
        cache_frame_data=False,
    )

    try:
        plt.show()
    finally:
        ser.close()
        with open(CSV_FILE, "w", newline="") as output:
            writer = csv.writer(output)
            writer.writerow(["elapsed_s", "t_ms", "count", "rpm"])
            writer.writerows(rows)
        fig.savefig(PLOT_FILE, dpi=150)
        print(f"Saved {len(rows)} RPM samples to {CSV_FILE}")
        print(f"Saved plot to {PLOT_FILE}")
        del animation


if __name__ == "__main__":
    main()
