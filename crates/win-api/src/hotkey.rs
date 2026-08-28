use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct HotkeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

impl Into<HOT_KEY_MODIFIERS> for HotkeyModifiers {
    fn into(self) -> HOT_KEY_MODIFIERS {
        let mut mods = HOT_KEY_MODIFIERS(0);
        if self.ctrl {
            mods |= MOD_CONTROL;
        }
        if self.shift {
            mods |= MOD_SHIFT;
        }
        if self.alt {
            mods |= MOD_ALT;
        }
        if self.win {
            mods |= MOD_WIN;
        }
        mods
    }
}

pub struct HotkeyManager {
    id: i32,
}

impl HotkeyManager {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    pub fn register(&self, modifiers: HotkeyModifiers, vk: u32) -> Result<(), windows::core::Error> {
        unsafe { RegisterHotKey(None, self.id, modifiers.into(), vk) }
    }

    pub fn unregister(&self) -> Result<(), windows::core::Error> {
        unsafe { UnregisterHotKey(None, self.id) }
    }

    pub fn listen<F>(&self, mut callback: F)
    where
        F: FnMut(),
    {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_HOTKEY {
                    callback();
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
