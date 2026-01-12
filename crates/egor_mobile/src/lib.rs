//! egor_mobile - Mobile FFI bindings for egor
//!
//! Provides C-compatible functions for integrating egor into Android and iOS apps.
//!
//! # Android Integration
//! Pass the ANativeWindow pointer from your native activity to `egor_init()`.
//!
//! # iOS Integration
//! Pass the CAMetalLayer pointer from your UIView to `egor_init()`.

use std::ffi::c_void;
use std::ptr;
use std::sync::Mutex;

mod renderer;

pub use egor_render::{GeometryBatch, vertex::Vertex};
pub use renderer::MobileRenderer;

// Global state (single instance for now)
// Using Mutex for safe access in Rust 2024
static RENDERER: Mutex<Option<MobileRenderer>> = Mutex::new(None);

// Pending geometry batches for current frame (texture_id -> batch)
static PENDING_BATCHES: Mutex<Vec<(usize, GeometryBatch)>> = Mutex::new(Vec::new());

/// Initialize logging for the platform
fn init_logging() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("egor"),
        );
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = env_logger::try_init();
    }
}

// ============================================================================
// C FFI Interface
// ============================================================================

/// Initialize the egor renderer with a native surface.
///
/// # Arguments
/// * `native_surface` - Platform-specific surface pointer:
///   - Android: ANativeWindow*
///   - iOS: CAMetalLayer*
/// * `width` - Surface width in pixels
/// * `height` - Surface height in pixels
///
/// # Returns
/// * 1 on success, 0 on failure
///
/// # Safety
/// The native_surface pointer must be valid for the lifetime of the renderer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_init(
    native_surface: *mut c_void,
    width: u32,
    height: u32,
) -> i32 {
    init_logging();
    log::info!("egor_init called: {}x{}", width, height);

    if native_surface.is_null() {
        log::error!("egor_init: native_surface is null");
        return 0;
    }

    match unsafe { MobileRenderer::new(native_surface, width, height) } {
        Ok(renderer) => {
            if let Ok(mut guard) = RENDERER.lock() {
                *guard = Some(renderer);
                log::info!("egor_init: success");
                1
            } else {
                log::error!("egor_init: failed to acquire lock");
                0
            }
        }
        Err(e) => {
            log::error!("egor_init failed: {}", e);
            0
        }
    }
}

/// Render a frame.
///
/// # Arguments
/// * `delta_ms` - Time since last frame in milliseconds
///
/// # Returns
/// * 1 on success, 0 on failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_render(delta_ms: f32) -> i32 {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            match renderer.render(delta_ms) {
                Ok(_) => 1,
                Err(e) => {
                    log::error!("egor_render failed: {}", e);
                    0
                }
            }
        } else {
            log::warn!("egor_render: not initialized");
            0
        }
    } else {
        log::error!("egor_render: failed to acquire lock");
        0
    }
}

// ============================================================================
// Geometry Drawing API
// ============================================================================

/// Draw a colored rectangle.
///
/// Call this between frames to queue geometry for rendering.
/// The rectangle will be drawn when `egor_render` is called.
///
/// # Arguments
/// * `x` - X position (top-left corner)
/// * `y` - Y position (top-left corner)
/// * `width` - Rectangle width
/// * `height` - Rectangle height
/// * `r`, `g`, `b`, `a` - RGBA color components (0.0 - 1.0)
/// * `texture_id` - Texture ID (use 0 for no texture / solid color)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_draw_rect(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    texture_id: u32,
) {
    let color = [r, g, b, a];
    let tex_id = texture_id as usize;

    // Create 4 vertices for the rectangle
    let vertices = [
        Vertex::new([x, y], color, [0.0, 0.0]),                          // top-left
        Vertex::new([x + width, y], color, [1.0, 0.0]),                  // top-right
        Vertex::new([x + width, y + height], color, [1.0, 1.0]),         // bottom-right
        Vertex::new([x, y + height], color, [0.0, 1.0]),                 // bottom-left
    ];

    // Two triangles: 0-1-2 and 0-2-3
    let indices = [0u16, 1, 2, 0, 2, 3];

    if let Ok(mut batches) = PENDING_BATCHES.lock() {
        // Find or create batch for this texture
        if let Some((_, batch)) = batches.iter_mut().find(|(id, _)| *id == tex_id) {
            batch.push(&vertices, &indices);
        } else {
            let mut batch = GeometryBatch::default();
            batch.push(&vertices, &indices);
            batches.push((tex_id, batch));
        }
    }
}

