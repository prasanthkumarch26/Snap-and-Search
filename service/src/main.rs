use chrono::Local;
use overlay::{Overlay, OverlayAction};
use settings::Config;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use win_api::hotkey::{HotkeyManager, HotkeyModifiers};
use win_api::registry::apply_startup;
use win_api::tray::TrayIcon;

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║       Snap and Search  v1.0.0        ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // ── Load configuration ────────────────────────────────────────────────
    let config = Config::load();
    println!("Config loaded from: {}", settings::config_path().display());
    println!("  hotkey          : {}", config.hotkey);
    println!("  screenshots_dir : {}", config.screenshots_dir);
    println!("  launch_on_startup: {}", config.launch_on_startup);
    println!();

    // Apply startup setting
    apply_startup(config.launch_on_startup);

    let screenshots_dir = config.screenshots_path();
    println!("Screenshots folder: {}", screenshots_dir.display());

    // ── System tray icon ─────────────────────────────────────────────────
    let tray = match TrayIcon::new() {
        Ok(t) => {
            println!("System tray icon registered.");
            Arc::new(t)
        }
        Err(e) => {
            eprintln!("Warning: could not create tray icon: {}", e);
            // Continue without tray — not fatal
            return;
        }
    };

    // ── Metrics file: record startup time ────────────────────────────────
    let startup_time = Instant::now();
    let metrics_path = std::env::current_dir()
        .unwrap_or_default()
        .join("metrics.txt");

    // ── Overlay + action handler ──────────────────────────────────────────
    let tray_clone = Arc::clone(&tray);
    let screenshots_dir_clone = screenshots_dir.clone();
    let metrics_path_clone = metrics_path.clone();

    let overlay = match Overlay::new(move |rect, action| {
        let t_capture_start = Instant::now();
        println!("\nAction: {:?} | Region: {:?}", action, rect);

        // Give DWM time to composite the hidden overlay out of the frame
        std::thread::sleep(std::time::Duration::from_millis(50));

        match capture::capture_region(rect.x, rect.y, rect.width, rect.height) {
            Ok(frame) => {
                let capture_ms = t_capture_start.elapsed().as_millis();
                println!(
                    "Captured {}x{} px in {}ms ({} bytes raw)",
                    frame.width, frame.height, capture_ms, frame.data.len()
                );

                match action {
                    // ── Search with Google Lens ───────────────────────────
                    OverlayAction::SearchLens => {
                        let t_enc = Instant::now();
                        match frame.to_jpeg_bytes(90) {
                            Ok(jpeg) => {
                                let enc_ms = t_enc.elapsed().as_millis();
                                let jpeg_size = jpeg.len();
                                println!("JPEG encoded in {}ms ({} bytes)", enc_ms, jpeg_size);

                                // Append metrics
                                append_metrics(
                                    &metrics_path_clone,
                                    &format!(
                                        "[{}] SearchLens | region={}x{} | capture={}ms | jpeg_enc={}ms | jpeg_size={}B\n",
                                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                                        frame.width, frame.height,
                                        capture_ms, enc_ms, jpeg_size
                                    ),
                                );

                                std::thread::spawn(move || {
                                    if let Err(e) = google_lens::upload_and_open(jpeg, "image/jpeg") {
                                        eprintln!("Google Lens upload failed: {}", e);
                                    }
                                });
                            }
                            Err(e) => eprintln!("JPEG encoding failed: {}", e),
                        }
                    }

                    // ── Save Screenshot as PNG ────────────────────────────
                    OverlayAction::SaveScreenshot => {
                        let t_enc = Instant::now();
                        match frame.to_png_bytes() {
                            Ok(png) => {
                                let enc_ms = t_enc.elapsed().as_millis();
                                let png_size = png.len();
                                let path = next_screenshot_path(&screenshots_dir_clone);
                                let filename = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();

                                match std::fs::write(&path, &png) {
                                    Ok(_) => {
                                        println!(
                                            "Screenshot saved: {} ({}ms enc, {} bytes)",
                                            path.display(), enc_ms, png_size
                                        );

                                        // Append metrics
                                        append_metrics(
                                            &metrics_path_clone,
                                            &format!(
                                                "[{}] SaveScreenshot | region={}x{} | capture={}ms | png_enc={}ms | png_size={}B | file={}\n",
                                                Local::now().format("%Y-%m-%d %H:%M:%S"),
                                                frame.width, frame.height,
                                                capture_ms, enc_ms, png_size, filename
                                            ),
                                        );

                                        // Balloon notification
                                        tray_clone.notify(
                                            "Screenshot saved",
                                            &format!("{}", filename),
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to save screenshot: {}", e);
                                        tray_clone.notify(
                                            "Save failed",
                                            &format!("Could not save screenshot: {}", e),
                                        );
                                    }
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

    // ── Global hotkey ─────────────────────────────────────────────────────
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

    // Write initial metrics header
    write_metrics_header(&metrics_path, startup_time.elapsed().as_millis());

    println!("Ready. Ctrl+Shift+S to capture. Right-click tray icon to exit.");
    println!("Metrics will be written to: {}", metrics_path.display());
    println!();

    hotkey_manager.listen(|| {
        println!("Hotkey pressed — showing overlay…");
        overlay.show();
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Returns the next available screenshot filename.
/// Format: `Screenshot_YYYY-MM-DD_001.png`
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

/// Write the metrics.txt header on first startup.
fn write_metrics_header(path: &PathBuf, startup_ms: u128) {
    let header = format!(
        "╔══════════════════════════════════════════════════════╗\n\
         ║           Snap and Search  —  Metrics Log            ║\n\
         ╚══════════════════════════════════════════════════════╝\n\
         \n\
         Generated : {}\n\
         Platform  : Windows (native Win32, Rust)\n\
         Startup   : {}ms\n\
         \n\
         Legend:\n\
           capture   = BitBlt + GetDIBits (screen region → raw BGRA)\n\
           jpeg_enc  = JPEG encoding at quality 90 (for Lens upload)\n\
           png_enc   = PNG encoding lossless (for Save Screenshot)\n\
           jpeg_size = JPEG file bytes uploaded to Google Lens\n\
           png_size  = PNG file bytes written to disk\n\
         \n\
         ── Capture Events ──────────────────────────────────────\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        startup_ms
    );

    // Only write header if file doesn't exist yet
    if !path.exists() {
        let _ = std::fs::write(path, header);
    }
}

/// Append a single line to the metrics file.
fn append_metrics(path: &PathBuf, line: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}
