"use client"

import "./App.css";
import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
  const overlayRef = useRef<HTMLElement>(null);

    useEffect(() => {
    console.log("🔥 React App Mounted!");
    
    try {
      const currentWindow = getCurrentWindow();
      console.log("✅ Current window identified:", currentWindow.label);

      const handleKeyDown = async (event: KeyboardEvent) => {
        console.log("⌨️ Key pressed:", event.key);
        if (event.key === "Escape") {
          console.log("Closing window...");
          await currentWindow.close();
        }
      };

      // Listen on the document instead of window
      document.addEventListener("keydown", handleKeyDown);

      return () => {
        document.removeEventListener("keydown", handleKeyDown);
      };
    } catch (error) {
      console.error("❌ Error in useEffect:", error);
    }
  }, []);

  return (
    <main
      ref={overlayRef}
    >
    </main>
  );
}

export default App;