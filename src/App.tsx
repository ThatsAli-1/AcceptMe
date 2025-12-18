import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import ChampionPreferences from "./ChampionPreferences";

interface AppState {
  isConnected: boolean;
  isRunning: boolean;
  status: string;
  matchFound: boolean;
}

type View = "main" | "preferences";

function App() {
  const [view, setView] = useState<View>("main");
  const [state, setState] = useState<AppState>({
    isConnected: false,
    isRunning: false,
    status: "Initializing...",
    matchFound: false,
  });
  const [delaySeconds, setDelaySeconds] = useState(0);

  useEffect(() => {
    checkConnection();
    loadDelay();
    const interval = setInterval(() => {
      updateStatus();
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadDelay = async () => {
    try {
      const delay = await invoke<number>("get_accept_delay");
      setDelaySeconds(delay);
    } catch (error) {
      console.error("Error loading delay:", error);
    }
  };

  const handleDelayChange = async (newDelay: number) => {
    setDelaySeconds(newDelay);
    try {
      await invoke("set_accept_delay", { delaySeconds: newDelay });
    } catch (error) {
      console.error("Error setting delay:", error);
    }
  };

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

  if (view === "preferences") {
    return <ChampionPreferences onBack={() => setView("main")} />;
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-indigo-950 to-slate-950 flex items-center justify-center p-6 relative overflow-hidden">
      {/* Animated gradient orbs */}
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-blue-500/20 rounded-full blur-3xl animate-pulse"></div>
      <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-purple-500/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
      
      <div className="w-full max-w-md space-y-5 relative z-10">
        {/* Header */}
        <div className="text-center space-y-3 mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-br from-blue-500 via-indigo-500 to-purple-500 shadow-lg shadow-blue-500/30 mb-2 relative overflow-hidden group">
            <div className="absolute inset-0 bg-gradient-to-br from-white/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
            <svg className="w-8 h-8 text-white relative z-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
            </svg>
          </div>
          <h1 className="text-4xl font-bold bg-gradient-to-r from-blue-400 via-indigo-400 to-purple-400 bg-clip-text text-transparent">
            AcceptMe
          </h1>
          <p className="text-slate-400 text-sm">Auto-accept League matches</p>
        </div>

        {/* Navigation */}
        <div className="flex gap-2 bg-slate-900/50 backdrop-blur-xl rounded-xl p-1 border border-slate-800/50">
          <button
            onClick={() => setView("main")}
            className="flex-1 py-2 px-4 rounded-lg font-medium text-sm text-white bg-gradient-to-r from-blue-500 to-indigo-500 shadow-lg shadow-blue-500/30 transition-all duration-200"
          >
            Auto Accept
          </button>
          <button
            onClick={() => setView("preferences")}
            className="flex-1 py-2 px-4 rounded-lg font-medium text-sm text-slate-300 hover:text-white transition-colors"
          >
            Preferences
          </button>
        </div>

        {/* Status Card */}
        <div className="bg-slate-900/40 backdrop-blur-xl rounded-2xl border border-slate-800/50 p-6 shadow-xl">
          {/* Connection Status */}
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className={`w-2.5 h-2.5 rounded-full ${state.isConnected ? 'bg-emerald-400 shadow-lg shadow-emerald-400/50' : 'bg-red-400 shadow-lg shadow-red-400/50'} animate-pulse`}></div>
              <span className="text-slate-300 text-sm font-medium">Connection</span>
            </div>
            <span className={`text-sm font-semibold ${state.isConnected ? 'text-emerald-400' : 'text-red-400'}`}>
              {state.isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>

          {/* Status Message */}
          <div className="mb-6">
            <div className="flex items-center gap-2 mb-2">
              <svg className="w-4 h-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span className="text-slate-500 text-xs font-medium uppercase tracking-wider">Status</span>
            </div>
            <p className="text-white text-lg font-semibold">{state.status}</p>
          </div>

          {/* Match Found Alert */}
          {state.matchFound && (
            <div className="p-4 bg-gradient-to-r from-emerald-500/10 to-blue-500/10 rounded-xl border border-emerald-500/30 animate-pulse">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-emerald-500/20 flex items-center justify-center">
                  <svg className="w-6 h-6 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div>
                  <p className="text-emerald-300 font-semibold">Match Found!</p>
                  <p className="text-slate-400 text-sm">
                    {delaySeconds > 0 
                      ? `Waiting ${delaySeconds}s before accepting...`
                      : "Auto-accepting..."}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Delay Slider */}
        <div className="bg-slate-900/40 backdrop-blur-xl rounded-2xl border border-slate-800/50 p-6 shadow-xl">
          <div className="mb-4">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <svg className="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="text-slate-300 text-sm font-medium">Accept Delay</span>
              </div>
              <span className="text-blue-400 text-sm font-semibold">
                {delaySeconds === 0 ? "Instant" : `${delaySeconds}s`}
              </span>
            </div>
            <p className="text-slate-500 text-xs">Delay before accepting a match</p>
          </div>
          
          <div className="space-y-3">
            <input
              type="range"
              min="0"
              max="10"
              value={delaySeconds}
              onChange={(e) => handleDelayChange(Number(e.target.value))}
              className="w-full h-2 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
              style={{
                background: `linear-gradient(to right, rgb(59, 130, 246) 0%, rgb(59, 130, 246) ${(delaySeconds / 10) * 100}%, rgb(30, 41, 59) ${(delaySeconds / 10) * 100}%, rgb(30, 41, 59) 100%)`
              }}
            />
            <div className="flex justify-between text-xs text-slate-500">
              <span>0s</span>
              <span>5s</span>
              <span>10s</span>
            </div>
          </div>
        </div>

        {/* Control Button */}
        <button
          onClick={toggleAutoAccept}
          disabled={!state.isConnected && !state.isRunning}
          className={`w-full py-3.5 px-6 rounded-xl font-semibold text-white transition-all duration-200 ${
            state.isRunning
              ? 'bg-gradient-to-r from-red-500 to-rose-600 hover:from-red-400 hover:to-rose-500 shadow-lg shadow-red-500/30 hover:shadow-red-500/50'
              : 'bg-gradient-to-r from-blue-500 to-indigo-600 hover:from-blue-400 hover:to-indigo-500 shadow-lg shadow-blue-500/30 hover:shadow-blue-500/50'
          } disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:shadow-none`}
        >
          <div className="flex items-center justify-center gap-2">
            {state.isRunning ? (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
                </svg>
                <span>Stop Auto Accept</span>
              </>
            ) : (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Start Auto Accept</span>
              </>
            )}
          </div>
        </button>

        {/* Info Footer */}
        <div className="text-center space-y-1">
          <p className="text-slate-500 text-xs">Make sure League of Legends client is running</p>
          <p className="text-slate-600 text-xs">The app will automatically detect and connect</p>
        </div>
      </div>
    </div>
  );
}

export default App;
