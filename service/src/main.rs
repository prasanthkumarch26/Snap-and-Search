use overlay::Overlay;
use win_api::hotkey::{HotkeyManager, HotkeyModifiers};

fn main() {
    println!("Starting Screen Intelligence Background Service...");

    // Create the native transparent overlay.
    // The on_capture closure fires when the user completes a selection.
    let overlay = match Overlay::new(|rect| {
        println!("Capture triggered for region: {:?}", rect);

        // Small sleep to allow Windows to actually hide the overlay before we capture
        std::thread::sleep(std::time::Duration::from_millis(50));

        match capture::capture_region(rect.x, rect.y, rect.width, rect.height) {
            Ok(frame) => {
                println!(
                    "Captured {}x{} pixels ({} bytes raw)",
                    frame.width,
                    frame.height,
                    frame.data.len()
                );

                // Save as lossless PNG to temp dir
                match frame.to_png_bytes() {
                    Ok(png) => {
                        let path = std::env::temp_dir().join("snap_and_search_capture.png");
                        std::fs::write(&path, &png)
                            .expect("Failed to write capture to disk");
                        println!("Capture saved to: {}", path.display());
                    }
                    Err(e) => eprintln!("PNG encoding failed: {}", e),
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
    println!("Overlay created. Service running silently.");

    // Register the global hotkey: Ctrl + Shift + S
    let hotkey_manager = HotkeyManager::new(1);
    let modifiers = HotkeyModifiers {
        ctrl: true,
        shift: true,
        alt: false,
        win: false,
    };
    // 0x53 = virtual key code for 'S'
    if let Err(e) = hotkey_manager.register(modifiers, 0x53) {
        eprintln!("Failed to register global hotkey: {}", e);
        return;
    }

    println!("Global hotkey registered: Ctrl + Shift + S");
    println!("Press the hotkey to begin a screen selection. Press Escape to cancel.");

    // Block here and process Windows messages forever.
    hotkey_manager.listen(|| {
        println!("Hotkey pressed — showing overlay...");
        overlay.show();
    });
}
