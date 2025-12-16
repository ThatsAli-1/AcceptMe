# AcceptMe - League of Legends Auto Accept

An application that automatically accepts League of Legends matches when found, so you can step away from your keyboard during queue times.

## Features

- 🎮 Automatic connection to League of Legends client
- ✅ Auto-accept matches when found
- 🖥️ Modern React + Tauri UI with Tailwind CSS
- ⚡ Fast Rust backend
- 📊 Real-time status updates

## Tech Stack

- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Desktop Framework**: Tauri (Rust)
- **Backend**: Rust (native, no dependencies)
- **League API**: LCU (League Client Update) API

## Prerequisites

- Node.js (v18 or higher)
- Rust (latest stable) - Install from [rustup.rs](https://rustup.rs/)
- League of Legends client installed

## Installation

1. Install dependencies:
```bash
npm install
```

2. Install Tauri CLI (if not already installed):
```bash
npm install -g @tauri-apps/cli
```

## Development

Run the app in development mode:
```bash
npm run tauri dev
```

## Building

Build the application:
```bash
npm run tauri build
```

## How It Works

1. The app automatically detects when League of Legends client is running
2. It reads the lockfile to get connection credentials
3. Monitors the matchmaking API for ready checks
4. Automatically accepts matches when found

## Future Features

This project is designed to be scalable. Future features can include:
- Custom delay before accepting
- Sound notifications
- Match history tracking
- Auto-pick champions
- And more...

## License

MIT

