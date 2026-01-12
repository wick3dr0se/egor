#!/bin/bash
# Build egor_mobile for iOS
#
# Prerequisites:
#   - macOS with Xcode installed
#   - Rust targets: rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$CRATE_DIR/mobile/ios/libs"

echo "Building egor_mobile for iOS..."
echo "Crate dir: $CRATE_DIR"
echo "Output dir: $OUTPUT_DIR"

# Check we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "Error: iOS builds require macOS"
    exit 1
fi

# Check for Xcode
if ! command -v xcrun &> /dev/null; then
    echo "Error: Xcode command line tools not found"
    exit 1
fi

cd "$CRATE_DIR"

# Find the workspace root (where Cargo.toml with [workspace] is)
WORKSPACE_ROOT="$(cd "$CRATE_DIR/../.." && pwd)"
TARGET_DIR="$WORKSPACE_ROOT/target"

echo "Workspace root: $WORKSPACE_ROOT"
echo "Target dir: $TARGET_DIR"

# Create output directories
mkdir -p "$OUTPUT_DIR"

# Build for iOS device (arm64)
echo ""
echo "Building for iOS device (aarch64-apple-ios)..."
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator (arm64 for M1+ Macs)
echo ""
echo "Building for iOS simulator arm64 (aarch64-apple-ios-sim)..."
cargo build --release --target aarch64-apple-ios-sim

# Build for iOS simulator (x86_64 for Intel Macs) - optional
echo ""
echo "Building for iOS simulator x86_64 (x86_64-apple-ios)..."
cargo build --release --target x86_64-apple-ios || echo "x86_64-apple-ios build skipped (may not be installed)"

# Copy static libraries
echo ""
echo "Copying static libraries..."

cp "$TARGET_DIR/aarch64-apple-ios/release/libegor_mobile.a" "$OUTPUT_DIR/libegor_mobile-ios.a"
cp "$TARGET_DIR/aarch64-apple-ios-sim/release/libegor_mobile.a" "$OUTPUT_DIR/libegor_mobile-ios-sim-arm64.a"

if [ -f "$TARGET_DIR/x86_64-apple-ios/release/libegor_mobile.a" ]; then
    cp "$TARGET_DIR/x86_64-apple-ios/release/libegor_mobile.a" "$OUTPUT_DIR/libegor_mobile-ios-sim-x86_64.a"
fi

# Create universal library for simulator (if both archs available)
if [ -f "$OUTPUT_DIR/libegor_mobile-ios-sim-x86_64.a" ]; then
    echo ""
    echo "Creating universal simulator library..."
    lipo -create \
        "$OUTPUT_DIR/libegor_mobile-ios-sim-arm64.a" \
        "$OUTPUT_DIR/libegor_mobile-ios-sim-x86_64.a" \
        -output "$OUTPUT_DIR/libegor_mobile-ios-sim.a"
else
    cp "$OUTPUT_DIR/libegor_mobile-ios-sim-arm64.a" "$OUTPUT_DIR/libegor_mobile-ios-sim.a"
fi

# Create XCFramework (modern Apple distribution format)
echo ""
echo "Creating XCFramework..."

# Remove old xcframework if exists
rm -rf "$OUTPUT_DIR/EgorMobile.xcframework"

xcodebuild -create-xcframework \
    -library "$OUTPUT_DIR/libegor_mobile-ios.a" \
    -headers "$CRATE_DIR/include" \
    -library "$OUTPUT_DIR/libegor_mobile-ios-sim.a" \
    -headers "$CRATE_DIR/include" \
    -output "$OUTPUT_DIR/EgorMobile.xcframework"

echo ""
echo "iOS build complete!"
echo "Outputs:"
echo "  - $OUTPUT_DIR/libegor_mobile-ios.a (device)"
echo "  - $OUTPUT_DIR/libegor_mobile-ios-sim.a (simulator)"
echo "  - $OUTPUT_DIR/EgorMobile.xcframework (universal framework)"
echo ""
echo "To use in Xcode:"
echo "  1. Drag EgorMobile.xcframework into your project"
echo "  2. Add include/egor_mobile.h to your bridging header (for Swift)"
echo "  3. Link against Metal.framework and QuartzCore.framework"
ls -la "$OUTPUT_DIR"
