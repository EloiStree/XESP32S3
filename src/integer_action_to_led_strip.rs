use smart_leds_trait::RGB8;

use crate::config::STRIP_LED_COUNT;

pub const LED_COMMAND_START: u32 = 200_000_000;
pub const LED_COMMAND_END: u32 = 299_999_999;

pub fn is_led_command(value: u32) -> bool {
    (LED_COMMAND_START..=LED_COMMAND_END).contains(&value)
}

// At 2.5 MHz, three SPI bits encode one WS2812B bit: 100 for zero, 110 for one.
pub const STRIP_RESET_BYTES: usize = 100;
pub const STRIP_FRAME_BYTES: usize = STRIP_RESET_BYTES * 2 + STRIP_LED_COUNT * 9;

pub fn encode_strip_frame(colors: &[RGB8; STRIP_LED_COUNT], frame: &mut [u8; STRIP_FRAME_BYTES]) {
    frame.fill(0);
    for (color, encoded) in colors.iter().zip(
        frame[STRIP_RESET_BYTES..STRIP_RESET_BYTES + STRIP_LED_COUNT * 9].chunks_exact_mut(9),
    ) {
        for (byte, encoded_byte) in [color.g, color.r, color.b]
            .into_iter()
            .zip(encoded.chunks_exact_mut(3))
        {
            let mut bits = 0u32;
            for shift in (0..8).rev() {
                bits = (bits << 3) | if byte & (1 << shift) == 0 { 0b100 } else { 0b110 };
            }
            encoded_byte.copy_from_slice(&bits.to_be_bytes()[1..]);
        }
    }
}

/// Holds the strip state and applies 02AARRGGBB integer commands.
pub struct IntegerActionToLedStrip {
    state: [RGB8; STRIP_LED_COUNT],
}

impl IntegerActionToLedStrip {
    pub fn new(initial_color: RGB8) -> Self {
        Self {
            state: [initial_color; STRIP_LED_COUNT],
        }
    }

    /// Returns whether the state changed, or the LED index if it is out of bounds.
    pub fn handle_integer(&mut self, value: u32) -> Result<bool, usize> {
        if !is_led_command(value) {
            return Ok(false);
        }

        let payload = value - LED_COMMAND_START;
        let led_index = (payload / 1_000_000) as usize;
        let color = RGB8::new(
            ((payload / 10_000) % 100) as u8,
            ((payload / 100) % 100) as u8,
            (payload % 100) as u8,
        );
        let Some(current) = self.state.get_mut(led_index) else {
            return Err(led_index);
        };
        if *current == color {
            return Ok(false);
        }
        *current = color;
        Ok(true)
    }

    pub fn state(&self) -> &[RGB8; STRIP_LED_COUNT] {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_only_the_addressed_led_and_detects_unchanged_colors() {
        let initial = RGB8::new(0, 0, 255);
        let mut strip = IntegerActionToLedStrip::new(initial);

        assert_eq!(strip.handle_integer(201_990_099), Ok(true));
        assert_eq!(strip.state()[1], RGB8::new(99, 0, 99));
        assert_eq!(strip.state()[0], initial);
        assert_eq!(strip.handle_integer(201_990_099), Ok(false));
    }

    #[test]
    fn accepts_range_boundaries_and_rejects_other_values() {
        let mut strip = IntegerActionToLedStrip::new(RGB8::new(1, 2, 3));

        assert_eq!(strip.handle_integer(199_999_999), Ok(false));
        assert_eq!(strip.handle_integer(300_000_000), Ok(false));
        assert_eq!(strip.state()[0], RGB8::new(1, 2, 3));
        assert_eq!(strip.handle_integer(200_000_000), Ok(true));
        assert_eq!(strip.state()[0], RGB8::new(0, 0, 0));
        if STRIP_LED_COUNT > 99 {
            assert_eq!(strip.handle_integer(299_999_999), Ok(true));
            assert_eq!(strip.state().get(99), Some(&RGB8::new(99, 99, 99)));
        } else {
            assert_eq!(strip.handle_integer(299_999_999), Err(99));
        }
    }

    #[test]
    fn sets_every_led_independently() {
        let mut strip = IntegerActionToLedStrip::new(RGB8::new(0, 0, 0));
        for index in 0..STRIP_LED_COUNT {
            let green = (index as u32 + 1).min(99);
            let command = LED_COMMAND_START + index as u32 * 1_000_000 + green * 100;
            assert_eq!(strip.handle_integer(command), Ok(true));
        }
        for index in 0..STRIP_LED_COUNT {
            let green = (index as u8 + 1).min(99);
            assert_eq!(strip.state()[index], RGB8::new(0, green, 0));
        }
    }

    #[test]
    fn rejects_led_indices_outside_the_configured_strip() {
        if STRIP_LED_COUNT < 100 {
            let mut strip = IntegerActionToLedStrip::new(RGB8::new(1, 2, 3));
            let command = LED_COMMAND_START + (STRIP_LED_COUNT as u32) * 1_000_000;
            assert_eq!(strip.handle_integer(command), Err(STRIP_LED_COUNT));
        }
    }

    #[test]
    fn encodes_all_leds_in_grb_order_with_reset_before_and_after() {
        if STRIP_LED_COUNT < 100 {
            return;
        }
        let mut strip = IntegerActionToLedStrip::new(RGB8::new(0, 0, 0));
        assert_eq!(strip.handle_integer(214_009_900), Ok(true));
        assert_eq!(strip.handle_integer(299_990_001), Ok(true));
        let mut frame = [0xff; STRIP_FRAME_BYTES];
        encode_strip_frame(strip.state(), &mut frame);

        assert!(frame[..STRIP_RESET_BYTES].iter().all(|&b| b == 0));
        assert!(frame[STRIP_FRAME_BYTES - STRIP_RESET_BYTES..].iter().all(|&b| b == 0));
        let off = [0x92, 0x49, 0x24];
        let green_99 = [0x9b, 0x49, 0x36];
        let one = [0x92, 0x49, 0x26];
        for index in 0..STRIP_LED_COUNT {
            let start = STRIP_RESET_BYTES + index * 9;
            let expected = match index {
                14 => [green_99, off, off],
                99 => [off, green_99, one],
                _ => [off, off, off],
            };
            for (channel, bytes) in expected.iter().enumerate() {
                assert_eq!(&frame[start + channel * 3..start + channel * 3 + 3], bytes);
            }
        }
    }
}
