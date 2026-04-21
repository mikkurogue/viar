# via-protocol

A Rust library for communicating with QMK keyboards over the VIA and Vial HID protocols.

This crate handles device discovery, command construction, keymap reading/writing, lighting control, and keyboard layout parsing. It is transport-level only -- no GUI dependencies.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
via-protocol = { path = "crates/via-protocol" }
```

### Discovering Keyboards

```rust
use via_protocol::{check_hid_permissions, discover_keyboards, HidAccessStatus};

let status = check_hid_permissions();
match status {
    HidAccessStatus::Ok => {},
    HidAccessStatus::NoPermission => {
        eprintln!("No permission to access HID devices");
        return;
    }
    HidAccessStatus::NoViaDevices => {
        eprintln!("No VIA keyboards found");
        return;
    }
    HidAccessStatus::InitFailed(e) => {
        eprintln!("HID init failed: {e}");
        return;
    }
}

let api = hidapi::HidApi::new().unwrap();
let keyboards = discover_keyboards(&api);

for kb in &keyboards {
    println!("{} {} ({})", kb.manufacturer, kb.product, kb.path);
}
```

Keyboards are identified by the VIA HID usage page (`0xFF60`) and usage ID (`0x61`).

### Connecting and Querying

```rust
use via_protocol::{KeyboardDevice, ViaProtocol};

let device = KeyboardDevice::open(&api, &keyboards[0]).unwrap();
let proto = ViaProtocol::new(&device);

let version = proto.get_protocol_version().unwrap();
let layers = proto.get_layer_count().unwrap();
println!("Protocol version: {version}, layers: {layers}");
```

### Reading the Keymap

```rust
// Read a single key (layer 0, row 0, col 0)
let keycode = proto.get_keycode(0, 0, 0).unwrap();
println!("Key: {keycode}"); // e.g. "KC_ESC"

// Read the entire keymap as a flat buffer
let rows = 4;
let cols = 12;
let keymap = proto.read_entire_keymap(layers, rows, cols).unwrap();
// keymap[layer][row * cols + col] = keycode as u16
```

### Writing Keys

```rust
use via_protocol::Keycode;

// Set layer 0, row 0, col 0 to KC_A
proto.set_keycode(0, 0, 0, Keycode(0x0004)).unwrap();
```

Changes take effect on the keyboard immediately. VIA firmware persists them to EEPROM automatically.

### Keycode Utilities

```rust
use via_protocol::{Keycode, all_basic_keycodes, keycode_groups};

let kc = Keycode(0x0004);
println!("{}", kc.name());        // "KC_A"
println!("{}", kc.short_name());  // "A"
println!("{:?}", kc.category());  // Basic

// All basic HID keycodes
let basics = all_basic_keycodes();

// Categorized groups (Letters, Numbers, Modifiers, Layers, etc.)
let groups = keycode_groups();
for group in &groups {
    println!("{}: {} keycodes", group.name, group.codes.len());
}
```

`Keycode` recognizes QMK-specific encodings: layer tap, mod tap, layer momentary, one-shot, tap dance, and more. `category()` returns a `KeycodeCategory` variant indicating how the keycode should be interpreted.

### Lighting Control

```rust
use via_protocol::LightingValues;

// Auto-detect the keyboard's lighting protocol
let protocol = proto.detect_lighting_protocol().unwrap();
println!("Lighting protocol: {protocol:?}");

// Read current values
let values = proto.read_lighting_values(&protocol).unwrap();
println!("Brightness: {}, Effect: {}", values.brightness, values.effect_id);

// Write new values
let new_values = LightingValues {
    brightness: 200,
    effect_id: 1,
    speed: 128,
    hue: 0,
    saturation: 255,
};
proto.write_lighting_values(&protocol, &new_values).unwrap();

// Persist to EEPROM
proto.save_lighting(&protocol).unwrap();
```

The library auto-detects whether the keyboard uses VialRGB, Vial legacy lighting, or VIA channel-based lighting. `LightingProtocol` variants:
- `VialRgb` -- Vial RGB Matrix protocol (command `0x80+`)
- `VialLegacy` -- Vial firmware with legacy lighting commands
- `Via { channel }` -- Standard VIA with a specific `LightingChannel`

### Keyboard Layouts

```rust
use via_protocol::{find_layout, generic_layout, parse_vial_definition, corne_layout};

// Built-in Corne v4.1 layout
let layout = corne_layout();

// Look up by VID:PID
if let Some(layout) = find_layout(0x4653, 0x0001) {
    println!("Found layout: {}", layout.name);
}

// Generic grid fallback
let layout = generic_layout(4, 12);

