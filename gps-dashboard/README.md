# GPS Dashboard

A Rust application that reads GPS telemetry from GPSD and serves a local web
dashboard. The dashboard displays the receiver connection, GPS fix, position,
flight path, speed, altitude, heading, and satellite information.

## Requirements

- Stable Rust (1.85 or newer)
- GPSD running on `127.0.0.1:2947` and connected to a GPS receiver
- An internet connection for the Leaflet map library and CARTO map tiles

The map uses CARTO's Positron basemap, including labels. The CARTO basemap API
key is configured as `CARTO_API_KEY` in `src/static/app.js`. Since the key is
used by browser JavaScript, it is visible to visitors; restrict it using
CARTO's available settings. OpenStreetMap and CARTO attribution is shown on the
map.

The package selects the stable Rust toolchain independently of the embedded
projects in the workspace.

## Run

Start GPSD with a connected receiver, then run the dashboard from this
directory:

```sh
cargo run
```

Open [http://localhost:8080](http://localhost:8080) in a browser. The server
also provides the current telemetry as JSON at `/api/gps` and as a live
Server-Sent Events stream at `/api/gps/stream`.

The server listens on port `8080` on all network interfaces and logs the
address selected by the system's default network route at startup. Open that
address from another device on the same network. GPSD must be reachable at
`127.0.0.1:2947`; this address is currently configured in the application.

Before a GPS fix is available, the map shows an assumed startup location. To
change it, edit `DEFAULT_POSITION` near the top of `src/static/app.js` and set
its `latitude` and `longitude`. The assumed marker is replaced by the GPS
position when a fix arrives.

## Checks

Run these commands from this directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```
