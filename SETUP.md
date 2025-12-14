# Setup Instructions

## Step 1: Install Dependencies

### Node.js Dependencies
```bash
npm install
```

### Python Dependencies
```bash
cd python_backend
pip install -r requirements.txt
cd ..
```

### Rust (for Tauri)
If you don't have Rust installed, install it from [rustup.rs](https://rustup.rs/)

## Step 2: Install Tauri CLI (if needed)
```bash
npm install -g @tauri-apps/cli
```

## Step 3: Run the Application

### Development Mode
```bash
npm run tauri dev
```

This will:
- Start the React dev server
- Compile the Rust backend
- Launch the Tauri application

## Step 4: Using the App

1. **Start League of Legends client** - Make sure the game client is running
2. **Launch AcceptMe** - Run `npm run tauri dev` or build the app
3. **Wait for connection** - The app will automatically detect your League client
4. **Start Auto-Accept** - Click the "Start Auto Accept" button
5. **Queue up** - Join a queue in League of Legends
6. **Relax** - The app will automatically accept matches for you!

## Building for Production

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release/`

## Notes

- The app reads the League client lockfile to get connection credentials
- It uses the LCU (League Client Update) API to interact with the client
- The app checks for matches every second when running
- Make sure League of Legends is installed in the default location

## Troubleshooting

### "Disconnected" Status
- Make sure League of Legends client is running
- Check that the client is fully loaded (not just the launcher)
- Try restarting the League client

### App Won't Start
- Make sure Rust is installed: `rustc --version`
- Make sure Node.js is installed: `node --version`
- Try deleting `node_modules` and running `npm install` again

### Match Not Being Accepted
- Make sure you clicked "Start Auto Accept"
- Check that you're actually in queue
- The app only accepts when a match is found (ready check appears)

