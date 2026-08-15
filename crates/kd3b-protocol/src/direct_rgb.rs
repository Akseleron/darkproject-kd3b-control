//! Typed immutable packets for the documented KD3B Direct RGB protocol.
//!
//! The encoder accepts one color per logical protocol key. For each
//! `key` in the canonical layout, `frame[key.index()]` is that key's RGB color.

use crate::{ALL_KEYS, DIRECT_PACKET_LEN, LOGICAL_KEY_COUNT, Rgb8};

const PACKET_A_HEADER: [u8; 5] = [0x08, 0x07, 0x00, 0x00, 0x00];
const PACKET_B_HEADER: [u8; 5] = [0x08, 0x07, 0x00, 0x01, 0x00];

const RED_CHANNEL_BASE: usize = 5;
const GREEN_CHANNEL_BASE: usize = 107;
const BLUE_CHANNEL_BASE: usize = 5;

/// The immutable Packet A and Packet B pair for one Direct RGB frame.
pub struct DirectRgbPackets {
    packet_a: [u8; DIRECT_PACKET_LEN],
    packet_b: [u8; DIRECT_PACKET_LEN],
}

/// Encodes one complete logical-key frame into documented Direct RGB packets.
#[must_use]
pub fn encode_direct_rgb(frame: &[Rgb8; LOGICAL_KEY_COUNT]) -> DirectRgbPackets {
    let mut packet_a = [0_u8; DIRECT_PACKET_LEN];
    let mut packet_b = [0_u8; DIRECT_PACKET_LEN];
    packet_a[..PACKET_A_HEADER.len()].copy_from_slice(&PACKET_A_HEADER);
    packet_b[..PACKET_B_HEADER.len()].copy_from_slice(&PACKET_B_HEADER);

    for key in ALL_KEYS {
        let color = frame[key.index()];
        let offset = usize::from(key.offset());
        packet_a[RED_CHANNEL_BASE + offset] = color.red;
        packet_a[GREEN_CHANNEL_BASE + offset] = color.green;
        packet_b[BLUE_CHANNEL_BASE + offset] = color.blue;
    }

    DirectRgbPackets { packet_a, packet_b }
}

impl DirectRgbPackets {
    /// Returns Packet A, containing the documented red and green channels.
    #[must_use]
    pub const fn packet_a(&self) -> &[u8; DIRECT_PACKET_LEN] {
        &self.packet_a
    }

    /// Returns Packet B, containing the documented blue channel.
    #[must_use]
    pub const fn packet_b(&self) -> &[u8; DIRECT_PACKET_LEN] {
        &self.packet_b
    }

    /// Returns the documented transport order: Packet A followed by Packet B.
    #[must_use]
    pub fn write_order(&self) -> [&[u8; DIRECT_PACKET_LEN]; 2] {
        [self.packet_a(), self.packet_b()]
    }
}
