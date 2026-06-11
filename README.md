# Rusty AutoClicker

<div align="center">

[![Latest Version](https://img.shields.io/github/v/tag/MrTanoshii/rusty-autoclicker.svg?label=Version&sort=semver&color=orange)](https://github.com/MrTanoshii/rusty-autoclicker/releases)
[![CC0-1.0 License](https://img.shields.io/badge/License-CC0--1.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.96+-blue)](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/Cargo.toml)
[![Rust Check](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml/badge.svg?branch=main)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml)
[![Linux Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml/badge.svg?branch=main)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml)
[![macOS Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml/badge.svg?branch=main)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml)
[![Windows Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml/badge.svg?branch=main)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml)

</div>

## Table of Content

- [Screenshots](#screenshots-top)
- [Features and Roadmap](#features--roadmap-top)
- [Installation](#installation-top)
- [Building from Source](#building-from-source-top)
- [Contributing](#contributing-top)

## Screenshots [:top:](#table-of-content)

<div align="center">

##### Main Interface

[![](/screenshots/v2.5.0/rusty-autoclicker_main_interface.png?raw=true "Main Interface")](#)

##### Hotkey Change

[![](/screenshots/v2.5.0/rusty-autoclicker_hotkey_change.png?raw=true "Hotkey Change")](#)

##### Setting Coordinates

[![](/screenshots/v2.5.0/rusty-autoclicker_setting_coordinates.png?raw=true "Setting Coordinates")](#)

</div>

## Features & Roadmap [:top:](#table-of-content)

Features that currently are or will be implemented.

- [x] Built in Rust
- [x] Cross-compatible with Linux, macOS and Windows
- [x] Free and Open Source Software
- [x] Bot Mode
- [ ] Humanlike Mode
  - [x] Tweening when moving to desired click position
  - [x] Randomizing time between clicks
  - [x] Randomizing click duration
  - [ ] Random mouse movement between clicks (if click interval permits)
  - [x] Setting for min/max move speed
- [x] Mouse & Preset Coordinates Mode
- [x] Infinite & fixed click amount
- [x] Left/Middle/Right mouse clicks
- [x] Single/Double mouse clicks
- [x] User customizable hotkeys
- [ ] Options (e.g. Display mouse & key info)

### Advanced features

These features are being considered but are not confirmed.

- Click sequence
- Profiles (e.g. Profile for a specific app/game)
- Time-based clicking (e.g. every day/week/fortnight/month at 08:00 am)

## Installation [:top:](#table-of-content)

Prebuilt binaries for Linux, macOS and Windows are attached to every [release](https://github.com/MrTanoshii/rusty-autoclicker/releases).

> [!NOTE]
> These binaries are **not code-signed or notarized** (that requires a paid Apple Developer account). They are safe, but the OS may warn you on first launch. See the platform notes below.

### macOS (manual download)

Because the binary is unsigned, double-clicking the `.tar.gz` in Finder makes macOS Gatekeeper show *"Apple could not verify … is free of malware"*. To avoid it, **extract from the Terminal instead of Finder** — the command-line `tar` does not propagate the quarantine flag:

```shell
# Download (curl does not set the quarantine flag either)
curl -LO https://github.com/MrTanoshii/rusty-autoclicker/releases/latest/download/rusty-autoclicker-macos.tar.gz

# Extract from Terminal (NOT by double-clicking in Finder)
tar -xzf rusty-autoclicker-macos.tar.gz

./rusty-autoclicker-macos
```

If you did download/extract via the browser and Finder and already hit the warning, clear the quarantine flag manually:

```shell
xattr -d com.apple.quarantine ./rusty-autoclicker-macos
```

…or open it once via **System Settings → Privacy & Security → "Open Anyway"**.

> [!IMPORTANT]
> macOS requires **Accessibility** *and* **Input Monitoring** permissions for the app to read hotkeys and move/click the mouse. Grant both under **System Settings → Privacy & Security** when prompted.

### Linux (manual download)

```shell
curl -LO https://github.com/MrTanoshii/rusty-autoclicker/releases/latest/download/rusty-autoclicker-linux.tar.gz
tar -xzf rusty-autoclicker-linux.tar.gz
./rusty-autoclicker-linux
```

### Windows

Download `rusty-autoclicker-windows.exe` from the [latest release](https://github.com/MrTanoshii/rusty-autoclicker/releases/latest) and run it. SmartScreen may warn about an unrecognized publisher — choose **More info → Run anyway**.

### Install from source with Cargo

If you have the [Rust toolchain](https://rustup.rs/), you can build and install directly — locally compiled binaries are never quarantined:

```shell
cargo install --git https://github.com/MrTanoshii/rusty-autoclicker
```

On Linux, install the [system dependencies](#linux) first.

## Building from Source [:top:](#table-of-content)

### OS specific requirements

#### Fedora Rawhide (not tested)

```shell
dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel
```

#### Linux

```shell
sudo apt-get install libx11-dev libxtst-dev libevdev-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev

# Install `libfontconfig-dev` if you get the following error
# error: failed to run custom build command for `servo-fontconfig-sys v5.1.0`
sudo apt-get install libfontconfig-dev
```

### Running

```shell
rustup update
cargo run --release
```

#### Linux crash fix

```shell
export WINIT_UNIX_BACKEND=x11
./launch_your_app
```

### Build

```shell
rustup update
cargo build --release
```

## Contributing [:top:](#table-of-content)

Please follow the [CONTRIBUTING.md](CONTRIBUTING.md) guide.
