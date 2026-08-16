use std::{error::Error, fmt};

pub const LINKTYPE_USBPCAP: u16 = 249;
pub const KD3B_CONFIGURATION_ENDPOINT: u8 = 3;

const SECTION_HEADER_BLOCK: u32 = 0x0A0D_0D0A;
const INTERFACE_DESCRIPTION_BLOCK: u32 = 1;
const ENHANCED_PACKET_BLOCK: u32 = 6;
const BYTE_ORDER_MAGIC: u32 = 0x1A2B_3C4D;
const USBPCAP_BASE_HEADER_LEN: usize = 27;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    fn read_u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }

    fn read_u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    offset: usize,
    message: String,
}

impl ParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "capture parse error at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedPacket {
    pub section_index: usize,
    pub interface_id: u32,
    pub linktype: u16,
    pub captured_len: u32,
    pub original_len: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PcapNgCapture {
    pub section_count: usize,
    pub interface_count: usize,
    pub packets: Vec<CapturedPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDirection {
    Out,
    In,
}

impl fmt::Display for UsbDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Out => formatter.write_str("OUT"),
            Self::In => formatter.write_str("IN"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbPcapPacket {
    pub section_index: usize,
    pub interface_id: u32,
    pub header_len: u16,
    pub irp_id: u64,
    pub status: u32,
    pub function: u16,
    pub info: u8,
    pub bus: u16,
    pub device: u16,
    pub endpoint: u8,
    pub transfer: u8,
    pub data_length: u32,
    pub payload: Vec<u8>,
}

impl UsbPcapPacket {
    #[must_use]
    pub const fn direction(&self) -> UsbDirection {
        if self.endpoint & 0x80 == 0 {
            UsbDirection::Out
        } else {
            UsbDirection::In
        }
    }

    #[must_use]
    pub const fn endpoint_number(&self) -> u8 {
        self.endpoint & 0x0F
    }

    #[must_use]
    pub const fn is_kd3b_configuration_out(&self) -> bool {
        matches!(self.direction(), UsbDirection::Out)
            && self.endpoint_number() == KD3B_CONFIGURATION_ENDPOINT
    }
}

#[derive(Debug, Default)]
struct SectionState {
    endianness: Option<Endianness>,
    section_index: Option<usize>,
    interfaces: Vec<u16>,
}

/// Parses the pcapng block structure needed by the KD3B research tooling.
///
/// # Errors
///
/// Returns [`ParseError`] when the block stream is truncated, structurally invalid, references an
/// unknown interface, or contains lengths that cannot be represented safely.
pub fn parse_pcapng(bytes: &[u8]) -> Result<PcapNgCapture, ParseError> {
    let mut capture = PcapNgCapture::default();
    let mut state = SectionState::default();
    let mut offset = 0usize;

    while offset < bytes.len() {
        if bytes.len() - offset < 12 {
            return Err(ParseError::new(offset, "truncated block header"));
        }
        let raw_type = array4(bytes, offset)?;
        if u32::from_le_bytes(raw_type) == SECTION_HEADER_BLOCK {
            let (endianness, total_length) = parse_section_header(bytes, offset)?;
            state.endianness = Some(endianness);
            state.section_index = Some(capture.section_count);
            state.interfaces.clear();
            capture.section_count += 1;
            offset += total_length;
            continue;
        }

        let endianness = state.endianness.ok_or_else(|| {
            ParseError::new(
                offset,
                "packet capture does not start with a section header",
            )
        })?;
        let section_index = state
            .section_index
            .ok_or_else(|| ParseError::new(offset, "capture parser has no active section index"))?;
        let block_type = endianness.read_u32(raw_type);
        let total_length = block_length(bytes, offset, endianness)?;
        validate_block(bytes, offset, total_length, endianness)?;
        let body_start = offset + 8;
        let body = &bytes[body_start..offset + total_length - 4];

        match block_type {
            INTERFACE_DESCRIPTION_BLOCK => {
                let linktype = parse_interface_description(body, body_start, endianness)?;
                state.interfaces.push(linktype);
                capture.interface_count += 1;
            }
            ENHANCED_PACKET_BLOCK => capture.packets.push(parse_enhanced_packet(
                body,
                body_start,
                endianness,
                section_index,
                &state.interfaces,
            )?),
            _ => {}
        }

        offset += total_length;
    }

    if capture.section_count == 0 {
        return Err(ParseError::new(0, "capture contains no section header"));
    }
    Ok(capture)
}

fn parse_section_header(bytes: &[u8], offset: usize) -> Result<(Endianness, usize), ParseError> {
    let byte_order_bytes = array4(bytes, offset + 8)?;
    let endianness = if u32::from_le_bytes(byte_order_bytes) == BYTE_ORDER_MAGIC {
        Endianness::Little
    } else if u32::from_be_bytes(byte_order_bytes) == BYTE_ORDER_MAGIC {
        Endianness::Big
    } else {
        return Err(ParseError::new(
            offset + 8,
            "invalid section byte-order magic",
        ));
    };
    let total_length = block_length(bytes, offset, endianness)?;
    validate_block(bytes, offset, total_length, endianness)?;
    if total_length < 28 {
        return Err(ParseError::new(offset, "section header block is too short"));
    }
    Ok((endianness, total_length))
}

fn parse_interface_description(
    body: &[u8],
    body_start: usize,
    endianness: Endianness,
) -> Result<u16, ParseError> {
    if body.len() < 8 {
        return Err(ParseError::new(
            body_start,
            "interface description block is too short",
        ));
    }
    Ok(endianness.read_u16([body[0], body[1]]))
}

fn parse_enhanced_packet(
    body: &[u8],
    body_start: usize,
    endianness: Endianness,
    section_index: usize,
    interfaces: &[u16],
) -> Result<CapturedPacket, ParseError> {
    if body.len() < 20 {
        return Err(ParseError::new(
            body_start,
            "enhanced packet block is too short",
        ));
    }
    let interface_id = endianness.read_u32([body[0], body[1], body[2], body[3]]);
    let captured_len = endianness.read_u32([body[12], body[13], body[14], body[15]]);
    let original_len = endianness.read_u32([body[16], body[17], body[18], body[19]]);
    let interface_index = usize::try_from(interface_id)
        .map_err(|_| ParseError::new(body_start, "interface id does not fit this platform"))?;
    let linktype = *interfaces.get(interface_index).ok_or_else(|| {
        ParseError::new(
            body_start,
            format!("enhanced packet references unknown interface {interface_id}"),
        )
    })?;
    let captured_len_usize = usize::try_from(captured_len).map_err(|_| {
        ParseError::new(
            body_start + 12,
            "captured length does not fit this platform",
        )
    })?;
    let packet_end = 20usize.checked_add(captured_len_usize).ok_or_else(|| {
        ParseError::new(body_start + 12, "captured length overflows address space")
    })?;
    if packet_end > body.len() {
        return Err(ParseError::new(
            body_start + 20,
            "enhanced packet data is truncated",
        ));
    }

    Ok(CapturedPacket {
        section_index,
        interface_id,
        linktype,
        captured_len,
        original_len,
        data: body[20..packet_end].to_vec(),
    })
}

/// Parses one packet carried by the USBPcap data-link type.
///
/// # Errors
///
/// Returns [`ParseError`] if a USBPcap packet has a truncated or inconsistent header or length.
pub fn parse_usbpcap_packet(packet: &CapturedPacket) -> Result<Option<UsbPcapPacket>, ParseError> {
    if packet.linktype != LINKTYPE_USBPCAP {
        return Ok(None);
    }
    if packet.data.len() < USBPCAP_BASE_HEADER_LEN {
        return Err(ParseError::new(
            0,
            format!(
                "USBPcap packet is shorter than the {USBPCAP_BASE_HEADER_LEN}-byte base header"
            ),
        ));
    }

    let header_len = u16::from_le_bytes([packet.data[0], packet.data[1]]);
    let header_len_usize = usize::from(header_len);
    if header_len_usize < USBPCAP_BASE_HEADER_LEN {
        return Err(ParseError::new(
            0,
            format!("USBPcap header length {header_len} is smaller than the base header"),
        ));
    }
    if header_len_usize > packet.data.len() {
        return Err(ParseError::new(
            0,
            format!("USBPcap header length {header_len} exceeds captured packet length"),
        ));
    }

    let data_length = u32::from_le_bytes([
        packet.data[23],
        packet.data[24],
        packet.data[25],
        packet.data[26],
    ]);
    let declared_payload_len = usize::try_from(data_length)
        .map_err(|_| ParseError::new(23, "USBPcap data length does not fit this platform"))?;
    let available_payload_len = packet.data.len() - header_len_usize;
    let payload_len = declared_payload_len.min(available_payload_len);

    Ok(Some(UsbPcapPacket {
        section_index: packet.section_index,
        interface_id: packet.interface_id,
        header_len,
        irp_id: u64::from_le_bytes([
            packet.data[2],
            packet.data[3],
            packet.data[4],
            packet.data[5],
            packet.data[6],
            packet.data[7],
            packet.data[8],
            packet.data[9],
        ]),
        status: u32::from_le_bytes([
            packet.data[10],
            packet.data[11],
            packet.data[12],
            packet.data[13],
        ]),
        function: u16::from_le_bytes([packet.data[14], packet.data[15]]),
        info: packet.data[16],
        bus: u16::from_le_bytes([packet.data[17], packet.data[18]]),
        device: u16::from_le_bytes([packet.data[19], packet.data[20]]),
        endpoint: packet.data[21],
        transfer: packet.data[22],
        data_length,
        payload: packet.data[header_len_usize..header_len_usize + payload_len].to_vec(),
    }))
}

/// Parses every USBPcap-linked packet in a capture, ignoring other link types.
///
/// # Errors
///
/// Returns [`ParseError`] when any USBPcap packet is structurally invalid.
pub fn usbpcap_packets(capture: &PcapNgCapture) -> Result<Vec<UsbPcapPacket>, ParseError> {
    capture
        .packets
        .iter()
        .map(parse_usbpcap_packet)
        .filter_map(Result::transpose)
        .collect()
}

/// Extracts non-empty OUT payloads sent to the KD3B configuration endpoint number 3.
///
/// # Errors
///
/// Returns [`ParseError`] when a USBPcap packet is structurally invalid.
pub fn kd3b_configuration_out_payloads(
    capture: &PcapNgCapture,
) -> Result<Vec<Vec<u8>>, ParseError> {
    Ok(usbpcap_packets(capture)?
        .into_iter()
        .filter(UsbPcapPacket::is_kd3b_configuration_out)
        .filter(|packet| !packet.payload.is_empty())
        .map(|packet| packet.payload)
        .collect())
}

fn block_length(bytes: &[u8], offset: usize, endianness: Endianness) -> Result<usize, ParseError> {
    let length_u32 = endianness.read_u32(array4(bytes, offset + 4)?);
    usize::try_from(length_u32)
        .map_err(|_| ParseError::new(offset + 4, "block length does not fit this platform"))
}

fn validate_block(
    bytes: &[u8],
    offset: usize,
    total_length: usize,
    endianness: Endianness,
) -> Result<(), ParseError> {
    if total_length < 12 {
        return Err(ParseError::new(
            offset + 4,
            "block length is smaller than 12",
        ));
    }
    if !total_length.is_multiple_of(4) {
        return Err(ParseError::new(
            offset + 4,
            "block length is not aligned to 32 bits",
        ));
    }
    let end = offset
        .checked_add(total_length)
        .ok_or_else(|| ParseError::new(offset + 4, "block length overflows address space"))?;
    if end > bytes.len() {
        return Err(ParseError::new(
            offset + 4,
            "block extends past end of file",
        ));
    }
    let trailing = endianness.read_u32(array4(bytes, end - 4)?);
    let trailing = usize::try_from(trailing)
        .map_err(|_| ParseError::new(end - 4, "trailing block length does not fit platform"))?;
    if trailing != total_length {
        return Err(ParseError::new(
            end - 4,
            "leading and trailing block lengths differ",
        ));
    }
    Ok(())
}

fn array4(bytes: &[u8], offset: usize) -> Result<[u8; 4], ParseError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| ParseError::new(offset, "four-byte read overflows address space"))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| ParseError::new(offset, "expected four bytes"))?;
    Ok([slice[0], slice[1], slice[2], slice[3]])
}

