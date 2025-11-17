#!/bin/bash
# Docker-based test script for VCV Rack CLI Client
#
# This script demonstrates how to test the CLI client using the Docker build environment
# as described in DOCKER_BUILD.md and DOCKER_QUICKSTART.md

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo ""
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${GREEN}======================================${NC}"
    echo ""
}

# Check if docker-build.sh exists
if [ ! -f "./docker-build.sh" ]; then
    log_error "docker-build.sh not found. Please run this script from the Rack root directory."
    exit 1
fi

log_info "VCV Rack CLI Client - Docker Test Suite"
echo ""
log_info "This script will:"
echo "  1. Build the Docker image (if needed)"
echo "  2. Build dependencies"
echo "  3. Build VCV Rack with HTTP API support"
echo "  4. Build the CLI client"
echo "  5. Run integration tests"
echo ""

read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Aborted by user"
    exit 0
fi

# Step 1: Build Docker image
log_step "Step 1: Building Docker image"
./docker-build.sh build-image

# Step 2: Build dependencies
log_step "Step 2: Building dependencies (this may take a while...)"
./docker-build.sh deps

# Step 3: Build VCV Rack
log_step "Step 3: Building VCV Rack"
./docker-build.sh build

# Step 4: Build CLI client
log_step "Step 4: Building CLI client"
./docker-build.sh exec make cli

log_success "CLI client built successfully!"
log_info "Location: tools/rack-cli"
echo ""

# Step 5: Run tests
log_step "Step 5: Running integration tests"

log_info "Starting VCV Rack with HTTP API in Docker container..."
log_info "This will start Rack in a background container."
echo ""

# Start Rack in background with HTTP API
./docker-build.sh run &
DOCKER_PID=$!

log_info "Waiting for API to be ready (30 seconds)..."
sleep 30

log_info "Testing CLI commands..."
echo ""

# Test 1: Check if API is responding
log_info "Test 1: Checking if HTTP API is responding..."
./docker-build.sh exec curl -s http://localhost:8080/api/plugins > /dev/null
if [ $? -eq 0 ]; then
    log_success "HTTP API is responding"
else
    log_error "HTTP API is not responding"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Test 2: Create test patch via API
log_info "Test 2: Creating test patch..."
VCO_RESPONSE=$(./docker-build.sh exec curl -s -X POST http://localhost:8080/api/modules \
    -H "Content-Type: application/json" \
    -d '{"pluginSlug":"Fundamental","modelSlug":"VCO-1","pos":{"x":0,"y":0}}')

VCO_ID=$(echo "$VCO_RESPONSE" | grep -o '"id":[0-9]*' | cut -d: -f2)

if [ -n "$VCO_ID" ]; then
    log_success "Created VCO module with ID $VCO_ID"
else
    log_error "Failed to create VCO module"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi

VCF_RESPONSE=$(./docker-build.sh exec curl -s -X POST http://localhost:8080/api/modules \
    -H "Content-Type: application/json" \
    -d '{"pluginSlug":"Fundamental","modelSlug":"VCF","pos":{"x":100,"y":0}}')

VCF_ID=$(echo "$VCF_RESPONSE" | grep -o '"id":[0-9]*' | cut -d: -f2)

if [ -n "$VCF_ID" ]; then
    log_success "Created VCF module with ID $VCF_ID"
else
    log_error "Failed to create VCF module"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi

# Connect modules
CABLE_RESPONSE=$(./docker-build.sh exec curl -s -X POST http://localhost:8080/api/cables \
    -H "Content-Type: application/json" \
    -d "{\"outputModuleId\":$VCO_ID,\"outputId\":0,\"inputModuleId\":$VCF_ID,\"inputId\":0}")

CABLE_ID=$(echo "$CABLE_RESPONSE" | grep -o '"id":[0-9]*' | cut -d: -f2)

if [ -n "$CABLE_ID" ]; then
    log_success "Created cable with ID $CABLE_ID"
else
    log_error "Failed to create cable"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Test 3: Run CLI list-modules
log_info "Test 3: Running 'rack-cli list-modules'"
echo ""
./docker-build.sh exec ./tools/rack-cli list-modules
echo ""
if [ $? -eq 0 ]; then
    log_success "list-modules command succeeded"
else
    log_error "list-modules command failed"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Test 4: Run CLI show-module
log_info "Test 4: Running 'rack-cli show-module $VCO_ID'"
echo ""
./docker-build.sh exec ./tools/rack-cli show-module "$VCO_ID"
echo ""
if [ $? -eq 0 ]; then
    log_success "show-module command succeeded"
else
    log_error "show-module command failed"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Test 5: Run CLI list-connections
log_info "Test 5: Running 'rack-cli list-connections'"
echo ""
./docker-build.sh exec ./tools/rack-cli list-connections
echo ""
if [ $? -eq 0 ]; then
    log_success "list-connections command succeeded"
else
    log_error "list-connections command failed"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Test 6: Run CLI list-connections with sorting
log_info "Test 6: Running 'rack-cli list-connections --sort-by type'"
echo ""
./docker-build.sh exec ./tools/rack-cli list-connections --sort-by type
echo ""
if [ $? -eq 0 ]; then
    log_success "list-connections --sort-by type command succeeded"
else
    log_error "list-connections --sort-by type command failed"
    kill $DOCKER_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Cleanup
log_info "Stopping Docker container..."
kill $DOCKER_PID 2>/dev/null || true
sleep 2

log_step "All Tests Passed!"
log_success "CLI client is working correctly in Docker environment"
echo ""
log_info "You can now use the CLI client by running:"
echo "  ./docker-build.sh exec ./tools/rack-cli <command>"
echo ""
log_info "For more information, see tools/README.md"
