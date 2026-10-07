import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface AppSettings {
  hotkey: string;
  save_dir: string;
}

function App() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [screenshots, setScreenshots] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [recordingHotkey, setRecordingHotkey] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((s) => setSettings(s))
      .catch(console.error);
      
    invoke<string[]>("get_screenshots")
      .then((s) => setScreenshots(s))
      .catch(console.error);
  }, []);

  const handleSave = async () => {
    if (settings) {
      try {
        await invoke("save_settings", { newSettings: settings });
        setStatus("Settings saved! (Restart app to apply hotkey changes)");
        setTimeout(() => setStatus(""), 4000);
      } catch (err) {
        setStatus(`Error saving settings: ${err}`);
      }
    }
  };

  const selectFolder = async () => {
    if (!settings) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select where to save screenshots"
      });
      if (selected && typeof selected === "string") {
        let finalPath = selected;
        // If they didn't explicitly pick a folder named 'Screen Captures', append it
        if (!finalPath.endsWith("Screen Captures")) {
          finalPath = finalPath.replace(/\\$/, "") + "\\Screen Captures";
        }
        setSettings({ ...settings, save_dir: finalPath });
      }
    } catch (err) {
      console.error(err);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!recordingHotkey) return;
    e.preventDefault();
    
    // Ignore lone modifier keys
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
      return;
    }
    
    let parts = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    
    let key = e.key.toUpperCase();
    if (key === " ") key = "SPACE";
    
    parts.push(key);
    
    setSettings(prev => prev ? { ...prev, hotkey: parts.join("+") } : null);
    setRecordingHotkey(false);
  };

  return (
    <main className="container">
      <header className="hero">
        <img src="/search.png" alt="Logo" className="logo" />
        <h1>Snap & Search</h1>
        <p>Configure your Preferences.</p>
      </header>
      
      {settings && (
        <section className="card settings-panel">
          <h2>Preferences</h2>
          
          <div className="setting-group">
            <label>Screenshots Save Directory</label>
            <div className="input-with-button">
              <input 
                type="text" 
                value={settings.save_dir} 
                onChange={(e) => setSettings({...settings, save_dir: e.target.value})} 
              />
              <button onClick={selectFolder} className="secondary-btn">Browse...</button>
            </div>
            <small>New snips will automatically be placed in a "Screen Captures" folder here.</small>
          </div>
          
          <div className="setting-group">
            <label>Global Activation Hotkey</label>
            <div className="input-with-button">
              <input 
                type="text" 
                value={recordingHotkey ? "Listening..." : settings.hotkey} 
                onFocus={() => setRecordingHotkey(true)}
                onBlur={() => setRecordingHotkey(false)}
                onKeyDown={handleKeyDown}
                className={recordingHotkey ? "recording" : ""}
                readOnly
              />
            </div>
            <small>Click to record a new shortcut (e.g. Ctrl+Shift+X).</small>
          </div>

          <div className="actions">
            <button onClick={handleSave} className="primary-btn">Save Changes</button>
            {status && <span className="status-msg">{status}</span>}
          </div>
        </section>
      )}

      <section className="card history-panel">
        <h2>Recent Captures ({screenshots.length})</h2>
        <div className="gallery">
          {screenshots.length === 0 ? (
             <div className="empty-state">No snaps yet. Try pressing your hotkey!</div>
          ) : null}
          {screenshots.slice(0, 5).map((name) => (
             <div key={name} className="gallery-item">
               <span className="file-icon">🖼️</span>
               <span>{name}</span>
             </div>
          ))}
          {screenshots.length > 5 && (
            <div className="gallery-item more-item">...and {screenshots.length - 5} more</div>
          )}
        </div>
      </section>
    </main>
  );
}

export default App;