# RoboRIO File Explorer

A lightweight, fast file explorer for RoboRIO robots. Connect via **Phoenix Tuner X Diagnostic Server** OR **SSH/SFTP** - your choice! Built with Rust + Tauri for maximum performance and minimal resource usage.

## Features

✨ **Dual Connection Modes** - Use Phoenix Tuner X or SSH/SFTP, whatever works best for you
📁 **File Management** - Browse, upload, download, and delete files on your RoboRIO
🔒 **Secure Connections** - Either Phoenix server or encrypted SSH/SFTP
💾 **Easy Windows Installation** - Single .msi installer, no dependencies needed
🎨 **Modern UI** - Clean, intuitive interface
⚡ **Quick Start** - Just enter your RoboRIO host and connect
🔌 **No Extra Setup** - Uses what's already on your robot

## Installation

### Windows (Recommended)
1. Download the latest `.msi` installer from [Releases](https://github.com/wolftech07/rio-file/releases)
2. Run the installer
3. Launch "RoboRIO File Explorer" from your Start Menu

### From Source
**Requirements:**
- Rust 1.70+ ([install here](https://www.rust-lang.org/tools/install))
- Node.js 16+ ([install here](https://nodejs.org))

```bash
# Clone repository
git clone https://github.com/wolftech07/rio-file.git
cd rio-file

# Install dependencies
npm install
cd src-tauri && cargo build --release

# Run in development
npm run dev
```

## Usage

### Phoenix Tuner X Mode (Recommended)
1. Select "Phoenix Tuner X" tab
2. Enter your RoboRIO host: `roboRIO-TEAM.local` or `10.xx.xx.2`
3. Click "Connect"
4. Ensure Phoenix Tuner X is running on your RoboRIO (it runs by default)

**Pros:**
- Simplest setup (no credentials needed)
- Fast and lightweight
- Uses existing Phoenix server
- Great for read-only file browsing

### SSH/SFTP Mode
1. Select "SSH/SFTP" tab
2. Enter RoboRIO host: `roboRIO-TEAM.local` or `10.xx.xx.2`
3. Enter username (default: `lvuser`)
4. Enter password
5. Click "Connect"

**Pros:**
- Works if Phoenix Tuner X is disabled
- Full command-line compatibility
- More file operation options
- Industry standard protocol

### File Operations
Both modes support:
- **Navigate** - Double-click folders to open
- **Download** - Save files from robot to your computer
- **Delete** - Remove files from robot
- **Refresh** - Reload current directory
- **Back** - Go to parent directory

## System Requirements

- **Windows 7+** (64-bit recommended)
- **100MB** disk space
- **256MB** RAM minimum
- Network connection to RoboRIO

## Troubleshooting

### Phoenix Tuner X Mode Issues

**Can't connect to Phoenix server**
- Ensure Phoenix Tuner X is running on the RoboRIO (check with Phoenix Tuner X app)
- Verify you can access: `http://roboRIO-TEAM.local:5800`
- Check RoboRIO is powered on and networked
- Ensure RoboRIO is connected to same network

**Port 5800 Not Responding**
- Phoenix Tuner X diagnostic server should be running (enabled by default)
- Restart RoboRIO if needed

### SSH/SFTP Mode Issues

**Authentication failed**
- Verify username is correct (default: `lvuser`)
- Check password is correct
- Ensure SSH is enabled on RoboRIO

**Can't connect to SSH**
- Ensure RoboRIO is connected to same network
- Check host name matches your team number
- Try pinging: `ping roboRIO-TEAM.local`
- Verify SSH service is running

### General Issues

**Permission Denied (SSH mode)**
- Some directories require elevated privileges
- Try `/home/lvuser` instead of `/root`
- Use Phoenix mode which has broader access

**Slow Performance**
- Check network connection quality
- Avoid large file operations over slow connections
- Consider SSH mode for better stability over poor connections

## Development

Built with:
- **Tauri** - Lightweight desktop app framework
- **Rust** - High-performance backend
- **Vue 3** - Modern, responsive UI
- **reqwest** - HTTP client for Phoenix Tuner X
- **ssh2** - SSH/SFTP client for direct connections

## Architecture

The app supports two independent connection modes:

**Phoenix Tuner X Mode (HTTP/REST)**
- Connects to port 5800 on RoboRIO
- Uses REST API endpoints for file operations
- No authentication required by default
- Perfect for quick file access

**SSH/SFTP Mode (Secure Shell)**
- Connects to port 22 on RoboRIO
- Uses standard SSH/SFTP protocols
- Username/password authentication
- Most compatible and flexible option

Both modes work independently - switch between them in the UI without restarting the app.

## License

MIT - Feel free to use and modify!

## Contributing

Found a bug? Have a feature idea? Open an issue or submit a pull request!