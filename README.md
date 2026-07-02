# Pulse - Self-Hosted System Monitor

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/pulse/main/frontend/Assets/favicon.svg" alt="Pulse Logo" width="128" height="128">
</p>

Pulse is a minimalist, high-performance, self-hosted system monitor. It runs a lightweight background thread on the host (or container) that queries CPU, RAM, Network I/O, and GPU usage metrics, streaming them to a sci-fi heads-up display (HUD) and an interactive diagnostic terminal in real-time. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## Key Features

*   **Samus-Visor HUD**: Immersive, radial circular progress gauges representing CPU, memory, and GPU usage with high-tech glow and warning alarms.
*   **Aura Terminal**: Monospace sci-fi terminal log view that streams details and handles interactive debug commands.
*   **Auto-Adapting GPU Passthrough**: Dynamically scans for NVIDIA (`nvidia-smi`) and AMD/Intel (`sysfs`) graphics cards and adapts widgets automatically.
*   **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
*   **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.

---

## Container Registry

The Docker image is built with **Nix** (no Alpine, fully reproducible) and published to Docker Hub:

*   **Docker Hub**: [ubermetroid/pulse](https://hub.docker.com/r/ubermetroid/pulse)

---

## Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4406` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. *(Supports fallback `PULSE_SITE_TITLE`)* | `Pulse` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies to ensure redirect and websocket links are resolved correctly. | `http://localhost:4406` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `PULSE_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `PULSE_REFRESH_INTERVAL` | Metrics collection and broadcast cycle (in seconds). | `2` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `false` |
| `ENABLE_THEMES` | Enable the Super Metroid theme selector in the navigation header (true/false). | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header (true/false). | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |
| `LOCKOUT_TIME` | Bruteforce lockout duration in minutes. | `15` |
| `COOKIE_MAX_AGE` | Duration in hours that the user's PIN session cookie remains valid. | `24` |
| `SHOW_VERSION` | Display the application version number in the footer (true/false). | `true` |
| `SHOW_GITHUB` | Display the GitHub repository link in the footer (true/false). | `true` |
| `TRUST_PROXY` | Set true if deploying behind reverse proxy (Nginx, Cloudflare). | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated list of trusted proxy CIDRs/IPs. | None |
| `PULSE_MONITOR_CPU` | Enable CPU usage metrics monitoring (true/false). | `true` |
| `PULSE_MONITOR_MEMORY` | Enable system memory usage metrics monitoring (true/false). | `true` |
| `PULSE_MONITOR_STORAGE` | Enable storage metrics monitoring (true/false). | `true` |
| `PULSE_MONITOR_NETWORK` | Enable network interface throughput monitoring (true/false). | `true` |
| `PULSE_MONITOR_GPU` | Enable GPU usage metrics monitoring (true/false). | `true` |
| `PULSE_ENABLE_COFFEE` | Enable coffee easter egg command in aura terminal. | `true` |
