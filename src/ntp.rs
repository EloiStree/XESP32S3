const NTP_EPOCH_OFFSET: u64 = 2_208_988_800;
const NTP_ERA_SECONDS: u64 = 1 << 32;

pub const NTP_PACKET_BYTES: usize = 48;

pub fn request(nonce: [u8; 8]) -> [u8; NTP_PACKET_BYTES] {
    let mut packet = [0; NTP_PACKET_BYTES];
    packet[0] = 0x23;
    packet[40..48].copy_from_slice(&nonce);
    packet
}

/// Returns UTC seconds since midnight for a matching, synchronized NTP server reply.
pub fn response_seconds_of_day(packet: &[u8], nonce: [u8; 8]) -> Option<u32> {
    if packet.len() < NTP_PACKET_BYTES
        || packet[0] >> 6 == 3
        || !matches!((packet[0] >> 3) & 7, 3 | 4)
        || packet[0] & 7 != 4
        || !(1..=15).contains(&packet[1])
        || packet[24..32] != nonce
        || packet[40..48] == [0; 8]
    {
        return None;
    }

    let seconds = u32::from_be_bytes(packet[40..44].try_into().ok()?) as u64;
    // Disambiguate NTP's 2036 rollover for dates from 1970 through 2106.
    let ntp_seconds = if seconds < NTP_EPOCH_OFFSET {
        seconds + NTP_ERA_SECONDS
    } else {
        seconds
    };
    Some(((ntp_seconds - NTP_EPOCH_OFFSET) % 86_400) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(nonce: [u8; 8], timestamp: u32) -> [u8; NTP_PACKET_BYTES] {
        let mut packet = [0; NTP_PACKET_BYTES];
        packet[0] = 0x24;
        packet[1] = 1;
        packet[24..32].copy_from_slice(&nonce);
        packet[40..44].copy_from_slice(&timestamp.to_be_bytes());
        packet
    }

    #[test]
    fn parses_utc_before_and_after_ntp_era_rollover() {
        let nonce = [1; 8];
        assert_eq!(response_seconds_of_day(&response(nonce, 2_208_988_800 + 45_296), nonce), Some(45_296));
        assert_eq!(response_seconds_of_day(&response(nonce, u32::MAX), nonce), Some(23_295));
        assert_eq!(response_seconds_of_day(&response(nonce, 1), nonce), Some(23_297));
        let mut rollover = response(nonce, 0);
        rollover[47] = 1;
        assert_eq!(response_seconds_of_day(&rollover, nonce), Some(23_296));
    }

    #[test]
    fn rejects_invalid_or_unrelated_responses() {
        let nonce = [1; 8];
        let packet = response(nonce, 2_208_988_801);
        assert_eq!(response_seconds_of_day(&packet[..47], nonce), None);
        assert_eq!(response_seconds_of_day(&packet, [2; 8]), None);
        let mut invalid = packet;
        invalid[0] = 0xe4;
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
        invalid = packet;
        invalid[1] = 0;
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
        invalid = packet;
        invalid[0] = 0x23;
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
        invalid = packet;
        invalid[0] = 0x14;
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
        invalid = packet;
        invalid[1] = 16;
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
        invalid = packet;
        invalid[40..48].fill(0);
        assert_eq!(response_seconds_of_day(&invalid, nonce), None);
    }
}
