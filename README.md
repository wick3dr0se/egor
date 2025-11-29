<div align="center">
<h1>egor</h1>
<p>A dead simple cross-platform 2D graphics engine</p>

![Screenshot](media/ss.png)

<a href="https://crates.io/crates/egor"><img src="https://img.shields.io/crates/v/egor?style=flat-square&color=fc8d62&logo=rust"></a>
<a href='#'><img src="https://img.shields.io/badge/Maintained%3F-Yes-green.svg?style=flat-square&labelColor=232329&color=5277C3"></img></a>  
<a href="https://opensourceforce.net/discord"><img src="https://discordapp.com/api/guilds/913584348937207839/widget.png?style=shield"/></a>

</div>

## Why Egor?

- **Stupid Simple** – You can grok the whole engine without falling into a rabbit hole
- **Cross-Platform** – Same code runs native & on the web via WASM
- **Zero Boilerplate** – Primitives, textures, input, text & more without writing a book
- **Extensible & Minimal** – If it's not required, it’s probably not included (by default)

## Features

- **Primitives**
- **Textures**
- **Input Handling**
- **Camera System**
- **Text/Fonts**

## Platform Support

| Target     | Backend(s)               | Status     |
| ---------- | ------------------------ | ---------- |
| Windows    | DX12, Vulkan, OpenGL     | ✅ Stable  |
| MacOS      | Metal, Vulkan (MoltenVK) | ✅ Stable  |
| Linux      | Vulkan, OpenGL           | ✅ Stable  |
| Web (WASM) | WebGPU, WebGL2           | ✅ Working |

> [!NOTE]
> Mobile (Android/iOS) isn't (intended to be) supported & neither is touch input

## Getting Started

Add **egor** to your project:

```bash
cargo add egor
```

Example:

```rust
let mut position = Vec2::ZERO;

App::new()
    .title("Egor Stateful Rectangle")
    .run(move |gfx, input, timer| {
        let dx = input.key_held(KeyCode::ArrowRight) as i8
            - input.key_held(KeyCode::ArrowLeft) as i8;
        let dy =
            input.key_held(KeyCode::ArrowDown) as i8 - input.key_held(KeyCode::ArrowUp) as i8;

        position += vec2(dx as f32, dy as f32) * 100.0 * timer.delta;

        gfx.rect().at(position).color(Color::RED);
    })
```

To see more of **egor** in action, checkout [demos/](demos)

> [!TIP]
> Running a demo for WASM? You’ll need to move [index.html](demos/index.html) into a demo, or just use the included [run.sh](demos/run.sh) script (see usage). It simplifies running native, WASM & hot-reload builds

### Running a Native Build

Simply run `cargo`:

```bash
cargo run
```

### Running a WASM Build

Run `trunk` (defer to [Trunk docs](https://docs.rs/crate/trunk/latest) for setup):

```bash
trunk serve
```

### Try Out Subsecond Hot-reloading

Add **egor_subsecond**:

```bash
cargo add egor_subsecond --git https://github.com/wick3dr0se/egor --package egor_subsecond
```

```rust
App:new().run(with_hot_reload(|gfx, input, timer| {...}))
```

Run `dioxus-cli` (defer to [Dioxus CLI docs](https://docs.rs/crate/dioxus-cli/latest) for setup):

```bash
dx serve --hot-patch
```

> [!NOTE]
> Subsecond hot-reloading is experimental; native is working

## Contributing

**egor** is moving fast.. Got an idea, bugfix, or question?
Check out some [issues](https://github.com/wick3dr0se/egor/issues), open a new one, drop a PR, or come hang in [Discord](https://opensourceforce.net/discord)

---

**egor** is maintained with ❤️ by [Open Source Force](https://github.com/opensource-force)
