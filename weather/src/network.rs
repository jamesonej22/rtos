use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_net::{
    Runner, StackResources,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_time::{Duration, Timer};
use esp_println::println;
use esp_radio::wifi::{Interface, WifiController};
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::{Method, RequestBuilder},
};

const API_KEY: &str = env!("OPENWEATHER_API_KEY");
const LATITUDE: &str = env!("LATITUDE");
const LONGITUDE: &str = env!("LONGITUDE");

pub type WeatherHttpClient =
    HttpClient<'static, TcpClient<'static, 1, 1500, 1500>, DnsSocket<'static>>;

pub fn request_url() -> heapless::String<256> {
    let mut url = heapless::String::new();
    write!(
        url,
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=imperial&appid={}",
        LATITUDE, LONGITUDE, API_KEY
    )
    .expect("weather request URL exceeds its fixed buffer");
    url
}

pub async fn initialize_network_client(
    spawner: Spawner,
    controller: WifiController<'static>,
    rng: &esp_hal::rng::Trng,
) -> WeatherHttpClient {
    let net_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    static STACK_RESOURCES: static_cell::StaticCell<StackResources<3>> =
        static_cell::StaticCell::new();
    let (stack, runner) = embassy_net::new(
        Interface::station(),
        net_config,
        STACK_RESOURCES.init(StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(controller).expect("Failed to spawn Wi-Fi connection task"));
    spawner.spawn(net_task(runner).expect("Failed to spawn network task"));

    println!("Waiting for Wi-Fi and DHCP...");
    stack.wait_config_up().await;
    if let Some(config) = stack.config_v4() {
        println!("Got IP: {}", config.address);
    }

    static TCP_CLIENT_STATE: static_cell::StaticCell<TcpClientState<1, 1500, 1500>> =
        static_cell::StaticCell::new();
    static TCP_CLIENT: static_cell::StaticCell<TcpClient<'static, 1, 1500, 1500>> =
        static_cell::StaticCell::new();
    static DNS_CLIENT: static_cell::StaticCell<DnsSocket<'static>> = static_cell::StaticCell::new();
    static TLS_READ_BUFFER: static_cell::StaticCell<[u8; 8192]> = static_cell::StaticCell::new();
    static TLS_WRITE_BUFFER: static_cell::StaticCell<[u8; 8192]> = static_cell::StaticCell::new();

    let tcp_client = TCP_CLIENT.init(TcpClient::new(
        stack,
        TCP_CLIENT_STATE.init(TcpClientState::<1, 1500, 1500>::new()),
    ));
    let dns_client = DNS_CLIENT.init(DnsSocket::new(stack));
    let tls_seed = ((rng.random() as u64) << 32) | rng.random() as u64;
    let tls_config = TlsConfig::new(
        tls_seed,
        TLS_READ_BUFFER.init([0; 8192]),
        TLS_WRITE_BUFFER.init([0; 8192]),
        TlsVerify::None,
    );

    HttpClient::new_with_tls(tcp_client, dns_client, tls_config)
}

pub async fn send_request<'a>(
    client: &mut WeatherHttpClient,
    rx_buffer: &'a mut [u8],
) -> Result<(u16, &'a [u8]), reqwless::Error> {
    let url = request_url();
    let mut request = client
        .request(Method::GET, &url)
        .await?
        .headers(&[("Connection", "close")]);
    let response = request.send(rx_buffer).await?;
    let status = response.status.0;
    let body = response.body().read_to_end().await?;
    Ok((status, body))
}

pub fn parse_response(
    body: &[u8],
) -> serde_json_core::de::Result<(crate::types::WeatherResponse<'_>, usize)> {
    serde_json_core::from_slice(body)
}

pub fn print_response(weather: &crate::types::WeatherResponse<'_>) {
    println!(
        "{}: {} ({})",
        weather.name, weather.main.temp, weather.weather[0].description
    );
    println!(
        "Feels like: {}, humidity: {}%",
        weather.main.feels_like, weather.main.humidity
    );
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    loop {
        println!("Connecting to Wi-Fi...");

        match controller.connect_async().await {
            Ok(info) => {
                println!("Wi-Fi connected: {:?}", info);
                controller.wait_for_disconnect_async().await.ok();
                println!("Wi-Fi disconnected.");
            }
            Err(error) => {
                println!("Wi-Fi connection failed: {:?}", error);
            }
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await;
}
