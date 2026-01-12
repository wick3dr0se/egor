//! Mobile renderer implementation
//!
//! Wraps egor_render::Renderer for mobile platforms, handling
//! platform-specific surface creation from raw handles.

use std::ffi::c_void;
use std::ptr::NonNull;

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use wgpu::{Instance, RequestAdapterOptions, SurfaceTargetUnsafe};

use egor_render::{GeometryBatch, Renderer};

/// Create a SurfaceTargetUnsafe from a raw platform pointer.
///
/// # Safety
/// The pointer must be valid for the lifetime of the renderer.
#[cfg(target_os = "android")]
unsafe fn create_surface_target(ptr: *mut c_void) -> Result<SurfaceTargetUnsafe, String> {
    use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle};

    let ptr =
        NonNull::new(ptr.cast()).ok_or_else(|| "Android window pointer is null".to_string())?;
    let window_handle = RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(ptr));
    let display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());

    Ok(SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: display_handle,
        raw_window_handle: window_handle,
    })
}

#[cfg(target_os = "ios")]
unsafe fn create_surface_target(ptr: *mut c_void) -> Result<SurfaceTargetUnsafe, String> {
    use raw_window_handle::{UiKitDisplayHandle, UiKitWindowHandle};

    let ptr = NonNull::new(ptr.cast()).ok_or_else(|| "iOS view pointer is null".to_string())?;
    let window_handle = RawWindowHandle::UiKit(UiKitWindowHandle::new(ptr));
    let display_handle = RawDisplayHandle::UiKit(UiKitDisplayHandle::new());

    Ok(SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: display_handle,
        raw_window_handle: window_handle,
    })
}

#[cfg(target_os = "macos")]
unsafe fn create_surface_target(ptr: *mut c_void) -> Result<SurfaceTargetUnsafe, String> {
    use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle};

    let ptr = NonNull::new(ptr.cast()).ok_or_else(|| "macOS view pointer is null".to_string())?;
    let window_handle = RawWindowHandle::AppKit(AppKitWindowHandle::new(ptr));
    let display_handle = RawDisplayHandle::AppKit(AppKitDisplayHandle::new());

    Ok(SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: display_handle,
        raw_window_handle: window_handle,
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_os = "macos")))]
unsafe fn create_surface_target(_ptr: *mut c_void) -> Result<SurfaceTargetUnsafe, String> {
    Err("Platform not supported for mobile renderer".to_string())
}

/// Mobile-specific renderer that wraps egor_render::Renderer
pub struct MobileRenderer {
    renderer: Renderer,

    // Input state
    touch_positions: [(f32, f32); 10],
    touch_active: [bool; 10],

    // Frame timing
    frame_count: u64,
}

