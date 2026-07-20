# Snap & Search — Architecture

## 1. Overview

Snap & Search is a Windows-native visual search application that allows users to select any region of their screen and instantly search it on the web.

The application is designed around a simple workflow:

```text
Global Shortcut
      │
      v
Selection Overlay
      │
      v
Region Selection
      │
      v
Screen Capture
      │
      v
Image Processing
      │
      v
Action Dispatcher
      │
      v
Image Search
      │
      v
Browser Results
```

The initial implementation focuses on visual image search.

The architecture, however, is designed so that the screen selection and capture pipeline can support additional actions in the future without requiring changes to the core user experience.

---

## 2. Architectural Goals

The architecture is designed around the following goals:

### 2.1 Native Windows Experience

Snap & Search should behave like a native Windows utility rather than a web application running inside a desktop wrapper.

The application should:

- Run in the background.
- Respond to a global keyboard shortcut.
- Display a native fullscreen selection overlay.
- Integrate with the Windows system tray.
- Support Windows-specific capabilities such as screen capture and multi-monitor environments.

---

### 2.2 Familiar User Experience

The selection workflow is intentionally inspired by the Windows screenshot experience.

The goal is to minimize the learning curve for Windows users.

The expected interaction is:

```text
Press Shortcut
      │
      v
Overlay Appears
      │
      v
Select Region
      │
      v
Release Mouse
      │
      v
Selected Region Is Captured
      │
      v
Search Begins
```

The user should not need to manually:

- Take a screenshot.
- Save an image.
- Open a browser.
- Upload the image.
- Start a search.

---

### 2.3 Separation of Responsibilities

The system is divided into independent components with clearly defined responsibilities.

The selection system should not be tightly coupled to a specific search provider.

The core workflow is:

```text
Selection
    │
    ▼
Capture
    │
    ▼
Action
```

This allows the same selection and capture pipeline to support future capabilities such as:

- OCR
- AI-powered visual analysis
- Translation
- Product search
- QR and barcode detection
- Code search

---

### 2.4 Extensibility

Image Search is the first action implemented by Snap & Search.

The architecture should allow new actions to be added without modifying the core selection engine.

Conceptually:

```text
                  ┌───────────────┐
                  │ Selection     │
                  │ Engine        │
                  └───────┬───────┘
                          │
                          v
                  ┌───────────────┐
                  │ Captured      │
                  │ Content       │
                  └───────┬───────┘
                          │
                          v
                  ┌───────────────┐
                  │ Action        │
                  │ Dispatcher    │
                  └───────┬───────┘
                          │
              ┌───────────┼───────────┐
              v           v           v
        Image Search     OCR          AI
```

The selection engine should only be responsible for selecting and capturing content.

The action system determines what should happen with the captured content.

---

# 3. High-Level Architecture

Snap & Search consists of several major subsystems.

