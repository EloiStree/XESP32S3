pub const SCREEN_FRAME_BYTES: usize = 128 * 64 / 8;

/// Identifies a complete SSD1306 128x64 framebuffer.
pub fn udp_screen_packet(packet: &[u8]) -> Option<&[u8; SCREEN_FRAME_BYTES]> {
    packet.try_into().ok()
}

/// Decodes the SSD1306's horizontal page order, with the top pixel in bit 0.
pub fn screen_pixels(frame: &[u8; SCREEN_FRAME_BYTES]) -> impl Iterator<Item = (u32, u32, bool)> + '_ {
    frame.iter().enumerate().flat_map(|(index, &byte)| {
        (0..8).map(move |bit| {
            ((index % 128) as u32, ((index / 128) * 8 + bit) as u32, byte & (1 << bit) != 0)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_and_decodes_screen_packets() {
        assert!(udp_screen_packet(&[0; 1023]).is_none());
        assert!(udp_screen_packet(&[0; 1025]).is_none());
        let mut frame = [0; SCREEN_FRAME_BYTES];
        frame[0] = 0b1000_0001;
        frame[128] = 0b0000_0010;
        frame[1023] = 0b1000_0000;
        let frame = udp_screen_packet(&frame).unwrap();
        assert_eq!(screen_pixels(frame).count(), 128 * 64);
        assert_eq!(screen_pixels(frame).nth(0), Some((0, 0, true)));
        assert_eq!(screen_pixels(frame).nth(1), Some((0, 1, false)));
        assert_eq!(screen_pixels(frame).nth(7), Some((0, 7, true)));
        assert_eq!(screen_pixels(frame).nth(128 * 8 + 1), Some((0, 9, true)));
        assert_eq!(screen_pixels(frame).nth(1023 * 8 + 7), Some((127, 63, true)));
    }
}
