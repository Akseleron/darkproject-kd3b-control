//! Pure protocol definitions for Dark Project KD3B rev.2.
//! Hardware I/O does not belong in this crate.

pub mod direct_rgb;
pub mod layout;

pub use direct_rgb::{DirectRgbPackets, encode_direct_rgb};
pub use layout::{ALL_KEYS, Key};

pub const USB_VENDOR_ID: u16 = 0x195d;
pub const USB_PRODUCT_ID: u16 = 0x2061;
pub const LOGICAL_KEY_COUNT: usize = 87;
pub const DIRECT_PACKET_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb8 {
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_identity_is_stable() {
        assert_eq!(USB_VENDOR_ID, 0x195d);
        assert_eq!(USB_PRODUCT_ID, 0x2061);
        assert_eq!(LOGICAL_KEY_COUNT, 87);
        assert_eq!(DIRECT_PACKET_LEN, 256);
    }
}