#[cfg(test)]
mod tests {
    use super::{
        KD3B_CONFIGURATION_ENDPOINT, LINKTYPE_USBPCAP, UsbDirection,
        kd3b_configuration_out_payloads, parse_pcapng, parse_usbpcap_packet,
    };

    #[test]
    fn parses_usbpcap_enhanced_packet_and_extracts_endpoint_three_out_payload() {
        let payload = [0x08, 0x07, 0x00, 0x00, 0x00, 0xAA, 0xBB, 0xCC];
        let capture_bytes = fixture_capture(0x03, 1, &payload);
        let capture = parse_pcapng(&capture_bytes).expect("fixture parses");

        assert_eq!(capture.section_count, 1);
        assert_eq!(capture.interface_count, 1);
        assert_eq!(capture.packets.len(), 1);
        assert_eq!(capture.packets[0].linktype, LINKTYPE_USBPCAP);

        let usb = parse_usbpcap_packet(&capture.packets[0])
            .expect("USBPcap header parses")
            .expect("packet uses USBPcap linktype");
        assert_eq!(usb.direction(), UsbDirection::Out);
        assert_eq!(usb.endpoint_number(), KD3B_CONFIGURATION_ENDPOINT);
        assert_eq!(usb.transfer, 1);
        assert_eq!(usb.payload, payload);

        assert_eq!(
            kd3b_configuration_out_payloads(&capture).expect("payload extraction succeeds"),
            vec![payload.to_vec()]
        );
    }

