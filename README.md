<div align="center">

<img src="assets/placeholder.png" alt="Snap & Search Logo" width="128" />

<h1>Snap & Search</h1>

<p><strong>Snap anything. Search instantly.</strong></p>

<p>
Turn anything on your screen into something you can <strong>search, scan, translate, or save</strong> —
without taking screenshots, manually uploading images, or interrupting your workflow.
</p>

<p><strong>Ctrl + Shift + S → Select → Act.</strong></p>

<a href="https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest/download/Snap_and_Search_0.1.0_x64_en-US.msi">
<b>📦 Download for Windows (.msi)</b>
</a>
&nbsp;&nbsp;|&nbsp;&nbsp;
<a href="https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest/download/Snap_and_Search_0.1.0_x64-setup.exe">
<b>🏃 Portable (.exe)</b>
</a>

<p>
Windows 10 / 11 · 64-bit · Free
</p>

</div>

---

## ✨ Why Snap & Search?

Ever seen something on your screen and wondered:

* 🛍️ **What product is this?**
* 🌐 **What does this foreign text mean?**
* 🐛 **What does this error message mean?**
* 🖼️ **Where does this image come from?**
* 🔗 **What's inside this QR code?**

Normally, you'd take a screenshot, save it, open a browser, upload the image, and search manually.

**Snap & Search removes those extra steps.**

```text
Ctrl + Shift + S
        ↓
Select anything on your screen
        ↓
🔍 Search   🔗 Scan QR   💾 Save
```

---

# 🚀 Features

## 🔍 Search Anything You See

Select any image, product, diagram, object, or visual content and search it with Google Lens.

Google Lens can help with:

* Reverse image search
* OCR and text extraction
* Translation
* Product discovery
* Visual search

No browser extension. No manual screenshot uploads.

---

## 🔗 Scan QR Codes Directly From Your Screen

No phone required.

Select a QR code visible anywhere on your screen and Snap & Search decodes it directly.

* 📋 Copies decoded text to your clipboard
* 🌐 Automatically opens URLs in your default browser

---

## 💾 Save Screenshots Instantly

Capture any region and save it as a lossless PNG.

Your screenshots stay local and can be accessed through the History dashboard.

---

## 🖥️ Works Across Multiple Monitors

Snap & Search supports the full Windows virtual desktop.

Select content across multiple monitors, including displays with negative coordinates.

---

## ⚡ Feels Like a Built-In Windows Feature

The capture experience uses native Windows APIs for fast, lightweight interaction.

* Global keyboard shortcut
* Native fullscreen overlay
* Crosshair cursor
* Rectangle selection
* Native action menu
* System tray integration

The interaction pipeline stays separate from the dashboard UI, keeping screen selection fast and responsive.

---

## 👻 Lightweight Background Service

Snap & Search stays quietly in your system tray until you need it.

* ~8 MB idle RAM
* ~0% idle CPU usage
* Event-driven architecture
* No browser extension required

---

## 📊 Settings & History Dashboard

A lightweight dashboard built with **Tauri + React + TypeScript** provides:

* Screenshot history
* Settings management
* Application configuration

The screen capture and selection pipeline remains native and independent from the UI layer.

---

# ⚡ Built to Feel Instant

The native interaction pipeline is designed to stay out of your way.

| Interaction                          |     Latency |
| ------------------------------------ | ----------: |
| Process launch → tray ready          | **< 150ms** |
| `Ctrl + Shift + S` → overlay visible |  **< 50ms** |
| Mouse release → action menu          |  **< 15ms** |

For visual search, most end-to-end delay comes from **network transfer and Google Lens processing**.

The native overlay, selection, and menu interactions are designed to feel near-instant.

---

# 📊 Performance

Performance primarily scales with the number of pixels in the selected region.

## 🖥️ Screen Capture Latency

| Resolution            |   P50 |        P95 |    P99 |
| --------------------- | ----: | ---------: | -----: |
| **1080p (1920×1080)** | 4.1ms |  **6.2ms** |  8.1ms |
| **4K (3840×2160)**    | 9.8ms | **14.5ms** | 18.2ms |

