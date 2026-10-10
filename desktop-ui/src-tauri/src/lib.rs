use chrono::Local;
use overlay::{Overlay, OverlayAction};
use std::path::PathBuf;
use win_api::hotkey::{HotkeyManager, HotkeyModifiers};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

fn run_background_service() {
    println!("Starting Snap and Search Background Service in Tauri thread...");

    let settings = settings::load_settings();
    let screenshots_dir = PathBuf::from(&settings.save_dir);



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
                                // Load fresh settings so we get the newly updated path without restarting
                                let current_settings = settings::load_settings();
                                let current_dir = PathBuf::from(current_settings.save_dir);
                                
                                // Ensure the folder exists just in case they deleted it or just changed it
                                let _ = std::fs::create_dir_all(&current_dir);
                                
                                let path = next_screenshot_path(&current_dir);
                                match std::fs::write(&path, &png) {
                                    Ok(_) => println!("Screenshot saved: {}", path.display()),
                                    Err(e) => eprintln!("Failed to save screenshot: {}", e),
                                }
                            }
                            Err(e) => eprintln!("PNG encoding failed: {}", e),
                        }
                    }
                    OverlayAction::CopyToClipboard => {
                        let rgba_img = frame.into_rgba_image();
                        let (width, height) = rgba_img.dimensions();
                        let img_data = arboard::ImageData {
                            width: width as usize,
                            height: height as usize,
                            bytes: std::borrow::Cow::Borrowed(rgba_img.as_raw()),
                        };
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Err(e) = clipboard.set_image(img_data) {
                                eprintln!("Failed to copy image to clipboard: {}", e);
                            } else {
                                println!("Image copied to clipboard!");
                            }
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
    
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut vk = 0x53; // Default 'S'

    let parts: Vec<&str> = settings.hotkey.split('+').collect();
    for part in parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => ctrl = true,
            "SHIFT" => shift = true,
            "ALT" => alt = true,
            key if key.len() == 1 => {
                vk = key.chars().next().unwrap() as u32;
            }
            "SPACE" => vk = 0x20,
            _ => {} // Ignore unknown
        }
    }

    let modifiers = HotkeyModifiers { ctrl, shift, alt, win: false };

    if let Err(e) = hotkey_manager.register(modifiers, vk) {
        eprintln!("Failed to register global hotkey: {}", e);
        return;
    }

    println!("Global hotkey registered: {}", settings.hotkey);

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
    let settings = settings::load_settings();
    let screenshots_dir = PathBuf::from(settings.save_dir);

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
    
    let settings = settings::load_settings();
    let screenshots_dir = PathBuf::from(settings.save_dir);
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_screenshots, get_screenshot_base64, get_settings, save_settings])
        .setup(|app| {
            // Spawn the native capture/overlay background thread
            std::thread::spawn(run_background_service);

            // Create tray menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
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
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(is_visible) = window.is_visible() {
                                if is_visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
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
#[tauri::command]
fn get_settings() -> settings::AppSettings {
    settings::load_settings()
}

#[tauri::command]
fn save_settings(new_settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&new_settings);
    // Restarting the background service to bind new hotkey is left as an exercise 
    // for this MVP, we just save it.
    Ok(())
}

