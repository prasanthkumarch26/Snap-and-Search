use tauri::{
    Manager,
    WebviewUrl,
    WebviewWindowBuilder,
    WindowEvent,
};

use tauri_plugin_global_shortcut::{
    Code,
    GlobalShortcutExt,
    Modifiers,
    Shortcut,
    ShortcutState,
};

const OVERLAY_LABEL: &str = "overlay";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())

        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    println!("Shortcut Pressed: {:?}", shortcut);

                    // Prevent multiple overlay windows
                    if app.get_webview_window(OVERLAY_LABEL).is_some() {
                        return;
                    }

                    let overlay = WebviewWindowBuilder::new(
                        app,
                        OVERLAY_LABEL,
                        WebviewUrl::App("index.html".into()),
                    )
                    .title("Snap & Search")
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .fullscreen(true)
                    .visible(false)
                    .build()
                    .expect("Failed to create overlay window");

                    let _ = overlay.show();
                    let _ = overlay.set_focus();

                    // overlay.open_devtools();

                    println!("Overlay created and focused");

                    // Close overlay when it loses focus
                    let overlay_clone = overlay.clone();

                    overlay.on_window_event(move |event| {
                        if let WindowEvent::Focused(false) = event {
                            println!("Overlay lost focus. Closing...");

                            let _ = overlay_clone.close();
                        }
                    });
                })
                .build(),
        )

        .setup(|app| {
            let shortcut =
                Shortcut::new(Some(Modifiers::SHIFT), Code::KeyS);

            app.global_shortcut()
                .register(shortcut)
                .expect("Failed to register Shift + S");

            println!("Shift + S Registered!");

            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}