// Parse a Vial definition (KLE JSON from firmware)
let json: serde_json::Value = /* ... */;
let layout = parse_vial_definition(&json).unwrap();
```

### Vial Firmware Definitions

Vial-enabled keyboards store a compressed layout definition in firmware. Retrieve it with:

```rust
let (uid, _flags) = proto.vial_get_keyboard_id().unwrap();
let definition = proto.vial_get_definition().unwrap(); // serde_json::Value
let layout = parse_vial_definition(&definition).unwrap();
```

The definition is fetched in chunks and LZMA-decompressed automatically.

### Dynamic Entries (Tap Dance, Combos, Key Overrides)

Vial firmware supports dynamic configuration of tap dances, combos, and key overrides. Each entry type is a fixed 10-byte struct stored in EEPROM.

```rust
use via_protocol::{TapDanceEntry, ComboEntry, KeyOverrideEntry};

// Query how many slots the keyboard supports
let counts = proto.get_dynamic_entry_counts().unwrap();
println!("TD: {}, Combos: {}, KO: {}", counts.tap_dance, counts.combo, counts.key_override);

// Read a tap dance entry
let td = proto.get_tap_dance(0).unwrap();
println!("On tap: 0x{:04X}, On hold: 0x{:04X}, term: {}ms",
    td.on_tap, td.on_hold, td.tapping_term);

// Write a tap dance entry
let td = TapDanceEntry {
    on_tap: 0x0004,      // KC_A
    on_hold: 0x00E0,     // KC_LCTL
    on_double_tap: 0x0005, // KC_B
    on_tap_hold: 0x0000,
    tapping_term: 200,
};
proto.set_tap_dance(0, &td).unwrap();

// Read/write combos
let combo = proto.get_combo(0).unwrap();
let combo = ComboEntry {
    input: [0x0004, 0x0005, 0, 0], // A + B
    output: 0x001B,                 // KC_ESC -> output Escape
};
proto.set_combo(0, &combo).unwrap();

// Read/write key overrides
let ko = proto.get_key_override(0).unwrap();
let ko = KeyOverrideEntry {
    trigger: 0x0004,      // KC_A
    replacement: 0x0005,  // KC_B
    layers: 0xFFFF,       // all layers
    trigger_mods: 0x02,   // LShift
    negative_mod_mask: 0,
    suppressed_mods: 0x02,
    options: 0x80,        // bit 7 = enabled
};
proto.set_key_override(0, &ko).unwrap();

// Bulk read all entries
let all_td = proto.get_all_tap_dances(counts.tap_dance).unwrap();
let all_combos = proto.get_all_combos(counts.combo).unwrap();
let all_ko = proto.get_all_key_overrides(counts.key_override).unwrap();
```

All dynamic entry operations go through the Vial prefix (`0xFE`) with sub-command `0x0D`. Data is serialized as raw little-endian structs matching the firmware's C layout.

## Protocol Details

All communication uses 32-byte HID reports (`VIA_REPORT_SIZE`). The first byte is the command ID. The library handles padding, chunked reads, and response parsing.

Key command IDs (see `ViaCommandId`):

| Command | ID | Description |
|---|---|---|
| `GetProtocolVersion` | `0x01` | Query VIA protocol version |
| `GetKeyboardValue` | `0x02` | Read keyboard state (uptime, layout options, etc.) |
| `DynamicKeymapGetKeycode` | `0x04` | Read a single keycode |
| `DynamicKeymapSetKeycode` | `0x05` | Write a single keycode |
| `DynamicKeymapGetBuffer` | `0x11` | Bulk keymap read |
| `GetLayerCount` | `0x11` | Get number of layers |
| `LightingGetValue` | `0x08` | Read lighting parameter |
| `LightingSetValue` | `0x09` | Write lighting parameter |
| `LightingSave` | `0x0A` | Persist lighting to EEPROM |
| `VialPrefix` | `0xFE` | Vial-specific command prefix |

## Error Handling

All fallible operations return `ViaResult<T>`, which wraps `ViaError`:

- `ViaError::Hid` -- USB HID communication failure
- `ViaError::Protocol` -- Unexpected response from keyboard
- `ViaError::NotViaDevice` -- Device does not support VIA
- `ViaError::Timeout` -- Communication timeout
- `ViaError::InvalidKeycode` -- Malformed keycode value

## Dependencies

- `hidapi` -- USB HID communication
- `lzma-rs` -- LZMA decompression for Vial definitions
- `serde_json` -- JSON parsing for keyboard definitions
- `tracing` -- Structured logging
- `thiserror` -- Error type derivation
