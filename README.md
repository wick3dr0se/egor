<div align="center">
<h1>egor</h1>
<p>A dead simple cross-platform 2D graphics engine</p>

![Screenshot](media/ss.png)

<a href="https://crates.io/crates/egor"><img src="https://img.shields.io/crates/v/egor?style=flat-square&color=fc8d62&logo=rust"></a>
<a href='#'><img src="https://img.shields.io/badge/Maintained%3F-Yes-green.svg?style=flat-square&labelColor=232329&color=5277C3"></img></a>  
<a href="https://opensourceforce.net/discord"><img src="https://discordapp.com/api/guilds/913584348937207839/widget.png?style=shield"/></a>

</div>

## Why Egor?

**Egor** is dead **simple**, **lightweight** and **cross-platform**. The same code runs on desktop, web (WASM) and mobile with minimal boilerplate. It's built from small, composable crates on top of modern graphics and windowing abstractions

**Egor** gives you the essentials for 2D apps and games:

- Efficient 2D rendering (shapes, textures, text)
- Clipping, draw layers and world/screen space
- Keyboard & mouse input
- Camera & world-space transforms
- Interactive UIs with optional **egui** integration
- Optional hot-reload during development

## Platform Support

| Target     | Backend(s)               | Status     |
| ---------- | ------------------------ | ---------- |
| Windows    | DX12, Vulkan, OpenGL     | ✅ Stable  |
| MacOS      | Metal, Vulkan (MoltenVK) | ✅ Stable  |
| Linux      | Vulkan, OpenGL           | ✅ Stable  |
| Web (WASM) | WebGPU, WebGL2           | ✅ Stable  |
| Android    | Vulkan, OpenGL           | ✅ Working |

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
    .run(move |FrameContext { gfx, input, timer, .. } | {
        let dx = input.key_held(KeyCode::ArrowRight) as i8
            - input.key_held(KeyCode::ArrowLeft) as i8;
        let dy =
            input.key_held(KeyCode::ArrowDown) as i8 - input.key_held(KeyCode::ArrowUp) as i8;

        position += vec2(dx as f32, dy as f32) * 100.0 * timer.delta;

        gfx.rect().at(position).color(Color::RED);
    })
```

For full documentation see the [official docs](https://docs.rs/egor/latest)

### Running a Native Build

Simply run `cargo`:

```bash
cargo run
```

### Running a WASM Build

**Setup**

Create an `index.html` next to your `Cargo.toml`:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <link data-trunk rel="rust" />
    <style>
      body {
        margin: 0;
      }
      canvas {
        display: block;
        width: 100vw;
        height: 100vh;
      }
    </style>
  </head>
</html>
```

Run `trunk` (defer to [Trunk docs](https://docs.rs/crate/trunk/latest) for setup):

```bash
trunk serve
```

### Running an Android Build

**Setup**

Make sure your crate is a library:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Have a lib.rs with your `egor::main` function:

```rust
egor::main!(main);
pub fn main() {
    App::new().run(...)
}
```

The `egor::main!(main)` macro defines the required `android_main()` entry point on Android and forwards execution to your `main()` function. On other platforms it expands to nothing (see the secs_particles demo for reference)

Add an Android build target (if needed):

```bash
rustup target add aarch64-linux-android
```

Egor defers Android toolchain setup and build workflow to xbuild. See the [xbuild - getting started](https://github.com/rust-mobile/xbuild?tab=readme-ov-file#getting-started) for SDK/NDK and device setup

Build with `xbuild`

```bash
x build
```

### Try Out Subsecond Hot-reloading

There are two ways to reload while the app runs. The first needs no tooling at all.

**Assets.** Read the file when you draw it and the edit is live on the next frame.
[demos/hot_postfx](demos/hot_postfx) reads `shaders/{effect}.wgsl` every frame, so a shader edit
shows up in a running app under plain `cargo run`.

**Code.** Compile with the `hot_reload` feature, which wraps `AppHandler::update`, and run the
[Dioxus CLI](https://docs.rs/crate/dioxus-cli/latest). Keep the CLI in the 0.7 line to match
`dioxus-devtools 0.7`.

```bash
dx serve --hot-patch
```

Frame logic and state both survive a patch. When you change how the state is built and want the old
one thrown away, diff the setup function's address each frame:

```rust
use egor::hot::{HotFn, HotFnPtr};

let mut game: Option<Game> = None;
let mut game_ptr: Option<HotFnPtr> = None;

// in the frame
let mut setup = HotFn::current(new_game);
let ptr = setup.ptr_address();
if game_ptr != Some(ptr) {
    game_ptr = Some(ptr);
    game = Some(setup.call((&mut ctx.gfx,)));
}
let game = game.as_mut().unwrap();
```

The setup function has to be a function item, not a closure, and everything the state needs belongs
in it. Editing a struct field used elsewhere, without touching that function, is still unsafe.
[demos/hot_postfx](demos/hot_postfx) uses this, holding the effect name in its setup function so
the rebuild shows on screen.

> [!NOTE]
> Subsecond hot-reloading is experimental; native is working. The watcher needs a terminal: piping
> `dx serve` to a file runs the app but never rebuilds, and the first patch after it starts can land
> one save late.

### Rendering Without a Window

`egor::render::render_image` draws one frame into a buffer, no window and no event loop:

```rust
let pixels = egor::render::render_image(640, 360, |gfx| {
    gfx.clear(Color::BLACK);
    gfx.rect().screen().at(pos).size(size).color(Color::WHITE);
    gfx.text("hello").at(pos).size(32.0);
});
```

It returns tightly packed RGBA8, top row first, `width * height * 4` bytes, ready for a texture or an encoder. Shapes, text and clipping behave as they do in a window, which makes it usable for thumbnails, image tests and hosts with no display.

## Performance

To stress test **egor**, [ferrismark](demos/ferrismark) is a bunnymark like demo: 129,989 Ferris crabs at 55 FPS on a Ryzen 5 1600 and a Radeon Pro WX 2100.

| Radeon Pro WX 2100, 129,989 crabs | llvmpipe, 10,000 crabs |
| --- | --- |
| <img src="media/ferrismark.png" height="320" /> | <img src="media/ferrismark-sw.png" height="320" /> |

### Software Rendering

wgpu picks the adapter, so a CPU driver works like any other. Mesa's lavapipe is its software Vulkan driver, packaged as `vulkan-swrast` on Arch:

```bash
sudo pacman -S vulkan-swrast
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo xtask run ferrismark --features log
```

`RUST_LOG=info` prints the adapter it picked: `adapter: llvmpipe (LLVM 23.1.5, 256 bits) (Vulkan, Cpu, driver llvmpipe)`.

## Demos

To try ferrismark or see more of **egor** in action, check out [demos/](demos)

Run any demo with (see `--help` usage) xtask:

```bash
cargo xtask run <demo>
```

## Contributing

**egor** is moving fast. Before opening a PR or submitting a change, please read [CONTRIBUTING.md](CONTRIBUTING.md)

Check out some [issues](https://github.com/wick3dr0se/egor/issues), open a new one, drop a PR or come hang in [Discord](https://opensourceforce.net/discord)

---

**egor** is maintained with ❤️ by [Open Source Force](https://github.com/opensource-force)
