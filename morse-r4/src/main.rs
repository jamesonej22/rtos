#![no_std]
#![no_main]

use core::{fmt::Write as _, str};

use arduino_uno_r4_hal::{Peripherals, serial::Serial};
use embedded_hal_nb::serial::{Read as _, Write as _};
use morse_r4::transmit_string;
use nb::block;
use panic_halt as _;

pub const ARDUINO_UNO_R4_BAUD_RATE: u32 = 115_200;

#[cortex_m_rt::entry]
/// Waits for serial input and transmits each completed line in Morse code.
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let mut delay = p.delay;
    let mut led = p.pins.d13.into_output();

    // SCI9 -> P109/P110 -> ESP32-S3 USB bridge -> /dev/ttyACM0
    let mut serial = Serial::new_sci9(p.sci9, ARDUINO_UNO_R4_BAUD_RATE, &p.clocks).unwrap();

    loop {
        serial.write_str("morse> ").unwrap();

        let mut line = [0u8; 64];
        let mut line_length = 0;

        loop {
            let byte = match block!(serial.read()) {
                Ok(byte) => byte,
                Err(_) => continue,
            };

            match byte {
                b'\r' | b'\n' => {
                    serial.write_str("\r\n").unwrap();
                    break;
                }

                byte if line_length < line.len() => {
                    line[line_length] = byte;
                    line_length += 1;
                    block!(serial.write(byte)).unwrap();
                }

                _ => {}
            }
        }

        if let Ok(text) = str::from_utf8(&line[..line_length]) {
            transmit_string(&mut led, &mut delay, text);
        }
    }
}
