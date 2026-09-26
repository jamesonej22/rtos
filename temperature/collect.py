import csv
import time

import matplotlib.pyplot as plt
import serial

SERIAL_PORT = "/dev/ttyACM0"
BAUD_RATE = 115200

DURATION_SECONDS = 12 * 60

CSV_FILE = "temperature.csv"
PLOT_FILE = "temperature_plot.png"


def main():
    print(f"Opening {SERIAL_PORT}...")
    ser = serial.Serial(SERIAL_PORT, BAUD_RATE, timeout=1)

    # Give the serial connection a moment to settle.
    time.sleep(2)

    print(f"Collecting data for {DURATION_SECONDS} seconds...\n")

    start_time = time.monotonic()

    rows = []

    while time.monotonic() - start_time < DURATION_SECONDS:
        line = ser.readline().decode("utf-8", errors="ignore").strip()

        if not line:
            continue

        print(line)

        # Ignore anything that isn't a CSV data row.
        if "," not in line:
            continue

        parts = line.split(",")

        if len(parts) != 2:
            continue

        try:
            time_s = int(parts[0])
            temp_f = float(parts[1])
        except ValueError:
            continue

        rows.append((time_s, temp_f))

    ser.close()

    print(f"\nCollection complete. Received {len(rows)} samples.")

    # Write clean CSV.
    with open(CSV_FILE, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["time_s", "temp_f"])
        writer.writerows(rows)

    print(f"Saved data to {CSV_FILE}")

    # Plot the data.
    times = [row[0] for row in rows]
    temperatures = [row[1] for row in rows]

    plt.figure(figsize=(10, 6))
    plt.plot(times, temperatures)

    plt.xlabel("Time (seconds)")
    plt.ylabel("Temperature (°F)")
    plt.title("Arduino Temperature vs. Time")

    plt.grid(True)
    plt.tight_layout()

    plt.savefig(PLOT_FILE, dpi=150)
    plt.show()

    print(f"Saved plot to {PLOT_FILE}")


if __name__ == "__main__":
    main()
