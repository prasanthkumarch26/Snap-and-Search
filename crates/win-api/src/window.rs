use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::{SetCapture, ReleaseCapture},
};

#[derive(Default)]
struct SelectionState {
    is_dragging: bool,
    start_point: POINT,
    current_point: POINT,
}

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
                hbrBackground: CreateSolidBrush(COLORREF(0x00000000)),
                ..Default::default()
            };

            let _atom = RegisterClassW(&wc);

            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let cx = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let cy = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            let state = Box::into_raw(Box::new(SelectionState::default()));

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                window_class,
                w!("Screen Intelligence Overlay"),
                WS_POPUP,
                x,
                y,
                cx,
                cy,
                None,
                None,
                Some(instance.into()),
                Some(state as *const _ as _),
            )?;

            SetLayeredWindowAttributes(hwnd, COLORREF(0), 128, LWA_ALPHA)?;

            Ok(Self { hwnd })
        }
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = UpdateWindow(self.hwnd);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn set_window_userdata(hwnd: HWND, ptr: isize) {
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr); }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn get_window_userdata(hwnd: HWND) -> isize {
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) }
    }

    #[cfg(target_arch = "x86")]
    unsafe fn set_window_userdata(hwnd: HWND, ptr: isize) {
        unsafe { SetWindowLongW(hwnd, GWLP_USERDATA, ptr as i32); }
    }

    #[cfg(target_arch = "x86")]
    unsafe fn get_window_userdata(hwnd: HWND) -> isize {
        unsafe { GetWindowLongW(hwnd, GWLP_USERDATA) as isize }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                if !create_struct.is_null() {
                    let state_ptr = (*create_struct).lpCreateParams;
                    Self::set_window_userdata(window, state_ptr as isize);
                }
                return DefWindowProcW(window, message, wparam, lparam);
            }

            let state_ptr = Self::get_window_userdata(window) as *mut SelectionState;
            
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
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
                            let _ = ShowWindow(window, SW_HIDE);
                            
                            // Let's print out the captured region for debugging
                            let min_x = state.start_point.x.min(state.current_point.x);
                            let min_y = state.start_point.y.min(state.current_point.y);
                            let max_x = state.start_point.x.max(state.current_point.x);
                            let max_y = state.start_point.y.max(state.current_point.y);
                            println!("Captured Region: x: {}, y: {}, w: {}, h: {}", min_x, min_y, max_x - min_x, max_y - min_y);
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
                                state.start_point.x,
                                state.start_point.y,
                                state.current_point.x,
                                state.current_point.y,
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
                            state.is_dragging = false;
                            let _ = ShowWindow(window, SW_HIDE);
                            LRESULT(0)
                        } else {
                            DefWindowProcW(window, message, wparam, lparam)
                        }
                    }
                    WM_DESTROY => {
                        let _ = Box::from_raw(state_ptr);
                        Self::set_window_userdata(window, 0);
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    _ => DefWindowProcW(window, message, wparam, lparam),
                }
            } else {
                DefWindowProcW(window, message, wparam, lparam)
            }
        }
    }
}
