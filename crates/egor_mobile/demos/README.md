# Egor Mobile Demos

Demo applications showcasing egor_mobile's 2D rendering on Android and iOS.

## Bouncing Boxes Demo

A simple but visually impressive demo featuring:
- Colorful rectangles with physics simulation
- Gravity, bouncing, and rotation
- Touch to spawn new boxes
- Smooth 60fps rendering

## Quick Start

### 1. Build egor_mobile

```bash
# Build for all platforms
./build_demos.sh
```

Or manually:

```bash
# Android (requires cargo-ndk)
cd ../mobile/android && ./build.sh

# iOS (requires Xcode)
cd ../mobile/ios && ./build.sh
```

### 2. Run Demos

**Android:**
```bash
cd android
# Open in Android Studio and run
```

**iOS:**
```bash
cd ios
open EgorDemo.xcodeproj
# Build and run in Xcode
```

## Project Structure

```
demos/
├── shared/                    # Platform-independent demo code
│   ├── bouncing_boxes.h       # Demo API
│   └── bouncing_boxes.c       # Physics + rendering logic
├── android/                   # Android Studio project
│   ├── app/src/main/
│   │   ├── cpp/               # JNI bridge
│   │   ├── java/              # Kotlin activity
│   │   └── jniLibs/           # Put .so files here
│   └── README.md
├── ios/                       # Xcode project
│   ├── EgorDemo.xcodeproj/
│   ├── EgorDemo/              # Swift sources
│   ├── libs/                  # Put .a file here
│   └── README.md
└── build_demos.sh             # Build helper script
```

## How It Works

The demo architecture separates platform code from rendering logic:

```
┌─────────────────────────────────────────────────────────┐
│              Platform Layer (Kotlin/Swift)              │
│  - Window/Surface management                            │
│  - Touch event handling                                 │
│  - Render loop timing                                   │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│              Shared Demo Code (C)                       │
│  - Physics simulation                                   │
│  - Game state management                                │
│  - Calls egor_draw_rect() to render                     │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│              egor_mobile (Rust FFI)                     │
│  - egor_init, egor_draw_rect, egor_render_frame         │
│  - Batches geometry for efficient rendering             │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│              egor_render (Rust)                         │
│  - Full 2D rendering engine                             │
│  - Vertex/index buffers, shaders, textures              │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                    wgpu                                 │
│  - Vulkan (Android) / Metal (iOS)                       │
└─────────────────────────────────────────────────────────┘
```

## Adding Your Own Demo

1. Create a new `.c` file in `shared/`
2. Use the egor_mobile FFI functions:
   ```c
   #include "egor_mobile.h"

   void my_demo_frame(float delta_ms) {
       // Draw a red rectangle
       egor_draw_rect(100, 100, 200, 150,
                      1.0f, 0.0f, 0.0f, 1.0f,  // RGBA
                      0);  // texture_id

       // Render everything
       egor_render_frame(delta_ms);
   }
   ```
3. Call your demo functions from the platform layer
