# DriftGuard 🛡️

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.85+-dea584?style=flat&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![API](https://img.shields.io/badge/API-REST-blue.svg)]()
[![Status](https://img.shields.io/badge/Status-Active-success.svg)]()

**Real-time web monitoring and change detection system for production environments.**

</div>

## Overview

DriftGuard is a high-performance web monitoring system written in Rust that detects changes to websites in real-time. It helps you track competitors, monitor price changes, detect content updates, and receive instant notifications when changes occur.

## Features

| Feature | Description |
|---------|-------------|
| 🌐 **Multi-URL Monitoring** | Monitor unlimited URLs with configurable check intervals |
| 🔍 **Smart Change Detection** | Detects content, HTML structure, metadata, title, links, and image changes |
| 📸 **Screenshot Capture** | Visual comparison with automatic screenshots |
| 🔔 **Webhook Alerts** | Real-time notifications when changes are detected |
| 📊 **REST API** | Full programmatic control for integration |
| 💾 **SQLite Storage** | Lightweight, reliable data persistence |

## Quick Start

### Installation

```bash
# From source
cargo install driftguard

# Or use Docker
docker pull ghcr.io/buildermaleon/driftguard:latest
```

### Basic Usage

```bash
# Add a URL to monitor
driftguard add https://example.com --interval 3600

# Check status
driftguard status

# View detected changes
driftguard changes

# Start API server
driftguard serve --port 8080
```

### Docker Compose

```yaml
version: '3.8'
services:
  driftguard:
    image: ghcr.io/buildermaleon/driftguard:latest
    ports:
      - "8080:8080"
    volumes:
      - data:/data
    environment:
      - DATABASE_URL=/data/driftguard.db
```

## API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/monitors` | List all monitored URLs |
| `POST` | `/api/monitors` | Add new URL to monitor |
| `DELETE` | `/api/monitors/:id` | Remove a monitor |
| `GET` | `/api/changes/:monitor_id` | Get change history |

## Architecture

```
driftguard/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── db.rs            # SQLite database
│   ├── monitor.rs       # URL checking logic
│   ├── detector.rs      # Change detection
│   ├── screenshot.rs   # Screenshot capture
│   └── api.rs           # REST API server
├── Cargo.toml
└── Dockerfile
```

## Use Cases

- 📈 **Price Monitoring** - Track competitor pricing changes
- 📰 **Content Surveillance** - Monitor news sites, blogs for updates
- 🔒 **Compliance** - Detect changes to terms of service
- 👀 **Competitive Intelligence** - Watch competitor landing pages
- 🏥 **Status Pages** - Monitor service health endpoints

## Performance

- Written in Rust for maximum performance
- ~10MB binary size
- <50MB RAM usage
- Sub-second check times

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

Built with 🔥 by [Anvil](https://github.com/buildermaleon)

</div>
