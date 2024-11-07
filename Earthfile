VERSION 0.8
PROJECT APT/LibreQoS
FROM debian:12.7-slim
WORKDIR /app

# Define global packaging variables
ARG PACKAGE_NAME="libreqos"
ARG PACKAGE_VERSION="1.4.1"
ARG ARCHITECTURE="amd64"

# Map ARG to ENV for access in all build stages
ENV PACKAGE_NAME=$PACKAGE_NAME
ENV PACKAGE_VERSION=$PACKAGE_VERSION
ENV ARCHITECTURE=$ARCHITECTURE
ENV DPKG_DIR="dist/${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCHITECTURE}"
ENV LQOS_DIR="${DPKG_DIR}/opt/libreqos/src"
ENV LQOS_EXAMPLE_DIR="${LQOS_DIR}/bin/"
ENV DEBIAN_DIR="${DPKG_DIR}/DEBIAN"
ENV SERVICE_DIR_PATH="${DPKG_DIR}/etc/systemd/system"


base-deps:
    WORKDIR /app

    RUN apt-get update && \
        apt-get install -y \
        bpftool \
        build-essential \
        clang \
        cmake \
        curl \
        esbuild \
        git \
        graphviz \
        llvm \
        mold \
        pkg-config \
        python3-pip \
        python3-venv \
        tree \
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

# INIT function to set environment and install necessary tools
INIT:
    FUNCTION
    RUN if [ -n "$EARTHLY_CACHE_PREFIX" ]; then \
      echo "+INIT has already been called in this build environment" ; \
      exit 1; \
    fi
    IF [ "$CARGO_HOME" = "" ]
        ENV CARGO_HOME="$HOME/.cargo"
    END
    IF ! echo $PATH | grep -E -q "(^|:)$CARGO_HOME/bin($|:)"
        ENV PATH="$PATH:$CARGO_HOME/bin"
    END

    # Install cargo-sweep
    RUN if [ ! -f $CARGO_HOME/bin/cargo-sweep ]; then \
          cargo install cargo-sweep@0.7.0 --locked --root $CARGO_HOME; \
        fi;

    # Set environment variables for caching
    ARG EARTHLY_TARGET_PROJECT_NO_TAG
    ARG OS_RELEASE=$(md5sum /etc/os-release | cut -d ' ' -f 1)
    ARG cache_prefix="${EARTHLY_TARGET_PROJECT_NO_TAG}#${OS_RELEASE}#earthly-cargo-cache"
    ENV EARTHLY_CACHE_PREFIX=$cache_prefix

    ARG keep_fingerprints=false
    ENV EARTHLY_KEEP_FINGERPRINTS=$keep_fingerprints

    ARG sweep_days=4
    ENV EARTHLY_SWEEP_DAYS=$sweep_days

    ENV CARGO_INSTALL_ROOT=$CARGO_HOME
    ENV CARGO_HOME="/tmp/earthly/.cargo"

    # Create local copy of scripts with debugging
    ENV LOCAL_FUNCTIONS_HOME="/tmp/local/functions"
    RUN mkdir -p $LOCAL_FUNCTIONS_HOME

    # Local copy-output.sh with debug statements
    RUN echo 'if [ -n "$1" ]; then \
                    mkdir -p /tmp/local/lib/rust; \
                    cd target || exit 1; \
                    find . -type f -regextype posix-egrep -regex "./$1" -exec cp --parents {} /tmp/local/lib/rust \; ; \
                    cd ..; \
                fi;' > $LOCAL_FUNCTIONS_HOME/local-copy-output.sh

    RUN chmod +x $LOCAL_FUNCTIONS_HOME/local-copy-output.sh

    # Local rename-output.sh with debug statements
    RUN echo 'mkdir -p target; \
                if [ "$(find /tmp/local/lib/rust -type f -printf . | wc -c)" -eq 0 ]; then \
                    echo "No files found within ./target matching the provided output regexp"; \
                else \
                    cp -ruT /tmp/local/lib/rust target; \
                    rm -rf /tmp/local/lib/rust; \
                fi;' > $LOCAL_FUNCTIONS_HOME/local-rename-output.sh

    RUN chmod +x $LOCAL_FUNCTIONS_HOME/local-rename-output.sh

# Set cache mounts (with FUNCTION keyword)
SET_CACHE_MOUNTS_ENV:
    FUNCTION
    ARG target_cache_suffix
    ARG TARGETPLATFORM
    ARG EARTHLY_TARGET_NAME
    ENV EARTHLY_RUST_CARGO_HOME_CACHE="type=cache,mode=0777,id=$EARTHLY_CACHE_PREFIX#cargo-home,sharing=shared,target=$CARGO_HOME"
    ENV EARTHLY_RUST_TARGET_CACHE="type=cache,mode=0777,id=$EARTHLY_CACHE_PREFIX#target#$EARTHLY_TARGET_NAME#$TARGETPLATFORM#$target_cache_suffix,sharing=locked,target=target"

# Prepare production release
prepare-prod:
    FROM +base-deps
    DO +INIT --keep_fingerprints=true
    WORKDIR /app
    COPY . .

# Build Rust binaries using cache and cargo-sweep
CARGO:
    FUNCTION
    DO +SET_CACHE_MOUNTS_ENV
    ARG --required args
    ARG output
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
        set -e; \
        cargo $args; \
        cargo sweep -r -t $EARTHLY_SWEEP_DAYS; \
        cargo sweep -r -i; \
        $LOCAL_FUNCTIONS_HOME/local-copy-output.sh "$output"
    RUN $LOCAL_FUNCTIONS_HOME/local-rename-output.sh