```text
┌──────────────────────────────────────────────────────────┐
│                    Snap & Search                         │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Background Application                │  │
│  │                                                    │  │
│  │  ┌──────────────┐       ┌──────────────────────┐   │  │
│  │  │    Global    │       │    System Tray       │   │  │
│  │  │    Hotkey    │       │    Integration       │   │  │
│  │  └──────┬───────┘       └──────────┬───────────┘   │  │
│  │         │                          │               │  │
│  │         v                          v               │  │
│  │  ┌────────────────────────────────────────────┐    │  │
│  │  │              Selection Engine              │    │  │
│  │  │                                            │    │  │
│  │  │  ┌────────────┐      ┌─────────────────┐   │    │  │
│  │  │  │  Overlay   │      │    Selection    │   │    │  │
│  │  │  │  Renderer  │─────>│    Controller   │   │    │  │
│  │  │  └────────────┘      └────────┬────────┘   │    │  │
│  │  │                               │            │    │  │
│  │  └───────────────────────────────┼────────────┘    │  │
│  │                                  v                 │  │
│  │                         ┌─────────────────┐        │  │
│  │                         │  Capture Engine │        │  │
│  │                         └────────┬────────┘        │  │
│  │                                  v                 │  │
│  │                         ┌─────────────────┐        │  │
│  │                         │     Image       │        │  │
│  │                         │   Processing    │        │  │
│  │                         └────────┬────────┘        │  │
│  │                                  v                 │  │
│  │                         ┌─────────────────┐        │  │
│  │                         │      Action     │        │  │
│  │                         │    Dispatcher   │        │  │
│  │                         └───────┬─────────┘        │  │
│  │                                 │                  │  │
│  │                  ┌──────────────┼──────────────┐   │  │
│  │                  v              v              v   │  │
│  │            Image Search        OCR            AI   │  │
│  │                (MVP)         (Future)      (Future)│  │
│  │                                                    │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │                 Desktop UI                         │  │
│  │                                                    │  │
│  │       Settings │ Search History │ Preferences      │  │
│  │                                                    │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

# 4. Core Components

## 4.1 Background Application

The background application is the primary runtime component of Snap & Search.

Responsibilities include:

- Application lifecycle.
- Global shortcut registration.
- Activation of the selection overlay.
- System tray integration.
- Coordination of the selection pipeline.
- Dispatching actions.
- Managing application state.

The application should be available without requiring a visible main window.

---

## 4.2 Global Shortcut Manager

The shortcut manager listens for the configured global keyboard shortcut.

When the shortcut is triggered:

```text
Shortcut Pressed
      │
      v
Activate Selection Engine
```

The shortcut should be configurable in the future.

The initial implementation will define a default shortcut while avoiding unnecessary conflicts with existing Windows shortcuts.

---

## 4.3 Selection Engine

The Selection Engine is the central interaction component.

Its responsibilities include:

- Displaying the selection overlay.
- Darkening the visible desktop.
- Tracking mouse input.
- Allowing the user to define a selection region.
- Handling keyboard cancellation.
- Returning the selected region.

The Selection Engine should not know what the selected content will be used for.

Its responsibility ends when it produces a valid selection.

```text
User Interaction
      │
      v
Selection Engine
      │
      v
Selection Region
```

---

## 4.4 Overlay Renderer

The Overlay Renderer is responsible for displaying the visual selection interface.

The initial experience should include:

- Fullscreen overlay.
- Dimmed background.
- Mouse interaction.
- Selection rectangle.
- Visual feedback during selection.
- Keyboard cancellation.

The overlay is intentionally designed to feel familiar to Windows users.

The overlay should be implemented independently from the settings and history interface because it has different performance and interaction requirements.

---

## 4.5 Capture Engine

The Capture Engine converts the selected screen region into image data.

```text
Selected Region
      │
      v
Capture Screen Pixels
      │
      v
Crop Region
      │
      v
Captured Image
```

The Capture Engine should expose a clean interface to the rest of the system and should not contain search-specific logic.

---

## 4.6 Image Processing

The image processing stage prepares captured content for downstream actions.

Potential responsibilities include:

- Cropping.
- Resizing.
- Encoding.
- Compression.
- Format conversion.

The initial implementation should keep processing minimal and only perform operations required for the Image Search workflow.

Additional processing can be introduced later for specific actions.

---

## 4.7 Action Dispatcher

The Action Dispatcher determines what should happen with captured content.

The conceptual flow is:

```text
Captured Content
      │
      v
Action Dispatcher
      │
      ├── Image Search
      ├── OCR
      ├── AI
      └── Future Actions
```

The dispatcher provides a boundary between the capture pipeline and the actions that consume captured content.

This prevents the selection engine from becoming tightly coupled to a particular capability.

---

## 4.8 Image Search Action

Image Search is the first action implemented by Snap & Search.

The initial workflow is:

```text
Captured Image
      │
      v
Prepare Search Request
      │
      v
Open Search Provider
      │
      v
Browser
      │
      v
