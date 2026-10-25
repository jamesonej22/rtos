#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use weather::{
    display::{show_local_conditions, show_sensor_error, show_weather},
    initialize_esp,
    network::{initialize_network_client, send_request},
    types::WeatherResponse,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
async fn main(spawner: Spawner) -> ! {
    let mut esp = initialize_esp();
    let mut client = initialize_network_client(spawner, esp.wifi_controller, &esp.rng).await;

    println!("HTTPS is enabled; server certificate verification is disabled.");
    loop {
        println!("Requesting current weather...");
        let mut rx_buf = [0u8; 4096];
        let (status, body) = match send_request(&mut client, &mut rx_buf).await {
            Ok(response) => response,
            Err(error) => {
                println!("Weather request failed: {:?}", error);
                Timer::after(Duration::from_secs(600)).await;
                continue;
            }
        };

        println!("HTTP status: {}", status);
        let (weather, _) = match serde_json_core::from_slice::<WeatherResponse<'_>>(body) {
            Ok(weather) => weather,
            Err(error) => {
                println!("Could not deserialize weather response: {:?}", error);
                Timer::after(Duration::from_secs(600)).await;
                continue;
            }
        };
        println!(
            "{}: {} ({})",
            weather.name, weather.main.temp, weather.weather[0].description
        );
        println!(
            "Feels like: {}, humidity: {}%",
            weather.main.feels_like, weather.main.humidity
        );
        show_weather(&mut esp.display, &weather);

        for sample in 0..60 {
            match esp.dht11.read() {
                Ok(reading) => {
                    if sample % 6 == 0 {
                        let temperature_f = f32::from(reading.temperature) * 9.0 / 5.0 + 32.0;
                        println!(
                            "Local conditions: {:.1} F, {}% humidity",
                            temperature_f, reading.humidity
                        );
                    }
                    show_local_conditions(&mut esp.display, reading.temperature, reading.humidity);
                }
                Err(error) => {
                    if sample % 6 == 0 {
                        println!("DHT11 read failed: {:?}", error);
                    }
                    show_sensor_error(&mut esp.display);
                }
            }
            Timer::after(Duration::from_secs(5)).await;
            show_weather(&mut esp.display, &weather);
            Timer::after(Duration::from_secs(5)).await;
        }
    }
}
