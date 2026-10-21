# syntax=docker/dockerfile:1.7
ARG BUILD_PLATFORM=linux/amd64
FROM --platform=${BUILD_PLATFORM} rust:latest AS builder

RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    libudev-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu wasm32-unknown-unknown

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install dioxus-cli --version "^0.7" --locked

ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
ENV AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_SYSROOT_DIR=/usr/aarch64-linux-gnu
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

WORKDIR /app
COPY . .

# Optional release profile overrides, set via --build-arg.
ARG CARGO_PROFILE_RELEASE_OPT_LEVEL
ARG CARGO_PROFILE_RELEASE_DEBUG
ARG CARGO_PROFILE_RELEASE_PANIC
ARG CARGO_PROFILE_RELEASE_STRIP
ARG CARGO_PROFILE_RELEASE_INCREMENTAL
ARG CARGO_PROFILE_RELEASE_LTO
ARG CARGO_PROFILE_RELEASE_CODEGEN_UNITS

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    rm -rf /app/target/dx/ruder_cam_beta/release/web && \
    dx bundle --release --web @server --server --target aarch64-unknown-linux-gnu && \
    mkdir -p /out && \
    cp /app/target/aarch64-unknown-linux-gnu/server-release/ruder_cam_beta /out/server && \
    cp -r /app/target/dx/ruder_cam_beta/release/web/public /out/public

FROM scratch AS artifacts
COPY --from=builder /out /

# Use: docker build --target artifacts -o type=local,dest=./OUT_DIR .
