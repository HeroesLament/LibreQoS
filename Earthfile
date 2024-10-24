VERSION 0.8
PROJECT APT/LibreQoS

# Base image using Debian Bookworm
FROM debian:bookworm-slim
WORKDIR /app

# Target to install required dependencies and Rust on Debian Bookworm
base-deps:
    WORKDIR /app

    # Install system dependencies
    RUN apt-get update && \
        apt-get install -y \
        # Non *-dev packages
        build-essential \
        clang \
        cmake \
        curl \
        git \
        graphviz \
        llvm \
        mold \
        nano \
        pkg-config \
        python3-pip \
        # *-dev packages
        libbpf-dev \
        libclang-dev \
        libc6-dev-i386 \
        libelf-dev \
        libpq-dev \
        libsqlite3-dev \
        libssl-dev \
        libz-dev \
        linux-libc-dev \
        zlib1g-dev \
        && apt-get clean


    # Install Rust
    RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        . "$HOME/.cargo/env" && \
        rustup install stable && \
        rustup default stable

# Prepare production release
prepare-prod:
    FROM +base-deps
    WORKDIR /app

    COPY . .

# Target to build Rust binaries
build-rust:
    FROM +prepare-prod
    WORKDIR /app/src/rust

    # Symlink the entire 'asm', 'bits', 'sys', and 'gnu' directories from x86_64-linux-gnu to /usr/include
    # Use -sf to avoid conflicts
    RUN ln -sf /usr/include/x86_64-linux-gnu/asm /usr/include/asm
    RUN ln -sf /usr/include/x86_64-linux-gnu/bits /usr/include/bits
    RUN ln -sf /usr/include/x86_64-linux-gnu/sys /usr/include/sys
    RUN ln -sf /usr/include/x86_64-linux-gnu/gnu /usr/include/gnu

    # Clean previous builds and build all Rust binaries in release mode
    RUN . "$HOME/.cargo/env" && cargo clean && cargo update
    RUN . "$HOME/.cargo/env" && cargo build --all --release

# Pipeline to run the full Rust build process
pipeline-rust:
    BUILD +base-deps
    BUILD +build-rust