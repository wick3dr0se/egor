# egor_mobile

Mobile FFI bindings for the egor 2D graphics engine. Enables egor to run on Android and iOS by providing a C-compatible API that can be called from native mobile apps.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your App (Java/Kotlin/Swift)         │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼ C FFI
┌─────────────────────────────────────────────────────────┐
│                    egor_mobile                          │
│  - egor_init(surface, w, h)                             │
│  - egor_render(delta_ms)                                │
│  - egor_on_touch_*(x, y, id)                           │
│  - egor_cleanup()                                       │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    egor_render (wgpu)                   │
│  - Vulkan on Android                                    │
│  - Metal on iOS                                         │
└─────────────────────────────────────────────────────────┘
```

## Building

### Prerequisites

**All platforms:**
```bash
# Install cbindgen for header generation
cargo install cbindgen
```

**Android:**
```bash
# Install cargo-ndk
cargo install cargo-ndk

# Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Set NDK path (example)
export ANDROID_NDK_ROOT="$HOME/Library/Android/sdk/ndk/25.2.9519653"
```

**iOS:**
```bash
# Add iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

### Build Commands

**Android:**
```bash
cd mobile/android
./build.sh
```

Output:
- `libs/arm64-v8a/libegor_mobile.so`
- `libs/armeabi-v7a/libegor_mobile.so`
- `libs/x86_64/libegor_mobile.so`
- `libs/x86/libegor_mobile.so`

**iOS:**
```bash
cd mobile/ios
./build.sh
```

Output:
- `libs/libegor_mobile-ios.a` (device)
- `libs/libegor_mobile-ios-sim.a` (simulator)
- `libs/EgorMobile.xcframework` (universal)

**Headers:**
Generated at `include/egor_mobile.h` during build.

## API Reference

### Initialization

```c
// Initialize with native surface
// Android: ANativeWindow* from NativeActivity
// iOS: CAMetalLayer* from UIView.layer
int egor_init(void* native_surface, uint32_t width, uint32_t height);

// Clean up resources
void egor_cleanup();

// Get version string
const char* egor_version();
```

### Rendering

```c
// Render a frame (call from your render loop)
int egor_render(float delta_ms);

// Resize the surface
void egor_resize(uint32_t width, uint32_t height);

// Set clear color (RGBA, 0.0-1.0)
void egor_set_clear_color(float r, float g, float b, float a);
```

### Input

```c
// Touch events (pointer_id for multi-touch)
void egor_on_touch_down(float x, float y, int pointer_id);
void egor_on_touch_up(float x, float y, int pointer_id);
void egor_on_touch_move(float x, float y, int pointer_id);

// Key events
void egor_on_key_down(int key_code);
void egor_on_key_up(int key_code);
```

### Callbacks

```c
// Register render callback (called each frame)
typedef void (*RenderCallback)(float delta_ms, void* user_data);
void egor_set_render_callback(RenderCallback callback, void* user_data);

// Register touch callbacks
typedef void (*TouchCallback)(float x, float y, int pointer_id, void* user_data);
void egor_set_touch_callbacks(
    TouchCallback on_down,
    TouchCallback on_up,
    TouchCallback on_move,
    void* user_data
);
```

## Android Integration

### 1. Copy Libraries

Copy `mobile/android/libs/*` to your Android project:
```
app/src/main/jniLibs/
├── arm64-v8a/
│   └── libegor_mobile.so
├── armeabi-v7a/
│   └── libegor_mobile.so
├── x86_64/
│   └── libegor_mobile.so
└── x86/
    └── libegor_mobile.so
```

### 2. JNI Wrapper (Kotlin)

```kotlin
class EgorRenderer {
    companion object {
        init {
            System.loadLibrary("egor_mobile")
        }
    }

    external fun init(surface: Surface, width: Int, height: Int): Int
    external fun render(deltaMs: Float): Int
    external fun resize(width: Int, height: Int)
    external fun onTouchDown(x: Float, y: Float, pointerId: Int)
    external fun onTouchUp(x: Float, y: Float, pointerId: Int)
    external fun onTouchMove(x: Float, y: Float, pointerId: Int)
    external fun cleanup()
}
```

### 3. SurfaceView Integration

```kotlin
class EgorSurfaceView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    private val renderer = EgorRenderer()

    init {
        holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        renderer.init(holder.surface, width, height)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        renderer.resize(width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        renderer.cleanup()
    }
}
```

## iOS Integration

### 1. Add Framework

Drag `EgorMobile.xcframework` into your Xcode project.

### 2. Bridging Header (Swift)

```objc
// EgorMobile-Bridging-Header.h
#import "egor_mobile.h"
```

### 3. Metal View Integration

```swift
import MetalKit

class EgorMetalView: MTKView {
    override init(frame: CGRect, device: MTLDevice?) {
        super.init(frame: frame, device: device)
        setupEgor()
    }

    private func setupEgor() {
        guard let metalLayer = self.layer as? CAMetalLayer else { return }

        let layerPtr = Unmanaged.passUnretained(metalLayer).toOpaque()
        egor_init(layerPtr, UInt32(bounds.width), UInt32(bounds.height))
    }

    override func draw(_ rect: CGRect) {
        let deltaMs: Float = 16.67 // ~60fps
        egor_render(deltaMs)
    }

    deinit {
        egor_cleanup()
    }
}
```

## Platform Notes

### Android
- Requires API level 24+ (Android 7.0) for Vulkan support
- Uses Vulkan backend
- ANativeWindow pointer from NativeActivity or SurfaceView

### iOS
- Requires iOS 13+ for Metal support
- Uses Metal backend
- CAMetalLayer from UIView.layer
- Link against Metal.framework and QuartzCore.framework

## License

MIT (same as egor)
