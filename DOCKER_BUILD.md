# Docker Build Environment for VCV Rack

This document explains how to build VCV Rack using Docker on Ubuntu 24.04. The Docker environment provides a consistent, reproducible build environment that works across different host systems.

## Overview

The Docker build environment includes:

- **Ubuntu 24.04** base image
- All required build tools (GCC, Make, CMake, etc.)
- Development libraries for audio, graphics, and MIDI
- Python 3 for testing the HTTP API
- Persistent volume caching for faster rebuilds

## Prerequisites

### Required Software

1. **Docker** (version 20.10 or later)
   ```bash
   # Install Docker on Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install docker.io
   sudo systemctl start docker
   sudo systemctl enable docker

   # Add your user to docker group (optional, to avoid sudo)
   sudo usermod -aG docker $USER
   # Log out and back in for group changes to take effect
   ```

2. **Docker Compose** (version 1.29 or later)
   ```bash
   # Install docker-compose
   sudo apt-get install docker-compose

   # Or install standalone
   sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
   sudo chmod +x /usr/local/bin/docker-compose
   ```

3. **Git** (for cloning the repository)
   ```bash
   sudo apt-get install git
   ```

### System Requirements

- **Disk Space**: At least 5GB free space for Docker images and build artifacts
- **RAM**: At least 4GB RAM (8GB recommended for parallel builds)
- **CPU**: Multi-core processor recommended for parallel compilation

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/VCVRack/Rack.git
cd Rack
```

### 2. Build the Docker Image

```bash
./docker-build.sh build-image
```

This creates a Docker image with all necessary build dependencies.

### 3. Build Dependencies

```bash
./docker-build.sh deps
```

This builds all third-party libraries (GLFW, GLEW, Jansson, etc.). This step takes 10-20 minutes on first run but results are cached.

### 4. Build VCV Rack

```bash
./docker-build.sh build
```

This compiles VCV Rack itself (libRack.so and the Rack executable).

### 5. Run VCV Rack

```bash
./docker-build.sh run
```

This runs VCV Rack in headless mode with the HTTP API enabled on port 8080.

## docker-build.sh Script

The `docker-build.sh` script provides convenient commands for building and running VCV Rack.

### Commands

| Command | Description |
|---------|-------------|
| `build-image` | Build the Docker image with all dependencies |
| `shell` | Open an interactive shell in the build container |
| `deps` | Build third-party dependencies |
| `build` | Build VCV Rack (libRack.so + Rack binary) |
| `clean` | Clean build artifacts |
| `cleandep` | Clean dependency build artifacts |
| `run` | Run VCV Rack with HTTP API |
| `test-api` | Test the HTTP API with test_http_api.py |
| `full` | Full build (deps + rack) |
| `help` | Show help message |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port PORT` | HTTP API port | 8080 |
| `-j, --jobs N` | Number of parallel build jobs | 4 |

### Examples

```bash
# Build everything from scratch
./docker-build.sh build-image
./docker-build.sh full -j8

# Rebuild just VCV Rack (after code changes)
./docker-build.sh build -j8

# Open interactive shell for debugging
./docker-build.sh shell

# Run with HTTP API on custom port
./docker-build.sh run --port 9000

# Clean and rebuild
./docker-build.sh clean
./docker-build.sh build
```

## Using Docker Compose Directly

You can also use `docker-compose` commands directly:

```bash
# Build the image
docker-compose build

# Build dependencies
docker-compose run --rm rack-build make dep -j4

# Build VCV Rack
docker-compose run --rm rack-build make all -j4

# Open shell
docker-compose run --rm rack-build /bin/bash

# Run with HTTP API
docker-compose up rack-build
```

## Docker Compose Services

The `docker-compose.yml` file defines two services:

### rack-build

Main build service for compiling VCV Rack.

- **Image**: vcv-rack-build:ubuntu24.04
- **Volumes**: Source code + build cache
- **Ports**: 8080 (HTTP API)

### rack-dev

Development service with persistent separate cache.

- **Image**: Same as rack-build
- **Volumes**: Separate build cache
- **Ports**: 8081 (HTTP API)
- **Use**: For experimental builds without affecting main build

## Volume Management

Docker Compose uses named volumes to persist build artifacts:

- `rack-build-cache`: Build output for main service
- `rack-dep-cache`: Dependency libraries for main service
- `rack-dev-build-cache`: Build output for dev service
- `rack-dev-dep-cache`: Dependency libraries for dev service

### Inspecting Volumes

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect rack_rack-build-cache

# Remove volumes (clean slate)
docker volume rm rack_rack-build-cache rack_rack-dep-cache
```

### Cleaning Up

```bash
# Remove all build artifacts (inside container)
./docker-build.sh clean

# Remove dependencies (inside container)
./docker-build.sh cleandep

# Remove Docker volumes
docker-compose down -v

# Remove Docker image
docker rmi vcv-rack-build:ubuntu24.04
```

## Development Workflow

### Typical Workflow

1. **Initial Setup** (once)
   ```bash
   ./docker-build.sh build-image
   ./docker-build.sh deps
   ```

2. **Code Changes**
   - Edit code on host system using your favorite editor
   - Build changes:
     ```bash
     ./docker-build.sh build
     ```

3. **Testing**
   ```bash
   ./docker-build.sh run
   # In another terminal:
   ./docker-build.sh test-api
   ```

4. **Debugging**
   ```bash
   ./docker-build.sh shell
   # Inside container:
   make clean
   make all -j4
   gdb ./Rack
   ```

### Incremental Builds

After the initial build, subsequent builds are much faster:

```bash
# Make code changes
vim src/httpapi.cpp

