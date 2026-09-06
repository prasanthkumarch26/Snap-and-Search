use chrono::Local;
use overlay::{Overlay, OverlayAction};
use std::path::PathBuf;
use win_api::hotkey::{HotkeyManager, HotkeyModifiers};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

fn run_background_service() {
    println!("Starting Screen Intelligence Background Service in Tauri thread...");

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

        std::thread::sleep(std::time::Duration::from_millis(50));

        match capture::capture_region(rect.x, rect.y, rect.width, rect.height) {
            Ok(frame) => {
                println!("Captured {}x{} pixels ({} bytes raw)", frame.width, frame.height, frame.data.len());
                match action {
                    OverlayAction::SearchLens => {
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
                    OverlayAction::ScanQrCode => {
                        let rgba_img = frame.into_rgba_image();
                        let luma_img = image::DynamicImage::ImageRgba8(rgba_img).into_luma8();
                        
                        let mut img = rqrr::PreparedImage::prepare(luma_img);
                        let grids = img.detect_grids();
                        
                        if grids.is_empty() {
                            println!("No QR codes found in selection.");
                        } else {
                            for grid in grids {
                                match grid.decode() {
                                    Ok((meta, content)) => {
                                        println!("QR Code detected ({}): {}", meta.version.to_size(), content);
                                        
                                        // Attempt to copy to clipboard
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            let _ = clipboard.set_text(content.clone());
                                            println!("Copied to clipboard!");
                                        }

                                        // If it's a URL, open it in the browser
                                        if content.starts_with("http://") || content.starts_with("https://") {
                                            if let Err(e) = webbrowser::open(&content) {
                                                eprintln!("Failed to open URL: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("Failed to decode QR grid: {}", e),
                                }
                            }
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

    let hotkey_manager = HotkeyManager::new(1);
    let modifiers = HotkeyModifiers { ctrl: true, shift: true, alt: false, win: false };

    if let Err(e) = hotkey_manager.register(modifiers, 0x53) {
        eprintln!("Failed to register global hotkey: {}", e);
        return;
    }

    println!("Global hotkey registered: Ctrl + Shift + S");

    hotkey_manager.listen(|| {
        println!("Hotkey pressed — showing overlay...");
        overlay.show();
    });
}

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

#[tauri::command]
fn get_screenshots() -> Vec<String> {
    let mut screenshots = Vec::new();
    let screenshots_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("screenshots");

    if let Ok(entries) = std::fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "png" {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                screenshots.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Sort descending by name so newest is first
    screenshots.sort_by(|a, b| b.cmp(a));
    screenshots
}

#[tauri::command]
fn get_screenshot_base64(name: String) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    
    let screenshots_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("screenshots");
    let path = screenshots_dir.join(&name);
    
    // Basic security check: ensure it's actually in the screenshots dir
    if path.parent() != Some(&screenshots_dir) {
        return Err("Invalid path".into());
    }
    
    match std::fs::read(&path) {
        Ok(bytes) => Ok(STANDARD.encode(&bytes)),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_screenshots, get_screenshot_base64])
        .setup(|app| {
            // Spawn the native capture/overlay background thread
            std::thread::spawn(run_background_service);

            // Create tray menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show History & Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Hide the main window initially (run in background)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // Instead of quitting, hide the window to tray
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}