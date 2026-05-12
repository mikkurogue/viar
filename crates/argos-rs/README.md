# argos-rs

Rust implementation of the [Bastard Keyboards](https://bastardkb.com/) Argos protocol for keyboard configuration over USB HID.

Argos is a QMK community module developed by Bastard Keyboards that extends the VIA protocol with additional configuration capabilities for their keyboards (Charybdis, Dilemma, etc.). It communicates over the same raw HID interface as VIA (usage page `0xFF60`, usage `0x61`) but uses a distinct command prefix (`0x90`) to avoid conflicts.

This crate is a host-side consumer of the Argos protocol, allowing applications to configure BKB keyboards that have the Argos module enabled.

## What Argos provides

- **Combos** -- up to 16 combo entries, each with 4 trigger keys and an output keycode
- **Tap dances** -- up to 16 tap dance entries with tap, hold, double-tap, and tap-hold actions
- **Pointing device configuration** -- DPI and sniping DPI for trackballs (Charybdis) and trackpads (Dilemma)
- **Global timing** -- configurable tapping term and combo term (in ms)
- **Theming** -- theme ID stored on the keyboard for the companion web app
- **Keymap testing** -- interactive key capture mode for testing key assignments

## Usage

`argos-rs` reuses the HID transport from `via-protocol`. After connecting to a keyboard, probe for Argos support:

```rust
use via_protocol::KeyboardDevice;
use argos_rs::ArgosProtocol;

// After opening a KeyboardDevice via via-protocol:
let argos = ArgosProtocol::new(&device);

// Probe for Argos support
if let Some(info) = argos.probe() {
    println!("Argos protocol v{:#06x}", info.protocol_version);
    println!("Combos: {}, Tap dances: {}", info.combo_entries, info.tap_dance_entries);

    // Read all combos
    let combos = argos.get_all_combos()?;

    // Set global tapping term to 200ms
    argos.set_global_tapping_term(200)?;
}
```

## Protocol overview

All Argos commands are 32-byte HID reports with the following layout:

| Byte | Field |
|------|-------|
| 0 | `0x90` (Argos command prefix) |
| 1 | Command ID |
| 2..31 | Command-specific data |

Responses echo the same prefix and command ID, followed by response data.

### Command reference

| ID | Command | Direction |
|----|---------|-----------|
| `0x01` | Get keyboard info | Request/Response |
| `0x02` | Get combo | Request/Response |
| `0x03` | Delete combo key | Request |
| `0x04` | Capture combo key | Request/Response (blocking) |
| `0x05` | Get theme ID | Request/Response |
| `0x06` | Set theme ID | Request/Response |
| `0x07` | Get tap dance | Request/Response |
| `0x08` | Set tap dance | Request/Response |
| `0x09` | Capture tap dance key | Request/Response (blocking) |
| `0x0A` | Delete tap dance key | Request |
| `0x0B` | Set DPI | Request/Response |
| `0x0C` | Get pointing device info | Request/Response |
| `0x0D` | Set sniping DPI | Request/Response |
| `0x0E` | Set combo | Request/Response |
| `0x0F` | Capture all keycodes | Request (async response on keypress) |
| `0x10` | Set welcome message displayed | Request/Response |
| `0x11` | Set global tapping term | Request/Response |
| `0x12` | Set global combo term | Request/Response |

## Upstream reference

The Argos firmware module lives at [Bastardkb/qmk_modules/argos](https://github.com/Bastardkb/qmk_modules/tree/main/argos).

## License

MIT
