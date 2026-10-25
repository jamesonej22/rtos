#![no_std]

pub mod display;
pub mod network;
pub mod types;

use embedded_dht_rs::dht11::Dht11;
use esp_hal::{
    Blocking,
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Flex, Level, OutputConfig},
    i2c::master::{Config as I2cConfig, I2c},
    ram,
    rng::{Trng, TrngSource},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ControllerConfig, WifiController,
    sta::StationConfig,
};
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*};

pub type WeatherDisplay = Ssd1306<
    I2CInterface<I2c<'static, Blocking>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub type Dht11Sensor = Dht11<Flex<'static>, Delay>;

pub struct EspResources {
    pub wifi_controller: WifiController<'static>,
    pub rng: Trng,
    pub display: WeatherDisplay,
    pub dht11: Dht11Sensor,
}

pub fn initialize_esp() -> EspResources {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut dht11_pin = Flex::new(peripherals.GPIO7);
    dht11_pin.apply_output_config(&OutputConfig::default().with_drive_mode(DriveMode::OpenDrain));
    dht11_pin.set_input_enable(true);
    dht11_pin.set_level(Level::High);
    dht11_pin.set_output_enable(true);
    let dht11 = Dht11::new(dht11_pin, Delay::new());

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_scl(peripherals.GPIO9)
    .with_sda(peripherals.GPIO8);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().expect("Failed to initialize OLED");

    static TRNG_SOURCE: static_cell::StaticCell<TrngSource<'static>> =
        static_cell::StaticCell::new();
    TRNG_SOURCE.init(TrngSource::new(peripherals.RNG, peripherals.ADC1));
    let rng =
        Trng::try_new().expect("Failed to initialize the hardware true random number generator");

    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let ssid = env!("SSID");
    let password = env!("PASSWORD");
    let station_config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(ssid.try_into().expect("SSID exceeds the supported length"))
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                password
                    .try_into()
                    .expect("Wi-Fi password exceeds the supported length"),
            )),
    );
    let wifi_controller = WifiController::new(
        peripherals.WIFI,
        ControllerConfig::default().with_initial_config(station_config),
    )
    .expect("Failed to initialize Wi-Fi controller");

    EspResources {
        wifi_controller,
        rng,
        display,
        dht11,
    }
}
