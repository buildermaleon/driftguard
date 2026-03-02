# DriftGuard

Real-time web monitoring and change detection system. Detects content changes, structure changes, metadata changes, and takes comparative screenshots.

## Features

- 🌐 **URL Monitoring** - Track multiple URLs with configurable intervals
- 🔍 **Change Detection** - Content, HTML structure, and metadata changes
- 📸 **Screenshot Comparison** - Visual diff between snapshots
- 📊 **Dashboard** - Web UI to view monitoring results
- 🔔 **Alerts** - Webhook notifications on changes
- 📡 **API** - REST API for integration

## Quick Start

### CLI

```bash
# Install
cargo install driftguard

# Add URL to monitor
driftguard add https://example.com --interval 3600

# Check status
driftguard status

# View changes
driftguard changes
```

### Docker

```bash
# Start the service
docker-compose up -d

# Access dashboard
open http://localhost:8080
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/monitors` | List all monitored URLs |
| POST | `/api/monitors` | Add new URL to monitor |
| DELETE | `/api/monitors/:id` | Remove URL |
| GET | `/api/changes/:monitor_id` | List detected changes |

## License

MIT
