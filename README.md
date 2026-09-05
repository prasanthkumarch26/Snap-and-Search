# Snap and Search

> **Instant screen capture + Google Lens search — the way Windows should have worked.**

[![Release](https://img.shields.io/github/v/release/prasanthkumarch26/Snap-and-Search?label=Download&logo=windows&style=for-the-badge)](https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)

---

## What is it?

**Snap and Search** is a lightweight native Windows utility that lets you instantly search anything on your screen using Google Lens — or save it as a lossless PNG — with a single keyboard shortcut.

No browser extension. No Electron. No overhead. Just **Ctrl+Shift+S → draw → done**.

---

## Features

- ⚡ **Native Win32** — zero UI framework overhead; starts in milliseconds
- 🖼️ **Snipping Tool-style** selection — dark overlay, crosshair, rectangle draw
- 🔍 **Google Lens search** — uploads your selection and opens results instantly
- 💾 **Save as PNG** — lossless screenshots saved as `Screenshot_YYYY-MM-DD_001.png`
- 🔔 **System tray** — lives quietly in your tray; right-click to exit
- 📊 **Metrics log** — every capture writes timing data to `metrics.txt`
- 🚀 **Tiny footprint** — ~5 MB exe, <10 MB RAM when idle
- 🖥️ **Multi-monitor** — full virtual screen support including negative coordinates

---

## Download

👉 **[Download the latest installer from the Releases page](https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest)**

| File | Description |
|---|---|
| `SnapAndSearch-Setup.exe` | Windows installer (recommended) |
| `SnapAndSearch.exe` | Portable — run without installing |

**System requirements:** Windows 10 / 11, 64-bit

---

## Usage

1. Run `SnapAndSearch-Setup.exe` and install.
2. The app starts silently in your **system tray**.
3. Press **Ctrl+Shift+S** anywhere on your screen.
4. **Click and drag** to select a region.
5. Choose an action from the popup menu:
   - 🔍 **Search with Google Lens** — opens your browser with results
   - 💾 **Save Screenshot as PNG** — saved to the `screenshots/` folder
6. Press **Escape** at any time to cancel.

---

## Configuration

Settings are stored at `%APPDATA%\SnapAndSearch\config.json`:

```json
{
  "hotkey": "Ctrl+Shift+S",
  "screenshots_dir": "C:\\path\\to\\screenshots",
  "launch_on_startup": false
}
```

Set `"launch_on_startup": true` to auto-start with Windows.

---

## Architecture

```
Ctrl+Shift+S (global hotkey — Win32 RegisterHotKey)
        │
        ▼
Native Transparent Overlay  ←── win-api crate (pure Win32/GDI)
  WS_EX_LAYERED | TOPMOST        144Hz capable, multi-monitor
        │
        ▼
  BitBlt Screen Capture      ←── capture crate
  GetDIBits → BGRA pixels
        │
        ▼
  Action Popup Menu          ←── Win32 TrackPopupMenu
        │
   ┌────┴────┐
   ▼         ▼
JPEG→Lens   PNG→Disk       ←── google-lens plugin / std::fs
(browser)   (file)
        │
        ▼
  metrics.txt  +  Tray Balloon notification
```

**Crate breakdown:**

| Crate | Responsibility |
|---|---|
| `service` | Orchestrator — wires everything together |
| `win-api` | All Win32 FFI: hotkeys, overlay window, tray, registry |
| `overlay` | Safe wrapper over `win-api`'s overlay window |
| `capture` | BitBlt screen capture, PNG + JPEG encoding |
| `settings` | JSON config load/save (`%APPDATA%`) |
| `plugins/google-lens` | Browser-form Google Lens upload |

---

## Build from Source

**Prerequisites:** [Rust stable](https://rustup.rs/) (MSVC toolchain)

```powershell
git clone https://github.com/prasanthkumarch26/Snap-and-Search.git
cd Snap-and-Search
cargo build --release -p service

# Run directly:
.\target\release\service.exe
```

**Build the installer** (requires [NSIS 3.x](https://nsis.sourceforge.io/)):
```powershell
Copy-Item target\release\service.exe target\release\SnapAndSearch.exe
makensis installer\setup.nsi
```

---

## Metrics

Every capture session appends a line to `metrics.txt` at the project root:

```
[2026-09-05 10:30:00] SearchLens | region=800x600 | capture=12ms | jpeg_enc=8ms | jpeg_size=42310B
[2026-09-05 10:31:00] SaveScreenshot | region=400x300 | capture=6ms | png_enc=45ms | png_size=183204B
```

**Typical performance on a mid-range PC:**

| Stage | Time |
|---|---|
| Hotkey → overlay visible | < 50ms |
| Mouse release → menu visible | < 100ms |
| BitBlt screen capture | 5–15ms |
| JPEG encoding (quality 90) | 5–20ms |
| PNG encoding (lossless) | 30–80ms |
| Browser open + Lens upload | 0.5–2s (network) |

---

## Contributing

PRs welcome! See the [project outline](project-outline.txt) for the full roadmap including planned OCR, AI, and translation actions.

---

## License

[MIT](LICENSE) © 2026 Prasanth Kumar
