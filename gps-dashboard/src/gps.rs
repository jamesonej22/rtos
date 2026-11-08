use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::watch,
    time::{Duration, sleep},
};

/// Latest GPS telemetry exposed to the rest of the application.
///
/// Fields are optional because GPSD may not have a valid fix yet.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GpsTelemetry {
    /// Whether the application is currently connected to GPSD.
    pub connected: bool,
    /// GPSD fix mode: 0 = unknown, 1 = no fix, 2 = 2D, 3 = 3D.
    pub mode: u8,
    /// Human-readable fix status.
    pub fix_status: String,
    /// UTC timestamp reported by the GPS receiver.
    pub gps_time: Option<String>,
    /// Latitude in decimal degrees.
    pub latitude: Option<f64>,
    /// Longitude in decimal degrees.
    pub longitude: Option<f64>,
    /// Altitude above mean sea level, in meters.
    pub altitude_m: Option<f64>,
    /// Ground speed, in meters per second.
    pub speed_mps: Option<f64>,
    /// Ground speed, in kilometers per hour.
    pub speed_kph: Option<f64>,
    /// Track/heading over ground, in degrees.
    pub heading_deg: Option<f64>,
    /// Vertical speed, in meters per second.
    pub climb_mps: Option<f64>,
    /// Number of satellites visible to the receiver.
    pub satellites_visible: Option<u32>,
    /// Number of satellites used in the current solution.
    pub satellites_used: Option<u32>,
    /// GPSD device path.
    pub device: Option<String>,
    /// Unix timestamp, in milliseconds, when this application
    /// last received a GPSD report.
    pub received_at_ms: Option<u128>,
}

impl GpsTelemetry {
    fn update_received_time(&mut self) {
        self.received_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis());
    }

    fn update_fix_status(&mut self) {
        self.fix_status = match self.mode {
            0 => "Unknown".to_string(),
            1 => "No Fix".to_string(),
            2 => "2D Fix".to_string(),
            3 => "3D Fix".to_string(),
            _ => "Unknown".to_string(),
        };
    }

    /// Apply a GPSD TPV (Time-Position-Velocity) report.
    fn update_tpv(&mut self, report: &Value) {
        if let Some(mode) = report["mode"].as_u64() {
            self.mode = mode as u8;
            self.update_fix_status();
        }

        // GPSD only includes position fields when they are available.
        // Preserve the previous values when a field is absent.
        if let Some(value) = report["time"].as_str() {
            self.gps_time = Some(value.to_string());
        }
        if let Some(value) = report["lat"].as_f64() {
            self.latitude = Some(value);
        }
        if let Some(value) = report["lon"].as_f64() {
            self.longitude = Some(value);
        }
        if let Some(value) = report["alt"].as_f64() {
            self.altitude_m = Some(value);
        }
        if let Some(value) = report["speed"].as_f64() {
            self.speed_mps = Some(value);
            self.speed_kph = Some(value * 3.6);
        }
        if let Some(value) = report["track"].as_f64() {
            self.heading_deg = Some(value);
        }
        if let Some(value) = report["climb"].as_f64() {
            self.climb_mps = Some(value);
        }
        if let Some(value) = report["device"].as_str() {
            self.device = Some(value.to_string());
        }
        self.update_received_time();
    }

    /// Apply a GPSD SKY (satellite information) report.
    fn update_sky(&mut self, report: &Value) {
        if let Some(value) = report["nSat"].as_u64() {
            self.satellites_visible = Some(value as u32);
        }
        if let Some(value) = report["uSat"].as_u64() {
            self.satellites_used = Some(value as u32);
        }
        if let Some(value) = report["device"].as_str() {
            self.device = Some(value.to_string());
        }
        self.update_received_time();
    }
}

/// Connect to GPSD, subscribe to reports, and continuously publish
/// the latest telemetry through a Tokio watch channel.
///
/// This function reconnects automatically if the GPSD connection drops.
pub async fn run_gpsd_client(telemetry_tx: watch::Sender<GpsTelemetry>) {
    const GPSD_ADDRESS: &str = "127.0.0.1:2947";
    let mut telemetry = GpsTelemetry::default();

    loop {
        tracing::info!("Connecting to GPSD at {GPSD_ADDRESS}");

        match connect_and_read(GPSD_ADDRESS, &mut telemetry, &telemetry_tx).await {
            Ok(()) => {
                tracing::warn!("GPSD connection closed");
            }
            Err(error) => {
                tracing::error!("GPSD client error: {error}");
            }
        }

        telemetry.connected = false;
        let _ = telemetry_tx.send(telemetry.clone());

        tracing::info!("Retrying GPSD connection in 2 seconds");
        sleep(Duration::from_secs(2)).await;
    }
}

/// Establish one GPSD connection and process reports until it closes.
async fn connect_and_read(
    address: &str,
    telemetry: &mut GpsTelemetry,
    telemetry_tx: &watch::Sender<GpsTelemetry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stream = TcpStream::connect(address).await?;

    tracing::info!("Connected to GPSD");
    telemetry.connected = true;
    let _ = telemetry_tx.send(telemetry.clone());

    let (read_half, mut write_half) = stream.into_split();

    // Subscribe to JSON reports.
    write_half
        .write_all(b"?WATCH={\"enable\":true,\"json\":true};\n")
        .await?;

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let report: Value = match serde_json::from_str(&line) {
            Ok(report) => report,
            Err(error) => {
                tracing::debug!(
                    "Ignoring malformed GPSD response: {error}; line={}",
                    line.trim()
                );
                continue;
            }
        };

        let report_class = report["class"].as_str().unwrap_or("");
        match report_class {
            "TPV" => {
                telemetry.update_tpv(&report);

                tracing::debug!(
                    "GPS position: lat={:?}, lon={:?}, mode={}",
                    telemetry.latitude,
                    telemetry.longitude,
                    telemetry.mode
                );
            }
            "SKY" => {
                telemetry.update_sky(&report);

                tracing::debug!(
                    "Satellites: visible={:?}, used={:?}",
                    telemetry.satellites_visible,
                    telemetry.satellites_used
                );
            }
            "DEVICE" => {
                if let Some(device) = report["path"].as_str() {
                    telemetry.device = Some(device.to_string());
                }
            }
            "ERROR" => {
                tracing::warn!(
                    "GPSD reported an error: {}",
                    report["message"].as_str().unwrap_or("unknown error")
                );
            }
            // VERSION, DEVICES, WATCH, and other GPSD messages
            // are not needed for our telemetry state yet.
            _ => {}
        }

        // Publish the updated state for the dashboard/API.
        let _ = telemetry_tx.send(telemetry.clone());
    }

    Ok(())
}
