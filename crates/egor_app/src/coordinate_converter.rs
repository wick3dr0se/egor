#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
    pub logical_width: f32,
    pub logical_height: f32,
    pub buffer_width: f32,
    pub buffer_height: f32,
}

/// Handles DPI scaling and logical-to-buffer coordinate conversion
#[derive(Clone, Copy, PartialEq)]
pub struct CoordinateConverter {
    logical_to_buffer_scale_x: f32,
    logical_to_buffer_scale_y: f32,
    scale_factor: f32,
}

impl CoordinateConverter {
    #[allow(unused)]
    pub fn new(display_info: DisplayInfo, scale_factor: f32) -> Self {
        Self {
            logical_to_buffer_scale_x: display_info.buffer_width / display_info.logical_width,
            logical_to_buffer_scale_y: display_info.buffer_height / display_info.logical_height,
            scale_factor,
        }
    }

    /// Convert window coordinates (from winit) to buffer coordinates
    pub fn window_to_buffer(&self, window_x: f32, window_y: f32) -> (f32, f32) {
        if self.scale_factor == 1.0 {
            (window_x, window_y)
        } else {
            let logical_x = window_x / self.scale_factor;
            let logical_y = window_y / self.scale_factor;
            (
                logical_x * self.logical_to_buffer_scale_x,
                logical_y * self.logical_to_buffer_scale_y,
            )
        }
    }
}

impl Default for CoordinateConverter {
    /// Default converter that does no conversion (pass-through)
    fn default() -> Self {
        Self {
            logical_to_buffer_scale_x: 1.0,
            logical_to_buffer_scale_y: 1.0,
            scale_factor: 1.0,
        }
    }
}

#[allow(unused)]
pub fn create_desktop_converter(
    physical_size: (f32, f32),
    scale_factor: f32,
) -> CoordinateConverter {
    let (physical_width, physical_height) = physical_size;

    // Logical size = physical size / scale factor
    let logical_width = physical_width / scale_factor;
    let logical_height = physical_height / scale_factor;

    // Buffer size equals physical size (common case for desktop)
    let (buffer_width, buffer_height) = physical_size;

    let display_info = DisplayInfo {
        logical_width,
        logical_height,
        buffer_width,
        buffer_height,
    };

    CoordinateConverter::new(display_info, scale_factor)
}
