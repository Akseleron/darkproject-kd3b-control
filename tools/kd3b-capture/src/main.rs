use std::{env, error::Error, fs, path::Path};

use kd3b_capture::{
    PcapNgCapture, UsbPcapPacket, kd3b_configuration_out_payloads, parse_pcapng,
    usbpcap_packets,
};

const USAGE: &str = "Usage:\n  kd3b-capture list <capture.pcapng>\n  kd3b-capture extract <capture.pcapng>\n  kd3b-capture diff <left.pcapng> <right.pcapng>\n";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let Some(command) = arguments.next() else {
        return Err(USAGE.into());
    };
    let command = command.to_string_lossy();

    match command.as_ref() {
        "list" => {
            let path = required_path(arguments.next())?;
            reject_extra(arguments.next())?;
            list_capture(&path)?;
        }
        "extract" => {
            let path = required_path(arguments.next())?;
            reject_extra(arguments.next())?;
            extract_capture(&path)?;
        }
        "diff" => {
            let left = required_path(arguments.next())?;
            let right = required_path(arguments.next())?;
            reject_extra(arguments.next())?;
            diff_captures(&left, &right)?;
        }
        "help" | "-h" | "--help" => {
            reject_extra(arguments.next())?;
            print!("{USAGE}");
        }
        _ => return Err(format!("unknown command '{command}'\n{USAGE}").into()),
    }

    Ok(())
}

fn required_path(value: Option<std::ffi::OsString>) -> Result<std::path::PathBuf, Box<dyn Error>> {
    value
        .map(std::path::PathBuf::from)
        .ok_or_else(|| format!("missing capture path\n{USAGE}").into())
}

fn reject_extra(value: Option<std::ffi::OsString>) -> Result<(), Box<dyn Error>> {
    if let Some(value) = value {
        return Err(format!("unexpected argument '{}'\n{USAGE}", value.to_string_lossy()).into());
    }
    Ok(())
}

fn read_capture(path: &Path) -> Result<PcapNgCapture, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    Ok(parse_pcapng(&bytes)?)
}

fn list_capture(path: &Path) -> Result<(), Box<dyn Error>> {
    let capture = read_capture(path)?;
    let usb_packets = usbpcap_packets(&capture)?;

    println!("file: {}", path.display());
    println!("sections: {}", capture.section_count);
    println!("interfaces: {}", capture.interface_count);
    println!("captured packets: {}", capture.packets.len());
    println!("USBPcap packets: {}", usb_packets.len());

    for (index, packet) in usb_packets.iter().enumerate() {
        println!("{}", render_usb_packet(index, packet));
    }

    Ok(())
}

fn extract_capture(path: &Path) -> Result<(), Box<dyn Error>> {
    let capture = read_capture(path)?;
    let payloads = kd3b_configuration_out_payloads(&capture)?;

    println!("file: {}", path.display());
    println!("endpoint-3 OUT payloads: {}", payloads.len());
    for (index, payload) in payloads.iter().enumerate() {
        println!("#{index:04} len={} {}", payload.len(), hex(payload));
    }

    Ok(())
}

fn diff_captures(left_path: &Path, right_path: &Path) -> Result<(), Box<dyn Error>> {
    let left = kd3b_configuration_out_payloads(&read_capture(left_path)?)?;
    let right = kd3b_configuration_out_payloads(&read_capture(right_path)?)?;

    println!("left: {} ({} payloads)", left_path.display(), left.len());
    println!("right: {} ({} payloads)", right_path.display(), right.len());

    let pair_count = left.len().max(right.len());
    for index in 0..pair_count {
        match (left.get(index), right.get(index)) {
            (Some(left_payload), Some(right_payload)) => {
                let changes = byte_changes(left_payload, right_payload);
                println!(
                    "payload #{index:04}: left_len={} right_len={} changed_bytes={}",
                    left_payload.len(),
                    right_payload.len(),
                    changes.len()
                );
                for change in changes {
                    println!("  {change}");
                }
            }
            (Some(left_payload), None) => println!(
                "payload #{index:04}: only left, len={} {}",
                left_payload.len(),
                hex(left_payload)
            ),
            (None, Some(right_payload)) => println!(
                "payload #{index:04}: only right, len={} {}",
                right_payload.len(),
                hex(right_payload)
            ),
            (None, None) => unreachable!("index is bounded by maximum payload count"),
        }
    }

    Ok(())
}

fn render_usb_packet(index: usize, packet: &UsbPcapPacket) -> String {
    format!(
        "#{index:04} section={} if={} bus={} dev={} ep=0x{:02x} {} transfer={} declared={} captured={} {}",
        packet.section_index,
        packet.interface_id,
        packet.bus,
        packet.device,
        packet.endpoint,
        packet.direction(),
        transfer_name(packet.transfer),
        packet.data_length,
        packet.payload.len(),
        hex(&packet.payload)
    )
}

fn transfer_name(transfer: u8) -> &'static str {
    match transfer {
        0 => "isochronous",
        1 => "interrupt",
        2 => "control",
        3 => "bulk",
        0xFE => "irp-info",
        _ => "unknown",
    }
}

fn byte_changes(left: &[u8], right: &[u8]) -> Vec<String> {
    let length = left.len().max(right.len());
    let mut changes = Vec::new();
    for offset in 0..length {
        let left_byte = left.get(offset).copied();
        let right_byte = right.get(offset).copied();
        if left_byte != right_byte {
            changes.push(format!(
                "offset {offset:04}: {} -> {}",
                optional_hex(left_byte),
                optional_hex(right_byte)
            ));
        }
    }
    changes
}

fn optional_hex(value: Option<u8>) -> String {
    value.map_or_else(|| "--".to_owned(), |byte| format!("{byte:02x}"))
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::byte_changes;

    #[test]
    fn byte_diff_reports_changed_and_missing_offsets() {
        let changes = byte_changes(&[0x08, 0x07, 0x00], &[0x08, 0x09]);
        assert_eq!(
            changes,
            vec![
                "offset 0001: 07 -> 09".to_owned(),
                "offset 0002: 00 -> --".to_owned(),
            ]
        );
    }
}
