use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const TRAY_ID: u32 = 1001;
// Menu item IDs
const MENU_ABOUT: usize = 2001;
const MENU_EXIT: usize = 2002;

pub struct TrayIcon {
    hwnd: HWND,
}

// SAFETY: HWND is just an integer handle. Shell_NotifyIconW and SendMessage
// are safe to call from any thread. We only use hwnd for notifications.
unsafe impl Send for TrayIcon {}
unsafe impl Sync for TrayIcon {}

impl TrayIcon {
    pub fn new() -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            let class_name = w!("SnapAndSearchTrayClass");

            // Register a minimal window class for the message-only tray window
            let mut wc_info = WNDCLASSW::default();
            if GetClassInfoW(Some(instance.into()), class_name, &mut wc_info).is_err() {
                let wc = WNDCLASSW {
                    hInstance: instance.into(),
                    lpszClassName: class_name,
                    lpfnWndProc: Some(tray_wndproc),
                    ..Default::default()
                };
                RegisterClassW(&wc);
            }

            // HWND_MESSAGE creates an invisible message-only window — no taskbar entry
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("SnapAndSearch Tray"),
                WS_OVERLAPPEDWINDOW,
                0, 0, 0, 0,
                Some(HWND_MESSAGE),
                None,
                Some(instance.into()),
                None,
            )?;

            // Build the NOTIFYICONDATAW structure
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: TRAY_ID,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                ..Default::default()
            };

            // Use the default Windows application icon as a fallback
            nid.hIcon = LoadIconW(None, IDI_APPLICATION)?;

            // Tooltip shown on hover
            let tooltip = "Snap and Search\0Press Ctrl+Shift+S to capture";
            let tooltip_wide: Vec<u16> = tooltip.encode_utf16().collect();
            let copy_len = tooltip_wide.len().min(nid.szTip.len() - 1);
            nid.szTip[..copy_len].copy_from_slice(&tooltip_wide[..copy_len]);

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            Ok(Self { hwnd })
        }
    }

    /// Show a balloon tip notification on the tray icon.
    pub fn notify(&self, title: &str, message: &str) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                uFlags: NIF_INFO,
                dwInfoFlags: NIIF_INFO,
                ..Default::default()
            };

            // Copy title
            let title_wide: Vec<u16> = title.encode_utf16().collect();
            let tlen = title_wide.len().min(nid.szInfoTitle.len() - 1);
            nid.szInfoTitle[..tlen].copy_from_slice(&title_wide[..tlen]);

            // Copy message
            let msg_wide: Vec<u16> = message.encode_utf16().collect();
            let mlen = msg_wide.len().min(nid.szInfo.len() - 1);
            nid.szInfo[..mlen].copy_from_slice(&msg_wide[..mlen]);

            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                ..Default::default()
            };
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }
}

unsafe extern "system" fn tray_wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAYICON => {
            // lparam low word = mouse event
            let event = (lparam.0 & 0xFFFF) as u32;
            if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                unsafe { show_tray_menu(window); }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as usize;
            match id {
                MENU_ABOUT => {
                    unsafe {
                        MessageBoxW(
                            Some(window),
                            w!("Snap and Search v1.0\n\nPress Ctrl+Shift+S to capture a region.\nRight-click this tray icon for options.\n\nGitHub: github.com/prasanthkumarch26/Snap-and-Search"),
                            w!("About Snap and Search"),
                            MB_OK | MB_ICONINFORMATION,
                        );
                    }
                }
                MENU_EXIT => {
                    unsafe { PostQuitMessage(0); }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let menu = match unsafe { CreatePopupMenu() } {
        Ok(m) => m,
        Err(_) => return,
    };

    unsafe {
        let _ = AppendMenuW(menu, MF_STRING, MENU_ABOUT, w!("About Snap and Search"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, w!("Exit"));
    }

    let mut cursor = POINT::default();
    unsafe { let _ = GetCursorPos(&mut cursor); }
    unsafe { let _ = SetForegroundWindow(hwnd); }

    unsafe {
        let _ = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
            cursor.x,
            cursor.y,
            Some(0),
            hwnd,
            None,
        );
    }

    unsafe { let _ = DestroyMenu(menu); }
}