    #[test]
    fn ignores_inbound_endpoint_three_for_configuration_output_extraction() {
        let capture = parse_pcapng(&fixture_capture(0x83, 1, &[1, 2, 3])).expect("fixture parses");

        assert!(
            kd3b_configuration_out_payloads(&capture)
                .expect("payload extraction succeeds")
                .is_empty()
        );
    }

    #[test]
    fn rejects_truncated_enhanced_packet_payload() {
        let mut bytes = fixture_capture(0x03, 1, &[1, 2, 3, 4]);
        let epb_start = 28 + 20;
        let captured_len_offset = epb_start + 20;
        bytes[captured_len_offset..captured_len_offset + 4].copy_from_slice(&64u32.to_le_bytes());

        let error = parse_pcapng(&bytes).expect_err("truncated packet must fail");
        assert!(error.to_string().contains("truncated"));
    }

    #[test]
    fn accepts_big_endian_section_and_interface_description() {
        let mut bytes = Vec::new();
        push_section_header_be(&mut bytes);
        push_interface_description_be(&mut bytes, LINKTYPE_USBPCAP);

        let capture = parse_pcapng(&bytes).expect("big-endian metadata parses");
        assert_eq!(capture.section_count, 1);
        assert_eq!(capture.interface_count, 1);
        assert!(capture.packets.is_empty());
    }

