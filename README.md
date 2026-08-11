# Soundcore Liberty 5

A small Windows desktop application for controlling Soundcore Liberty 5 earbuds.

## What it does

The app provides a simple interface to:

- Discover available Liberty 5 devices.
- Connect to and disconnect from an earbud pair.
- Check the battery levels of the left earbud, right earbud, and charging case.
- Switch between active noise cancellation, transparency, and off modes.
- Toggle Game Mode.
- Select verified equalizer presets when available.
- Keep the application available from the Windows system tray.

## Project status

This is an experimental, hobby project built with the help of AI. It is intended for learning, exploration, and personal use rather than as an official Soundcore product or a production-ready replacement for the Soundcore application.

Device support and available controls may change as the project is tested. Use it at your own risk, especially when experimenting with undocumented device communication.

## Technology

- Rust
- Tauri 2
- A small HTML, CSS, and JavaScript frontend
- Bluetooth communication for Liberty 5 device control

## Building on Windows

Install the Rust toolchain and the Tauri prerequisites for Windows, then run:

```powershell
cargo tauri build
```

The NSIS installer is generated under:

```text
target/release/bundle/nsis/
```

## Disclaimer

Soundcore and Liberty 5 are trademarks of their respective owner. This project is unofficial and is not affiliated with or endorsed by Soundcore or Anker.
