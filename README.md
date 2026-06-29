# Pulse

A minimalist, high-performance, self-hosted system monitor container. Pulse runs a lightweight background thread on the host (or container) that queries CPU, RAM, Network I/O, and GPU usage metrics, streaming them to a sci-fi heads-up display (HUD) and an interactive diagnostic terminal in real-time.

## Features

*   **Samus-Visor HUD Mode**: Immersive, radial circular progress gauges representing CPU, memory, and GPU usage with high-tech glow and warning alarms.
*   **Aura Terminal Mode**: Monospace sci-fi terminal log view that streams details and handles interactive debug commands.
*   **Auto-Adapting GPU Passthrough**: Dynamically scans for NVIDIA (`nvidia-smi`) and AMD/Intel (`sysfs`) graphics cards and adapts widgets automatically.
*   **Security Primitives**: Features cookie-based PIN authentication, origin validation, CORS settings, HSTS, and X-Frame security headers.
*   **Zero JS Bloat**: Written in 100% Rust (Axum backend) and compiled to WebAssembly (Yew 0.23 frontend).
*   **Reproducible Nix Container**: Bundled as a secure, minimal Nix container running under `nobody:users`.

## Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | Listening Port | `4406` |
| `PULSE_SITE_TITLE` | Site Title displayed in the Header | `Pulse` |
| `PULSE_PIN` | Lock GUI with numeric PIN authentication | None |
| `PULSE_REFRESH_INTERVAL` | Metrics collection cycle (in seconds) | `2` |
| `TRUST_PROXY` | Set true if running behind a reverse proxy | `false` |

## Local Development

Start the development server:

```bash
# Start backend
cd backend && cargo run

# Start frontend (requires Trunk)
cd frontend && trunk serve
```
