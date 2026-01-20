# Build Notes

## Building the Application

### Prerequisites
- Rust 1.70+: https://www.rust-lang.org/tools/install
- Node.js 16+: https://nodejs.org
- On Windows, Visual Studio Build Tools are required for Rust compilation

### Development Build
```bash
npm install
npm run dev
```

### Production Build (Windows Installer)
```bash
npm install
npm run tauri build
```

The `.msi` installer will be in `src-tauri/target/release/bundle/msi/`

## Project Structure

```
rio-file/
├── src/                    # Vue.js frontend source
│   ├── App.vue            # Main application component
│   ├── main.js            # Entry point
│   └── style.css          # Styling
├── src-tauri/             # Rust/Tauri backend
│   ├── src/
│   │   └── main.rs        # Rust backend logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Node.js dependencies
├── vite.config.js         # Vite configuration
└── index.html             # HTML entry point
```

## Key Dependencies

- **Tauri 1.5** - Desktop app framework
- **Vue 3** - Frontend framework
- **Vite** - Build tool
- **Paramiko/SSH2** - SFTP protocol
- **Rust stdlib** - No heavyweight dependencies

## Size & Performance

- Binary size: ~15-20MB (including runtime)
- Memory usage: 50-100MB at idle
- Startup time: <1 second
- File operations: Limited by network speed, not application

## Windows Installer Details

The installer (`rio-file-explorer_1.0.0_x64_en-US.msi`) includes:
- NSIS installer option
- Single-click or advanced installation
- Auto-updates ready
- Uninstall support

## Building for Other Platforms

### macOS
```bash
npm run tauri build -- --target universal-apple-darwin
```

### Linux (AppImage)
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```
