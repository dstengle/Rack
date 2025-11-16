#!/bin/bash
# Docker build script for VCV Rack
# This script provides convenient commands for building VCV Rack in Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show usage
usage() {
    cat <<EOF
VCV Rack Docker Build Script

Usage: $0 <command> [options]

Commands:
    build-image         Build the Docker image
    shell              Open a shell in the build container
    deps               Build dependencies only
    build              Build VCV Rack (libRack.so + Rack binary)
    clean              Clean build artifacts
    cleandep           Clean dependencies
    run                Run VCV Rack with HTTP API
    test-api           Test the HTTP API
    full               Build everything (deps + rack)
    help               Show this help message

Options:
    -p, --port PORT    HTTP API port (default: 8080)
    -j, --jobs N       Number of parallel jobs (default: 4)

Examples:
    $0 build-image           # Build the Docker image
    $0 deps                  # Build dependencies
    $0 build                 # Build VCV Rack
    $0 full                  # Build deps + rack
    $0 run --port 9000       # Run with API on port 9000
    $0 shell                 # Open interactive shell

EOF
}

# Build Docker image
build_image() {
    info "Building Docker image..."
    docker-compose build rack-build
    success "Docker image built successfully"
}

# Open shell in container
open_shell() {
    info "Opening shell in build container..."
    docker-compose run --rm rack-build /bin/bash
}

# Build dependencies
build_deps() {
    local jobs="${1:-4}"
    info "Building dependencies with $jobs parallel jobs..."
    docker-compose run --rm rack-build bash -c "make dep -j$jobs"
    success "Dependencies built successfully"
}

# Build Rack
build_rack() {
    local jobs="${1:-4}"
    info "Building VCV Rack with $jobs parallel jobs..."
    docker-compose run --rm rack-build bash -c "make all -j$jobs"
    success "VCV Rack built successfully"
}

# Clean build artifacts
clean_build() {
    info "Cleaning build artifacts..."
    docker-compose run --rm rack-build make clean
    success "Build artifacts cleaned"
}

# Clean dependencies
clean_deps() {
    warning "This will remove all built dependencies!"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Cleaning dependencies..."
        docker-compose run --rm rack-build make cleandep
        success "Dependencies cleaned"
    else
        info "Cancelled"
    fi
}

# Run Rack with HTTP API
run_rack() {
    local port="${1:-8080}"
    info "Running VCV Rack with HTTP API on port $port..."
    info "Press Ctrl+C to stop"
    docker-compose run --rm --service-ports rack-build bash -c "./Rack --httpapi=$port"
}

# Test HTTP API
test_api() {
    local port="${1:-8080}"
    info "Testing HTTP API on port $port..."
    if ! command -v python3 &> /dev/null; then
        error "Python 3 is required for API testing"
        exit 1
    fi
    docker-compose run --rm rack-build python3 test_http_api.py --port "$port"
}

# Full build (deps + rack)
full_build() {
    local jobs="${1:-4}"
    info "Starting full build (dependencies + rack)..."
    build_deps "$jobs"
    build_rack "$jobs"
    success "Full build completed successfully"
}

# Parse command line arguments
COMMAND=""
PORT="8080"
JOBS="4"

while [[ $# -gt 0 ]]; do
    case $1 in
        build-image|shell|deps|build|clean|cleandep|run|test-api|full|help)
            COMMAND="$1"
            shift
            ;;
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -j|--jobs)
            JOBS="$2"
            shift 2
            ;;
        *)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    error "docker-compose is not installed. Please install docker-compose first."
    exit 1
fi

# Execute command
case $COMMAND in
    build-image)
        build_image
        ;;
    shell)
        open_shell
        ;;
    deps)
        build_deps "$JOBS"
        ;;
    build)
        build_rack "$JOBS"
        ;;
    clean)
        clean_build
        ;;
    cleandep)
        clean_deps
        ;;
    run)
        run_rack "$PORT"
        ;;
    test-api)
        test_api "$PORT"
        ;;
    full)
        full_build "$JOBS"
        ;;
    help|"")
        usage
        ;;
    *)
        error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
