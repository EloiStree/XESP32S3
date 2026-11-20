use smart_leds_trait::RGB8;

use crate::config::STRIP_LED_COUNT;

pub const LED_COMMAND_START: u32 = 200_000_000;
pub const LED_COMMAND_END: u32 = 299_999_999;

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
        if !(LED_COMMAND_START..=LED_COMMAND_END).contains(&value) {
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
    fn rejects_led_indices_outside_the_configured_strip() {
        if STRIP_LED_COUNT < 100 {
            let mut strip = IntegerActionToLedStrip::new(RGB8::new(1, 2, 3));
            let command = LED_COMMAND_START + (STRIP_LED_COUNT as u32) * 1_000_000;
            assert_eq!(strip.handle_integer(command), Err(STRIP_LED_COUNT));
        }
    }
}
