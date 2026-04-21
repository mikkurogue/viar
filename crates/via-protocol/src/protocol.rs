use crate::{
    command::{LightingChannel, RgbValueId},
    device::KeyboardDevice,
    ViaCommand, ViaResult,
};
use tracing::debug;

/// High-level VIA protocol interface for a connected keyboard.
pub struct ViaProtocol<'a> {
    device: &'a KeyboardDevice,
}

impl<'a> ViaProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self { device }
    }

    /// Get the VIA protocol version supported by the keyboard.
    /// Returns (major, minor) — e.g. VIA V3 keyboards return (12, _) or similar.
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

    /// Read a chunk of the keymap buffer. Used for bulk-reading the entire keymap.
    /// Returns the payload bytes (up to `size` bytes) from the buffer at `offset`.
    pub fn get_keymap_buffer(&self, offset: u16, size: u8) -> ViaResult<Vec<u8>> {
        let resp = self
            .device
            .send_command(&ViaCommand::get_keymap_buffer(offset, size))?;
        // Response: [cmd_id, offset_hi, offset_lo, payload...]
        let payload_start = 4; // cmd + offset(2) + size(1)
        let end = (payload_start + size as usize).min(resp.len());
        Ok(resp[payload_start..end].to_vec())
    }

    /// Read the entire dynamic keymap as a flat Vec of u16 keycodes.
    /// `rows` and `cols` must match the keyboard's matrix dimensions.
    pub fn read_entire_keymap(
        &self,
        layers: u8,
        rows: u8,
        cols: u8,
    ) -> ViaResult<Vec<Vec<Vec<u16>>>> {
        let total_keys = layers as usize * rows as usize * cols as usize;
        let total_bytes = total_keys * 2; // 2 bytes per keycode

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

        // Parse into [layer][row][col]
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

    // -- Lighting --

    /// Get RGB brightness (0-255) for a given lighting channel.
    pub fn get_rgb_brightness(&self, channel: LightingChannel) -> ViaResult<u8> {
        let resp = self.device.send_command(&ViaCommand::get_lighting_value(
            channel as u8,
            RgbValueId::Brightness as u8,
        ))?;
        let val = resp[2];
        debug!(channel = ?channel, brightness = val, "get_rgb_brightness");
        Ok(val)
    }

    /// Get current RGB effect index.
    pub fn get_rgb_effect(&self, channel: LightingChannel) -> ViaResult<u8> {
        let resp = self.device.send_command(&ViaCommand::get_lighting_value(
            channel as u8,
            RgbValueId::Effect as u8,
        ))?;
        let val = resp[2];
        debug!(channel = ?channel, effect = val, "get_rgb_effect");
        Ok(val)
    }

    /// Get RGB effect speed (0-255).
    pub fn get_rgb_speed(&self, channel: LightingChannel) -> ViaResult<u8> {
        let resp = self.device.send_command(&ViaCommand::get_lighting_value(
            channel as u8,
            RgbValueId::EffectSpeed as u8,
        ))?;
        let val = resp[2];
        debug!(channel = ?channel, speed = val, "get_rgb_speed");
        Ok(val)
    }

    /// Get RGB color as (hue, saturation). Both 0-255.
    pub fn get_rgb_color(&self, channel: LightingChannel) -> ViaResult<(u8, u8)> {
        let resp = self.device.send_command(&ViaCommand::get_lighting_value(
            channel as u8,
            RgbValueId::Color as u8,
        ))?;
        let hue = resp[2];
        let sat = resp[3];
        debug!(channel = ?channel, hue, sat, "get_rgb_color");
        Ok((hue, sat))
    }

    /// Set RGB brightness (0-255).
    pub fn set_rgb_brightness(&self, channel: LightingChannel, brightness: u8) -> ViaResult<()> {
        debug!(channel = ?channel, brightness, "set_rgb_brightness");
        self.device.send_command(&ViaCommand::set_lighting_value(
            channel as u8,
            RgbValueId::Brightness as u8,
            &[brightness],
        ))?;
        Ok(())
    }

    /// Set RGB effect index.
    pub fn set_rgb_effect(&self, channel: LightingChannel, effect: u8) -> ViaResult<()> {
        debug!(channel = ?channel, effect, "set_rgb_effect");
        self.device.send_command(&ViaCommand::set_lighting_value(
            channel as u8,
            RgbValueId::Effect as u8,
            &[effect],
        ))?;
        Ok(())
    }

    /// Set RGB effect speed (0-255).
    pub fn set_rgb_speed(&self, channel: LightingChannel, speed: u8) -> ViaResult<()> {
        debug!(channel = ?channel, speed, "set_rgb_speed");
        self.device.send_command(&ViaCommand::set_lighting_value(
            channel as u8,
            RgbValueId::EffectSpeed as u8,
            &[speed],
        ))?;
        Ok(())
    }

    /// Set RGB color (hue, saturation). Both 0-255.
    pub fn set_rgb_color(&self, channel: LightingChannel, hue: u8, sat: u8) -> ViaResult<()> {
        debug!(channel = ?channel, hue, sat, "set_rgb_color");
        self.device.send_command(&ViaCommand::set_lighting_value(
            channel as u8,
            RgbValueId::Color as u8,
            &[hue, sat],
        ))?;
        Ok(())
    }

    /// Save custom values (lighting) to EEPROM.
    pub fn custom_save(&self) -> ViaResult<()> {
        debug!("custom_save");
        self.device.send_command(&ViaCommand::custom_save())?;
        Ok(())
    }

    /// Try to detect which lighting channel the keyboard supports.
    /// Sends a get_brightness to each channel and returns the first one that responds
    /// without error.
    pub fn detect_lighting_channel(&self) -> Option<LightingChannel> {
        for channel in [
            LightingChannel::QmkRgbMatrix,
            LightingChannel::QmkRgblight,
            LightingChannel::QmkLed,
        ] {
            if self.get_rgb_brightness(channel).is_ok() {
                debug!(channel = ?channel, "detected lighting channel");
                return Some(channel);
            }
        }
        debug!("no lighting channel detected");
        None
    }
}
