# Viar

A native desktop keyboard configurator for QMK keyboards with VIA/Vial support, written in Rust.

Viar is a lightweight alternative to the Electron-based [VIA](https://usevia.app/) and [Vial](https://get.vial.today/) configurators. It communicates with keyboards over USB HID to remap keys and control RGB lighting in real time.

## Features

- Auto-detection and connection to VIA/Vial-enabled keyboards
- Visual keyboard layout rendering with accurate physical key positioning
- Real-time key remapping (changes take effect immediately on the device)
- RGB lighting control: brightness, effect, speed, hue, saturation
- Save lighting configuration to keyboard EEPROM
- Export and import keymaps as JSON
- Undo support (Ctrl+Z)
- Automatic lighting protocol detection (VialRGB, Vial legacy, VIA channels)
- Vial keyboard definition parsing (KLE format, LZMA-compressed from firmware)
- Graceful device disconnection handling

## Project Structure

```
crates/
  via-protocol/   # Library: VIA/Vial HID protocol implementation
  viar/           # Binary: egui desktop application
```

`via-protocol` is a standalone library with no GUI dependencies. It can be used independently to build other tools that communicate with VIA/Vial keyboards. See its [README](crates/via-protocol/README.md) for API documentation.

`viar` is the GUI application built with [egui](https://github.com/emilk/egui)/[eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

## Building

Requires Rust edition 2024.

```sh
cargo build --release
```

## Running

```sh
cargo run -p viar
```

With debug logging:

```sh
RUST_LOG=debug cargo run -p viar
```

## Platform Setup

### Linux

HID device access requires appropriate permissions. Add a udev rule:

```sh
# /etc/udev/rules.d/99-hid.rules
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", MODE="0666"
```

Then reload:

```sh
sudo udevadm control --reload-rules && sudo udevadm trigger
```

### Requirements

- A keyboard running QMK firmware with VIA or Vial enabled
- USB connection (no Bluetooth support)

## Architecture

The application follows a layered design:

1. **Raw HID** -- `KeyboardDevice` sends and receives 32-byte HID reports
2. **Command builder** -- `ViaCommand` constructs typed protocol messages
3. **Protocol interface** -- `ViaProtocol` provides high-level operations (read keymap, detect lighting, etc.)
4. **Application** -- `ViarApp` manages UI state, user interaction, and device communication

The GUI uses egui's immediate mode rendering with a state machine (`AppScreen`) driving screen transitions: detection, permission errors, device selection, loading, and the connected view with keymap and lighting tabs.

## License

See repository for license information.
