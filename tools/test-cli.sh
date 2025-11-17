#!/bin/bash
# Test script for VCV Rack CLI Client
#
# This script tests the rack-cli tool by:
# 1. Starting VCV Rack with HTTP API enabled (in background)
# 2. Creating a simple test patch via the API
# 3. Running CLI commands to verify functionality
# 4. Cleaning up

set -e

# Configuration
RACK_BINARY="${RACK_BINARY:-./Rack}"
CLI_BINARY="${CLI_BINARY:-./tools/rack-cli}"
API_PORT="${API_PORT:-8080}"
API_HOST="${API_HOST:-localhost}"
API_BASE="http://${API_HOST}:${API_PORT}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if binaries exist
check_binaries() {
    log_info "Checking for required binaries..."

    if [ ! -f "$RACK_BINARY" ]; then
        log_error "Rack binary not found at $RACK_BINARY"
        log_info "Build it with: make"
        exit 1
    fi

    if [ ! -f "$CLI_BINARY" ]; then
        log_error "CLI binary not found at $CLI_BINARY"
        log_info "Build it with: make cli"
        exit 1
    fi

    log_success "Binaries found"
}

# Start Rack in background
start_rack() {
    log_info "Starting VCV Rack with HTTP API on port $API_PORT..."

    # Kill any existing Rack instances
    pkill -f "Rack.*httpapi" || true
    sleep 1

    # Start Rack in background
    "$RACK_BINARY" --httpapi="$API_PORT" &
    RACK_PID=$!

    log_info "Rack started with PID $RACK_PID"

    # Wait for API to be ready
    log_info "Waiting for HTTP API to be ready..."
    for i in {1..30}; do
        if curl -s "$API_BASE/api/plugins" > /dev/null 2>&1; then
            log_success "HTTP API is ready"
            return 0
        fi
        echo -n "."
        sleep 1
    done

    log_error "HTTP API did not become ready within 30 seconds"
    cleanup
    exit 1
}

# Create a test patch via API
create_test_patch() {
    log_info "Creating test patch via HTTP API..."

    # Add VCO module
    VCO_ID=$(curl -s -X POST "$API_BASE/api/modules" \
        -H "Content-Type: application/json" \
        -d '{"pluginSlug":"Fundamental","modelSlug":"VCO-1","pos":{"x":0,"y":0}}' \
        | grep -o '"id":[0-9]*' | cut -d: -f2)

    if [ -z "$VCO_ID" ]; then
        log_error "Failed to create VCO module"
        return 1
    fi
    log_success "Created VCO module with ID $VCO_ID"

    # Add VCF module
    VCF_ID=$(curl -s -X POST "$API_BASE/api/modules" \
        -H "Content-Type: application/json" \
        -d '{"pluginSlug":"Fundamental","modelSlug":"VCF","pos":{"x":100,"y":0}}' \
        | grep -o '"id":[0-9]*' | cut -d: -f2)

    if [ -z "$VCF_ID" ]; then
        log_error "Failed to create VCF module"
        return 1
    fi
    log_success "Created VCF module with ID $VCF_ID"

    # Add VCA module
    VCA_ID=$(curl -s -X POST "$API_BASE/api/modules" \
        -H "Content-Type: application/json" \
        -d '{"pluginSlug":"Fundamental","modelSlug":"VCA-1","pos":{"x":200,"y":0}}' \
        | grep -o '"id":[0-9]*' | cut -d: -f2)

    if [ -z "$VCA_ID" ]; then
        log_error "Failed to create VCA module"
        return 1
    fi
    log_success "Created VCA module with ID $VCA_ID"

    # Connect VCO -> VCF
    CABLE1_ID=$(curl -s -X POST "$API_BASE/api/cables" \
        -H "Content-Type: application/json" \
        -d "{\"outputModuleId\":$VCO_ID,\"outputId\":0,\"inputModuleId\":$VCF_ID,\"inputId\":0}" \
        | grep -o '"id":[0-9]*' | cut -d: -f2)

    if [ -z "$CABLE1_ID" ]; then
        log_error "Failed to create cable VCO->VCF"
        return 1
    fi
    log_success "Created cable VCO->VCF with ID $CABLE1_ID"

    # Connect VCF -> VCA
    CABLE2_ID=$(curl -s -X POST "$API_BASE/api/cables" \
        -H "Content-Type: application/json" \
        -d "{\"outputModuleId\":$VCF_ID,\"outputId\":0,\"inputModuleId\":$VCA_ID,\"inputId\":0}" \
        | grep -o '"id":[0-9]*' | cut -d: -f2)

    if [ -z "$CABLE2_ID" ]; then
        log_error "Failed to create cable VCF->VCA"
        return 1
    fi
    log_success "Created cable VCF->VCA with ID $CABLE2_ID"

    log_success "Test patch created successfully"

    # Export module IDs for testing
    export VCO_ID VCF_ID VCA_ID CABLE1_ID CABLE2_ID
}

# Test CLI commands
test_cli_commands() {
    log_info "Testing CLI commands..."
    echo ""

    # Test 1: list-modules
    log_info "Test 1: list-modules"
    echo "Command: $CLI_BINARY --port $API_PORT list-modules"
    echo ""
    "$CLI_BINARY" --port "$API_PORT" list-modules
    echo ""
    log_success "list-modules test passed"
    echo ""

    # Test 2: show-module
    log_info "Test 2: show-module $VCO_ID"
    echo "Command: $CLI_BINARY --port $API_PORT show-module $VCO_ID"
    echo ""
    "$CLI_BINARY" --port "$API_PORT" show-module "$VCO_ID"
    echo ""
    log_success "show-module test passed"
    echo ""

    # Test 3: list-connections (no sort)
    log_info "Test 3: list-connections"
    echo "Command: $CLI_BINARY --port $API_PORT list-connections"
    echo ""
    "$CLI_BINARY" --port "$API_PORT" list-connections
    echo ""
    log_success "list-connections test passed"
    echo ""

    # Test 4: list-connections (sort by module)
    log_info "Test 4: list-connections --sort-by module"
    echo "Command: $CLI_BINARY --port $API_PORT list-connections --sort-by module"
    echo ""
    "$CLI_BINARY" --port "$API_PORT" list-connections --sort-by module
    echo ""
    log_success "list-connections --sort-by module test passed"
    echo ""

    # Test 5: list-connections (sort by type)
    log_info "Test 5: list-connections --sort-by type"
    echo "Command: $CLI_BINARY --port $API_PORT list-connections --sort-by type"
    echo ""
    "$CLI_BINARY" --port "$API_PORT" list-connections --sort-by type
    echo ""
    log_success "list-connections --sort-by type test passed"
    echo ""

    log_success "All CLI tests passed!"
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."

    if [ -n "$RACK_PID" ]; then
        log_info "Stopping Rack (PID $RACK_PID)..."
        kill "$RACK_PID" 2>/dev/null || true
        wait "$RACK_PID" 2>/dev/null || true
    fi

    # Kill any remaining Rack processes
    pkill -f "Rack.*httpapi" || true

    log_success "Cleanup complete"
}

# Trap exit to ensure cleanup
trap cleanup EXIT INT TERM

# Main execution
main() {
    log_info "VCV Rack CLI Client Test Script"
    echo ""

    check_binaries
    start_rack
    sleep 2  # Give Rack time to fully initialize
    create_test_patch
    sleep 1  # Give patch time to stabilize
    test_cli_commands

    log_success "All tests completed successfully!"
}

# Run main if not sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
