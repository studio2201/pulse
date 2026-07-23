<h1 align="center">
  <img src="assets/icon.png?v=1.0.31" width="48" height="48" valign="middle"> Pulse
</h1>

<p align="center">
  <b>Real-time self-hosted system resource monitor and telemetry dashboard written in Rust.</b>
</p>

---

### Instant One-Line Install (Docker Container)

Run the official zero-dependency container on port 4501:

```bash
docker run -d --name pulse -p 4501:4501 -v /mnt/user/appdata/pulse:/config ghcr.io/studio2201/pulse:latest
```

Open your browser to `http://localhost:4501` to view real-time system metrics immediately.

---

### One-Line Install (Native Package Manager)

On Debian, Ubuntu, Fedora, or RHEL:

```bash
curl -fsSL https://studio2201.github.io/packages/install.sh | sudo bash
```

---

### Unraid NAS Deployment

Deploy via the official Unraid Template:

1. Copy [`pulse.xml`](pulse.xml) to your Unraid flash drive under `/boot/config/plugins/dockerMan/templates-user/`.
2. Open **Docker** -> **Add Container** -> Select **pulse** from the template dropdown.
3. Click **Apply**.

---

### Environment Configuration

The backend service can be customized using the following environment variables:

| Variable | Description | Default |
| :--- | :--- | :---: |
| `PORT` | Network port the web server binds to | `4501` |
| `PULSE_PIN` | Security PIN required for telemetry access | *(Disabled)* |
| `PULSE_DATA_DIR` | Directory path for persistent configuration and logs | `/config` |
| `PULSE_ALLOWED_ORIGINS` | CORS allowed origins list (comma-separated) | `*` |
| `TRUST_PROXY` | Honor reverse proxy headers (`X-Forwarded-For`) | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated CIDR list of trusted reverse proxies | *(None)* |
| `LOG_LEVEL` | Tracing filter (`error`, `warn`, `info`, `debug`) | `info` |

---

### Administration CLI & TUI Dashboard

Every container and package includes a built-in administration utility (`pulse`).

Launch interactive TUI dashboard:
```bash
docker exec -it pulse pulse tui
```

System diagnostics and self-healing check:
```bash
docker exec -it pulse pulse doctor
```

CLI Command Reference:
- `pulse tui` — Interactive terminal user interface.
- `pulse doctor` — Diagnoses system permissions, ports, and telemetry sources.
- `pulse status` — Displays network configuration and security parameters.
- `pulse data stats` — Shows storage utilization and log entry metrics.

---

### Architecture & Security

- **Axum Web Backend**: High-concurrency async WebSocket/HTTP runtime built on Tokio.
- **Yew WebAssembly Frontend**: Type-safe client bundle running natively in browser WASM runtime.
- **Real-Time Telemetry Engine**: Low-overhead hardware counters for CPU, RAM, Disk, and Network stats.
- **Fail-Closed Security PIN Authentication**: Rate-limited brute force protection with automatic lockout timers.

---

### License

Distributed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

### Project Banner Showcase

Official **Pulse** project banner reflecting real-time system monitoring telemetry graphs and metrics visual architecture.

<p align="center">
  <a href="https://github.com/studio2201/pulse">
    <img src="assets/pulse-header.jpg" alt="studio2201 banner" width="100%">
  </a>
</p>
