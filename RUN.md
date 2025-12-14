# How to Run AcceptMe

## Quick Start

1. **Open PowerShell** in the project directory:
   ```powershell
   cd C:\Users\ActualAli\Desktop\Codes\PROJECTS\AceeptMe2
   ```

2. **Refresh PATH** (only needed once per PowerShell session):
   ```powershell
   $env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
   ```

3. **Run the app**:
   ```powershell
   npm run tauri dev
   ```

## What Happens:

1. **First time**: Takes 3-5 minutes (downloads and compiles Rust dependencies)
2. **Subsequent runs**: Takes 10-30 seconds (only recompiles changed code)
3. **App window opens**: Automatically when ready

## If You Get "cargo not found" Error:

The PATH refresh command above should fix it. If not:
- Close and reopen PowerShell
- Or restart your computer (after first Rust installation)

## To Stop the App:

Press `Ctrl+C` in the PowerShell window

## Troubleshooting:

- **"cargo not found"**: Run the PATH refresh command above
- **"icons not found"**: Run `python create_icons.py` first
- **Build errors**: Make sure you're in the project root directory

