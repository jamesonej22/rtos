//! Reads lines over serial and displays them as Morse code on the built-in LED.
#![no_std]
#![no_main]

use arduino_hal::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};
use core::str;
use morse::transmit_string;
use panic_halt as _;

pub const ARDUINO_UNO_R3_BAUD_RATE: u32 = 57_600;

#[arduino_hal::entry]
/// Waits for serial input and transmits each completed line in Morse code.
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    let mut serial = arduino_hal::default_serial!(dp, pins, ARDUINO_UNO_R3_BAUD_RATE);

    'main: loop {
        for byte in b"morse> ".iter().copied() {
            nb::block!(serial.write(byte)).unwrap();
        }

        let mut line = [0u8; 64];
        let mut line_length = 0;

        loop {
            let byte = nb::block!(serial.read()).unwrap();

            match byte {
                b'\r' | b'\n' => {
                    nb::block!(serial.write(b'\r')).unwrap();
                    nb::block!(serial.write(b'\n')).unwrap();
                    break;
                }
                b'#' => break 'main,
                byte if line_length < line.len() => {
                    line[line_length] = byte;
                    line_length += 1;
                    nb::block!(serial.write(byte)).unwrap();
                }
                _ => {}
            }
        }

        if let Ok(text) = str::from_utf8(&line[..line_length]) {
            transmit_string(&mut led, text);
        }
    }

    panic!()
}