/// Add raw vertices and indices to the render queue.
///
/// This is a low-level function for submitting custom geometry.
/// Each vertex has: position (x, y), color (r, g, b, a), tex_coords (u, v).
///
/// # Arguments
/// * `vertices` - Pointer to vertex data (8 floats per vertex: x, y, r, g, b, a, u, v)
/// * `vertex_count` - Number of vertices
/// * `indices` - Pointer to index data (u16)
/// * `index_count` - Number of indices
/// * `texture_id` - Texture ID for this geometry
///
/// # Safety
/// Pointers must be valid and point to arrays of the specified sizes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_add_vertices(
    vertices: *const f32,
    vertex_count: u32,
    indices: *const u16,
    index_count: u32,
    texture_id: u32,
) {
    if vertices.is_null() || indices.is_null() {
        log::error!("egor_add_vertices: null pointer");
        return;
    }

    let tex_id = texture_id as usize;
    let vert_count = vertex_count as usize;
    let idx_count = index_count as usize;

    // Parse vertices (8 floats each: x, y, r, g, b, a, u, v)
    let vert_slice = unsafe { std::slice::from_raw_parts(vertices, vert_count * 8) };
    let parsed_verts: Vec<Vertex> = vert_slice
        .chunks_exact(8)
        .map(|v| {
            Vertex::new(
                [v[0], v[1]],           // position
                [v[2], v[3], v[4], v[5]], // color
                [v[6], v[7]],           // tex_coords
            )
        })
        .collect();

    // Parse indices
    let idx_slice = unsafe { std::slice::from_raw_parts(indices, idx_count) };

    if let Ok(mut batches) = PENDING_BATCHES.lock() {
        if let Some((_, batch)) = batches.iter_mut().find(|(id, _)| *id == tex_id) {
            batch.push(&parsed_verts, idx_slice);
        } else {
            let mut batch = GeometryBatch::default();
            batch.push(&parsed_verts, idx_slice);
            batches.push((tex_id, batch));
        }
    }
}

/// Clear all pending geometry.
///
/// Call this to discard any queued geometry without rendering.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_clear_geometry() {
    if let Ok(mut batches) = PENDING_BATCHES.lock() {
        batches.clear();
    }
}

/// Render a frame with all pending geometry.
///
/// This renders all geometry queued via `egor_draw_rect` and `egor_add_vertices`,
/// then clears the pending geometry for the next frame.
///
/// # Arguments
/// * `delta_ms` - Time since last frame in milliseconds
///
/// # Returns
/// * 1 on success, 0 on failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_render_frame(delta_ms: f32) -> i32 {
    // Get pending batches
    let batches = if let Ok(mut guard) = PENDING_BATCHES.lock() {
        std::mem::take(&mut *guard)
    } else {
        return 0;
    };

    // Render with geometry
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            match renderer.render_with_geometry(delta_ms, &batches) {
                Ok(_) => 1,
                Err(e) => {
                    log::error!("egor_render_frame failed: {}", e);
                    0
                }
            }
        } else {
            log::warn!("egor_render_frame: not initialized");
            0
        }
    } else {
        log::error!("egor_render_frame: failed to acquire lock");
        0
    }
}

// ============================================================================
// Surface Management
// ============================================================================

