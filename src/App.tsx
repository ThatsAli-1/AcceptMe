import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

interface AppState {
  isConnected: boolean;
  isRunning: boolean;
  status: string;
  matchFound: boolean;
}

function App() {
  const [state, setState] = useState<AppState>({
    isConnected: false,
    isRunning: false,
    status: "Initializing...",
    matchFound: false,
  });

  useEffect(() => {
    // Check connection status on mount
    checkConnection();
    
    // Set up interval to check status
    const interval = setInterval(() => {
      updateStatus();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  const checkConnection = async () => {
    try {
      const connected = await invoke<boolean>("check_league_connection");
      setState((prev) => ({ ...prev, isConnected: connected }));
    } catch (error) {
      console.error("Error checking connection:", error);
      setState((prev) => ({ ...prev, isConnected: false }));
    }
  };

  const updateStatus = async () => {
    try {
      const status = await invoke<string>("get_status");
      const running = await invoke<boolean>("is_running");
      const matchFound = await invoke<boolean>("is_match_found");
      
      setState((prev) => ({
        ...prev,
        status,
        isRunning: running,
        matchFound,
      }));
    } catch (error) {
      console.error("Error updating status:", error);
    }
  };

  const toggleAutoAccept = async () => {
    try {
      if (state.isRunning) {
        await invoke("stop_auto_accept");
      } else {
        await invoke("start_auto_accept");
      }
      await updateStatus();
    } catch (error) {
      console.error("Error toggling auto accept:", error);
    }
  };

  return (
    <div className="container">
      <h1>AcceptMe - League Auto Accept</h1>
      
      <div className="status-card">
        <div className="status-item">
          <span className="label">Connection:</span>
          <span className={`value ${state.isConnected ? "connected" : "disconnected"}`}>
            {state.isConnected ? "✓ Connected" : "✗ Disconnected"}
          </span>
        </div>
        
        <div className="status-item">
          <span className="label">Status:</span>
          <span className="value">{state.status}</span>
        </div>
        
        {state.matchFound && (
          <div className="match-found">
            🎮 Match Found! Auto-accepting...
          </div>
        )}
      </div>

      <button
        className={`control-button ${state.isRunning ? "stop" : "start"}`}
        onClick={toggleAutoAccept}
        disabled={!state.isConnected && !state.isRunning}
      >
        {state.isRunning ? "Stop Auto Accept" : "Start Auto Accept"}
      </button>

      <div className="info">
        <p>Make sure League of Legends client is running</p>
        <p>The app will automatically detect and connect to your client</p>
      </div>
    </div>
  );
}

export default App;