# Rebuild (only changed files recompile)
./docker-build.sh build -j8

# Test
./docker-build.sh run
```

### Parallel Builds

Use multiple CPU cores for faster compilation:

```bash
# Use 8 parallel jobs
./docker-build.sh build -j8

# Use all available cores
./docker-build.sh build -j$(nproc)
```

## Interactive Development

### Opening a Shell

Get an interactive shell in the build container:

```bash
./docker-build.sh shell
```

Inside the shell, you have full access to the build environment:

```bash
# Build manually
make clean
make dep -j4
make all -j4

# Run Rack
./Rack --headless --httpapi

# Test
python3 test_http_api.py

# Debug
gdb ./Rack
```

### Editing Files

Edit files on your host system with your preferred editor. Changes are immediately visible in the container through volume mounts.

## Testing the HTTP API

### Using the Test Script

```bash
# Run all tests
./docker-build.sh test-api

# Run specific tests
docker-compose run --rm rack-build python3 test_http_api.py --test basic
docker-compose run --rm rack-build python3 test_http_api.py --test modules
docker-compose run --rm rack-build python3 test_http_api.py --test cables
```

### Manual Testing

```bash
# Start Rack with HTTP API
./docker-build.sh run

# In another terminal, test endpoints
curl http://localhost:8080/api/plugins
curl http://localhost:8080/api/models
curl http://localhost:8080/api/modules

# Create a module
curl -X POST http://localhost:8080/api/modules \
  -H "Content-Type: application/json" \
  -d '{"pluginSlug":"Fundamental","modelSlug":"VCO-1","pos":{"x":0,"y":0}}'
```

## Troubleshooting

### Build Fails

**Problem**: Dependency download fails

**Solution**: Network issues or proxy settings
```bash
# Set proxy in Dockerfile if needed
ENV http_proxy=http://proxy:port
ENV https_proxy=http://proxy:port
```

**Problem**: Out of disk space

**Solution**: Clean up Docker
```bash
docker system prune -a
docker volume prune
```

### Container Won't Start

**Problem**: Port already in use

**Solution**: Use different port
```bash
./docker-build.sh run --port 9000
```

**Problem**: Permission denied

**Solution**: Check volume permissions
```bash
# Run as root in container (not recommended for production)
docker-compose run --rm --user root rack-build /bin/bash
```

### Slow Builds

**Problem**: Builds take too long

**Solution**: Increase parallel jobs
```bash
./docker-build.sh build -j$(nproc)
```

**Solution**: Use build cache
- Don't remove volumes between builds
- Keep dependency cache (don't run cleandep)

### HTTP API Not Accessible

**Problem**: Can't connect to HTTP API

**Solution**: Check port mapping
```bash
# Verify container is running
docker ps

# Check port mapping
docker port rack-builder

# Verify inside container
docker-compose run --rm rack-build curl http://localhost:8080/api/plugins
```

## Advanced Usage

### Building for Different Architectures

The current Docker image builds for the host architecture. To cross-compile:

```bash
# Build for ARM64 on x86_64 (requires QEMU)
docker buildx build --platform linux/arm64 -t vcv-rack-build:arm64 .
```

### Custom Build Flags

Pass custom flags to the build:

```bash
docker-compose run --rm rack-build bash -c "make FLAGS='-O2 -g3' -j4"
```

### Extracting Build Artifacts

Copy built binaries from container:

```bash
# Get container ID
CONTAINER=$(docker create vcv-rack-build:ubuntu24.04)

# Copy files
docker cp $CONTAINER:/workspace/Rack ./Rack-linux
docker cp $CONTAINER:/workspace/libRack.so ./libRack.so

# Clean up
docker rm $CONTAINER
```

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build VCV Rack

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker image
        run: ./docker-build.sh build-image
      - name: Build dependencies
        run: ./docker-build.sh deps
      - name: Build VCV Rack
        run: ./docker-build.sh build
      - name: Test HTTP API
        run: ./docker-build.sh test-api
```

## Performance Comparison

Typical build times on a modern system (i7-8700K, 16GB RAM, SSD):

| Operation | First Run | Cached |
|-----------|-----------|--------|
| Build Docker image | 2-3 min | 10 sec |
| Build dependencies | 15-20 min | 30 sec |
| Build VCV Rack | 3-5 min | 30 sec |
| Full clean build | 20-25 min | 1 min |

## Security Considerations

### Running as Non-Root

The Docker image creates a `builder` user for compilation. This follows security best practices:

```dockerfile
USER builder
```

### Volume Permissions

Files created in the container are owned by the `builder` user (UID 1000). On most Linux systems, this matches your user ID.

If you encounter permission issues:

```bash
# Change ownership on host
sudo chown -R $USER:$USER .
```

### Network Isolation

The HTTP API binds to all interfaces in the container but is only exposed on localhost through port mapping.

## Maintenance

### Updating the Build Environment

To update the Docker image with new dependencies:

1. Edit `Dockerfile`
2. Rebuild image:
   ```bash
   docker-compose build --no-cache rack-build
   ```

### Updating VCV Rack

Pull latest changes and rebuild:

```bash
git pull
./docker-build.sh build
```

If dependencies changed:

```bash
./docker-build.sh cleandep
./docker-build.sh deps
./docker-build.sh build
```

## Support

For issues with:

- **Docker setup**: Check Docker documentation
- **VCV Rack build**: Check VCV Rack GitHub issues
- **HTTP API**: See HTTP_API.md documentation

## License

The Docker build environment scripts are part of VCV Rack and follow the same license terms.
