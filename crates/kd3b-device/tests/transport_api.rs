use std::convert::Infallible;

use kd3b_device::{MockTransport, PacketTransport, write_direct_rgb};
use kd3b_protocol::{DirectRgbPackets, LOGICAL_KEY_COUNT, Rgb8, encode_direct_rgb};

#[derive(Default)]
struct RecordingTransport {
    writes: Vec<Vec<u8>>,
}

impl PacketTransport for RecordingTransport {
    type Error = Infallible;

    fn write_packet(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.writes.push(bytes.to_vec());
        Ok(())
    }
}

fn direct_rgb_packets() -> DirectRgbPackets {
    let frame = [Rgb8::new(0x12, 0x34, 0x56); LOGICAL_KEY_COUNT];
    encode_direct_rgb(&frame)
}

#[test]
fn generic_transport_accepts_non_256_byte_slices() {
    // Given
    let mut transport = RecordingTransport::default();
    let encoded_packet = [0x04, 0x17, 0xa5];

    // When
    let result = transport.write_packet(&encoded_packet);

    // Then
    assert_eq!(result, Ok(()));
    assert_eq!(transport.writes, [encoded_packet]);
}

#[test]
fn direct_rgb_forwards_packet_a_then_packet_b_exactly() {
    // Given
    let packets = direct_rgb_packets();
    let expected = packets.write_order();
    let mut transport = RecordingTransport::default();

    // When
    let result = write_direct_rgb(&mut transport, &packets);

    // Then
    assert_eq!(result, Ok(()));
    assert_eq!(transport.writes.len(), 2);
    assert_eq!(transport.writes[0], expected[0]);
    assert_eq!(transport.writes[1], expected[1]);
}

#[test]
fn mock_exposes_recorded_writes_as_read_only_slice() {
    // Given
    let mut transport = MockTransport::new();
    let bytes = [0x21, 0x43, 0x65];

    // When
    let result = transport.write_packet(&bytes);

    // Then
    let recorded: &[Vec<u8>] = transport.recorded_writes();
    assert_eq!(result, Ok(()));
    assert_eq!(recorded, [bytes]);
}

#[test]
fn direct_rgb_stops_after_packet_a_failure() {
    // Given
    let packets = direct_rgb_packets();
    let mut transport = MockTransport::new();
    transport.fail_on_write(0);

    // When
    let result = write_direct_rgb(&mut transport, &packets);

    // Then
    assert_eq!(result.map_err(|error| error.write_index()), Err(0));
    assert!(transport.recorded_writes().is_empty());
}

#[test]
fn direct_rgb_returns_packet_b_failure_after_packet_a_succeeds() {
    // Given
    let packets = direct_rgb_packets();
    let expected_packet_a = packets.packet_a();
    let mut transport = MockTransport::new();
    transport.fail_on_write(1);

    // When
    let result = write_direct_rgb(&mut transport, &packets);

    // Then
    assert_eq!(result.map_err(|error| error.write_index()), Err(1));
    assert_eq!(transport.recorded_writes(), [expected_packet_a]);
}

#[test]
fn mock_counter_advances_after_injected_failure() {
    // Given
    let mut transport = MockTransport::new();
    transport.fail_on_write(0);
    let failed_bytes = [0xaa];
    let post_failure_bytes = [0xbb, 0xcc];

    // When
    let failure = transport.write_packet(&failed_bytes);
    let post_failure_result = transport.write_packet(&post_failure_bytes);

    // Then
    assert_eq!(failure.map_err(|error| error.write_index()), Err(0));
    assert_eq!(post_failure_result, Ok(()));
    assert_eq!(transport.recorded_writes(), [post_failure_bytes]);
}

#[test]
fn mock_successful_write_owns_copy_of_caller_bytes() {
    // Given
    let mut transport = MockTransport::new();
    let mut caller_buffer = vec![0x04, 0x17, 0xa5];

    // When
    let result = transport.write_packet(&caller_buffer);
    caller_buffer[0] = 0xff;

    // Then
    assert_eq!(result, Ok(()));
    assert_eq!(transport.recorded_writes(), [vec![0x04, 0x17, 0xa5]]);
}
