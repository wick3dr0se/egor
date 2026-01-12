#!/bin/bash
#
# Build egor_mobile and copy to demo projects
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
EGOR_MOBILE_DIR="$SCRIPT_DIR/.."

echo "=== Building egor_mobile for demos ==="

# Build for Android
echo ""
echo "--- Building for Android ---"
if command -v cargo-ndk &> /dev/null; then
    cd "$EGOR_MOBILE_DIR/mobile/android"
    ./build.sh

    echo "Copying Android libraries..."
    mkdir -p "$SCRIPT_DIR/android/app/src/main/jniLibs"
    cp -r libs/* "$SCRIPT_DIR/android/app/src/main/jniLibs/"
    echo "Done! Libraries copied to android/app/src/main/jniLibs/"
else
    echo "SKIP: cargo-ndk not found. Install with: cargo install cargo-ndk"
fi

# Build for iOS
echo ""
echo "--- Building for iOS ---"
if [[ "$OSTYPE" == "darwin"* ]]; then
    cd "$EGOR_MOBILE_DIR/mobile/ios"
    ./build.sh

    echo "Copying iOS library..."
    mkdir -p "$SCRIPT_DIR/ios/libs"
    cp libs/libegor_mobile-ios.a "$SCRIPT_DIR/ios/libs/libegor_mobile.a"
    echo "Done! Library copied to ios/libs/"
else
    echo "SKIP: iOS build requires macOS"
fi

echo ""
echo "=== Build complete! ==="
echo ""
echo "Next steps:"
echo "  Android: Open demos/android in Android Studio"
echo "  iOS:     Run 'open demos/ios/EgorDemo.xcodeproj'"
