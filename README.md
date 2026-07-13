<p align="center">
  <a href="https://github.com/etecoons">
    <img src="assets/header.jpg" alt="etecoons banner" width="100%">
  </a>
</p>

# Pulse

[![CI](https://github.com/etecoons/pulse/actions/workflows/ci.yml/badge.svg)](https://github.com/etecoons/pulse/actions/workflows/ci.yml)

Real-time system monitoring panel built in Rust.

## Quick Start

### Self-Hosting (Docker)
Pull and run the official Docker container:
```bash
docker run -d -p 4406:4406 -v /path/to/appdata:/app/data -v /proc:/host/proc:ro -v /sys:/sys:ro ghcr.io/etecoons/pulse:latest
```
