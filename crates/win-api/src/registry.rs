use windows::{
    Win32::Foundation::ERROR_SUCCESS,
    Win32::System::Registry::*,
};

const RUN_KEY: windows::core::PCWSTR = windows::core::w!(
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"
);
const APP_NAME: windows::core::PCWSTR = windows::core::w!("SnapAndSearch");

/// Register the current executable to run on Windows startup.
pub fn enable_startup() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("Startup: cannot get exe path: {}", e); return; }
    };

    let path_str = exe_path.to_string_lossy().to_string();
    // Registry string values are UTF-16 with null terminator
    let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
    let path_bytes = unsafe {
        std::slice::from_raw_parts(path_wide.as_ptr() as *const u8, path_wide.len() * 2)
    };

    unsafe {
        let mut hkey = HKEY::default();
        let err = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, Some(0), KEY_SET_VALUE, &mut hkey);
        if err != ERROR_SUCCESS {
            eprintln!("Startup: RegOpenKeyExW failed: {:?}", err);
            return;
        }

        let err = RegSetValueExW(hkey, APP_NAME, Some(0), REG_SZ, Some(path_bytes));
        if err != ERROR_SUCCESS {
            eprintln!("Startup: RegSetValueExW failed: {:?}", err);
        } else {
            println!("Startup on login: enabled ({})", path_str);
        }

        let _ = RegCloseKey(hkey);
    }
}

/// Remove the app from Windows startup.
pub fn disable_startup() {
    unsafe {
        let mut hkey = HKEY::default();
        let err = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, Some(0), KEY_SET_VALUE, &mut hkey);
        if err != ERROR_SUCCESS {
            // Key might not exist — not an error
            return;
        }
        // Ignore error if the value doesn't exist
        let _ = RegDeleteValueW(hkey, APP_NAME);
        let _ = RegCloseKey(hkey);
    }
    println!("Startup on login: disabled");
}

/// Apply the `launch_on_startup` setting.
pub fn apply_startup(enabled: bool) {
    if enabled {
        enable_startup();
    } else {
        disable_startup();
    }
}
