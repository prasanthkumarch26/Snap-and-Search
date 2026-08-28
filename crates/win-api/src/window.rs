use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

pub struct OverlayWindow {
    hwnd: HWND,
}

impl OverlayWindow {
    pub fn new() -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            debug_assert!(!instance.0.is_null());

            let window_class = w!("ScreenIntelligenceOverlayClass");

            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_CROSS)?,
                hInstance: instance.into(),
                lpszClassName: window_class,
                lpfnWndProc: Some(Self::wndproc),
                // We use a dark brush for the background
                hbrBackground: CreateSolidBrush(COLORREF(0x00000000)),
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            debug_assert!(atom != 0);

            // Get virtual screen dimensions to cover all monitors
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let cx = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let cy = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            let hwnd = CreateWindowExW(
                // WS_EX_LAYERED for transparency, WS_EX_TOPMOST to stay on top, WS_EX_TOOLWINDOW to hide from taskbar
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                window_class,
                w!("Screen Intelligence Overlay"),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                cx,
                cy,
                None,
                None,
                Some(instance.into()),
                None,
            )?;

            // Make it semi-transparent (128/255 opacity)
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 128, LWA_ALPHA)?;

            Ok(Self { hwnd })
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_KEYDOWN => {
                    // 0x1B is VK_ESCAPE
                    if wparam.0 == 0x1B {
                        ShowWindow(window, SW_HIDE);
                        LRESULT(0)
                    } else {
                        DefWindowProcW(window, message, wparam, lparam)
                    }
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
    }
}