/// Handle surface resize.
///
/// # Arguments
/// * `width` - New width in pixels
/// * `height` - New height in pixels
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_resize(width: u32, height: u32) {
    log::info!("egor_resize: {}x{}", width, height);
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.resize(width, height);
        }
    }
}

/// Handle touch/mouse down event.
///
/// # Arguments
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `pointer_id` - Touch pointer ID (0 for mouse)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_on_touch_down(x: f32, y: f32, pointer_id: i32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.on_touch_down(x, y, pointer_id);
        }
    }
}

/// Handle touch/mouse up event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_on_touch_up(x: f32, y: f32, pointer_id: i32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.on_touch_up(x, y, pointer_id);
        }
    }
}

/// Handle touch/mouse move event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_on_touch_move(x: f32, y: f32, pointer_id: i32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.on_touch_move(x, y, pointer_id);
        }
    }
}

/// Handle key down event.
///
/// # Arguments
/// * `key_code` - Platform-specific key code
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_on_key_down(key_code: i32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.on_key_down(key_code);
        }
    }
}

/// Handle key up event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_on_key_up(key_code: i32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.on_key_up(key_code);
        }
    }
}

/// Set the clear color.
///
/// # Arguments
/// * `r`, `g`, `b`, `a` - RGBA components (0.0 - 1.0)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_set_clear_color(r: f32, g: f32, b: f32, a: f32) {
    if let Ok(mut guard) = RENDERER.lock() {
        if let Some(renderer) = guard.as_mut() {
            renderer.set_clear_color(r, g, b, a);
        }
    }
}

/// Clean up and release resources.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_cleanup() {
    log::info!("egor_cleanup");
    if let Ok(mut guard) = RENDERER.lock() {
        *guard = None;
    }
}

/// Get the last error message.
///
/// # Returns
/// Pointer to null-terminated error string, or null if no error.
/// The string is valid until the next egor call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_get_error() -> *const i8 {
    // TODO: Implement error message storage
    ptr::null()
}

/// Get version string.
#[unsafe(no_mangle)]
pub extern "C" fn egor_version() -> *const i8 {
    // Include null terminator
    c"0.1.0".as_ptr().cast()
}

// ============================================================================
// Callback registration for game logic
// ============================================================================

/// Function pointer type for the render callback.
/// Called each frame with delta time in milliseconds.
pub type RenderCallback = unsafe extern "C" fn(delta_ms: f32, user_data: *mut c_void);

/// Function pointer type for touch callbacks.
pub type TouchCallback = unsafe extern "C" fn(x: f32, y: f32, pointer_id: i32, user_data: *mut c_void);

static mut RENDER_CALLBACK: Option<RenderCallback> = None;
static mut RENDER_USER_DATA: *mut c_void = ptr::null_mut();

static mut TOUCH_DOWN_CALLBACK: Option<TouchCallback> = None;
static mut TOUCH_UP_CALLBACK: Option<TouchCallback> = None;
static mut TOUCH_MOVE_CALLBACK: Option<TouchCallback> = None;
static mut TOUCH_USER_DATA: *mut c_void = ptr::null_mut();

/// Register a callback for rendering.
/// The callback will be invoked each frame with the delta time.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_set_render_callback(
    callback: RenderCallback,
    user_data: *mut c_void,
) {
    unsafe {
        RENDER_CALLBACK = Some(callback);
        RENDER_USER_DATA = user_data;
    }
}

/// Register callbacks for touch events.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn egor_set_touch_callbacks(
    on_down: TouchCallback,
    on_up: TouchCallback,
    on_move: TouchCallback,
    user_data: *mut c_void,
) {
    unsafe {
        TOUCH_DOWN_CALLBACK = Some(on_down);
        TOUCH_UP_CALLBACK = Some(on_up);
        TOUCH_MOVE_CALLBACK = Some(on_move);
        TOUCH_USER_DATA = user_data;
    }
}
