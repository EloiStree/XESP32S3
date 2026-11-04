//! Minimal mDNS responder helpers: answers A queries for a single `<host>.local` name.

pub const MDNS_PORT: u16 = 5353;
pub const MDNS_GROUP: [u8; 4] = [224, 0, 0, 251];

const TYPE_A: u16 = 1;
const TYPE_ANY: u16 = 255;

/// Returns true when `packet` is a query with a question asking for the A record of `<host>.local`.
pub fn is_query_for(packet: &[u8], host: &str) -> bool {
    if packet.len() < 12 || packet[2] & 0x80 != 0 {
        return false;
    }

    let questions = u16::from_be_bytes([packet[4], packet[5]]);
    let mut pos = 12;

    for _ in 0..questions {
        let mut matches = true;
        let mut label_index = 0;

        loop {
            let Some(&len) = packet.get(pos) else {
                return false;
            };
            pos += 1;

            if len == 0 {
                break;
            }
            // Compression pointers are not expected in questions.
            if len & 0xC0 != 0 {
                return false;
            }

            let Some(label) = packet.get(pos..pos + len as usize) else {
                return false;
            };
            pos += len as usize;

            let expected: &[u8] = match label_index {
                0 => host.as_bytes(),
                1 => b"local",
                _ => &[],
            };
            if !label.eq_ignore_ascii_case(expected) {
                matches = false;
            }
            label_index += 1;
        }

        let Some(tail) = packet.get(pos..pos + 4) else {
            return false;
        };
        pos += 4;

        let qtype = u16::from_be_bytes([tail[0], tail[1]]);
        if matches && label_index == 2 && (qtype == TYPE_A || qtype == TYPE_ANY) {
            return true;
        }
    }

    false
}

/// Builds an authoritative answer for `<host>.local` -> `ip`. Returns the number of bytes written.
pub fn build_answer(buf: &mut [u8], host: &str, ip: [u8; 4]) -> Option<usize> {
    let needed = 12 + 1 + host.len() + 1 + 5 + 1 + 10 + 4;
    if buf.len() < needed || host.len() > 63 {
        return None;
    }

    buf[..12].copy_from_slice(&[0, 0, 0x84, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
    let mut pos = 12;

    buf[pos] = host.len() as u8;
    pos += 1;
    buf[pos..pos + host.len()].copy_from_slice(host.as_bytes());
    pos += host.len();

    buf[pos] = 5;
    buf[pos + 1..pos + 6].copy_from_slice(b"local");
    buf[pos + 6] = 0;
    pos += 7;

    buf[pos..pos + 2].copy_from_slice(&TYPE_A.to_be_bytes());
    buf[pos + 2..pos + 4].copy_from_slice(&0x8001u16.to_be_bytes()); // IN + cache-flush
    buf[pos + 4..pos + 8].copy_from_slice(&120u32.to_be_bytes());
    buf[pos + 8..pos + 10].copy_from_slice(&4u16.to_be_bytes());
    buf[pos + 10..pos + 14].copy_from_slice(&ip);

    Some(pos + 14)
}
