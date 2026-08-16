use std::{error::Error, fmt};

use hidapi::{BusType as HidApiBusType, DeviceInfo, HidApi};

use crate::TargetIdentity;

/// Project-owned transport-bus classification for discovery output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    Unknown,
    Usb,
    Bluetooth,
    I2c,
    Spi,
}

/// Failure to initialize the read-only HID metadata discovery context.
#[derive(Debug)]
pub struct DeviceDiscoveryError {
    source: hidapi::HidError,
}

impl fmt::Display for DeviceDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to initialize HID metadata discovery: {}; no privileged recovery was attempted",
            self.source
        )
    }
}

impl Error for DeviceDiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for BusType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unknown => "unknown",
            Self::Usb => "usb",
            Self::Bluetooth => "bluetooth",
            Self::I2c => "i2c",
            Self::Spi => "spi",
        };

        formatter.write_str(name)
    }
}

/// Owned, HIDAPI-free metadata for one enumerated HID interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredHidInterface {
    pub interface_number: i32,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_string: Option<String>,
    pub manufacturer_string: Option<String>,
    pub serial_number: Option<String>,
    pub release_number: u16,
    pub bus_type: BusType,
    pub path: String,
}

impl DiscoveredHidInterface {
    /// Reports interface 2 only as an unvalidated configuration-interface candidate.
    #[must_use]
    pub const fn is_unvalidated_configuration_interface_candidate(&self) -> bool {
        self.interface_number == 2
    }
}

/// Enumerates target HID interface metadata without opening a device handle.
///
/// # Errors
/// Returns [`DeviceDiscoveryError`] when HIDAPI cannot initialize its metadata context.
pub fn enumerate_target_hid_interfaces() -> Result<Vec<DiscoveredHidInterface>, DeviceDiscoveryError>
{
    let api = HidApi::new().map_err(|source| DeviceDiscoveryError { source })?;
    let interfaces = api.device_list().map(map_device_info).collect::<Vec<_>>();

    Ok(filter_target_interfaces(
        TargetIdentity::default(),
        &interfaces,
    ))
}

fn map_device_info(device: &DeviceInfo) -> DiscoveredHidInterface {
    DiscoveredHidInterface {
        interface_number: device.interface_number(),
        vendor_id: device.vendor_id(),
        product_id: device.product_id(),
        product_string: device.product_string().map(str::to_owned),
        manufacturer_string: device.manufacturer_string().map(str::to_owned),
        serial_number: device.serial_number().map(str::to_owned),
        release_number: device.release_number(),
        bus_type: map_bus_type(device.bus_type()),
        path: escape_hid_path(device.path().to_bytes()),
    }
}

const fn map_bus_type(bus_type: HidApiBusType) -> BusType {
    match bus_type {
        HidApiBusType::Unknown => BusType::Unknown,
        HidApiBusType::Usb => BusType::Usb,
        HidApiBusType::Bluetooth => BusType::Bluetooth,
        HidApiBusType::I2c => BusType::I2c,
        HidApiBusType::Spi => BusType::Spi,
    }
}

/// Copies every matching VID/PID record in source order without deduplication.
#[must_use]
pub fn filter_target_interfaces(
    target: TargetIdentity,
    interfaces: &[DiscoveredHidInterface],
) -> Vec<DiscoveredHidInterface> {
    interfaces
        .iter()
        .filter(|interface| {
            interface.vendor_id == target.vendor_id && interface.product_id == target.product_id
        })
        .cloned()
        .collect()
}

/// Escapes path bytes obtained without a C string's trailing NUL for terminal output.
#[must_use]
pub fn escape_hid_path(path_bytes: &[u8]) -> String {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

    let mut escaped = String::with_capacity(path_bytes.len());
    for &byte in path_bytes {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            0x20..=0x7e => escaped.push(char::from(byte)),
            _ => {
                escaped.push_str("\\x");
                escaped.push(char::from(LOWER_HEX[usize::from(byte >> 4)]));
                escaped.push(char::from(LOWER_HEX[usize::from(byte & 0x0f)]));
            }
        }
    }

    escaped
}
