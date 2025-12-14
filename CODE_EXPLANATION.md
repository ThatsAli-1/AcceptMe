# Code Explanation - AcceptMe App

## 📋 Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Frontend (React/TypeScript)](#frontend-reacttypescript)
3. [Backend (Rust/Tauri)](#backend-rusttauri)
4. [How They Communicate](#how-they-communicate)
5. [League Client Integration](#league-client-integration)
6. [Auto-Accept Logic](#auto-accept-logic)

---

## 🏗️ Architecture Overview

The app uses a **Tauri** architecture:
- **Frontend**: React + TypeScript (runs in a webview)
- **Backend**: Rust (runs as native code)
- **Communication**: Tauri's `invoke` system (like a bridge)

```
┌─────────────────┐
│  React UI       │  ← User sees this
│  (TypeScript)   │
└────────┬────────┘
         │ invoke("command_name")
         ▼
┌─────────────────┐
│  Tauri Bridge   │  ← Communication layer
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Rust Backend   │  ← Does the actual work
│  (main.rs)      │
└────────┬────────┘
         │ HTTP requests
         ▼
┌─────────────────┐
│  League Client  │  ← League of Legends
│  (LCU API)      │
└─────────────────┘
```

---

## 🎨 Frontend (React/TypeScript)

### File: `src/App.tsx`

This is the user interface - what you see on screen.

#### **State Management** (Lines 5-18)
```typescript
interface AppState {
  isConnected: boolean;    // Is League client connected?
  isRunning: boolean;      // Is auto-accept active?
  status: string;          // Current status message
  matchFound: boolean;     // Has a match been found?
}
```
- Stores the app's current state
- Updates when backend sends new info

#### **useEffect Hook** (Lines 20-30)
```typescript
useEffect(() => {
  checkConnection();              // Check on startup
  const interval = setInterval(() => {
    updateStatus();                // Update every 2 seconds
  }, 2000);
  return () => clearInterval(interval);  // Cleanup
}, []);
```
- Runs when component loads
- Sets up a timer to check status every 2 seconds
- Cleans up when component unmounts

#### **checkConnection()** (Lines 32-40)
```typescript
const connected = await invoke<boolean>("check_league_connection");
```
- Calls the Rust backend function `check_league_connection`
- `invoke` is Tauri's way to call backend functions from frontend
- Updates UI with connection status

#### **updateStatus()** (Lines 42-57)
```typescript
const status = await invoke<string>("get_status");
const running = await invoke<boolean>("is_running");
const matchFound = await invoke<boolean>("is_match_found");
```
- Gets current status from backend
- Updates all UI elements with latest info
- Called every 2 seconds automatically

#### **toggleAutoAccept()** (Lines 59-70)
```typescript
if (state.isRunning) {
  await invoke("stop_auto_accept");
} else {
  await invoke("start_auto_accept");
}
```
- Starts or stops the auto-accept feature
- Called when user clicks the button

#### **UI Rendering** (Lines 72-108)
- Shows connection status (green checkmark or red X)
- Displays current status message
- Shows "Match Found!" notification when match is detected
- Button to start/stop auto-accept

---

## ⚙️ Backend (Rust/Tauri)

### File: `src-tauri/src/main.rs`

This is the "brain" - it does all the actual work.

#### **Data Structures** (Lines 8-34)

**LeagueClientInfo** (Lines 8-13)
```rust
struct LeagueClientInfo {
    port: u16,           // Port number (e.g., 53677)
    password: String,    // Password from lockfile
    protocol: String,    // "https" or "http"
}
```
- Stores connection info needed to talk to League client
- Extracted from the lockfile

**AppState** (Lines 15-22)
```rust
struct AppState {
    is_running: Arc<Mutex<bool>>,      // Thread-safe boolean
    is_connected: Arc<Mutex<bool>>,     // Thread-safe boolean
    status: Arc<Mutex<String>>,         // Thread-safe string
    match_found: Arc<Mutex<bool>>,       // Thread-safe boolean
    client_info: Arc<Mutex<Option<LeagueClientInfo>>>,
}
```
- `Arc` = Atomic Reference Counted (shared ownership)
- `Mutex` = Mutual Exclusion (thread-safe access)
- Multiple parts of code can safely read/write this state

#### **get_league_client_info()** (Lines 37-77)

**Purpose**: Find and read the League client lockfile

**How it works**:
1. Tries multiple possible file paths:
   ```rust
   "C:\Riot Games\League of Legends\lockfile"
   "%PROGRAMFILES%\Riot Games\League of Legends\lockfile"
   // ... etc
   ```

2. Reads the lockfile (format: `name:pid:port:password:protocol`)
   ```
   Example: LeagueClient:37388:53677:eRhBMl_CYffcQEb0yAnHZw:https
            └─name─┘ └pid┘ └port┘ └──password──┘ └protocol┘
   ```

3. Extracts port, password, and protocol
4. Returns `LeagueClientInfo` if found, `None` if not

#### **check_league_connection_internal()** (Lines 80-106)

**Purpose**: Verify League client is running and accessible

**How it works**:
1. Gets lockfile info
2. Creates HTTP client (allows invalid SSL certs - needed for localhost)
3. Makes GET request to: `https://127.0.0.1:PORT/lol-summoner/v1/current-summoner`
4. Uses Basic Auth with username "riot" and password from lockfile
5. If successful → client is connected!
6. Saves connection info for later use

#### **check_match_found()** (Lines 132-157)

**Purpose**: Check if a match has been found (ready check appeared)

**How it works**:
1. Gets saved connection info
2. Makes GET request to: `https://127.0.0.1:PORT/lol-matchmaking/v1/ready-check`
3. Parses JSON response
4. Checks if `state` field equals `"InProgress"`
5. Returns `true` if match found, `false` otherwise

#### **accept_match()** (Lines 109-129)

**Purpose**: Accept the match when found

**How it works**:
1. Gets saved connection info
2. Makes POST request to: `https://127.0.0.1:PORT/lol-matchmaking/v1/ready-check/accept`
3. Uses Basic Auth
4. Returns `true` if successful (HTTP 204), `false` if failed

#### **auto_accept_loop()** (Lines 160-193)

**Purpose**: The main monitoring loop that runs in background

**How it works**:
```rust
loop {  // Infinite loop
    // 1. Check if auto-accept is enabled
    if !is_running {
        sleep 1 second
        continue  // Skip to next iteration
    }
    
    // 2. Check if League client is connected
    if !connected {
        status = "Waiting for League client..."
        sleep 2 seconds
        continue
    }
    
    // 3. Check if match found
    if match_found {
        status = "Match found! Auto-accepting..."
        accept_match()  // Try to accept
        if successful {
            status = "Match accepted!"
            sleep 3 seconds
        }
    } else {
        status = "Waiting for match..."
    }
    
    sleep 1 second  // Wait before next check
}
```

**Key Points**:
- Runs continuously in background
- Only works when `is_running` is `true`
- Checks every 1 second
- Updates status messages for UI

#### **Tauri Commands** (Lines 195-227)

These are functions the frontend can call:

```rust
#[tauri::command]  // Makes function callable from frontend
async fn check_league_connection(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(check_league_connection_internal(&state).await)
}
```

**Available Commands**:
- `check_league_connection` → Returns if connected
- `get_status` → Returns current status string
- `is_running` → Returns if auto-accept is active
- `is_match_found` → Returns if match was found
- `start_auto_accept` → Starts monitoring
- `stop_auto_accept` → Stops monitoring

#### **main() Function** (Lines 229-250)

**Purpose**: Application entry point

**How it works**:
1. Creates `AppState` (shared state)
2. Starts background loop in separate thread:
   ```rust
   tauri::async_runtime::spawn(async move {
       auto_accept_loop(state_clone).await;
   });
   ```
3. Registers Tauri commands
4. Starts Tauri app (opens window, loads React UI)

---

## 🔗 How They Communicate

### Frontend → Backend

**React calls Rust function**:
```typescript
const result = await invoke<boolean>("check_league_connection");
```

**What happens**:
1. Frontend sends message: "call check_league_connection"
2. Tauri bridge receives it
3. Rust function `check_league_connection()` executes
4. Result sent back to frontend
5. React updates UI with result

### Backend → Frontend

**Rust updates state, frontend polls**:
- Rust updates `AppState.status` in background loop
- Frontend calls `get_status()` every 2 seconds
- Gets latest status and updates UI

**Alternative** (not used here, but possible):
- Rust could emit events to frontend
- Frontend listens for events
- More efficient, but polling works fine for this app

---

## 🎮 League Client Integration

### What is the LCU API?

**LCU** = League Client Update API
- League client runs a local web server
- Exposes REST API endpoints
- Used by client UI to get game data
- We can use it too!

### The Lockfile

**Location**: `C:\Riot Games\League of Legends\lockfile`

**Format**: `name:pid:port:password:protocol`
```
LeagueClient:37388:53677:eRhBMl_CYffcQEb0yAnHZw:https
```

**Why it exists**:
- League client needs to communicate with itself
- Lockfile provides credentials for local API
- Changes every time client starts (new port/password)

### API Endpoints Used

1. **Get Summoner Info** (Connection Check)
   ```
   GET https://127.0.0.1:PORT/lol-summoner/v1/current-summoner
   ```
   - Tests if API is accessible
   - Returns 200 if connected

2. **Check Ready Check** (Match Detection)
   ```
   GET https://127.0.0.1:PORT/lol-matchmaking/v1/ready-check
   ```
   - Returns JSON with `state` field
   - `"InProgress"` = match found, need to accept
   - `null` or other = no match

3. **Accept Match**
   ```
   POST https://127.0.0.1:PORT/lol-matchmaking/v1/ready-check/accept
   ```
   - Accepts the match
   - Returns 204 (No Content) on success

### Authentication

All requests use **HTTP Basic Authentication**:
- Username: `"riot"`
- Password: From lockfile (changes each session)

---

## ✅ Auto-Accept Logic Flow

### Complete Flow Diagram

```
1. App Starts
   ↓
2. Background Loop Starts (auto_accept_loop)
   ↓
3. User Clicks "Start Auto Accept"
   ↓
4. Loop Checks: is_running = true?
   ├─ No → Sleep 1s, repeat
   └─ Yes → Continue
   ↓
5. Check League Connection
   ├─ Not Connected → Status: "Waiting for League client..."
   │                    Sleep 2s, repeat from step 4
   └─ Connected → Continue
   ↓
6. Check for Match (every 1 second)
   ├─ No Match → Status: "Waiting for match..."
   │              Sleep 1s, repeat from step 4
   └─ Match Found → Continue
   ↓
7. Match Found!
   ↓
8. Status: "Match found! Auto-accepting..."
   ↓
9. Call accept_match()
   ├─ Success → Status: "Match accepted!"
   │             Sleep 3s, repeat from step 4
   └─ Failed → Status: "Failed to accept"
                Sleep 1s, repeat from step 4
```

### Key Timing

- **Connection check**: Every 2 seconds (when not connected)
- **Match check**: Every 1 second (when connected)
- **After accept**: Wait 3 seconds before checking again
- **UI update**: Frontend polls every 2 seconds

---

## 📦 Dependencies

### Frontend (`package.json`)
- **React**: UI framework
- **TypeScript**: Type safety
- **Vite**: Build tool & dev server
- **@tauri-apps/api**: Communication with Rust

### Backend (`Cargo.toml`)
- **tauri**: Desktop app framework
- **tokio**: Async runtime (for background tasks)
- **reqwest**: HTTP client (for API calls)
- **serde/serde_json**: JSON serialization
- **winreg**: Windows registry (not used yet, but available)

---

## 🔧 Key Concepts Explained

### Thread Safety
- Multiple parts of code access same data
- `Arc<Mutex<T>>` ensures safe concurrent access
- `Arc` = share data between threads
- `Mutex` = only one thread can modify at a time

### Async/Await
- Non-blocking operations
- Can wait for network requests without freezing
- `async fn` = function that can wait
- `.await` = wait for async operation to complete

### Option Type
- Rust's way of handling "maybe exists"
- `Option<T>` = either `Some(value)` or `None`
- Prevents null pointer errors

### Result Type
- Rust's way of handling errors
- `Result<T, E>` = either `Ok(value)` or `Err(error)`
- Forces error handling

---

## 🚀 How to Extend

### Add New Feature: Sound Notification

**Frontend** (`src/App.tsx`):
```typescript
// Add sound when match found
useEffect(() => {
  if (state.matchFound) {
    const audio = new Audio('/notification.mp3');
    audio.play();
  }
}, [state.matchFound]);
```

**Backend** (`src-tauri/src/main.rs`):
```rust
// Add command to play sound
#[tauri::command]
async fn play_sound() -> Result<(), String> {
    // Use Windows API or play file
    Ok(())
}
```

### Add New Feature: Delay Before Accept

**Backend** (`src-tauri/src/main.rs`):
```rust
// In auto_accept_loop, after match found:
if match_found {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;  // Wait 2 seconds
    accept_match(&state).await;
}
```

---

## 📝 Summary

1. **Frontend (React)**: Shows UI, calls backend functions
2. **Backend (Rust)**: Does the work, talks to League API
3. **Communication**: Tauri's `invoke` system
4. **League API**: Local REST API, credentials from lockfile
5. **Auto-Accept**: Background loop checks every second, accepts when match found

The app is **scalable** - easy to add features like:
- Sound notifications
- Custom delays
- Match history
- Statistics
- Settings panel

