#!/bin/bash
set -e

TYPE=${1:-release}
HOST_TRIPLE=$2

# Pass as 2nd argument: aarch64-unknown-linux-gnu or armv7-unknown-linux-gnueabihf
# Else if blank then build for host architecture
if [ -n "$HOST_TRIPLE" ]; then
    TARGET="--target $HOST_TRIPLE"
    . $HOME/.cargo/env

    # Check if Rust toolchain is installed
    if ! rustup target list | grep -q "$HOST_TRIPLE"; then
        echo "Adding Rust target: $HOST_TRIPLE"
        rustup target add $HOST_TRIPLE
    fi
fi

# Build the project
echo "Building geckodriver with type: $TYPE and target: $HOST_TRIPLE"
if [ "$TYPE" = "release" ]; then
    cargo build --release $TARGET
else
    cargo build $TARGET
fi

# Copy the built binary to /media/host
TARGET_DIR="/opt/geckodriver/target"
if [ -z "$HOST_TRIPLE" ]; then
    TARGET_FILE="$TARGET_DIR/$TYPE/geckodriver"
else
    TARGET_FILE="$TARGET_DIR/$HOST_TRIPLE/$TYPE/geckodriver"
fi

if [ -f "$TARGET_FILE" ]; then
    echo "Copying $TARGET_FILE to /media/host"
    cp "$TARGET_FILE" /media/host
else
    echo "Error: $TARGET_FILE not found!"
    exit 1
fi
