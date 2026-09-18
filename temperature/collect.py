import serial

ser = serial.Serial("/dev/ttyACM0", 115200)

with open("temperature.csv", "w") as f:
    while True:
        line = ser.readline().decode("utf-8", errors="ignore")

        print(line, end="")
        f.write(line)
        f.flush()
