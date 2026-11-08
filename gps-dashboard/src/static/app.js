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

const map = L.map("map", { zoomControl: true }).setView([39.0, -96.0], 4);

L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 19,
    attribution:
        '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

let positionMarker = null;
let flightPath = null;
let hasCenteredMap = false;

const trackPoints = [];
const MAX_TRACK_POINTS = 500;
const mapPlaceholder = document.getElementById("map-placeholder");

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
        if (positionMarker === null) {
            mapPlaceholder.classList.remove("hidden");
        }

        return;
    }

    const position = [latitude, longitude];

    mapPlaceholder.classList.add("hidden");
    setText("latitude", formatCoordinate(latitude, "N", "S"));
    setText("longitude", formatCoordinate(longitude, "E", "W"));
    setText("map-coordinates", `${latitude.toFixed(6)}, ${longitude.toFixed(6)}`);

    if (positionMarker === null) {
        positionMarker = L.circleMarker(position, {
            radius: 8,
            color: "#ffffff",
            weight: 2,
            fillColor: "#38bdf8",
            fillOpacity: 1,
        })
            .addTo(map)
            .bindPopup("Current GPS position");

        flightPath = L.polyline([], {
            color: "#38bdf8",
            weight: 3,
            opacity: 0.85,
        }).addTo(map);
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
        map.setView(position, 16);
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
