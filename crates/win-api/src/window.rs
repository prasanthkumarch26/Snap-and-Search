use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::{SetCapture, ReleaseCapture},
};

/// Coordinates of a completed screen selection, in virtual screen space.
#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

type CaptureCallback = Box<dyn Fn(SelectionRect) + Send + 'static>;

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
    pub fn new(on_capture: impl Fn(SelectionRect) + Send + 'static) -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            debug_assert!(!instance.0.is_null());

            let window_class = w!("ScreenIntelligenceOverlayClass");

            // Avoid re-registering the class if already registered
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
                x,
                y,
                cx,
                cy,
                None,
                None,
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
        if ptr == 0 {
            None
        } else {
            Some(unsafe { &mut *(ptr as *mut WindowState) })
        }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                if !create_struct.is_null() {
                    Self::set_state(window, (*create_struct).lpCreateParams as isize);
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

                        // Normalize so start is always top-left
                        let x = state.start_point.x.min(state.current_point.x);
                        let y = state.start_point.y.min(state.current_point.y);
                        let w = (state.start_point.x - state.current_point.x).abs();
                        let h = (state.start_point.y - state.current_point.y).abs();

                        // Only fire capture if the selection is meaningful (> 5px)
                        if w > 5 && h > 5 {
                            // Hide the overlay BEFORE capturing so it doesn't appear in screenshot
                            let _ = ShowWindow(window, SW_HIDE);

                            // We need to account for the virtual screen offset
                            let vscreen_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
                            let vscreen_y = GetSystemMetrics(SM_YVIRTUALSCREEN);

                            let rect = SelectionRect {
                                x: x + vscreen_x,
                                y: y + vscreen_y,
                                width: w,
                                height: h,
                            };
                            println!("Selection complete: {:?}", rect);
                            (state.on_capture)(rect);
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
                        // Draw a semi-transparent dark fill over the selection
                        let sel_rect = RECT {
                            left: state.start_point.x.min(state.current_point.x),
                            top: state.start_point.y.min(state.current_point.y),
                            right: state.start_point.x.max(state.current_point.x),
                            bottom: state.start_point.y.max(state.current_point.y),
                        };

                        // White outline pen (2px solid)
                        let pen = CreatePen(PS_SOLID, 2, COLORREF(0x00FFFFFF));
                        let old_pen = SelectObject(hdc, pen.into());
                        // Hollow brush so the interior is transparent
                        let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH).into());

                        let _ = Rectangle(
                            hdc,
                            sel_rect.left,
                            sel_rect.top,
                            sel_rect.right,
                            sel_rect.bottom,
                        );

                        SelectObject(hdc, old_brush);
                        SelectObject(hdc, old_pen);
                        let _ = DeleteObject(pen.into());
                    }

                    let _ = EndPaint(window, &ps);
                    LRESULT(0)
                }
                WM_KEYDOWN => {
                    // Escape cancels the selection
                    if wparam.0 == 0x1B {
                        state.is_dragging = false;
                        let _ = ShowWindow(window, SW_HIDE);
                        LRESULT(0)
                    } else {
                        DefWindowProcW(window, message, wparam, lparam)
                    }
                }
                WM_DESTROY => {
                    // Free the state box when the window is destroyed
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
