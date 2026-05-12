use crate::ARGOS_CMD_PREFIX;

/// Argos command IDs as defined in the firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArgosCommandId {
    GetKbInfo = 0x01,
    GetCombo = 0x02,
    DeleteComboKey = 0x03,
    CaptureComboKey = 0x04,
    GetThemeId = 0x05,
    SetThemeId = 0x06,
    GetTapDance = 0x07,
    SetTapDance = 0x08,
    CaptureTapDanceKey = 0x09,
    DeleteTapDanceKey = 0x0A,
    SetDpi = 0x0B,
    GetPointingDeviceInfo = 0x0C,
    SetSnipingDpi = 0x0D,
    SetCombo = 0x0E,
    CaptureAllKeycodes = 0x0F,
    SetWelcomeMessageDisplayed = 0x10,
    SetGlobalTappingTerm = 0x11,
    SetGlobalComboTerm = 0x12,
}

/// Build a 33-byte HID report for an Argos command.
///
/// Layout: `[0x00 (report ID), ARGOS_CMD_PREFIX, command_id, data...]`
pub fn build_report(command_id: ArgosCommandId, data: &[u8]) -> [u8; 33] {
    let mut report = [0u8; 33];
    report[0] = 0x00; // HID report ID
    report[1] = ARGOS_CMD_PREFIX;
    report[2] = command_id as u8;

    let copy_len = data.len().min(30); // 33 - 3 header bytes = 30 max payload
    report[3..3 + copy_len].copy_from_slice(&data[..copy_len]);

    report
}
