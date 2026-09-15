#![no_std]
#![no_main]

use arduino_hal::{
    Peripherals,
    adc::{Adc, AdcSettings, ReferenceVoltage},
    default_serial, delay_ms, pins,
};
use panic_halt as _;
use ufmt::uwriteln;
use ufmt_float::uFmt_f32;
const ARDUINO_INTERNAL_REFERENCE_VOLTAGE: f32 = 1.1;

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

    loop {
        let adc_value: u16 = temperature_sensor.analog_read(&mut adc);

        let voltage = (adc_value as f32) * ARDUINO_INTERNAL_REFERENCE_VOLTAGE / 1023.0;
        let temperature_c = (voltage - 0.500) / 0.010;
        let temperature_f = temperature_c * 9.0 / 5.0 + 32.0;

        let voltage = uFmt_f32::Five(voltage);
        let temperature_c = uFmt_f32::Five(temperature_c);
        let temperature_f = uFmt_f32::Five(temperature_f);

        uwriteln!(
            &mut serial,
            "ADC={} Voltage={} TempC={} TempF={}\r",
            adc_value,
            voltage,
            temperature_c,
            temperature_f
        )
        .unwrap();

        delay_ms(1000);
    }
}
