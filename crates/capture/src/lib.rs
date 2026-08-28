use windows::{
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::GetDesktopWindow,
};

/// Represents a captured region of the screen as raw RGBA pixels.
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    /// Raw BGRA pixel data, row by row, from top-left to bottom-right.
    pub data: Vec<u8>,
}

impl CapturedFrame {
    /// Convert to an RgbaImage usable by the `image` crate.
    pub fn into_rgba_image(self) -> image::RgbaImage {
        // BitBlt gives us BGRA, so we need to swap B and R channels
        let mut pixels = self.data;
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // swap B <-> R
        }
        image::RgbaImage::from_raw(self.width, self.height, pixels)
            .expect("Buffer dimensions must match width * height * 4")
    }

    /// Encode this frame to lossless PNG bytes.
    ///
    /// PNG is preferred over JPEG for screen captures because it is lossless,
    /// preserving sharp text, UI edges and icons exactly — which matters
    /// for image recognition and OCR.
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, String> {
        // Clone and swap BGR -> RGB channel order for the image crate
        let mut pixels = self.data.clone();
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // B <-> R
        }
        let rgba_img = image::RgbaImage::from_raw(self.width, self.height, pixels)
            .ok_or("Invalid pixel buffer dimensions")?;

        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        image::ImageEncoder::write_image(
            encoder,
            rgba_img.as_raw(),
            self.width,
            self.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG encoding failed: {}", e))?;

        Ok(png_bytes)
    }
}

/// Capture a rectangular region of the screen using Win32 GDI BitBlt.
///
/// `x`, `y` are the top-left corner in virtual screen coordinates.
/// `width` and `height` define the size of the rectangle.
pub fn capture_region(x: i32, y: i32, width: i32, height: i32) -> Result<CapturedFrame, String> {
    if width <= 0 || height <= 0 {
        return Err(format!(
            "Invalid capture dimensions: {}x{}",
            width, height
        ));
    }

    unsafe {
        // Get the Device Context for the entire desktop
        let hwnd_desktop = GetDesktopWindow();
        let hdc_screen = GetDC(Some(hwnd_desktop));
        if hdc_screen.is_invalid() {
            return Err("Failed to get screen Device Context".to_string());
        }

        // Create a compatible in-memory DC and bitmap to copy pixels into
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            ReleaseDC(Some(hwnd_desktop), hdc_screen);
            return Err("Failed to create compatible Memory Device Context".to_string());
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(Some(hwnd_desktop), hdc_screen);
            return Err("Failed to create compatible bitmap".to_string());
        }

        // Select the bitmap into our memory DC so BitBlt writes into it
        let old_bm = SelectObject(hdc_mem, hbm.into());

        // Bit-Block Transfer: copy pixels from screen DC into memory DC
        let blt_result = BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
        );

        if blt_result.is_err() {
            SelectObject(hdc_mem, old_bm);
            let _ = DeleteObject(hbm.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(Some(hwnd_desktop), hdc_screen);
            return Err("BitBlt screen capture failed".to_string());
        }

        // Read the raw pixel data out of the bitmap
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Negative = top-down (row 0 at top)
                biPlanes: 1,
                biBitCount: 32, // 32-bit BGRA
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let buf_size = (width * height * 4) as usize;
        let mut pixel_data: Vec<u8> = vec![0u8; buf_size];

        let lines = GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            Some(pixel_data.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI resources
        SelectObject(hdc_mem, old_bm);
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(Some(hwnd_desktop), hdc_screen);

        if lines == 0 {
            return Err("GetDIBits returned 0 scan lines".to_string());
        }

        Ok(CapturedFrame {
            width: width as u32,
            height: height as u32,
            data: pixel_data,
        })
    }
}
