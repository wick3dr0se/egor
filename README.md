<div align="center">
<h1>egor</h1>
<p>A dead simple cross-platform 2D graphics engine</p>

![Screenshot](media/ss.png)

<a href="https://crates.io/crates/egor"><img src="https://img.shields.io/crates/v/egor?style=flat-square&color=fc8d62&logo=rust"></a>
<a href='#'><img src="https://img.shields.io/badge/Maintained%3F-Yes-green.svg?style=flat-square&labelColor=232329&color=5277C3"></img></a>
<a href="https://opensourceforce.net/discord"><img src="https://discordapp.com/api/guilds/913584348937207839/widget.png?style=shield"/></a>
</div>

## Why Egor?
- **Stupid Simple** – You can grok the whole engine without diving into a rabbit hole
- **Cross-Platform** – Same code runs native & on the web via WASM
- **Zero Boilerplate** – Primitives, textures & input without writing a book
- **Minimalist by Design** – If it's not required, it’s probably not here

## Features
- **Primitives**
- **Textures**
- **Input Handling**
- **Camera System**
- **Font**

## Platform Support
Target | Backend(s) | Status
---|---|---
Windows	| DX12, Vulkan, OpenGL | ✅ Stable
MacOS | Metal, Vulkan (MoltenVK) | ✅ Stable
Linux | Vulkan, OpenGL | ✅ Stable
Web (WASM) | WebGPU, WebGL2 | ✅ Working

> Mobile (Android/iOS) isn't supported. It's theoretically possible but life is finite

## Getting Started
Get **egor**
```bash
cargo add egor
```

## Roadmap
- **Custom Shaders**
- **Blend Modes**
- **UI**

## Contributing
Egor could always use help.. Feel free to open an issue or PR. Contributions are much appreciated!