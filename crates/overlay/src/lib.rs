use win_api::window::OverlayWindow;

pub struct Overlay {
    window: OverlayWindow,
}

impl Overlay {
    pub fn new() -> Result<Self, String> {
        let window = OverlayWindow::new()
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
