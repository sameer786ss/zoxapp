# ZOX Development Setup (macOS)

## 1. Prerequisites

### System Tools
Ensure you have the Xcode Command Line Tools installed (required for the Rust linker):
```bash
xcode-select --install
```

### Node.js (Frontend)
Install Node.js (v18 or newer recommended).
```bash
node -v
# Should be v18.0.0 or higher
```

### Rust (Backend)
Install Rust via `rustup`:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Restart your terminal or run `source $HOME/.cargo/env`.

Verify installation:
```bash
rustc --version
# Should be 1.75+ or newer
```

## 2. Installation

Navigate to the project directory:
```bash
cd /Users/shaiksameer/.gemini/antigravity/scratch/my-agent-ide
```

Install frontend dependencies:
```bash
npm install
```

## 3. Running Development Mode

To start the app in development mode (hot-reload for both Frontend and Rust):

```bash
npm run tauri dev
```

> **Note:** The first run will take several minutes to compile all Rust dependencies. Subsequent runs will be much faster.

## 4. Building for Production

To create a release application (`.app` and `.dmg`):

```bash
npm run tauri build
```
The output will be in `src-tauri/target/release/bundle/macos/`.

## Troubleshooting

- **Permissions:** If you get permission errors, ensure you have read/write access to the `src-tauri` directory.
- **Microphone/Camera:** If you use features requiring permissions, you may need to add entries to `Info.plist` (though this app looks like a coding agent, so primarily file access).
