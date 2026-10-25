use core::fmt::Write;

use crate::{WeatherDisplay, types::WeatherResponse};
use embedded_graphics::{
    image::Image,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use embedded_iconoir::prelude::*;

pub fn show_weather(display: &mut WeatherDisplay, weather: &WeatherResponse<'_>) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let location = truncated_text::<10>(weather.name);
    let mut header = heapless::String::<16>::new();
    write!(&mut header, "OUT: {}", location).unwrap();
    let mut temperature = heapless::String::<20>::new();
    write!(&mut temperature, "Temp: {:.1} F", weather.main.temp).unwrap();
    let description = truncated_text::<20>(weather.weather[0].description);
    let mut details = heapless::String::<24>::new();
    write!(
        &mut details,
        "Feels {:.1} F  RH {}%",
        weather.main.feels_like, weather.main.humidity
    )
    .unwrap();

    display.clear(BinaryColor::Off).unwrap();

    draw_weather_icon(display, weather);

    Text::new(&header, Point::new(0, 10), style)
        .draw(display)
        .unwrap();

    Text::new(&temperature, Point::new(0, 28), style)
        .draw(display)
        .unwrap();

    Text::new(&description, Point::new(0, 40), style)
        .draw(display)
        .unwrap();

    Text::new(&details, Point::new(0, 52), style)
        .draw(display)
        .unwrap();

    display.flush().expect("Failed to update OLED");
}

pub fn show_local_conditions(display: &mut WeatherDisplay, temperature_c: u8, humidity: u8) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let temperature_f = f32::from(temperature_c) * 9.0 / 5.0 + 32.0;

    let mut temperature = heapless::String::<32>::new();
    write!(&mut temperature, "Temp: {:.1} F", temperature_f).unwrap();

    let mut humidity_text = heapless::String::<24>::new();
    write!(&mut humidity_text, "Humidity: {}%", humidity).unwrap();

    display.clear(BinaryColor::Off).unwrap();

    Text::new("LOCAL DHT11", Point::new(0, 10), style)
        .draw(display)
        .unwrap();

    Text::new(&temperature, Point::new(0, 28), style)
        .draw(display)
        .unwrap();

    Text::new(&humidity_text, Point::new(0, 40), style)
        .draw(display)
        .unwrap();

    Text::new("Sensor readings", Point::new(0, 52), style)
        .draw(display)
        .unwrap();

    display.flush().expect("Failed to update OLED");
}

pub fn show_sensor_error(display: &mut WeatherDisplay) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    display.clear(BinaryColor::Off).unwrap();

    Text::new("LOCAL DHT11", Point::new(0, 10), style)
        .draw(display)
        .unwrap();

    Text::new("Sensor read failed", Point::new(0, 28), style)
        .draw(display)
        .unwrap();

    Text::new("Check GPIO7", Point::new(0, 40), style)
        .draw(display)
        .unwrap();

    display.flush().expect("Failed to update OLED");
}

fn truncated_text<const N: usize>(text: &str) -> heapless::String<N> {
    let mut result = heapless::String::new();
    for character in text.chars() {
        if result.push(character).is_err() {
            break;
        }
    }
    result
}

fn draw_weather_icon(display: &mut WeatherDisplay, weather: &WeatherResponse<'_>) {
    let is_night = weather.dt < weather.sys.sunrise || weather.dt >= weather.sys.sunset;
    let icon = weather.weather[0].icon;

    macro_rules! draw_icon {
        ($icon:expr) => {
            Image::new(&$icon, Point::new(100, 2))
                .draw(display)
                .unwrap()
        };
    }

    // Use a moon icon for clear nighttime conditions.
    if is_night && icon.starts_with("01") {
        draw_icon!(icons::size24px::other::HalfMoon::new(BinaryColor::On));
        return;
    }

    match icon {
        "01d" => draw_icon!(icons::size24px::weather::SunLight::new(BinaryColor::On)),
        "01n" => draw_icon!(icons::size24px::weather::Cloud::new(BinaryColor::On)),
        "02d" | "02n" => draw_icon!(icons::size24px::weather::CloudSunny::new(BinaryColor::On)),
        "03d" | "03n" | "04d" | "04n" => {
            draw_icon!(icons::size24px::weather::Cloud::new(BinaryColor::On))
        }
        "09d" | "09n" | "10d" | "10n" => {
            draw_icon!(icons::size24px::weather::Rain::new(BinaryColor::On))
        }
        "11d" | "11n" => draw_icon!(icons::size24px::weather::Thunderstorm::new(BinaryColor::On)),
        "13d" | "13n" => draw_icon!(icons::size24px::weather::Snow::new(BinaryColor::On)),
        "50d" | "50n" => draw_icon!(icons::size24px::weather::Fog::new(BinaryColor::On)),
        _ => draw_icon!(icons::size24px::weather::Cloud::new(BinaryColor::On)),
    }
}
