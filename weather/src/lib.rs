#![no_std]

pub mod network;
pub mod types;

use esp_hal::{
    clock::CpuClock,
    ram,
    rng::{Trng, TrngSource},
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ControllerConfig, WifiController,
    sta::StationConfig,
};

pub struct EspResources {
    pub wifi_controller: WifiController<'static>,
    pub rng: Trng,
}

pub fn initialize_esp() -> EspResources {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

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
    }
}
