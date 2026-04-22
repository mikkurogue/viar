use tracing::{
    debug,
    info,
    trace,
    warn,
};

use crate::{
    VIA_USAGE,
    VIA_USAGE_PAGE,
    ViaError,
    ViaResult,
};

/// Information about a detected VIA-compatible keyboard.
#[derive(Debug, Clone)]
pub struct KeyboardInfo {
    pub vendor_id:     u16,
    pub product_id:    u16,
    pub manufacturer:  String,
    pub product:       String,
    pub serial_number: String,
    pub path:          String,
}

impl std::fmt::Display for KeyboardInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({:04x}:{:04x})",
            self.manufacturer, self.product, self.vendor_id, self.product_id
        )
    }
}

/// A handle to an opened VIA-compatible keyboard HID device.
pub struct KeyboardDevice {
    pub info: KeyboardInfo,
    device:   hidapi::HidDevice,
}

impl KeyboardDevice {
    /// Open a keyboard by its HID path.
    pub fn open(api: &hidapi::HidApi, info: KeyboardInfo) -> ViaResult<Self> {
        debug!(path = %info.path, keyboard = %info, "opening HID device");
        let device = api.open_path(std::ffi::CString::new(info.path.clone()).unwrap().as_ref())?;
        device.set_blocking_mode(true)?;
        info!(keyboard = %info, "connected to keyboard");
        Ok(Self { info, device })
    }

    /// Send a raw 33-byte report (report ID + 32 bytes) and read the response.
    pub fn raw_hid_send(&self, report: &[u8; 33]) -> ViaResult<[u8; 32]> {
        trace!(cmd = report[1], "HID write {} bytes", report.len());
        self.device.write(report)?;
        let mut buf = [0u8; 32];
        let n = self.device.read_timeout(&mut buf, 1000)?;
        if n == 0 {
            warn!("HID read timeout");
            return Err(ViaError::Timeout);
        }
        trace!(cmd = buf[0], "HID read {n} bytes");
        Ok(buf)
    }

    /// Send a VIA command and return the raw 32-byte response.
    pub fn send_command(&self, cmd: &crate::ViaCommand) -> ViaResult<[u8; 32]> {
        let report = cmd.to_report();
        self.raw_hid_send(&report)
    }
}

/// Scan for all connected VIA-compatible keyboards.
///
/// This looks for HID devices with usage page 0xFF60 and usage 0x61,
/// which is the standard VIA raw HID interface.
pub fn discover_keyboards(api: &hidapi::HidApi) -> Vec<KeyboardInfo> {
    debug!("scanning for VIA keyboards (usage_page={VIA_USAGE_PAGE:#06x}, usage={VIA_USAGE:#04x})");

    let keyboards: Vec<_> = api
        .device_list()
        .filter(|dev| dev.usage_page() == VIA_USAGE_PAGE && dev.usage() == VIA_USAGE)
        .map(|dev| KeyboardInfo {
            vendor_id:     dev.vendor_id(),
            product_id:    dev.product_id(),
            manufacturer:  dev.manufacturer_string().unwrap_or_default().to_string(),
            product:       dev.product_string().unwrap_or_default().to_string(),
            serial_number: dev.serial_number().unwrap_or_default().to_string(),
            path:          dev.path().to_string_lossy().into_owned(),
        })
        .collect();

    info!(count = keyboards.len(), "discovered VIA keyboards");
    for kb in &keyboards {
        debug!(keyboard = %kb, path = %kb.path, "found keyboard");
    }

    keyboards
}

/// Check whether HID access is likely blocked by permissions.
/// On Linux, this typically means missing udev rules for hidraw devices.
pub fn check_hid_permissions() -> HidAccessStatus {
    match hidapi::HidApi::new() {
        Ok(api) => {
            // We got an API handle — check if we can actually see any HID devices at all
            let total_devices = api.device_list().count();
            let via_devices = api
                .device_list()
                .filter(|dev| dev.usage_page() == VIA_USAGE_PAGE && dev.usage() == VIA_USAGE)
                .count();

            if total_devices == 0 {
                debug!("HID API initialized but no devices visible — likely permission issue");
                HidAccessStatus::NoPermission
            } else if via_devices == 0 {
                debug!(
                    total_devices,
                    "HID devices visible but none are VIA keyboards"
                );
                HidAccessStatus::NoViaDevices
            } else {
                HidAccessStatus::Ok
            }
        }
        Err(e) => {
            warn!(error = %e, "failed to initialize HID API");
            HidAccessStatus::InitFailed(e.to_string())
        }
    }
}

/// Result of checking HID subsystem access.
#[derive(Debug, Clone)]
pub enum HidAccessStatus {
    /// Everything works, VIA devices are visible.
    Ok,
    /// HID API works but zero devices are visible — likely a permissions issue.
    NoPermission,
    /// HID devices are visible but none match VIA usage page/usage.
    NoViaDevices,
    /// HID API failed to initialize entirely.
    InitFailed(String),
}
