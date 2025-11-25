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
    libasound2-dev \
    libjack-jackd2-dev \
    libpulse-dev \
    libgl1-mesa-dev \
    libssl-dev \
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
    requests \
    jsonschema

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# Accept user ID and group ID as build arguments to match host user
ARG USER_ID=1000
ARG GROUP_ID=1000

# Create build user with matching UID/GID (non-root for security)
# First remove default ubuntu user if it exists to avoid UID/GID conflicts
RUN if getent passwd ubuntu >/dev/null; then userdel -r ubuntu; fi && \
    if getent group ubuntu >/dev/null; then groupdel ubuntu; fi && \
    groupadd -g ${GROUP_ID} builder && \
    useradd -m -u ${USER_ID} -g ${GROUP_ID} -s /bin/bash builder && \
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
