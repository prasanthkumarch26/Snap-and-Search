use chrono::Local;
use overlay::{Overlay, OverlayAction};
use std::path::PathBuf;
use win_api::hotkey::{HotkeyManager, HotkeyModifiers};

fn main() {
    println!("Starting Screen Intelligence Background Service...");

    // Ensure the screenshots folder exists next to wherever we're run from
    let screenshots_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("screenshots");

    if let Err(e) = std::fs::create_dir_all(&screenshots_dir) {
        eprintln!("Warning: could not create screenshots dir: {}", e);
    } else {
        println!("Screenshots will be saved to: {}", screenshots_dir.display());
    }

    let overlay = match Overlay::new(move |rect, action| {
        println!("Action: {:?} | Region: {:?}", action, rect);

        // Give Windows time to composite the hidden overlay out of the display
        std::thread::sleep(std::time::Duration::from_millis(50));

        match capture::capture_region(rect.x, rect.y, rect.width, rect.height) {
            Ok(frame) => {
                println!(
                    "Captured {}x{} pixels ({} bytes raw)",
                    frame.width, frame.height, frame.data.len()
                );

                match action {
                    // ── Search with Google Lens ───────────────────────────────
                    OverlayAction::SearchLens => {
                        // JPEG quality 90: ~5-8x smaller than PNG → faster upload
                        match frame.to_jpeg_bytes(90) {
                            Ok(jpeg) => {
                                std::thread::spawn(move || {
                                    if let Err(e) = google_lens::upload_and_open(jpeg, "image/jpeg") {
                                        eprintln!("Google Lens upload failed: {}", e);
                                    }
                                });
                            }
                            Err(e) => eprintln!("JPEG encoding failed: {}", e),
                        }
                    }

                    // ── Save Screenshot as PNG ────────────────────────────────
                    OverlayAction::SaveScreenshot => {
                        match frame.to_png_bytes() {
                            Ok(png) => {
                                let path = next_screenshot_path(&screenshots_dir);
                                match std::fs::write(&path, &png) {
                                    Ok(_) => println!("Screenshot saved: {}", path.display()),
                                    Err(e) => eprintln!("Failed to save screenshot: {}", e),
                                }
                            }
                            Err(e) => eprintln!("PNG encoding failed: {}", e),
                        }
                    }
                }
            }
            Err(e) => eprintln!("Screen capture failed: {}", e),
        }
    }) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to initialize overlay: {}", e);
            return;
        }
    };

    println!("Overlay ready.");

    // Register the global hotkey: Ctrl + Shift + S
    let hotkey_manager = HotkeyManager::new(1);
    let modifiers = HotkeyModifiers {
        ctrl: true,
        shift: true,
        alt: false,
        win: false,
    };

    if let Err(e) = hotkey_manager.register(modifiers, 0x53) {
        eprintln!("Failed to register global hotkey: {}", e);
        return;
    }

    println!("Global hotkey registered: Ctrl + Shift + S");
    println!("Draw a selection, then pick an action from the menu. Escape cancels.");

    hotkey_manager.listen(|| {
        println!("Hotkey pressed — showing overlay...");
        overlay.show();
    });
}

/// Returns the next available screenshot path.
///
/// Format: `screenshots/Screenshot_YYYY-MM-DD_001.png`
/// The sequential number increments until a free filename is found.
fn next_screenshot_path(dir: &std::path::Path) -> PathBuf {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut seq: u32 = 1;

    loop {
        let name = format!("Screenshot_{}_{:03}.png", today, seq);
        let path = dir.join(&name);
        if !path.exists() {
            return path;
        }
        seq += 1;
    }
}
