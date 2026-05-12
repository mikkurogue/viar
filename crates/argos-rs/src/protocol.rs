use tracing::{
    debug,
    error,
    trace,
};
use via_protocol::KeyboardDevice;

use crate::{
    ARGOS_CMD_PREFIX,
    ARGOS_COMBO_ENTRIES,
    ARGOS_KEYS_PER_COMBO,
    ARGOS_TAP_DANCE_ENTRIES,
    command::{
        ArgosCommandId,
        build_report,
    },
    error::{
        ArgosError,
        ArgosResult,
    },
    types::{
        ArgosCombo,
        ArgosKbInfo,
        ArgosTapDance,
        CapturedKeycode,
        PointingDeviceInfo,
    },
};

/// High-level interface for the Argos protocol on a connected keyboard.
///
/// Borrows a [`KeyboardDevice`] (from `via-protocol`) and provides typed
/// methods for every Argos command.
pub struct ArgosProtocol<'a> {
    device: &'a KeyboardDevice,
}

impl<'a> ArgosProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self { device }
    }

    // ── helpers ──────────────────────────────────────────────────────────

    /// Send an Argos command and return the 32-byte raw response.
    fn send(&self, command_id: ArgosCommandId, data: &[u8]) -> ArgosResult<[u8; 32]> {
        let report = build_report(command_id, data);
        let resp = self.device.raw_hid_send(&report)?;
        Ok(resp)
    }

    /// Send an Argos command, verify the response prefix matches, and return
    /// the 30-byte command_data portion (bytes 2..32 of the response).
    fn send_and_verify(&self, command_id: ArgosCommandId, data: &[u8]) -> ArgosResult<[u8; 30]> {
        let resp = self.send(command_id, data)?;

        if resp[0] != ARGOS_CMD_PREFIX || resp[1] != command_id as u8 {
            return Err(ArgosError::Protocol(format!(
                "unexpected response header: [{:#04x}, {:#04x}], expected [{:#04x}, {:#04x}]",
                resp[0], resp[1], ARGOS_CMD_PREFIX, command_id as u8,
            )));
        }

        let mut cmd_data = [0u8; 30];
        cmd_data.copy_from_slice(&resp[2..]);
        Ok(cmd_data)
    }

    /// Probe whether this keyboard supports the Argos protocol.
    ///
    /// Sends a `GetKbInfo` command and returns `Some(info)` if the device
    /// responds with a valid Argos header, or `None` otherwise.
    pub fn probe(&self) -> Option<ArgosKbInfo> {
        match self.get_kb_info() {
            Ok(info) => {
                debug!(
                    protocol_version = info.protocol_version,
                    "argos protocol detected"
                );
                Some(info)
            }
            Err(e) => {
                error!("argos probe failed: {e}");
                trace!("unsupported or non official bastard keyboard device");
                None
            }
        }
    }

    /// Get keyboard information (protocol version, capabilities, config).
    pub fn get_kb_info(&self) -> ArgosResult<ArgosKbInfo> {
        let d = self.send_and_verify(ArgosCommandId::GetKbInfo, &[])?;

        Ok(ArgosKbInfo {
            protocol_version: u16::from_be_bytes([d[0], d[1]]),
            tap_dance_entries: d[2],
            combo_entries: d[3],
            keys_per_combo: d[4],
            theme_id: d[5],
            keycodes_version: [d[6], d[7], d[8]],
            welcome_displayed: d[9] != 0,
            global_tapping_term: u16::from_be_bytes([d[10], d[11]]),
            global_combo_term: u16::from_be_bytes([d[12], d[13]]),
        })
    }

    /// Get the current theme ID.
    pub fn get_theme_id(&self) -> ArgosResult<u8> {
        let d = self.send_and_verify(ArgosCommandId::GetThemeId, &[])?;
        Ok(d[0])
    }

    /// Set the theme ID.
    pub fn set_theme_id(&self, theme_id: u8) -> ArgosResult<()> {
        self.send_and_verify(ArgosCommandId::SetThemeId, &[theme_id])?;
        Ok(())
    }

    /// Mark the welcome message as displayed (or not).
    pub fn set_welcome_message_displayed(&self, displayed: bool) -> ArgosResult<()> {
        self.send_and_verify(
            ArgosCommandId::SetWelcomeMessageDisplayed,
            &[u8::from(displayed)],
        )?;
        Ok(())
    }

    /// Set the global tapping term (milliseconds).
    pub fn set_global_tapping_term(&self, ms: u16) -> ArgosResult<()> {
        let bytes = ms.to_be_bytes();
        self.send_and_verify(ArgosCommandId::SetGlobalTappingTerm, &bytes)?;
        Ok(())
    }

    /// Set the global combo term (milliseconds).
    pub fn set_global_combo_term(&self, ms: u16) -> ArgosResult<()> {
        let bytes = ms.to_be_bytes();
        self.send_and_verify(ArgosCommandId::SetGlobalComboTerm, &bytes)?;
        Ok(())
    }

    /// Get a combo entry by index.
    pub fn get_combo(&self, index: u8) -> ArgosResult<ArgosCombo> {
        if index >= ARGOS_COMBO_ENTRIES {
            return Err(ArgosError::IndexOutOfRange {
                index,
                max: ARGOS_COMBO_ENTRIES - 1,
            });
        }

        let d = self.send_and_verify(ArgosCommandId::GetCombo, &[index])?;

        let enabled = d[1] != 0;
        let keycode = u16::from_le_bytes([d[2], d[3]]);
        // bytes 4-5 reserved for custom tapping term
        let mut keys = [0u16; 4];
        for i in 0..ARGOS_KEYS_PER_COMBO as usize {
            keys[i] = u16::from_le_bytes([d[6 + i * 2], d[7 + i * 2]]);
        }

        Ok(ArgosCombo {
            enabled,
            keycode,
            keys,
        })
    }

    /// Set a combo entry by index.
    ///
    /// The firmware wire format for `SetCombo` is:
    /// `[combo_index, keycode_lo, keycode_hi, key0_lo, key0_hi, key1_lo, key1_hi, ...]`
    pub fn set_combo(&self, index: u8, combo: &ArgosCombo) -> ArgosResult<()> {
        if index >= ARGOS_COMBO_ENTRIES {
            return Err(ArgosError::IndexOutOfRange {
                index,
                max: ARGOS_COMBO_ENTRIES - 1,
            });
        }

        let kc = combo.keycode.to_le_bytes();
        // Build payload: [index, kc_lo, kc_hi, key0_lo, key0_hi, ...]
        let mut payload = [0u8; 1 + 2 + 8]; // 1 index + 2 keycode + 4 keys * 2 bytes
        payload[0] = index;
        payload[1] = kc[0];
        payload[2] = kc[1];
        for i in 0..ARGOS_KEYS_PER_COMBO as usize {
            let kb = combo.keys[i].to_le_bytes();
            payload[3 + i * 2] = kb[0];
            payload[4 + i * 2] = kb[1];
        }

        self.send_and_verify(ArgosCommandId::SetCombo, &payload)?;
        Ok(())
    }

    /// Get all combo entries.
    pub fn get_all_combos(&self) -> ArgosResult<Vec<ArgosCombo>> {
        let mut combos = Vec::with_capacity(ARGOS_COMBO_ENTRIES as usize);
        for i in 0..ARGOS_COMBO_ENTRIES {
            combos.push(self.get_combo(i)?);
        }
        Ok(combos)
    }

    /// Delete (reset) a combo key at the given key index.
    pub fn delete_combo_key(&self, key_index: u8) -> ArgosResult<()> {
        self.send(ArgosCommandId::DeleteComboKey, &[key_index])?;
        Ok(())
    }

    /// Start capturing the next key press for a combo slot.
    ///
    /// This is a blocking call -- the firmware will wait for a key press
    /// and respond with the captured keycode.
    pub fn capture_combo_key(&self, combo_index: u8, key_index: u8) -> ArgosResult<u16> {
        let d = self.send_and_verify(ArgosCommandId::CaptureComboKey, &[combo_index, key_index])?;
        Ok(u16::from_le_bytes([d[0], d[1]]))
    }

    /// Get a tap dance entry by index.
    pub fn get_tap_dance(&self, index: u8) -> ArgosResult<ArgosTapDance> {
        if index >= ARGOS_TAP_DANCE_ENTRIES {
            return Err(ArgosError::IndexOutOfRange {
                index,
                max: ARGOS_TAP_DANCE_ENTRIES - 1,
            });
        }

        let d = self.send_and_verify(ArgosCommandId::GetTapDance, &[index])?;

        // Response layout (command_data, starting at d[0] which is the index echo):
        // d[0] = index (echo), d[1..2] = on_tap LE, d[3..4] = on_hold LE, etc.
        Ok(ArgosTapDance {
            on_tap: u16::from_le_bytes([d[1], d[2]]),
            on_hold: u16::from_le_bytes([d[3], d[4]]),
            on_double_tap: u16::from_le_bytes([d[5], d[6]]),
            on_tap_hold: u16::from_le_bytes([d[7], d[8]]),
            custom_tapping_term: u16::from_le_bytes([d[9], d[10]]),
        })
    }

    /// Set a tap dance entry by index.
    ///
    /// Wire format: `[index, tap_lo, tap_hi, hold_lo, hold_hi, dtap_lo, dtap_hi, thold_lo,
    /// thold_hi]`
    pub fn set_tap_dance(&self, index: u8, td: &ArgosTapDance) -> ArgosResult<()> {
        if index >= ARGOS_TAP_DANCE_ENTRIES {
            return Err(ArgosError::IndexOutOfRange {
                index,
                max: ARGOS_TAP_DANCE_ENTRIES - 1,
            });
        }

        let mut payload = [0u8; 9]; // 1 index + 4 keycodes * 2 bytes
        payload[0] = index;
        let tap = td.on_tap.to_le_bytes();
        let hold = td.on_hold.to_le_bytes();
        let dtap = td.on_double_tap.to_le_bytes();
        let thold = td.on_tap_hold.to_le_bytes();
        payload[1] = tap[0];
        payload[2] = tap[1];
        payload[3] = hold[0];
        payload[4] = hold[1];
        payload[5] = dtap[0];
        payload[6] = dtap[1];
        payload[7] = thold[0];
        payload[8] = thold[1];

        self.send_and_verify(ArgosCommandId::SetTapDance, &payload)?;
        Ok(())
    }

    /// Get all tap dance entries.
    pub fn get_all_tap_dances(&self) -> ArgosResult<Vec<ArgosTapDance>> {
        let mut entries = Vec::with_capacity(ARGOS_TAP_DANCE_ENTRIES as usize);
        for i in 0..ARGOS_TAP_DANCE_ENTRIES {
            entries.push(self.get_tap_dance(i)?);
        }
        Ok(entries)
    }

    /// Delete (reset) a tap dance key at the given index.
    pub fn delete_tap_dance_key(&self, index: u8) -> ArgosResult<()> {
        self.send(ArgosCommandId::DeleteTapDanceKey, &[index])?;
        Ok(())
    }

    /// Start capturing the next key press for a tap dance slot.
    ///
    /// This is a blocking call -- the firmware will wait for a key press
    /// and respond with the captured keycode.
    pub fn capture_tap_dance_key(&self, td_index: u8, key_index: u8) -> ArgosResult<u16> {
        let d = self.send_and_verify(ArgosCommandId::CaptureTapDanceKey, &[td_index, key_index])?;
        Ok(u16::from_le_bytes([d[0], d[1]]))
    }

    /// Get pointing device information.
    pub fn get_pointing_device_info(&self) -> ArgosResult<PointingDeviceInfo> {
        let d = self.send_and_verify(ArgosCommandId::GetPointingDeviceInfo, &[])?;
        Ok(PointingDeviceInfo { raw: d })
    }

    /// Set the main DPI.
    ///
    /// The exact payload format depends on the firmware (typically a u16 LE).
    pub fn set_dpi(&self, dpi: u16) -> ArgosResult<()> {
        let bytes = dpi.to_le_bytes();
        self.send_and_verify(ArgosCommandId::SetDpi, &bytes)?;
        Ok(())
    }

    /// Set the sniping DPI.
    pub fn set_sniping_dpi(&self, dpi: u16) -> ArgosResult<()> {
        let bytes = dpi.to_le_bytes();
        self.send_and_verify(ArgosCommandId::SetSnipingDpi, &bytes)?;
        Ok(())
    }

    /// Enable or disable keymap test mode (capture all keycodes).
    ///
    /// When enabled, the firmware captures the next key press/release and
    /// sends it back. The webapp is expected to re-send the capture command
    /// after each received keycode to continue listening.
    pub fn capture_all_keycodes(&self, enable: bool) -> ArgosResult<()> {
        // The firmware does NOT ack this command -- the ack comes when a key
        // is actually pressed. So we just send without verifying the response.
        self.send(ArgosCommandId::CaptureAllKeycodes, &[u8::from(enable)])?;
        Ok(())
    }

    /// Read a captured keycode response.
    ///
    /// Call this after `capture_all_keycodes(true)` to block until the user
    /// presses a key. The firmware sends the keycode in the response.
    ///
    /// Returns `None` if the response doesn't match the expected format
    /// (e.g. timeout was handled at a lower level).
    pub fn read_captured_keycode(&self) -> ArgosResult<Option<CapturedKeycode>> {
        // We need to read a response without sending a new command.
        // The firmware will send data when a key is pressed.
        // We use the device's raw read with a longer timeout.
        let mut buf = [0u8; 32];
        // Read with a generous timeout (5 seconds) since we're waiting for user input
        let report = self
            .device
            .raw_hid_send(&build_report(ArgosCommandId::CaptureAllKeycodes, &[1]))?;

        buf.copy_from_slice(&report);

        if buf[0] != ARGOS_CMD_PREFIX || buf[1] != ArgosCommandId::CaptureAllKeycodes as u8 {
            return Ok(None);
        }

        Ok(Some(CapturedKeycode {
            pressed: buf[2] != 0,
            keycode: u16::from_le_bytes([buf[3], buf[4]]),
        }))
    }
}
