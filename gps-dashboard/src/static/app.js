"use strict";

function setText(id, value) {
    document.getElementById(id).textContent = value;
}

function formatNumber(value, decimals = 2) {
    if (value === null || value === undefined || !Number.isFinite(value)) {
        return "--";
    }

    return value.toFixed(decimals);
}

function formatCoordinate(value, positive, negative) {
    if (value === null || value === undefined || !Number.isFinite(value)) {
        return "--";
    }

    const direction = value >= 0 ? positive : negative;
    return `${Math.abs(value).toFixed(6)}° ${direction}`;
}

// Replace these coordinates with your preferred startup map location.
const DEFAULT_POSITION = { latitude: 38.88697, longitude: -77.09401 };
const DEFAULT_ZOOM = 17;
// Add your CARTO API key here. Browser-based map keys are visible to visitors.
const CARTO_API_KEY = "CARTO_API_KEY";

const map = L.map("map", { zoomControl: true }).setView(
    [DEFAULT_POSITION.latitude, DEFAULT_POSITION.longitude],
    DEFAULT_ZOOM,
);

L.tileLayer(
    `https://{s}.basemaps.cartocdn.com/rastertiles/dark_nolabels/{z}/{x}/{y}.png?key=${encodeURIComponent(CARTO_API_KEY)}`,
    {
        maxZoom: 19,
        subdomains: "abcd",
        attribution:
            '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>',
    },
).addTo(map);

let positionMarker = L.circleMarker(
    [DEFAULT_POSITION.latitude, DEFAULT_POSITION.longitude],
    {
        radius: 8,
        color: "#ffffff",
        weight: 2,
        fillColor: "#facc15",
        fillOpacity: 1,
    },
)
    .addTo(map)
    .bindPopup("Assumed default location (waiting for GPS fix)");
let flightPath = null;
let hasCenteredMap = false;
let hasGpsPosition = false;

const trackPoints = [];
const MAX_TRACK_POINTS = 500;

function updateConnectionStatus(connected) {
    const dot = document.getElementById("connection-dot");

    dot.classList.toggle("online", connected);
    dot.classList.toggle("offline", !connected);
    setText("connection-status", connected ? "Connected" : "Disconnected");
    setText("gpsd-status", connected ? "Connected" : "Disconnected");
}

function updateFixStatus(mode, status) {
    const badge = document.getElementById("fix-status");

    badge.textContent = (status || "Unknown").toUpperCase();
    badge.classList.remove("no-fix", "fix-2d", "fix-3d");

    if (mode === 2) {
        badge.classList.add("fix-2d");
    } else if (mode === 3) {
        badge.classList.add("fix-3d");
    } else {
        badge.classList.add("no-fix");
    }

    setText("gps-mode", status || "Unknown");
}

function updatePosition(latitude, longitude) {
    const validPosition =
        Number.isFinite(latitude) &&
        Number.isFinite(longitude) &&
        Math.abs(latitude) <= 90 &&
        Math.abs(longitude) <= 180;

    if (!validPosition) {
        return;
    }

    const position = [latitude, longitude];

    document.getElementById("map-position-notice").classList.add("hidden");
    setText("latitude", formatCoordinate(latitude, "N", "S"));
    setText("longitude", formatCoordinate(longitude, "E", "W"));
    setText("map-coordinates", `${latitude.toFixed(6)}, ${longitude.toFixed(6)}`);

    if (!hasGpsPosition) {
        positionMarker.setStyle({ fillColor: "#38bdf8" });
        positionMarker.setPopupContent("Current GPS position");
        flightPath = L.polyline([], {
            color: "#38bdf8",
            weight: 3,
            opacity: 0.85,
        }).addTo(map);
        hasGpsPosition = true;
    }

    positionMarker.setLatLng(position);

    // Record only changed positions and keep the track bounded.
    const previous = trackPoints[trackPoints.length - 1];
    if (!previous || previous[0] !== latitude || previous[1] !== longitude) {
        trackPoints.push(position);

        if (trackPoints.length > MAX_TRACK_POINTS) {
            trackPoints.shift();
        }

        flightPath.setLatLngs(trackPoints);
    }

    if (!hasCenteredMap) {
        map.setView(position, DEFAULT_ZOOM);
        hasCenteredMap = true;
    }
}

function updateSatellites(used, visible) {
    const satelliteBar = document.getElementById("satellite-bar-fill");

    setText("satellites-used", used ?? "--");
    setText("satellites-visible", visible ?? "--");

    if (visible === null || visible === undefined || visible === 0) {
        setText("satellite-caption", "No satellites currently visible");
        satelliteBar.style.width = "0%";
        return;
    }

    const percentage = Math.min(100, ((used ?? 0) / visible) * 100);
    satelliteBar.style.width = `${percentage}%`;
    setText(
        "satellite-caption",
        `${used ?? 0} of ${visible} visible satellites used in solution`,
    );
}

function updateTelemetry(data) {
    updateConnectionStatus(data.connected);
    updateFixStatus(data.mode, data.fix_status);

    setText("altitude", formatNumber(data.altitude_m, 1));
    setText("speed", formatNumber(data.speed_kph, 1));
    setText("heading", formatNumber(data.heading_deg, 1));
    setText("climb", formatNumber(data.climb_mps, 2));
    setText("gps-device", data.device || "--");
    setText("gps-time", data.gps_time || "--");

    updateSatellites(data.satellites_used, data.satellites_visible);
    updatePosition(data.latitude, data.longitude);

    if (data.received_at_ms) {
        setText("last-update", new Date(data.received_at_ms).toLocaleTimeString());
    }
}

const eventSource = new EventSource("/api/gps/stream");

eventSource.onopen = () => {
    console.info("Connected to GPS telemetry stream");
};

eventSource.onmessage = (event) => {
    try {
        const telemetry = JSON.parse(event.data);
        updateTelemetry(telemetry);
        console.debug("GPS telemetry updated", telemetry);
    } catch (error) {
        console.error("Unable to parse GPS telemetry:", error);
    }
};

eventSource.onerror = () => {
    console.warn("GPS telemetry stream disconnected; reconnecting...");
    updateConnectionStatus(false);
};

// Show the current state immediately; the event stream handles live updates.
fetch("/api/gps")
    .then((response) => {
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        return response.json();
    })
    .then(updateTelemetry)
    .catch((error) => {
        console.error("Unable to retrieve initial GPS state:", error);
    });
