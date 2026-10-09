# Rust and Cargo run inside GitHub Codespaces, not on the host computer.
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

USER root

# Match the native Linux dependencies installed by .github/workflows/ci.yml.
# These enable building PdfCraft's egui/winit/rfd desktop components.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        pkg-config \
        libxkbcommon-dev \
        libwayland-dev \
        libx11-dev \
        libxrandr-dev \
        libxi-dev \
        libgl1-mesa-dev \
        libgtk-3-dev \
    && rm -rf /var/lib/apt/lists/*
