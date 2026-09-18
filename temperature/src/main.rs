#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::cell::Cell;

use arduino_hal::{
    Peripherals,
    adc::{Adc, AdcSettings, ReferenceVoltage},
    default_serial, pins,
};
use avr_device::interrupt::Mutex;
use panic_halt as _;
use ufmt::uwriteln;
use ufmt_float::uFmt_f32;

const ARDUINO_INTERNAL_REFERENCE_VOLTAGE: f32 = 1.1;

static SAMPLE_REQUESTED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static SECONDS: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));
static ELAPSED_SECONDS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    avr_device::interrupt::free(|cs| {
        // Keep track of elapsed time.
        let elapsed = ELAPSED_SECONDS.borrow(cs).get() + 1;
        ELAPSED_SECONDS.borrow(cs).set(elapsed);

        // Request a temperature sample every 10 seconds.
        let seconds = SECONDS.borrow(cs).get() + 1;

        if seconds >= 10 {
            SECONDS.borrow(cs).set(0);
            SAMPLE_REQUESTED.borrow(cs).set(true);
        } else {
            SECONDS.borrow(cs).set(seconds);
        }
    });
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);

    let mut serial = default_serial!(dp, pins, 115200);
    let mut adc = Adc::new(
        dp.ADC,
        AdcSettings {
            ref_voltage: ReferenceVoltage::Internal,
            ..Default::default()
        },
    );
    let temperature_sensor = pins.a0.into_analog_input(&mut adc);

    // Configure Timer1 for a 10-second interrupt
    let tc1 = dp.TC1;
    tc1.tccr1a().write(|w| unsafe { w.wgm1().bits(0b00) });
    tc1.tccr1b()
        .write(|w| unsafe { w.wgm1().bits(0b01).cs1().prescale_1024() });
    tc1.ocr1a().write(|w| w.set(15624));
    tc1.timsk1().write(|w| w.ocie1a().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }
    ufmt::uwriteln!(&mut serial, "time_s,temp_f\r").unwrap();

    loop {
        let sample = avr_device::interrupt::free(|cs| {
            let requested = SAMPLE_REQUESTED.borrow(cs).get();

            if requested {
                SAMPLE_REQUESTED.borrow(cs).set(false);
            }

            requested
        });

        if sample {
            let elapsed_seconds =
                avr_device::interrupt::free(|cs| ELAPSED_SECONDS.borrow(cs).get());

            let adc_value: u16 = temperature_sensor.analog_read(&mut adc);
            let voltage = (adc_value as f32) * ARDUINO_INTERNAL_REFERENCE_VOLTAGE / 1023.0;
            let temperature_c = (voltage - 0.500) / 0.010;
            let temperature_f = temperature_c * 9.0 / 5.0 + 32.0;

            uwriteln!(
                &mut serial,
                "{},{}\r",
                elapsed_seconds,
                uFmt_f32::Five(temperature_f)
            )
            .unwrap();
        }
    }
}
