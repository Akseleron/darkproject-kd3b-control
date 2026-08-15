use kd3b_protocol::DirectRgbPackets;

/// Low-level boundary for writing already-encoded immutable packet bytes.
///
/// Implementations do not know the packet format or length and are not
/// responsible for transport selection or device discovery. This is an
/// internal real-backend seam, not a user-facing arbitrary-send API.
pub trait PacketTransport {
    type Error;

    /// Writes one already-encoded packet byte slice.
    ///
    /// # Errors
    /// Returns the implementation-specific write error when the transport
    /// cannot accept the already-encoded bytes.
    fn write_packet(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

/// Forwards the encoded Direct RGB packet pair in its documented write order.
///
/// # Errors
/// Returns the first transport write error without attempting later packets.
pub fn write_direct_rgb<T: PacketTransport>(
    transport: &mut T,
    packets: &DirectRgbPackets,
) -> Result<(), T::Error> {
    for packet in packets.write_order() {
        transport.write_packet(packet)?;
    }

    Ok(())
}
