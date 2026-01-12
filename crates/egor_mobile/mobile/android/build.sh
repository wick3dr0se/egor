#!/bin/bash
# Build egor_mobile for Android
#
# Prerequisites:
#   - Android NDK installed
#   - ANDROID_NDK_ROOT or ANDROID_NDK_HOME environment variable set
#   - cargo-ndk installed: cargo install cargo-ndk
#   - Rust targets: rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$CRATE_DIR/mobile/android/libs"

echo "Building egor_mobile for Android..."
echo "Crate dir: $CRATE_DIR"
echo "Output dir: $OUTPUT_DIR"

# Check for cargo-ndk
if ! command -v cargo-ndk &> /dev/null; then
    echo "Error: cargo-ndk not found. Install with: cargo install cargo-ndk"
    exit 1
fi

# Check for NDK
if [ -z "$ANDROID_NDK_ROOT" ] && [ -z "$ANDROID_NDK_HOME" ]; then
    # Try common locations
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        # Find latest NDK version
        ANDROID_NDK_ROOT=$(ls -d "$HOME/Library/Android/sdk/ndk"/*/ 2>/dev/null | sort -V | tail -1)
    elif [ -d "$HOME/Android/Sdk/ndk" ]; then
        ANDROID_NDK_ROOT=$(ls -d "$HOME/Android/Sdk/ndk"/*/ 2>/dev/null | sort -V | tail -1)
    fi
fi

NDK_PATH="${ANDROID_NDK_ROOT:-$ANDROID_NDK_HOME}"
if [ -z "$NDK_PATH" ] || [ ! -d "$NDK_PATH" ]; then
    echo "Error: Android NDK not found. Set ANDROID_NDK_ROOT or ANDROID_NDK_HOME"
    exit 1
fi

echo "Using NDK: $NDK_PATH"

# Create output directories
mkdir -p "$OUTPUT_DIR/arm64-v8a"
mkdir -p "$OUTPUT_DIR/armeabi-v7a"
mkdir -p "$OUTPUT_DIR/x86_64"
mkdir -p "$OUTPUT_DIR/x86"

cd "$CRATE_DIR"

# Find the workspace root (where Cargo.toml with [workspace] is)
WORKSPACE_ROOT="$(cd "$CRATE_DIR/../.." && pwd)"
TARGET_DIR="$WORKSPACE_ROOT/target"

# Build for each architecture
# Use -P for platform/API level
# API level 24 (Android 7.0) is minimum for Vulkan support

echo ""
echo "Building for arm64-v8a (aarch64)..."
cargo ndk -t arm64-v8a -P 24 build --release
cp "$TARGET_DIR/aarch64-linux-android/release/libegor_mobile.so" "$OUTPUT_DIR/arm64-v8a/"

echo ""
echo "Building for armeabi-v7a (armv7)..."
cargo ndk -t armeabi-v7a -P 24 build --release
cp "$TARGET_DIR/armv7-linux-androideabi/release/libegor_mobile.so" "$OUTPUT_DIR/armeabi-v7a/"

echo ""
echo "Building for x86_64..."
cargo ndk -t x86_64 -P 24 build --release
cp "$TARGET_DIR/x86_64-linux-android/release/libegor_mobile.so" "$OUTPUT_DIR/x86_64/"

echo ""
echo "Building for x86..."
cargo ndk -t x86 -P 24 build --release
cp "$TARGET_DIR/i686-linux-android/release/libegor_mobile.so" "$OUTPUT_DIR/x86/"

echo ""
echo "Android build complete!"
echo "Libraries are in: $OUTPUT_DIR"
echo ""
echo "To use in Android Studio:"
echo "  1. Copy libs/* to app/src/main/jniLibs/"
echo "  2. Copy include/egor_mobile.h to your JNI code"
echo "  3. Load library: System.loadLibrary(\"egor_mobile\")"
ls -la "$OUTPUT_DIR"/*/*.so
