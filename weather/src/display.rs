use core::fmt::Write;

use crate::{WeatherDisplay, types::WeatherResponse};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

pub fn show_weather(display: &mut WeatherDisplay, weather: &WeatherResponse<'_>) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let mut location_temp = heapless::String::<32>::new();
    write!(
        &mut location_temp,
        "{} {:.1} F",
        weather.name, weather.main.temp
    )
    .unwrap();

    let mut feels_like = heapless::String::<24>::new();
    write!(&mut feels_like, "Feels: {:.1} F", weather.main.feels_like).unwrap();

    let mut humidity = heapless::String::<24>::new();
    write!(&mut humidity, "Humidity: {}%", weather.main.humidity).unwrap();

    display.clear(BinaryColor::Off).unwrap();

    Text::new(&location_temp, Point::new(0, 10), style)
        .draw(display)
        .unwrap();

    Text::new(weather.weather[0].description, Point::new(0, 25), style)
        .draw(display)
        .unwrap();

    Text::new(&feels_like, Point::new(0, 40), style)
        .draw(display)
        .unwrap();

    Text::new(&humidity, Point::new(0, 55), style)
        .draw(display)
        .unwrap();

    display.flush().expect("Failed to update OLED");
}