impl MobileRenderer {
    /// Create a new mobile renderer from a native surface pointer.
    ///
    /// # Safety
    /// The native_surface_ptr must be a valid platform-specific surface pointer:
    /// - Android: ANativeWindow*
    /// - iOS: UIView* with CAMetalLayer
    /// - macOS: NSView* with CAMetalLayer
    pub unsafe fn new(
        native_surface_ptr: *mut c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        log::info!("MobileRenderer::new {}x{}", width, height);

        if native_surface_ptr.is_null() {
            return Err("native_surface_ptr is null".to_string());
        }

        // Create wgpu instance with appropriate backend
        // On Android, use GLES which is more reliable on emulators (SwiftShader Vulkan is buggy)
        // Real devices also support GLES, so this works everywhere
        #[cfg(target_os = "android")]
        let backends = wgpu::Backends::GL;
        #[cfg(target_os = "ios")]
        let backends = wgpu::Backends::METAL;
        #[cfg(target_os = "macos")]
        let backends = wgpu::Backends::METAL;
        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "macos")))]
        let backends = wgpu::Backends::all();

        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        // Create surface from raw platform handle
        let surface_target = unsafe { create_surface_target(native_surface_ptr)? };
        let surface = unsafe {
            instance
                .create_surface_unsafe(surface_target)
                .map_err(|e| format!("Failed to create surface: {}", e))?
        };

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(|e| format!("Failed to find suitable adapter: {}", e))?;

        log::info!("Adapter: {:?}", adapter.get_info());

        // Create the egor renderer using the new_from_surface method
        let renderer = pollster::block_on(Renderer::new_from_surface(surface, adapter, width, height));

        log::info!("MobileRenderer initialized successfully");

        let mut mobile_renderer = Self {
            renderer,
            touch_positions: [(0.0, 0.0); 10],
            touch_active: [false; 10],
            frame_count: 0,
        };

        // Set up orthographic projection for 2D rendering
        mobile_renderer.update_camera_matrix(width, height);

        Ok(mobile_renderer)
    }

    /// Render a frame.
    pub fn render(&mut self, _delta_ms: f32) -> Result<(), String> {
        self.frame_count += 1;

        // Begin frame
        let Some(mut frame) = self.renderer.begin_frame() else {
            return Err("Failed to begin frame".to_string());
        };

        // Begin render pass (clears with current clear color)
        {
            let _render_pass = self.renderer.begin_render_pass(&mut frame.encoder, &frame.view);
            // TODO: User can draw geometry batches here via callbacks
        }

        // End frame
        self.renderer.end_frame(frame);

        Ok(())
    }

    /// Render a frame with geometry batches.
    pub fn render_with_geometry(
        &mut self,
        _delta_ms: f32,
        batches: &[(usize, GeometryBatch)],
    ) -> Result<(), String> {
        self.frame_count += 1;

        let Some(mut frame) = self.renderer.begin_frame() else {
            return Err("Failed to begin frame".to_string());
        };

        {
            let mut render_pass = self.renderer.begin_render_pass(&mut frame.encoder, &frame.view);

            for (texture_id, batch) in batches {
                self.renderer.draw_batch(&mut render_pass, batch, *texture_id);
            }
        }

        self.renderer.end_frame(frame);

        Ok(())
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.update_camera_matrix(width, height);
        log::info!("Resized to {}x{}", width, height);
    }

    /// Update the orthographic projection matrix for 2D rendering.
    /// Maps screen coordinates (0,0 top-left to width,height bottom-right) to clip space.
    fn update_camera_matrix(&mut self, width: u32, height: u32) {
        // Create orthographic projection: top-left is (0,0), bottom-right is (width, height)
        let ortho = glam::Mat4::orthographic_rh(
            0.0,              // left
            width as f32,     // right
            height as f32,    // bottom (flipped for screen coords)
            0.0,              // top
            -1.0,             // near
            1.0,              // far
        );
        self.renderer.upload_camera_matrix(ortho);
        log::debug!("Camera matrix updated for {}x{}", width, height);
    }

    /// Set the clear color.
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.renderer
            .set_clear_color([r as f64, g as f64, b as f64, a as f64]);
    }

    /// Handle touch down.
    pub fn on_touch_down(&mut self, x: f32, y: f32, pointer_id: i32) {
        let idx = (pointer_id as usize).min(9);
        self.touch_positions[idx] = (x, y);
        self.touch_active[idx] = true;
        log::debug!("Touch down: {} @ ({}, {})", pointer_id, x, y);
    }

    /// Handle touch up.
    pub fn on_touch_up(&mut self, x: f32, y: f32, pointer_id: i32) {
        let idx = (pointer_id as usize).min(9);
        self.touch_positions[idx] = (x, y);
        self.touch_active[idx] = false;
        log::debug!("Touch up: {} @ ({}, {})", pointer_id, x, y);
    }

    /// Handle touch move.
    pub fn on_touch_move(&mut self, x: f32, y: f32, pointer_id: i32) {
        let idx = (pointer_id as usize).min(9);
        self.touch_positions[idx] = (x, y);
    }

    /// Handle key down.
    pub fn on_key_down(&mut self, key_code: i32) {
        log::debug!("Key down: {}", key_code);
    }

    /// Handle key up.
    pub fn on_key_up(&mut self, key_code: i32) {
        log::debug!("Key up: {}", key_code);
    }

    /// Get the underlying renderer for advanced usage.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Get mutable access to the underlying renderer.
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    /// Get surface dimensions.
    pub fn surface_size(&self) -> (f32, f32) {
        self.renderer.surface_size()
    }

    /// Upload camera matrix.
    pub fn upload_camera_matrix(&mut self, mat: glam::Mat4) {
        self.renderer.upload_camera_matrix(mat);
    }

    /// Add a texture from raw RGBA bytes.
    pub fn add_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> usize {
        self.renderer.add_texture_raw(w, h, data)
    }

    /// Add a texture from encoded image bytes (PNG, etc).
    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        self.renderer.add_texture(data)
    }
}
