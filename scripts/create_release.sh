#!/bin/bash

set -e

VERSION=${VERSION:=0.0.0}
BUILD_DIR=".build-release"
RELEASE_DIR=".release"

# Ensure clean release folder
rm -rf "$RELEASE_DIR" || true
mkdir -p "$RELEASE_DIR"

# Ensure clean build folder
rm -rf "$BUILD_DIR" || true
mkdir -p "$BUILD_DIR"

# Build release from scratch
cargo build --release --locked --verbose --target-dir "$BUILD_DIR"

# Copy binaries (or files) to release folder
cp "$BUILD_DIR/release/aliasx" "$RELEASE_DIR/aliasx-$VERSION"

# Create tarball with all builds
pushd "$RELEASE_DIR"
tar -czvf "aliasx-$VERSION.tar.gz" *
popd

echo "All builds completed successfully!"