    fn fixture_capture(endpoint: u8, transfer: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_section_header_le(&mut bytes);
        push_interface_description_le(&mut bytes, LINKTYPE_USBPCAP);

        let usb = usbpcap_packet(endpoint, transfer, payload);
        let mut body = Vec::new();
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        let usb_len = u32::try_from(usb.len()).expect("fixture length fits u32");
        body.extend_from_slice(&usb_len.to_le_bytes());
        body.extend_from_slice(&usb_len.to_le_bytes());
        body.extend_from_slice(&usb);
        pad4(&mut body);
        push_block_le(&mut bytes, 6, &body);
        bytes
    }

    fn usbpcap_packet(endpoint: u8, transfer: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&27u16.to_le_bytes());
        packet.extend_from_slice(&0x1122_3344_5566_7788u64.to_le_bytes());
        packet.extend_from_slice(&0u32.to_le_bytes());
        packet.extend_from_slice(&9u16.to_le_bytes());
        packet.push(0);
        packet.extend_from_slice(&1u16.to_le_bytes());
        packet.extend_from_slice(&2u16.to_le_bytes());
        packet.push(endpoint);
        packet.push(transfer);
        let payload_len = u32::try_from(payload.len()).expect("fixture length fits u32");
        packet.extend_from_slice(&payload_len.to_le_bytes());
        packet.extend_from_slice(payload);
        packet
    }

    fn push_section_header_le(bytes: &mut Vec<u8>) {
        let mut body = Vec::new();
        body.extend_from_slice(&0x1A2B_3C4Du32.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&u64::MAX.to_le_bytes());
        push_block_le(bytes, 0x0A0D_0D0A, &body);
    }

    fn push_interface_description_le(bytes: &mut Vec<u8>, linktype: u16) {
        let mut body = Vec::new();
        body.extend_from_slice(&linktype.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&65_535u32.to_le_bytes());
        push_block_le(bytes, 1, &body);
    }

    fn push_section_header_be(bytes: &mut Vec<u8>) {
        let mut body = Vec::new();
        body.extend_from_slice(&0x1A2B_3C4Du32.to_be_bytes());
        body.extend_from_slice(&1u16.to_be_bytes());
        body.extend_from_slice(&0u16.to_be_bytes());
        body.extend_from_slice(&u64::MAX.to_be_bytes());
        push_block_be(bytes, 0x0A0D_0D0A, &body);
    }

    fn push_interface_description_be(bytes: &mut Vec<u8>, linktype: u16) {
        let mut body = Vec::new();
        body.extend_from_slice(&linktype.to_be_bytes());
        body.extend_from_slice(&0u16.to_be_bytes());
        body.extend_from_slice(&65_535u32.to_be_bytes());
        push_block_be(bytes, 1, &body);
    }

    fn push_block_le(bytes: &mut Vec<u8>, block_type: u32, body: &[u8]) {
        let length = u32::try_from(body.len() + 12).expect("fixture block length fits u32");
        bytes.extend_from_slice(&block_type.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(body);
        bytes.extend_from_slice(&length.to_le_bytes());
    }

    fn push_block_be(bytes: &mut Vec<u8>, block_type: u32, body: &[u8]) {
        let length = u32::try_from(body.len() + 12).expect("fixture block length fits u32");
        bytes.extend_from_slice(&block_type.to_be_bytes());
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(body);
        bytes.extend_from_slice(&length.to_be_bytes());
    }

    fn pad4(bytes: &mut Vec<u8>) {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
    }
}