Search Results
```

The initial implementation will focus on providing a seamless search experience using the user's browser.

The search provider should be isolated behind an action boundary so that additional providers can be supported in the future.

---

## 4.9 Search History

Search History stores previous searches locally.

A history record may include:

- Timestamp.
- Captured image or a reference to it.
- Search provider.
- Action used.

The history system should be independent from the search implementation.

Users should be able to:

- View previous searches.
- Revisit a previous search.
- Clear individual records.
- Clear all history.

---

## 4.10 Desktop UI

The Desktop UI provides functionality that does not need to be part of the always-visible overlay experience.

Initial responsibilities include:

- Search history.
- Settings.
- Shortcut configuration.
- Application preferences.
- Future action configuration.

The UI is separate from the native selection workflow.

The overlay is optimized for immediate interaction.

The Desktop UI is optimized for configuration and management.

---

# 5. Selection Pipeline

The selection pipeline is the primary user interaction.

```text
┌───────────────┐
│ User Presses  │
│ Global Hotkey │
└───────┬───────┘
        │
        v
┌───────────────┐
│ Activate      │
│ Overlay       │
└───────┬───────┘
        │
        v
┌───────────────┐
│ User Selects  │
│ Screen Region │
└───────┬───────┘
        │
        v
┌───────────────┐
│ Selection     │
│ Confirmed     │
└───────┬───────┘
        │
        v
┌───────────────┐
│ Capture       │
│ Selected Area │
└───────┬───────┘
        │
        v
┌───────────────┐
│ Process       │
│ Captured Data │
└───────┬───────┘
        │
        v
┌───────────────┐
│ Dispatch      │
│ Action        │
└───────────────┘
```

---

# 6. Action Pipeline

The Action Pipeline begins after content has been captured.

```text
Captured Content
      │
      v
Action Selection
      │
      v
Action Execution
      │
      v
Result
```

For the MVP:

```text
Captured Image
      │
      v
Image Search
      │
      v
Browser Results
```

Future actions can use the same captured content:

```text
Captured Content
      │
      ├── Image Search
      │
      ├── OCR
      │
      ├── AI Analysis
      │
      ├── Translation
      │
      └── Other Actions
```

---

# 7. Background Runtime Model

Snap & Search is designed to run primarily as a background application.

The normal lifecycle is:

```text
Application Starts
      │
      v
Initialize Runtime
      │
      ├── Register Global Shortcut
      ├── Initialize System Tray
      ├── Load Configuration
      └── Initialize Services
      │
      v
Wait for User Interaction
      │
      v
Shortcut Triggered
      │
      v
Run Selection Pipeline
      │
      v
Return to Background State
```

The application should not require a visible window to remain active.

---

# 8. Failure Handling

Each major stage should be able to fail independently.

Potential failures include:

- Shortcut registration failure.
- Overlay initialization failure.
- Screen capture failure.
- Invalid selection.
- Image processing failure.
- Search provider failure.
- Browser launch failure.

Failures should be handled explicitly rather than causing the entire application to terminate unexpectedly.

The application should:

1. Record useful diagnostic information.
2. Provide appropriate user feedback where necessary.
3. Return to a usable background state whenever possible.

---

# 9. Future Evolution

The architecture is intentionally designed to support additional screen intelligence capabilities.

The long-term model is:

```text
                    Screen Region
                         │
                         v
                    Selection
                         │
                         v
                    Capture
                         │
                         v
              ┌────────────────────┐
              │   Action System    │
              └─────────┬──────────┘
                        │
       ┌────────────────┼────────────────┐
       v                v                v
 Image Search          OCR              AI
       │                │                │
       v                v                v
  Web Results      Extracted Text    Analysis
```

The core interaction remains unchanged as new actions are introduced.

This allows Snap & Search to expand while preserving the experience users already understand.

---

# 10. Summary

Snap & Search is built around a focused pipeline:

```text
Shortcut
   │
   v
Selection
   │
   v
Capture
   │
   v
Process
   │
   v
Action
   │
   v
Result
```

The most important architectural principle is the separation between:

- **How content is selected**
- **How content is captured**
- **What happens to the captured content**

This separation allows the initial Image Search experience to remain focused while providing a foundation for future OCR, AI, translation, and other screen intelligence capabilities.

The system is designed to be:

- Native to Windows.
- Fast and responsive.
- Familiar to users.
- Modular.
- Extensible.
- Focused on screen-based search and understanding.