# Target to build Rust binaries
build-rust:
    FROM +prepare-prod
    WORKDIR /app/src/rust

    # Symlink necessary directories
    RUN ln -sf /usr/include/x86_64-linux-gnu/asm /usr/include/asm
    RUN ln -sf /usr/include/x86_64-linux-gnu/bits /usr/include/bits
    RUN ln -sf /usr/include/x86_64-linux-gnu/sys /usr/include/sys
    RUN ln -sf /usr/include/x86_64-linux-gnu/gnu /usr/include/gnu

    # Use Earthly caching for Rust build
    DO +CARGO --args="build --all --release" --output="(release/[^\./]+|release/liblqos_python\.so)"

# Target to handle Python dependencies and build the virtual environment
build-python:
    FROM +build-rust
    WORKDIR /app/src
    RUN python3 -m venv venv && \
        . venv/bin/activate && \
        python3 -m pip install --upgrade pip && \
        python3 -m pip install -r ../requirements.txt

# Target to package as Debian .deb
package-deb:
    FROM +build-rust
    WORKDIR /app/src

    # Create packaging structure
    RUN mkdir -p $DEBIAN_DIR $LQOS_DIR/bin/static2

    # Copy Debian packaging files from version-controlled directory
    RUN mkdir -p $DPKG_DIR/DEBIAN && \
        cp ../packaging/control $DPKG_DIR/DEBIAN/control && \
        cp ../packaging/postinst $DPKG_DIR/DEBIAN/postinst && \
        cp ../packaging/postrm $DPKG_DIR/DEBIAN/postrm && \
        chmod +x $DPKG_DIR/DEBIAN/postinst $DPKG_DIR/DEBIAN/postrm && \
        ls -la $DPKG_DIR/DEBIAN  # Debug check

    # Copy Rust binaries
    RUN cp rust/target/release/* $LQOS_DIR/bin/

    # Move liblqos_python.so to the correct directory
    RUN mv $LQOS_DIR/bin/liblqos_python.so $LQOS_DIR/

    RUN mkdir -p $LQOS_DIR $SERVICE_DIR_PATH $LQOS_EXAMPLE_DIR && \
        while IFS= read -r entry; do \
            target=$(echo "$entry" | cut -d':' -f1); \
            file=$(echo "$entry" | cut -d':' -f2); \
            echo "Processing: target=$target, file=$file"; \
            if [ "$target" = "src" ]; then \
                cp "$file" "$LQOS_DIR"; \
                echo "$(ls -la ${LQOS_DIR})"; \
            elif [ "$target" = "svc" ]; then \
                cp "$file" "$SERVICE_DIR_PATH"; \
                echo "$(ls -la ${SERVICE_DIR_PATH})"; \
            elif [ "$target" = "example" ]; then \
                echo "Attempting to copy example file: $file"; \
                if [ -f "$file" ]; then \
                    cp "$file" "$LQOS_EXAMPLE_DIR"; \
                    echo "Copied to ${LQOS_EXAMPLE_DIR}"; \
                else \
                    echo "File not found: $file"; \
                fi; \
                echo "$(ls -la ${LQOS_EXAMPLE_DIR})"; \
            else \
                echo "Unknown target: $target"; \
            fi; \
        done < ../packaging/filelist


    # Copy web resources
    RUN mkdir -p $LQOS_DIR/bin && \
        cp rust/lqos_node_manager/Rocket.toml $LQOS_DIR/bin && \
        mkdir -p $LQOS_DIR/bin/static && \
        cp -R rust/lqos_node_manager/static/* $LQOS_DIR/bin/static && \
        # Debugging: list the contents of the target directory
        ls -la $LQOS_DIR/bin/static

    # Debugging: print the tree of the DPKG_DIR
    RUN echo "Directory tree of $DPKG_DIR:" && \
        tree $DPKG_DIR

    # Final assembly into Debian package
    RUN echo "Building .deb package in $DPKG_DIR" && \
        ls -la $DPKG_DIR && \
        dpkg-deb --root-owner-group --verbose --build "$DPKG_DIR"

# Target to copy the output artifacts to a local artifacts directory
copy-artifacts:
    FROM +package-deb
    WORKDIR /app/src

    RUN DEB_PATH="dist/${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCHITECTURE}.deb" && \
        echo "$DEB_PATH" > /tmp/deb_path.txt

    # Save the built .deb package as a local artifact using variables
    SAVE ARTIFACT $(cat /tmp/deb_path.txt) AS LOCAL artifacts/

# Target to push .deb to staging server and install locally
push-staging:
    # Run the dev pipeline to test, build, and package
    BUILD +ci-pipeline

    # Use a more complete base image with SCP and SSH for deployment
    FROM ubuntu:latest
    WORKDIR /app

    # Install necessary tools for deployment
    RUN apt-get update && \
        apt-get install -y openssh-client

    COPY artifacts/libreqos_1.4.1_amd64.deb /app/

    # Use SCP to copy the .deb package to the remote staging server
    RUN scp /app/libreqos_1.4.1_amd64.deb mwynkoop@172.23.0.6:/tmp/

# Pipeline to run the full Rust build process
ci-pipeline:
    BUILD +base-deps
    BUILD +build-rust
    BUILD +package-deb
    BUILD +copy-artifacts