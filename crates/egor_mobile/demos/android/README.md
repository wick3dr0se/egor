# Egor Mobile - Android Demo

Bouncing boxes demo showcasing egor_mobile on Android.

## Prerequisites

1. Android Studio (Arctic Fox or newer)
2. Android NDK (r25 or newer)
3. Rust with Android targets installed

## Building

### 1. Build egor_mobile for Android

```bash
# From the egor_mobile directory
cd ../../mobile/android
./build.sh
```

This creates `.so` files in `mobile/android/libs/`.

### 2. Copy libraries to demo project

```bash
# Copy to jniLibs
cp -r ../../mobile/android/libs/* app/src/main/jniLibs/
```

### 3. Open in Android Studio

1. Open Android Studio
2. File -> Open -> Select this `android` directory
3. Wait for Gradle sync
4. Build -> Make Project
5. Run on device/emulator

## What it does

- Displays colorful bouncing boxes with physics
- Tap anywhere to add more boxes!
- Boxes bounce off walls and floor
- Gravity pulls boxes down
- Each box has random color and rotation

## Project Structure

```
android/
├── app/
│   └── src/main/
│       ├── cpp/
│       │   ├── CMakeLists.txt    # Native build config
│       │   └── native-lib.cpp    # JNI bridge
│       ├── java/com/egor/demo/
│       │   └── MainActivity.kt   # Kotlin activity
│       ├── jniLibs/              # Put libegor_mobile.so here
│       └── AndroidManifest.xml
├── shared/                       # Shared demo code (symlink)
└── build.gradle
```

## Architecture

```
┌─────────────────────────────────────┐
│  MainActivity.kt (Kotlin)          │
│  - SurfaceView setup                │
│  - Touch handling                   │
│  - Render loop thread               │
└─────────────────┬───────────────────┘
                  │ JNI
┌─────────────────▼───────────────────┐
│  native-lib.cpp (C++)               │
│  - JNI function implementations     │
│  - ANativeWindow handling           │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  bouncing_boxes.c (shared demo)     │
│  - Physics simulation               │
│  - Box rendering via egor FFI       │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  libegor_mobile.so (Rust)           │
│  - wgpu rendering                   │
│  - Vulkan backend                   │
└─────────────────────────────────────┘
```
