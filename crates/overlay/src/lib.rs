pub use win_api::window::OverlayAction;
use win_api::window::{OverlayWindow, SelectionRect};

pub struct Overlay {
    window: OverlayWindow,
}

impl Overlay {
    pub fn new(on_capture: impl Fn(SelectionRect, OverlayAction) + Send + 'static) -> Result<Self, String> {
        let window = OverlayWindow::new(on_capture)
            .map_err(|e| format!("Failed to create overlay window: {}", e))?;
        Ok(Self { window })
    }

    pub fn show(&self) {
        self.window.show();
    }

    pub fn hide(&self) {
        self.window.hide();
    }
}
