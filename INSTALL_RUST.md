# Installing Rust for Tauri

## Quick Installation

### Option 1: Using rustup (Recommended)

1. **Download rustup-init.exe**
   - Visit: https://rustup.rs/
   - Or download directly: https://win.rustup.rs/x86_64

2. **Run the installer**
   - Double-click `rustup-init.exe`
   - Follow the prompts (default options are fine)
   - It will install Rust and Cargo

3. **Restart your terminal/PowerShell**
   - Close and reopen PowerShell/CMD
   - This ensures PATH is updated

4. **Verify installation**
   ```powershell
   rustc --version
   cargo --version
   ```

### Option 2: Using Chocolatey (if you have it)

```powershell
choco install rust
```

### Option 3: Using Scoop (if you have it)

```powershell
scoop install rust
```

## After Installation

1. **Restart your terminal** - This is important!
2. **Verify it works:**
   ```powershell
   cargo --version
   ```
3. **Then run your Tauri app:**
   ```powershell
   npm run tauri dev
   ```

## Troubleshooting

### If cargo is still not found after installation:
1. Close ALL terminal windows
2. Open a NEW PowerShell window
3. Try `cargo --version` again

### If it still doesn't work:
- Check if Rust is in your PATH:
  ```powershell
  $env:PATH -split ';' | Select-String rust
  ```
- Manually add Rust to PATH if needed (usually `C:\Users\YourName\.cargo\bin`)

