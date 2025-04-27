<div align="center">
<h1>egor</h1>
<p>A stupid simple cross-platform 2D graphics engine</p>
<img src="examples/assets/ss.png"/>

<a href="https://crates.io/crates/egor"><img src="https://img.shields.io/crates/v/egor?style=flat-square&color=fc8d62&logo=rust"></a>
<a href='#'><img src="https://img.shields.io/badge/Maintained%3F-Yes-green.svg?style=flat-square&labelColor=232329&color=5277C3"></img></a>
<a href="https://opensourceforce.net/discord"><img src="https://discordapp.com/api/guilds/913584348937207839/widget.png?style=shield"/></a>
</div>

## Features
Component | Highlight
---|---
**Windowing** | Pure Rust (no GLFW/SDL), WASM-ready
**Primitive** & **Texture Rendering** | Zero-config builders with NDC conversion
**Input Handling** | Snapshot polling (no callbacks)
**Timing** | Delta time + FPS tracking


## Getting Started
Get **egor**
```bash
cargo add --git https://github.com/wick3dr0se/egor
```

Example: Creating a window, clearing the surface and drawing a circle with 100 segments
```rust
App::init(|_ctx| {})
    .run(|ctx| {
        ctx.renderer.clear(Color::GREEN);

        ctx.renderer.circle().segments(100).draw();
    });
```

See more examples in [examples/](examples/)

You can run any examples with `cargo` or run the cross-platform example via `bash run.sh [native|webgpu|webgl] [debug|relesae]`. The script will execute either the former or `trunk serve` to run a web build, based on the arguments passed. It'll also setup a WASM target and `trunk` if needed

## Platform Support
Target | Backend(s) | Status
---|---|---
Windows	| DX12, Vulkan, OpenGL | ✅ Working
MacOS | Metal, Vulkan (MoltenVK) | ✅ Working
Linux | Vulkan, OpenGL | ✅ Working
Web (WASM) | WebGPU, WebGL2 | ✅ Working
Android | Vulkan, OpenGL ES | ⏳ Future
iOS | Metal | ⏳ Future

## Roadmap
- **Text**
- **Custom Shaders**
- **Blend Modes**
- **Camera System**
- **Immediate UI**

## Contributing
Egor could always use help.. Feel free to open an issue or PR. Contributions are much appreciated!