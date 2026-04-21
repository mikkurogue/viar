use crate::{
    command::{
        ComboEntry, DynamicEntryCounts, KeyOverrideEntry, LightingChannel, LightingProtocol,
        RgbValueId, TapDanceEntry, VialRgbValueId,
    },
    device::KeyboardDevice,
    ViaCommand, ViaError, ViaResult,
};
use tracing::{debug, info};

/// High-level VIA protocol interface for a connected keyboard.
pub struct ViaProtocol<'a> {
    device: &'a KeyboardDevice,
}

impl<'a> ViaProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self { device }
    }

    /// Get the VIA protocol version supported by the keyboard.
    pub fn get_protocol_version(&self) -> ViaResult<u16> {
        let resp = self
            .device
            .send_command(&ViaCommand::get_protocol_version())?;
        debug!(raw = ?&resp[..4], "get_protocol_version response");
        let version = u16::from_be_bytes([resp[1], resp[2]]);
        debug!(version, "protocol version");
        Ok(version)
    }

    /// Get the number of layers in the dynamic keymap.
    pub fn get_layer_count(&self) -> ViaResult<u8> {
        let resp = self.device.send_command(&ViaCommand::get_layer_count())?;
        debug!(raw = ?&resp[..4], "get_layer_count response");
        let count = resp[1];
        debug!(count, "layer count");
        Ok(count)
    }

    /// Get a single keycode at (layer, row, col).
    pub fn get_keycode(&self, layer: u8, row: u8, col: u8) -> ViaResult<u16> {
        let resp = self
            .device
            .send_command(&ViaCommand::get_keycode(layer, row, col))?;
        let keycode = u16::from_be_bytes([resp[4], resp[5]]);
        Ok(keycode)
    }

    /// Set a single keycode at (layer, row, col).
    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> ViaResult<()> {
        self.device
            .send_command(&ViaCommand::set_keycode(layer, row, col, keycode))?;
        Ok(())
    }

    /// Read a chunk of the keymap buffer.
    pub fn get_keymap_buffer(&self, offset: u16, size: u8) -> ViaResult<Vec<u8>> {
        let resp = self
            .device
            .send_command(&ViaCommand::get_keymap_buffer(offset, size))?;
        let payload_start = 4;
        let end = (payload_start + size as usize).min(resp.len());
        Ok(resp[payload_start..end].to_vec())
    }

    /// Read the entire dynamic keymap as [layer][row][col].
    pub fn read_entire_keymap(
        &self,
        layers: u8,
        rows: u8,
        cols: u8,
    ) -> ViaResult<Vec<Vec<Vec<u16>>>> {
        let total_keys = layers as usize * rows as usize * cols as usize;
        let total_bytes = total_keys * 2;

        let mut raw = Vec::with_capacity(total_bytes);
        let mut offset: u16 = 0;
        while (offset as usize) < total_bytes {
            let remaining = total_bytes - offset as usize;
            let chunk_size = 28usize.min(remaining) as u8;
            debug!(
                offset,
                chunk_size, remaining, total_bytes, "reading keymap chunk"
            );
            let chunk = self.get_keymap_buffer(offset, chunk_size)?;
            debug!(offset, returned = chunk.len(), "got keymap chunk");
            raw.extend_from_slice(&chunk);
            offset += chunk_size as u16;
        }

        let mut keymap = Vec::with_capacity(layers as usize);
        let mut idx = 0;
        for _ in 0..layers {
            let mut layer = Vec::with_capacity(rows as usize);
            for _ in 0..rows {
                let mut row_keys = Vec::with_capacity(cols as usize);
                for _ in 0..cols {
                    if idx + 1 < raw.len() {
                        let kc = u16::from_be_bytes([raw[idx], raw[idx + 1]]);
                        row_keys.push(kc);
                    } else {
                        row_keys.push(0);
                    }
                    idx += 2;
                }
                layer.push(row_keys);
            }
            keymap.push(layer);
        }

        Ok(keymap)
    }

    /// Get the number of macros supported.
    pub fn get_macro_count(&self) -> ViaResult<u8> {
        let resp = self.device.send_command(&ViaCommand::get_macro_count())?;
        Ok(resp[1])
    }

    /// Get the total macro buffer size in bytes.
    pub fn get_macro_buffer_size(&self) -> ViaResult<u16> {
        let resp = self
            .device
            .send_command(&ViaCommand::get_macro_buffer_size())?;
        Ok(u16::from_be_bytes([resp[1], resp[2]]))
    }

    // ========================================================================
    // Vial firmware — keyboard definition fetching
    // ========================================================================

    /// Get the Vial keyboard ID, protocol version, and UID.
    /// Returns (vial_protocol_version, uid_bytes).
    pub fn vial_get_keyboard_id(&self) -> ViaResult<(u32, [u8; 8])> {
        let resp = self
            .device
            .send_command(&ViaCommand::vial_get_keyboard_id())?;
        // Firmware overwrites entire msg buffer:
        // msg[0..3] = VIAL_PROTOCOL_VERSION (u32 LE)
        // msg[4..11] = keyboard UID (8 bytes)
        // msg[12] = vialrgb flag (optional)
        let version = u32::from_le_bytes([resp[0], resp[1], resp[2], resp[3]]);
        let mut uid = [0u8; 8];
        uid.copy_from_slice(&resp[4..12]);
        info!(vial_protocol_version = version, uid = ?uid, "vial keyboard ID");
        Ok((version, uid))
    }

    /// Get the size of the compressed Vial keyboard definition.
    pub fn vial_get_definition_size(&self) -> ViaResult<u32> {
        let resp = self.device.send_command(&ViaCommand::vial_get_size())?;
        // Firmware writes size at msg[0..3] (u32 LE)
        let size = u32::from_le_bytes([resp[0], resp[1], resp[2], resp[3]]);
        info!(compressed_size = size, "vial definition size");
        Ok(size)
    }

    /// Fetch and decompress the full Vial keyboard definition JSON.
    /// Returns the parsed JSON string.
    pub fn vial_get_definition(&self) -> ViaResult<String> {
        // First check if this is a Vial keyboard
        let (version, _uid) = self.vial_get_keyboard_id()?;
        if version == 0 {
            return Err(ViaError::Protocol("not a Vial keyboard (version 0)".into()));
        }

        let size = self.vial_get_definition_size()? as usize;
        if size == 0 || size > 1_000_000 {
            return Err(ViaError::Protocol(format!(
                "invalid definition size: {size}"
            )));
        }

        // Fetch compressed data page by page (32 bytes per page from byte index 1..33 of response)
        let mut compressed = Vec::with_capacity(size);
        let page_size = 32usize; // each response gives us 32 bytes of definition data
        let num_pages = (size + page_size - 1) / page_size;
        for page in 0..num_pages {
            let resp = self
                .device
                .send_command(&ViaCommand::vial_get_def(page as u16))?;
            let remaining = size - compressed.len();
            let take = remaining.min(page_size);
            // The definition data starts at byte 0 of the response for vial_get_def
            // (the 0xFE prefix is consumed by the firmware, response is raw data)
            compressed.extend_from_slice(&resp[..take]);
            debug!(
                page,
                bytes = take,
                total = compressed.len(),
                size,
                "fetching vial definition"
            );
        }

        info!(
            compressed_bytes = compressed.len(),
            first_bytes = ?&compressed[..compressed.len().min(16)],
            "vial definition fetched, decompressing"
        );

        // Try LZMA decompression first, fall back to XZ if that fails
        let mut decompressed = Vec::new();
        let lzma_result =
            lzma_rs::lzma_decompress(&mut std::io::Cursor::new(&compressed), &mut decompressed);
        if lzma_result.is_err() {
            decompressed.clear();
            lzma_rs::xz_decompress(&mut std::io::Cursor::new(&compressed), &mut decompressed)
                .map_err(|e| {
                    ViaError::Protocol(format!("decompression failed (tried LZMA and XZ): {e}"))
                })?;
        }

        let json = String::from_utf8(decompressed)
            .map_err(|e| ViaError::Protocol(format!("definition is not valid UTF-8: {e}")))?;

        info!(json_len = json.len(), "vial definition decompressed");
        debug!(json = %json, "vial definition JSON");

        Ok(json)
    }

    // ========================================================================
    // Vial Dynamic Entries (Tap Dance, Combos, Key Overrides)
    // ========================================================================

    /// Get the counts of dynamic entries supported by the keyboard.
    pub fn get_dynamic_entry_counts(&self) -> ViaResult<DynamicEntryCounts> {
        let resp = self
            .device
            .send_command(&ViaCommand::vial_get_dynamic_entry_count())?;
        // Response: firmware overwrites buffer starting at 0
        // [td_count, combo_count, ko_count, arep_count, ...]
        let counts = DynamicEntryCounts {
            tap_dance: resp[0],
            combo: resp[1],
            key_override: resp[2],
            alt_repeat: resp[3],
        };
        info!(
            tap_dance = counts.tap_dance,
            combo = counts.combo,
            key_override = counts.key_override,
            alt_repeat = counts.alt_repeat,
            "dynamic entry counts"
        );
        Ok(counts)
    }

    /// Get a tap dance entry by index.
    pub fn get_tap_dance(&self, idx: u8) -> ViaResult<TapDanceEntry> {
        let resp = self
            .device
            .send_command(&ViaCommand::vial_tap_dance_get(idx))?;
        // Response: [status, <10 bytes struct>]
        // Vial firmware writes status at msg[0], struct at msg[1..11]
        let entry = TapDanceEntry::from_bytes(&resp[1..11]);
        debug!(idx, ?entry, "get_tap_dance");
        Ok(entry)
    }

    /// Set a tap dance entry by index.
    pub fn set_tap_dance(&self, idx: u8, entry: &TapDanceEntry) -> ViaResult<()> {
        let data = entry.to_bytes();
        self.device
            .send_command(&ViaCommand::vial_tap_dance_set(idx, &data))?;
        debug!(idx, ?entry, "set_tap_dance");
        Ok(())
    }

    /// Get all tap dance entries.
    pub fn get_all_tap_dances(&self, count: u8) -> ViaResult<Vec<TapDanceEntry>> {
        let mut entries = Vec::with_capacity(count as usize);
        for i in 0..count {
            entries.push(self.get_tap_dance(i)?);
        }
        Ok(entries)
    }

    /// Get a combo entry by index.
    pub fn get_combo(&self, idx: u8) -> ViaResult<ComboEntry> {
        let resp = self.device.send_command(&ViaCommand::vial_combo_get(idx))?;
        let entry = ComboEntry::from_bytes(&resp[1..11]);
        debug!(idx, ?entry, "get_combo");
        Ok(entry)
    }

    /// Set a combo entry by index.
    pub fn set_combo(&self, idx: u8, entry: &ComboEntry) -> ViaResult<()> {
        let data = entry.to_bytes();
        self.device
            .send_command(&ViaCommand::vial_combo_set(idx, &data))?;
        debug!(idx, ?entry, "set_combo");
        Ok(())
    }

    /// Get all combo entries.
    pub fn get_all_combos(&self, count: u8) -> ViaResult<Vec<ComboEntry>> {
        let mut entries = Vec::with_capacity(count as usize);
        for i in 0..count {
            entries.push(self.get_combo(i)?);
        }
        Ok(entries)
    }

    /// Get a key override entry by index.
    pub fn get_key_override(&self, idx: u8) -> ViaResult<KeyOverrideEntry> {
        let resp = self
            .device
            .send_command(&ViaCommand::vial_key_override_get(idx))?;
        let entry = KeyOverrideEntry::from_bytes(&resp[1..11]);
        debug!(idx, ?entry, "get_key_override");
        Ok(entry)
    }

    /// Set a key override entry by index.
    pub fn set_key_override(&self, idx: u8, entry: &KeyOverrideEntry) -> ViaResult<()> {
        let data = entry.to_bytes();
        self.device
            .send_command(&ViaCommand::vial_key_override_set(idx, &data))?;
        debug!(idx, ?entry, "set_key_override");
        Ok(())
    }

    /// Get all key override entries.
    pub fn get_all_key_overrides(&self, count: u8) -> ViaResult<Vec<KeyOverrideEntry>> {
        let mut entries = Vec::with_capacity(count as usize);
        for i in 0..count {
            entries.push(self.get_key_override(i)?);
        }
        Ok(entries)
    }

    // ========================================================================
    // Lighting — unified API
    // ========================================================================

    /// Detect the lighting protocol supported by the keyboard.
    /// Tries VialRGB first, then Vial legacy, then VIA channels.
    pub fn detect_lighting_protocol(&self) -> Option<LightingProtocol> {
        // Try VialRGB first (most likely for Vial firmware)
        info!("probing VialRGB protocol");
        if self.try_vialrgb() {
            return Some(LightingProtocol::VialRgb);
        }

        // Try Vial legacy (VIA_QMK_RGB_MATRIX_ENABLE without VIALRGB)
        info!("VialRGB not detected, probing Vial legacy lighting");
        if self.try_vial_legacy() {
            return Some(LightingProtocol::VialLegacy);
        }

        // Fall back to VIA channel-based probing
        info!("Vial legacy not detected, probing VIA channels");
        self.try_via_channels()
    }

    /// Read all current lighting values for a detected protocol.
    pub fn read_lighting_values(&self, proto: &LightingProtocol) -> ViaResult<LightingValues> {
        match proto {
            LightingProtocol::VialRgb => self.vialrgb_read_values(),
            LightingProtocol::VialLegacy => self.vial_legacy_read_values(),
            LightingProtocol::Via { channel } => self.via_read_values(*channel),
        }
    }

    /// Apply lighting values to the device.
    pub fn write_lighting_values(
        &self,
        proto: &LightingProtocol,
        vals: &LightingValues,
    ) -> ViaResult<()> {
        match proto {
            LightingProtocol::VialRgb => {
                // VialRGB sets everything in one command
                self.device.send_command(&ViaCommand::vialrgb_set_mode(
                    vals.effect_id,
                    vals.speed,
                    vals.hue,
                    vals.saturation,
                    vals.brightness,
                ))?;
                Ok(())
            }
            LightingProtocol::VialLegacy => {
                self.vial_legacy_set(VialRgbValueId::Brightness as u8, &[vals.brightness])?;
                self.vial_legacy_set(VialRgbValueId::Effect as u8, &[vals.effect_id as u8])?;
                self.vial_legacy_set(VialRgbValueId::EffectSpeed as u8, &[vals.speed])?;
                self.vial_legacy_set(VialRgbValueId::Color as u8, &[vals.hue, vals.saturation])?;
                Ok(())
            }
            LightingProtocol::Via { channel } => {
                let ch = *channel as u8;
                self.device.send_command(&ViaCommand::set_lighting_value(
                    ch,
                    RgbValueId::Brightness as u8,
                    &[vals.brightness],
                ))?;
                self.device.send_command(&ViaCommand::set_lighting_value(
                    ch,
                    RgbValueId::Effect as u8,
                    &[vals.effect_id as u8],
                ))?;
                self.device.send_command(&ViaCommand::set_lighting_value(
                    ch,
                    RgbValueId::EffectSpeed as u8,
                    &[vals.speed],
                ))?;
                self.device.send_command(&ViaCommand::set_lighting_value(
                    ch,
                    RgbValueId::Color as u8,
                    &[vals.hue, vals.saturation],
                ))?;
                Ok(())
            }
        }
    }

    /// Save lighting values to EEPROM.
    pub fn save_lighting(&self, proto: &LightingProtocol) -> ViaResult<()> {
        debug!(?proto, "save_lighting");
        match proto {
            LightingProtocol::Via { channel } => {
                self.device
                    .send_command(&ViaCommand::custom_save(*channel as u8))?;
            }
            LightingProtocol::VialLegacy | LightingProtocol::VialRgb => {
                self.device.send_command(&ViaCommand::vial_custom_save())?;
            }
        }
        Ok(())
    }

    /// Get the list of supported VialRGB effect IDs from the keyboard.
    pub fn vialrgb_get_supported_effects(&self) -> ViaResult<Vec<u16>> {
        let mut effects = Vec::new();
        let mut gt: u16 = 0;
        loop {
            let resp = self
                .device
                .send_command(&ViaCommand::vialrgb_get_supported(gt))?;
            // Response: [cmd, sub_cmd, id_lo, id_hi, id_lo, id_hi, ..., 0xFF, 0xFF]
            let data = &resp[2..];
            let mut found_any = false;
            for chunk in data.chunks_exact(2) {
                let id = u16::from_le_bytes([chunk[0], chunk[1]]);
                if id == 0xFFFF {
                    return Ok(effects);
                }
                effects.push(id);
                gt = id;
                found_any = true;
            }
            if !found_any {
                break;
            }
        }
        Ok(effects)
    }

    /// Get VialRGB info (protocol version, max brightness).
    pub fn vialrgb_get_info(&self) -> ViaResult<VialRgbInfo> {
        let resp = self.device.send_command(&ViaCommand::vialrgb_get_info())?;
        // [cmd, 0x40, version_lo, version_hi, max_brightness]
        let version = u16::from_le_bytes([resp[2], resp[3]]);
        let max_brightness = resp[4];
        Ok(VialRgbInfo {
            protocol_version: version,
            max_brightness,
        })
    }

    // ========================================================================
    // Internal: VialRGB
    // ========================================================================

    fn try_vialrgb(&self) -> bool {
        match self.device.send_command(&ViaCommand::vialrgb_get_info()) {
            Ok(resp) => {
                info!(raw = ?&resp[..8], "VialRGB info probe");
                if resp[0] == 0xFF {
                    info!("VialRGB: command unhandled");
                    return false;
                }
                let version = u16::from_le_bytes([resp[2], resp[3]]);
                let max_brightness = resp[4];
                info!(version, max_brightness, "VialRGB detected");
                // Confirm by also reading mode
                if let Ok(mode_resp) = self.device.send_command(&ViaCommand::vialrgb_get_mode()) {
                    let mode_id = u16::from_le_bytes([mode_resp[2], mode_resp[3]]);
                    let speed = mode_resp[4];
                    let hue = mode_resp[5];
                    let sat = mode_resp[6];
                    let val = mode_resp[7];
                    info!(mode_id, speed, hue, sat, val, "VialRGB current mode");
                }
                version > 0 || max_brightness > 0
            }
            Err(e) => {
                info!(error = %e, "VialRGB probe error");
                false
            }
        }
    }

    fn vialrgb_read_values(&self) -> ViaResult<LightingValues> {
        let resp = self.device.send_command(&ViaCommand::vialrgb_get_mode())?;
        // [cmd, 0x41, mode_lo, mode_hi, speed, hue, sat, val]
        let effect_id = u16::from_le_bytes([resp[2], resp[3]]);
        let speed = resp[4];
        let hue = resp[5];
        let sat = resp[6];
        let brightness = resp[7];
        debug!(
            effect_id,
            speed, hue, sat, brightness, "vialrgb_read_values"
        );
        Ok(LightingValues {
            effect_id,
            brightness,
            speed,
            hue,
            saturation: sat,
        })
    }

    // ========================================================================
    // Internal: Vial Legacy (no channel, 0x80+ value IDs)
    // ========================================================================

    fn try_vial_legacy(&self) -> bool {
        let cmd = ViaCommand::vial_get_lighting_value(VialRgbValueId::Brightness as u8);
        match self.device.send_command(&cmd) {
            Ok(resp) => {
                info!(raw = ?&resp[..8], "Vial legacy brightness probe");
                if resp[0] == 0xFF {
                    return false;
                }
                let brightness = resp[2];
                let effect = self
                    .device
                    .send_command(&ViaCommand::vial_get_lighting_value(
                        VialRgbValueId::Effect as u8,
                    ))
                    .map(|r| r[2])
                    .unwrap_or(0);
                let has_data = brightness > 0 || effect > 0;
                if has_data {
                    info!(brightness, effect, "Vial legacy lighting detected");
                    return true;
                }
                // Write-readback test
                let _ = self
                    .device
                    .send_command(&ViaCommand::vial_set_lighting_value(
                        VialRgbValueId::Effect as u8,
                        &[1],
                    ));
                let readback = self
                    .device
                    .send_command(&ViaCommand::vial_get_lighting_value(
                        VialRgbValueId::Effect as u8,
                    ))
                    .map(|r| r[2])
                    .unwrap_or(0);
                let _ = self
                    .device
                    .send_command(&ViaCommand::vial_set_lighting_value(
                        VialRgbValueId::Effect as u8,
                        &[0],
                    ));
                info!(readback, "Vial legacy write-readback test");
                readback == 1
            }
            Err(_) => false,
        }
    }

    fn vial_legacy_read_values(&self) -> ViaResult<LightingValues> {
        let brightness = self
            .device
            .send_command(&ViaCommand::vial_get_lighting_value(
                VialRgbValueId::Brightness as u8,
            ))
            .map(|r| r[2])
            .unwrap_or(0);
        let effect = self
            .device
            .send_command(&ViaCommand::vial_get_lighting_value(
                VialRgbValueId::Effect as u8,
            ))
            .map(|r| r[2])
            .unwrap_or(0);
        let speed = self
            .device
            .send_command(&ViaCommand::vial_get_lighting_value(
                VialRgbValueId::EffectSpeed as u8,
            ))
            .map(|r| r[2])
            .unwrap_or(0);
        let color_resp = self
            .device
            .send_command(&ViaCommand::vial_get_lighting_value(
                VialRgbValueId::Color as u8,
            ))?;
        let hue = color_resp[2];
        let sat = color_resp[3];
        Ok(LightingValues {
            effect_id: effect as u16,
            brightness,
            speed,
            hue,
            saturation: sat,
        })
    }

    fn vial_legacy_set(&self, value_id: u8, payload: &[u8]) -> ViaResult<()> {
        self.device
            .send_command(&ViaCommand::vial_set_lighting_value(value_id, payload))?;
        Ok(())
    }

    // ========================================================================
    // Internal: Stock VIA (channel-based)
    // ========================================================================

    fn try_via_channels(&self) -> Option<LightingProtocol> {
        let channels = [
            (LightingChannel::QmkBacklight, "backlight"),
            (LightingChannel::QmkRgblight, "rgblight"),
            (LightingChannel::QmkRgbMatrix, "rgb_matrix"),
            (LightingChannel::QmkAudio, "audio"),
            (LightingChannel::QmkLedMatrix, "led_matrix"),
        ];

        for (channel, name) in channels {
            let ch = channel as u8;
            info!(channel = ch, name, "probing VIA lighting channel");

            let brightness = match self.device.send_command(&ViaCommand::get_lighting_value(
                ch,
                RgbValueId::Brightness as u8,
            )) {
                Ok(resp) => {
                    if resp[0] == 0xFF {
                        continue;
                    }
                    resp[3]
                }
                Err(_) => continue,
            };

            let effect = self
                .device
                .send_command(&ViaCommand::get_lighting_value(
                    ch,
                    RgbValueId::Effect as u8,
                ))
                .map(|r| r[3])
                .unwrap_or(0);

            let has_nonzero = brightness > 0 || effect > 0;
            if has_nonzero {
                info!(
                    channel = ch,
                    name, brightness, effect, "VIA channel confirmed"
                );
                return Some(LightingProtocol::Via { channel });
            }

            // Write-readback test
            let _ = self.device.send_command(&ViaCommand::set_lighting_value(
                ch,
                RgbValueId::Effect as u8,
                &[1],
            ));
            let readback = self
                .device
                .send_command(&ViaCommand::get_lighting_value(
                    ch,
                    RgbValueId::Effect as u8,
                ))
                .map(|r| r[3])
                .unwrap_or(0);
            let _ = self.device.send_command(&ViaCommand::set_lighting_value(
                ch,
                RgbValueId::Effect as u8,
                &[0],
            ));

            if readback == 1 {
                info!(
                    channel = ch,
                    name, "VIA channel confirmed via write-readback"
                );
                return Some(LightingProtocol::Via { channel });
            }
        }

        info!("no VIA lighting channels detected");
        None
    }

    fn via_read_values(&self, channel: LightingChannel) -> ViaResult<LightingValues> {
        let ch = channel as u8;
        let brightness = self
            .device
            .send_command(&ViaCommand::get_lighting_value(
                ch,
                RgbValueId::Brightness as u8,
            ))
            .map(|r| r[3])
            .unwrap_or(0);
        let effect = self
            .device
            .send_command(&ViaCommand::get_lighting_value(
                ch,
                RgbValueId::Effect as u8,
            ))
            .map(|r| r[3])
            .unwrap_or(0);
        let speed = self
            .device
            .send_command(&ViaCommand::get_lighting_value(
                ch,
                RgbValueId::EffectSpeed as u8,
            ))
            .map(|r| r[3])
            .unwrap_or(0);
        let color_resp = self
            .device
            .send_command(&ViaCommand::get_lighting_value(ch, RgbValueId::Color as u8))?;
        let hue = color_resp[3];
        let sat = color_resp[4];
        Ok(LightingValues {
            effect_id: effect as u16,
            brightness,
            speed,
            hue,
            saturation: sat,
        })
    }
}

/// Lighting values read from the device.
#[derive(Debug, Clone)]
pub struct LightingValues {
    /// Effect ID — for VialRGB this is a 16-bit VialRGB effect ID,
    /// for VIA/Vial legacy this is an 8-bit QMK effect index.
    pub effect_id: u16,
    pub brightness: u8,
    pub speed: u8,
    pub hue: u8,
    pub saturation: u8,
}

/// VialRGB protocol info.
#[derive(Debug, Clone)]
pub struct VialRgbInfo {
    pub protocol_version: u16,
    pub max_brightness: u8,
}
