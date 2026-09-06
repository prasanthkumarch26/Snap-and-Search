"use client"

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Screenshot {
  name: string;
  data: string;
}

function App() {
  const [screenshots, setScreenshots] = useState<Screenshot[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadScreenshots();
  }, []);

  const loadScreenshots = async () => {
    try {
      setLoading(true);
      const names: string[] = await invoke("get_screenshots");
      
      const loaded: Screenshot[] = [];
      for (const name of names) {
        try {
          const base64Data: string = await invoke("get_screenshot_base64", { name });
          loaded.push({ name, data: `data:image/png;base64,${base64Data}` });
        } catch (e) {
          console.error(`Failed to load ${name}`, e);
        }
      }
      
      setScreenshots(loaded);
    } catch (error) {
      console.error("Failed to load screenshots:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="container">
      <header>
        <h1>Snap & Search</h1>
        <p>Your screen intelligence history and settings</p>
      </header>
      
      <section className="settings-panel">
        <h2>Settings</h2>
        <div className="setting-item">
          <label>Global Hotkey:</label>
          <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>S</kbd>
        </div>
        <p className="setting-note">
          Keep this app running in the background. It hides to the system tray when closed.
        </p>
      </section>

      <section className="history-panel">
        <div className="history-header">
          <h2>Capture History</h2>
          <button onClick={loadScreenshots} className="refresh-btn">
            ↻ Refresh
          </button>
        </div>
        
        {loading ? (
          <p className="loading">Loading your history...</p>
        ) : screenshots.length === 0 ? (
          <div className="empty-state">
            <p>No screenshots saved yet.</p>
            <p>Use the hotkey and choose "Save Screenshot" to see them here.</p>
          </div>
        ) : (
          <div className="gallery">
            {screenshots.map((s) => (
              <div key={s.name} className="gallery-item">
                <img src={s.data} alt={s.name} loading="lazy" />
                <div className="item-name">{s.name}</div>
              </div>
            ))}
          </div>
        )}
      </section>
    </main>
  );
}

export default App;