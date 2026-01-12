# Egor Mobile - iOS Demo

Bouncing boxes demo showcasing egor_mobile on iOS.

## Prerequisites

1. Xcode 15 or newer
2. iOS device or simulator (iOS 13+)
3. Rust with iOS targets installed

## Building

### 1. Build egor_mobile for iOS

```bash
# From the egor_mobile directory
cd ../../mobile/ios
./build.sh
```

This creates static libraries in `mobile/ios/libs/`.

### 2. Copy library to demo project

```bash
# Create libs directory and copy
mkdir -p libs
cp ../../mobile/ios/libs/libegor_mobile-ios.a libs/libegor_mobile.a

# For simulator testing:
# cp ../../mobile/ios/libs/libegor_mobile-ios-sim.a libs/libegor_mobile.a
```

### 3. Open in Xcode

```bash
open EgorDemo.xcodeproj
```

4. Select your team for signing
5. Build and run on device/simulator

## What it does

- Displays colorful bouncing boxes with physics
- Tap anywhere to add more boxes!
- Boxes bounce off walls and floor
- Gravity pulls boxes down
- Each box has random color and rotation

## Project Structure

```
ios/
├── EgorDemo.xcodeproj/
│   └── project.pbxproj
├── EgorDemo/
│   ├── AppDelegate.swift
│   ├── EgorViewController.swift
│   ├── EgorDemo-Bridging-Header.h
│   └── Info.plist
├── libs/                    # Put libegor_mobile.a here
└── shared/                  # Shared demo code (symlink)
```

## Architecture

```
┌─────────────────────────────────────┐
│  AppDelegate.swift                  │
│  EgorViewController.swift           │
│  - CAMetalLayer setup               │
│  - CADisplayLink render loop        │
│  - Touch handling                   │
└─────────────────┬───────────────────┘
                  │ Swift -> C
┌─────────────────▼───────────────────┐
│  bouncing_boxes.c (shared demo)     │
│  - Physics simulation               │
│  - Box rendering via egor FFI       │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  libegor_mobile.a (Rust)            │
│  - wgpu rendering                   │
│  - Metal backend                    │
└─────────────────────────────────────┘
```

## Troubleshooting

### "No such module" errors

Make sure the bridging header path is correct and the header search paths include:
- `$(PROJECT_DIR)/../shared`
- `$(PROJECT_DIR)/../../include`

### Linker errors

Ensure `libegor_mobile.a` is in the `libs/` directory and `Library Search Paths` includes `$(PROJECT_DIR)/libs`.

### Signing issues

Select your development team in Project Settings -> Signing & Capabilities.
