use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeatherResponse<'a> {
    pub coord: Coord,
    pub weather: [WeatherDescription<'a>; 1],
    pub base: &'a str,
    pub main: Weather,
    pub visibility: u32,
    pub wind: Wind,
    pub clouds: Clouds,
    pub dt: u64,
    pub sys: Sys<'a>,
    pub timezone: i32,
    pub id: u32,
    pub name: &'a str,
    pub cod: u32,
}

#[derive(Deserialize)]
pub struct Coord {
    pub lon: f32,
    pub lat: f32,
}

#[derive(Deserialize)]
pub struct WeatherDescription<'a> {
    pub id: u32,
    pub main: &'a str,
    pub description: &'a str,
    pub icon: &'a str,
}

#[derive(Deserialize)]
pub struct Weather {
    pub temp: f32,
    pub feels_like: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub pressure: u32,
    pub humidity: u32,
    #[serde(default)]
    pub sea_level: Option<u32>,
    #[serde(default)]
    pub grnd_level: Option<u32>,
}

#[derive(Deserialize)]
pub struct Wind {
    pub speed: f32,
    pub deg: u32,
    #[serde(default)]
    pub gust: Option<f32>,
}

#[derive(Deserialize)]
pub struct Clouds {
    pub all: u32,
}

#[derive(Deserialize)]
pub struct Sys<'a> {
    #[serde(default)]
    pub r#type: Option<u32>,
    #[serde(default)]
    pub id: Option<u32>,
    pub country: &'a str,
    pub sunrise: u64,
    pub sunset: u64,
}
