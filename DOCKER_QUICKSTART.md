# Docker Quick Start Guide

Build VCV Rack in 5 minutes using Docker!

## Prerequisites

- Docker installed (`docker --version`)
- Docker Compose installed (`docker-compose --version`)
- At least 5GB free disk space

## Build in 4 Steps

### 1. Build Docker Image

```bash
./docker-build.sh build-image
```

⏱️ Takes ~3 minutes

### 2. Build Dependencies

```bash
./docker-build.sh deps
```

⏱️ Takes ~15 minutes (cached for future builds)

### 3. Build VCV Rack

```bash
./docker-build.sh build
```

⏱️ Takes ~5 minutes

### 4. Run with HTTP API

```bash
./docker-build.sh run
```

🎉 VCV Rack is now running with HTTP API on http://localhost:8080

## Test the HTTP API

In another terminal:

```bash
# Test basic endpoints
curl http://localhost:8080/api/plugins
curl http://localhost:8080/api/models

# Run full test suite
./docker-build.sh test-api
```

## Common Commands

```bash
# Open interactive shell
./docker-build.sh shell

# Rebuild after code changes
./docker-build.sh build -j8

# Clean build artifacts
./docker-build.sh clean

# Full rebuild
./docker-build.sh cleandep
./docker-build.sh full -j8
```

## What's Included

✅ Ubuntu 24.04 build environment
✅ All build dependencies pre-installed
✅ Persistent build cache (fast rebuilds)
✅ HTTP API for remote control
✅ Python test suite
✅ Multi-core parallel builds

## Troubleshooting

**Port 8080 already in use?**
```bash
./docker-build.sh run --port 9000
```

**Build too slow?**
```bash
./docker-build.sh build -j$(nproc)
```

**Need to start fresh?**
```bash
docker-compose down -v
./docker-build.sh build-image
```

## Next Steps

- Read [DOCKER_BUILD.md](DOCKER_BUILD.md) for detailed documentation
- Read [HTTP_API.md](HTTP_API.md) for HTTP API documentation
- Check [CLAUDE.md](CLAUDE.md) for codebase structure

## Quick Reference

| Command | What it does |
|---------|-------------|
| `build-image` | Create Docker image |
| `deps` | Build dependencies |
| `build` | Build VCV Rack |
| `run` | Run with HTTP API |
| `shell` | Open shell in container |
| `clean` | Remove build files |
| `test-api` | Test HTTP API |

---

**Need help?** Run `./docker-build.sh help`
