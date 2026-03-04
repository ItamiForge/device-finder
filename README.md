# device-finder

Cross-platform device discovery CLI and TUI for developers. Find iOS simulators, Android devices/emulators, and web containers instantly.

## Features

- Detect iOS simulators via `xcrun simctl`
- Discover Android devices and emulators via `adb`
- Find available browsers and Docker web containers
- List all devices with full details
- Search devices by name, ID, or model
- Show detailed device information
- Launch devices (boot simulators, start emulators)
- Install apps on devices
- Connect to devices
- Interactive TUI mode (`df`)
- JSON output for all commands
- Stable exit codes for automation
- Configuration file support
- Works on macOS, Linux, and Windows

## Install

### From crates.io

```bash
cargo install device-finder
```

### From GitHub

```bash
cargo install --git https://github.com/ItamiForge/device-finder.git
```

### From local source

```bash
cargo install --path .
```

### Verify

```bash
df --version
```

## Quick usage

```bash
df
df list
df list --platform ios
df find "iPhone"
df find "Pixel" --json
df info "device-id"
df launch "device-id"
df install "device-id" "/path/to/bundle"
df connect "device-id"
```

## Command reference

- `df`: launch interactive TUI mode
- `df list [--platform <ios|android|web>] [--verbose] [--json]`: list all devices
- `df find <query> [--json]`: search devices by name, ID, or model
- `df info <device-id> [--json]`: show detailed device information
- `df launch <device-id> [--json]`: boot/launch a device
- `df install <device-id> <bundle-path> [--json]`: install app on device
- `df connect <device-id> [--json]`: establish connection to device

### JSON output

All commands support `--json` for machine-readable output:

- `df list --json`: JSON array of device objects
- `df find "query" --json`: object with `query`, `found`, and `devices` array
- `df info "id" --json`: single device object or `{found: false}`
- `df launch "id" --json`: action result with status and message
- `df install "id" "bundle" --json`: action result with status and message
- `df connect "id" --json`: action result with status and message

### Exit codes

Commands use stable exit codes for automation:

- `0`: success (device found, action executed, etc.)
- `1`: command/runtime error
- `3`: device not found (for `find`, `info`)

### Platform filtering

```bash
# List only iOS simulators
df list --platform ios

# List only Android devices and emulators
df list --platform android

# List only web containers
df list --platform web
```

### Search

Find devices by any of: name, ID, or model:

```bash
df find "iPhone 16"
df find "emulator-5554"
df find "Pixel 8"
```

### Configuration

Device-finder reads optional configuration from `~/.config/itamiforge/device-finder/config.toml`:

```toml
[platforms]
# Filter devices by platform on startup
# default_filter = ["ios", "android"]

# Exclude specific devices
# exclude_devices = ["device-to-ignore"]

[tools]
# Override tool paths if they're not in PATH
# adb_path = "/path/to/adb"
# xcrun_path = "/path/to/xcrun"
# emulator_path = "/path/to/emulator"
```

Create an example config:

```bash
mkdir -p ~/.config/itamiforge/device-finder
# Then edit the file and add your settings
```

## TUI mode

Launch with `df` (no subcommand) for interactive device browser:

- `↑` / `↓`: navigate devices
- `Tab`: cycle through platform filters (All → iOS → Android → Web → All)
- `r`: refresh device list
- `q` / `Esc`: quit

## Requirements

| Platform | Required Tools | Notes |
|----------|---|---|
| macOS | Xcode Command Line Tools | For iOS simulator detection via `xcrun simctl` |
| Android | `adb` in PATH | Install via Android SDK or `brew install android-platform-tools` |
| Web | Docker (optional) | For detecting web containers; browsers detected automatically |

## Building from source

```bash
# Clone the repository
git clone https://github.com/ItamiForge/device-finder.git
cd device-finder

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Binary location
./target/release/df
```

## Testing

```bash
cargo test
cargo test -- --nocapture  # Show output
```

## Architecture

- **lib.rs**: Core device detection traits and Device struct
- **ios.rs**: iOS simulator detection via `xcrun simctl`
- **android.rs**: Android device and emulator detection via `adb`
- **web.rs**: Web browser and Docker container detection
- **cli.rs**: Command-line interface and argument parsing
- **config.rs**: Configuration file loading and defaults
- **tui.rs**: Interactive terminal UI mode
- **main.rs**: Entry point, command dispatch, and JSON output

## Performance

Most operations complete instantly:

- List devices: < 100ms (macOS) to 500ms (Android with multiple devices)
- Find device: < 50ms
- Device info: < 100ms

On systems with many devices, expect slightly longer times. Results are cached within a single run.

## License

MIT

