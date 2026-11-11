# syntax=docker/dockerfile:1.7
ARG BUILD_PLATFORM=linux/amd64
FROM --platform=${BUILD_PLATFORM} rust:latest AS builder

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu

RUN rustup target add aarch64-unknown-linux-gnu

ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --target aarch64-unknown-linux-gnu && \
    mkdir -p /out && \
    cp /app/target/aarch64-unknown-linux-gnu/release/update_service /out/update_service

FROM scratch AS artifacts
COPY --from=builder /out /

# Use: docker build --target artifacts -o type=local,dest=./build .
