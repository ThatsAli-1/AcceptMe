import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

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
    <div className="flex flex-col items-center justify-center min-h-screen p-8 bg-gradient-to-br from-indigo-500 via-purple-500 to-purple-600 text-white font-sans">
      <h1 className="mb-8 text-4xl text-center drop-shadow-lg">
        AcceptMe - League Auto Accept
      </h1>
      
      <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 mb-8 min-w-[400px] shadow-2xl border border-white/20">
        <div className="flex justify-between items-center mb-4 text-lg last:mb-0">
          <span className="font-semibold opacity-90">Connection:</span>
          <span className={`font-medium ${state.isConnected ? "text-green-400" : "text-red-400"}`}>
            {state.isConnected ? "✓ Connected" : "✗ Disconnected"}
          </span>
        </div>
        
        <div className="flex justify-between items-center mb-4 text-lg last:mb-0">
          <span className="font-semibold opacity-90">Status:</span>
          <span className="font-medium">{state.status}</span>
        </div>
        
        {state.matchFound && (
          <div className="mt-4 p-4 bg-green-500/20 rounded-lg text-center font-semibold text-xl animate-pulse">
            🎮 Match Found! Auto-accepting...
          </div>
        )}
      </div>

      <button
        className={`px-8 py-4 text-xl font-semibold border-none rounded-xl cursor-pointer transition-all duration-300 shadow-lg min-w-[200px] ${
          state.isRunning
            ? "bg-red-500 hover:bg-red-600 hover:-translate-y-0.5 hover:shadow-red-500/40"
            : "bg-green-500 hover:bg-green-600 hover:-translate-y-0.5 hover:shadow-green-500/40"
        } disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0`}
        onClick={toggleAutoAccept}
        disabled={!state.isConnected && !state.isRunning}
      >
        {state.isRunning ? "Stop Auto Accept" : "Start Auto Accept"}
      </button>

      <div className="mt-8 text-center opacity-80 text-sm">
        <p className="my-2">Make sure League of Legends client is running</p>
        <p className="my-2">The app will automatically detect and connect to your client</p>
      </div>
    </div>
  );
}

export default App;

