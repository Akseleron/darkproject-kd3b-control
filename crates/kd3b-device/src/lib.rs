//! Device orchestration and transport abstractions for KD3B.
//! The initial repository baseline intentionally contains no real HID backend.

mod mock;
mod transport;

pub use mock::{MockTransport, MockTransportError};
pub use transport::{PacketTransport, write_direct_rgb};

use kd3b_protocol::{USB_PRODUCT_ID, USB_VENDOR_ID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetIdentity {
    pub vendor_id: u16,
    pub product_id: u16,
}

impl Default for TargetIdentity {
    fn default() -> Self {
        Self {
            vendor_id: USB_VENDOR_ID,
            product_id: USB_PRODUCT_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_identity_matches_protocol_crate() {
        assert_eq!(
            TargetIdentity::default(),
            TargetIdentity {
                vendor_id: 0x195d,
                product_id: 0x2061,
            }
        );
    }
}