The capture pipeline uses native **Win32/GDI APIs** to extract selected regions into raw BGRA pixel buffers.

---

## 🖼️ Image Encoding

| Format                | Resolution |    P50 |    P95 |     P99 |
| --------------------- | ---------- | -----: | -----: | ------: |
| **JPEG (Quality 90)** | 1080p      | 77.0ms | 92.7ms | 112.0ms |
| **PNG (Lossless)**    | 1080p      | 20.8ms | 27.4ms |  35.1ms |

**JPEG** is used for Google Lens uploads because its smaller payload can reduce network transfer time.

**PNG** is used for local screenshots where lossless output is preferred.

---

# 🧪 Reliability

Snap & Search was tested across repeated capture cycles:

* **1,000 consecutive captures**
* **100% successful completion**
* **No observable memory growth** during repeated capture cycles

---

# 🏗️ How It Works

Snap & Search combines a native Rust capture engine with a lightweight modern desktop UI.

```text
Ctrl + Shift + S
        │
        ▼
Win32 Global Hotkey
        │
        ▼
Native Selection Overlay
(Rust + Win32 + GDI)
        │
        ▼
Multi-Monitor Screen Capture
(BitBlt → BGRA Buffer)
        │
        ▼
Native Action Menu
        │
   ┌────┼─────────────┐
   ▼    ▼             ▼
 Lens  QR Scan      Save PNG
   │      │             │
   ▼      ▼             ▼
Browser Clipboard      Disk
```

## Architecture

* **Rust + Win32/GDI** — Global hotkeys, layered windows, native menus, and screen capture.
* **Native Capture Engine** — Converts selected screen regions into raw BGRA pixel buffers.
* **Action Pipeline** — Handles Google Lens search, QR decoding, and PNG storage.
* **Tauri + React** — Provides settings and screenshot history through a lightweight desktop dashboard.
* **Event-Driven Service** — Remains idle until triggered by Windows events.

---

# 📦 Resource Usage

| Metric          |        Value |
| --------------- | -----------: |
| Idle RAM        |    **~8 MB** |
| Idle CPU        |      **~0%** |
| Executable Size | **~9.07 MB** |

The application remains event-driven while idle and waits for Windows messages such as `WM_HOTKEY`.

---

# 📥 Download

## Windows Installer

Recommended for most users:

👉 <a href="https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest/download/Snap_and_Search_0.1.0_x64_en-US.msi"><strong>📦 Download Snap & Search for Windows</strong></a>

## Portable Version

Run without installation:

👉 <a href="https://github.com/prasanthkumarch26/Snap-and-Search/releases/latest/download/Snap_and_Search_0.1.0_x64-setup.exe"><strong>🏃 Download Portable .exe</strong></a>

### System Requirements

* Windows 10 or Windows 11
* 64-bit system

---

# 🛠️ Build From Source

## Prerequisites

* [Rust](https://www.rust-lang.org/tools/install)
* [Node.js](https://nodejs.org/en/)

```bash
# Clone the repository
git clone https://github.com/prasanthkumarch26/Snap-and-Search.git

# Open the desktop application
cd Snap-and-Search/desktop-ui

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Build for Release

```bash
npm run tauri build
```

---

# 🛣️ Roadmap

## ✅ Completed

* [x] Native Win32 selection overlay
* [x] Multi-monitor virtual screen support
* [x] Google Lens visual search
* [x] On-screen QR code decoding
* [x] System tray application
* [x] Local PNG screenshot saving
* [x] Settings and History dashboard

## 🔜 Planned

* [ ] ☁️ Cloud share links
* [ ] 📌 Pin to Screen / Reference Mode
* [ ] 🧠 Local AI Vision with Ollama

---

# 🤝 Contributing

Contributions, ideas, and feedback are welcome!

Feel free to:

* 🐛 Report bugs
* 💡 Suggest features
* 🛠️ Open pull requests
* ⭐ Star the repository if you find Snap & Search useful

---

<div align="center">

### Snap anything. Search instantly.

Made for Windows users who want to act on what's already on their screen.

</div>
