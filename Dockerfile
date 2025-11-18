# VCV Rack Build Environment - Ubuntu 24.04
FROM ubuntu:24.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Set locale
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

# Install build dependencies
RUN apt-get update && apt-get install -y \
    # Core build tools
    build-essential \
    gcc \
    g++ \
    make \
    cmake \
    git \
    wget \
    curl \
    unzip \
    tar \
    gzip \
    pkg-config \
    autoconf \
    automake \
    libtool \
    # Libraries for VCV Rack
    libx11-dev \
    libglu1-mesa-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxi-dev \
    libasound2t64-dev \
    libjack-jackd2-dev \
    libpulse-dev \
    libgl1-mesa-dev \
    # Additional utilities
    xxd \
    python3 \
    python3-pip \
    markdown \
    jq \
    vim \
    nano \
    && rm -rf /var/lib/apt/lists/*

# Install Python packages for testing
RUN pip3 install --no-cache-dir --break-system-packages \
    requests \
    jsonschema

# Create build user (non-root for security)
RUN useradd -m -s /bin/bash builder && \
    mkdir -p /workspace && \
    chown -R builder:builder /workspace

# Set up working directory
WORKDIR /workspace

# Switch to builder user
USER builder

# Set environment variables for build
ENV RACK_DIR=/workspace
ENV PATH="/workspace:${PATH}"

# Default command
CMD ["/bin/bash"]

# Labels
LABEL maintainer="VCV Rack Build"
LABEL description="Ubuntu 24.04 build environment for VCV Rack"
LABEL version="1.0"
