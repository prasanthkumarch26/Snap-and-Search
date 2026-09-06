use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::{SetCapture, ReleaseCapture},
};

/// The action chosen by the user from the post-selection menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    SearchLens,
    SaveScreenshot,
    ScanQrCode,
}

/// Coordinates of a completed screen selection, in virtual screen space.
#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

type CaptureCallback = Box<dyn Fn(SelectionRect, OverlayAction) + Send + 'static>;

struct WindowState {
    is_dragging: bool,
    start_point: POINT,
    current_point: POINT,
    on_capture: CaptureCallback,
}

pub struct OverlayWindow {
    hwnd: HWND,
}

impl OverlayWindow {
    pub fn new(on_capture: impl Fn(SelectionRect, OverlayAction) + Send + 'static) -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;

            let window_class = w!("ScreenIntelligenceOverlayClass");

            let mut wc_info = WNDCLASSW::default();
            if GetClassInfoW(Some(instance.into()), window_class, &mut wc_info).is_err() {
                let wc = WNDCLASSW {
                    hCursor: LoadCursorW(None, IDC_CROSS)?,
                    hInstance: instance.into(),
                    lpszClassName: window_class,
                    lpfnWndProc: Some(Self::wndproc),
                    hbrBackground: CreateSolidBrush(COLORREF(0x00000000)),
                    style: CS_HREDRAW | CS_VREDRAW,
                    ..Default::default()
                };
                RegisterClassW(&wc);
            }

            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let cx = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let cy = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            let state = Box::into_raw(Box::new(WindowState {
                is_dragging: false,
                start_point: POINT::default(),
                current_point: POINT::default(),
                on_capture: Box::new(on_capture),
            }));

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                window_class,
                w!("Screen Intelligence Overlay"),
                WS_POPUP,
                x, y, cx, cy,
                None, None,
                Some(instance.into()),
                Some(state as *const _ as _),
            )?;

            SetLayeredWindowAttributes(hwnd, COLORREF(0), 160, LWA_ALPHA)?;

            Ok(Self { hwnd })
        }
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = SetForegroundWindow(self.hwnd);
            let _ = UpdateWindow(self.hwnd);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    unsafe fn set_state(hwnd: HWND, ptr: isize) {
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr); }
    }

    unsafe fn get_state<'a>(hwnd: HWND) -> Option<&'a mut WindowState> {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
        if ptr == 0 { None } else { Some(unsafe { &mut *(ptr as *mut WindowState) }) }
    }

    /// Show a native Win32 popup menu at the current cursor position.
    /// `hwnd` must be a window on the calling thread (used as menu owner).
    /// Returns the chosen `OverlayAction`, or `None` if dismissed.
    unsafe fn show_action_menu(hwnd: HWND) -> Option<OverlayAction> {
        let mut cursor = POINT::default();
        unsafe { let _ = GetCursorPos(&mut cursor); }

        let menu = unsafe { CreatePopupMenu().ok()? };

        unsafe {
            let _ = AppendMenuW(menu, MF_STRING, 1, w!("🔍  Search with Google Lens"));
            let _ = AppendMenuW(menu, MF_STRING, 2, w!("💾  Save Screenshot as PNG"));
            let _ = AppendMenuW(menu, MF_STRING, 3, w!("🔗  Scan QR Code"));
        }

        // TrackPopupMenu requires the parent window to be the foreground window.
        // This is a documented Win32 requirement, otherwise the menu silently fails.
        unsafe { let _ = SetForegroundWindow(hwnd); }

        let result = unsafe {
            TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RETURNCMD,
                cursor.x,
                cursor.y,
                Some(0),
                hwnd,   // must be a window on THIS thread, not GetDesktopWindow()
                None,
            )
        };

        // Required after TrackPopupMenu to flush the message queue correctly
        unsafe { let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0)); }

        unsafe { let _ = DestroyMenu(menu); }

        match result.0 {
            1 => Some(OverlayAction::SearchLens),
            2 => Some(OverlayAction::SaveScreenshot),
            3 => Some(OverlayAction::ScanQrCode),
            _ => None,
        }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTW;
                if !cs.is_null() {
                    Self::set_state(window, (*cs).lpCreateParams as isize);
                }
                return DefWindowProcW(window, message, wparam, lparam);
            }

            let state = match Self::get_state(window) {
                Some(s) => s,
                None => return DefWindowProcW(window, message, wparam, lparam),
            };

            match message {
                WM_LBUTTONDOWN => {
                    state.is_dragging = true;
                    let x = (lparam.0 & 0xFFFF) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                    state.start_point = POINT { x, y };
                    state.current_point = state.start_point;
                    SetCapture(window);
                    LRESULT(0)
                }
                WM_MOUSEMOVE => {
                    if state.is_dragging {
                        let x = (lparam.0 & 0xFFFF) as i16 as i32;
                        let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                        state.current_point = POINT { x, y };
                        let _ = InvalidateRect(Some(window), None, true);
                    }
                    LRESULT(0)
                }
                WM_LBUTTONUP => {
                    if state.is_dragging {
                        state.is_dragging = false;
                        let _ = ReleaseCapture();

                        let x = state.start_point.x.min(state.current_point.x);
                        let y = state.start_point.y.min(state.current_point.y);
                        let w = (state.start_point.x - state.current_point.x).abs();
                        let h = (state.start_point.y - state.current_point.y).abs();

                        // Only act on meaningful selections (> 5px in each dimension)
                        if w > 5 && h > 5 {
                            // Hide the overlay BEFORE capturing so it won't appear in screenshot
                            let _ = ShowWindow(window, SW_HIDE);
                            let _ = UpdateWindow(window);

                            // Account for virtual screen offset (multi-monitor support)
                            let vscreen_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
                            let vscreen_y = GetSystemMetrics(SM_YVIRTUALSCREEN);

                            let rect = SelectionRect {
                                x: x + vscreen_x,
                                y: y + vscreen_y,
                                width: w,
                                height: h,
                            };

                            // Show the action menu; if dismissed (Escape), do nothing
                            if let Some(action) = Self::show_action_menu(window) {
                                (state.on_capture)(rect, action);
                            }
                        } else {
                            let _ = ShowWindow(window, SW_HIDE);
                        }
                    }
                    LRESULT(0)
                }
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    let hdc = BeginPaint(window, &mut ps);

                    if state.is_dragging {
                        let pen = CreatePen(PS_SOLID, 2, COLORREF(0x00FFFFFF));
                        let old_pen = SelectObject(hdc, pen.into());
                        let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH).into());

                        let _ = Rectangle(
                            hdc,
                            state.start_point.x.min(state.current_point.x),
                            state.start_point.y.min(state.current_point.y),
                            state.start_point.x.max(state.current_point.x),
                            state.start_point.y.max(state.current_point.y),
                        );

                        SelectObject(hdc, old_brush);
                        SelectObject(hdc, old_pen);
                        let _ = DeleteObject(pen.into());
                    }

                    let _ = EndPaint(window, &ps);
                    LRESULT(0)
                }
                WM_KEYDOWN => {
                    if wparam.0 == 0x1B {
                        // Escape cancels
                        state.is_dragging = false;
                        let _ = ShowWindow(window, SW_HIDE);
                        LRESULT(0)
                    } else {
                        DefWindowProcW(window, message, wparam, lparam)
                    }
                }
                WM_DESTROY => {
                    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA);
                    if ptr != 0 {
                        let _ = Box::from_raw(ptr as *mut WindowState);
                        Self::set_state(window, 0);
                    }
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
    }
}
