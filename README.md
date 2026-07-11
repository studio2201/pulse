<p align="center">
  <a href="https://github.com/etecoons">
    <img src="assets/header.jpg" alt="etecoons banner" width="100%">
  </a>
</p>

# Pulse — Self-Hosted System Monitor <img src="https://raw.githubusercontent.com/etecoons/unraid-apps/main/icons/pulse.png" width="48" height="48" alt="pulse logo" align="right">

Pulse is a minimalist, high-performance, self-hosted system monitor. It runs a lightweight background thread on the host (or container) that queries CPU, RAM, Network I/O, and GPU usage metrics, streaming them to a sci-fi heads-up display (HUD) and an interactive diagnostic terminal in real-time. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## Architecture & Stack
* **Frontend**: Yew (WASM)
* **Backend**: Axum (Rust) / Tokio (SSE Broadcaster)
* **Deployment**: UBI container (Red Hat UBI9) on Docker Hub / Unraid / Podman / Docker Compose

---

## Key Features
* **Samus-Visor HUD**: Immersive, radial circular progress gauges representing CPU, memory, and GPU usage with high-tech glow and warning alarms.
* **Aura Terminal**: Monospace sci-fi terminal log view that streams details and handles interactive debug commands.
* **Auto-Adapting GPU Passthrough**: Dynamically scans for NVIDIA (`nvidia-smi`) and AMD/Intel (`sysfs`) graphics cards and adapts widgets automatically.
* **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
* **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.

---

## Deployment & Installation

### Container images (Docker Hub)

Images are **UBI9-minimal** based (Red Hat Universal Base Image). Tags:

| Tag | Meaning |
| :--- | :--- |
| `latest` | Current recommended build |
| `ubi` | Explicit UBI image (same lineage as `latest`) |
| `1.3.34` | Immutable release pin |

```bash
# Pull examples
podman pull docker.io/ghcr.io/etecoons/pulse:latest
podman pull docker.io/ghcr.io/etecoons/pulse:ubi
podman pull docker.io/ghcr.io/etecoons/pulse:1.3.34
```

Hub: [https://hub.docker.com/r/ghcr.io/etecoons/pulse](https://hub.docker.com/r/ghcr.io/etecoons/pulse)

### Docker Compose
Create a `docker-compose.yml` file with the following service definition:

```yaml
services:
 pulse:
 image: ghcr.io/etecoons/pulse:latest
 container_name: pulse
 restart: unless-stopped
 ports:
 - ${PORT:-4406}:4406
 volumes:
 - ${PULSE_DATA_PATH:-./data}:/app/data
 # Optional: bind-mount host proc for accurate metrics inside docker
 - /proc:/host/proc:ro
 # Optional: pass through sysfs for AMD/Intel GPU detection
 - /sys:/sys:ro
 environment:
 PORT: 4406
 SITE_TITLE: ${PULSE_SITE_TITLE:-Pulse}
 PULSE_PIN: ${PULSE_PIN:-}
 BASE_URL: ${PULSE_BASE_URL:-http://localhost:4406}
 ALLOWED_ORIGINS: ${PULSE_ALLOWED_ORIGINS:-*}
 TZ: ${TZ:-UTC}
 ENABLE_TRANSLATION: ${ENABLE_TRANSLATION:-false}
 ENABLE_THEMES: ${ENABLE_THEMES:-true}
 MAX_ATTEMPTS: ${MAX_ATTEMPTS:-5}
 PULSE_REFRESH_INTERVAL: ${PULSE_REFRESH_INTERVAL:-2}
```

### Build the UBI image locally

Requires [Podman](https://podman.io/) (or Docker) and network access to pull base images and crates.

```bash
# From the repository root
podman build --format docker -f Containerfile.ubi \
 -t docker.io/ghcr.io/etecoons/pulse:1.3.34 \
 -t docker.io/ghcr.io/etecoons/pulse:latest \
 -t docker.io/ghcr.io/etecoons/pulse:ubi \
 .

# Optional: push all three tags
podman push docker.io/ghcr.io/etecoons/pulse:1.3.34
podman push docker.io/ghcr.io/etecoons/pulse:latest
podman push docker.io/ghcr.io/etecoons/pulse:ubi
```

---

## Configuration Options

| Environment Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4406` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. | `Pulse` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies. | `http://localhost:4406` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). | `*` |
| `PULSE_PIN` | Optional 4–10 digit numerical PIN to lock access to the interface. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `PULSE_REFRESH_INTERVAL` | Metrics collection and broadcast cycle (in seconds). | `2` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header. | `false` |
| `ENABLE_THEMES` | Enable the Super Metroid theme selector in the navigation header. | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header. | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before rate lockout. | `5` |
| `LOCKOUT_TIME` | Bruteforce lockout duration in minutes. | `15` |
| `COOKIE_MAX_AGE` | Duration in hours that the user's PIN session cookie remains valid. | `24` |
| `SHOW_VERSION` | Display the application version number in the footer. | `true` |
| `SHOW_GITHUB` | Display the GitHub repository link in the footer. | `true` |
| `TRUST_PROXY` | Set true if deploying behind reverse proxy (Nginx, Cloudflare). | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated list of trusted proxy CIDRs/IPs. | None |
| `PULSE_MONITOR_CPU` | Enable CPU usage metrics monitoring. | `true` |
| `PULSE_MONITOR_MEMORY` | Enable system memory usage metrics monitoring. | `true` |
| `PULSE_MONITOR_STORAGE` | Enable storage metrics monitoring. | `true` |
| `PULSE_MONITOR_NETWORK` | Enable network interface throughput monitoring. | `true` |
| `PULSE_MONITOR_GPU` | Enable GPU usage metrics monitoring. | `true` |
| `PULSE_ENABLE_COFFEE` | Enable coffee easter egg command in aura terminal. | `true` |

---

## Local Development

Ensure you have the Rust toolchain and Trunk installed.

```bash
# 1. Run workspace tests
cargo test

# 2. Run clippy workspace checks
cargo clippy --workspace --all-targets

# 3. Start frontend Yew dev server (from frontend/)
cd frontend && trunk serve

# 4. Start backend Axum server (from backend/)
cd backend && cargo run
```

---

## License
Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 etecoons.
