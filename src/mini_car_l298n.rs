//! Mini Car L298N Motor Driver Controller
//!
//! Handles UDP commands to control a mini car via L298N motor driver.
//! 
//! ## GPIO Pin Configuration
//! - **Pins 17-18**: Analog PWM speed control via ASCII characters '0'-'9'
//! - **Pins 10-13**: Boolean direction control
//!
//! ## UDP Message Format
//! Message format: "CAR:AADDDD" (10 bytes, case-insensitive header)
//! - `CAR`: Message identifier (case-insensitive: CAR:, car:, Car:, etc.)
//! - `AA`: Two ASCII characters ('0'-'9') for pins 17-18 analog control
//! - `DDDD`: 4 binary digits for pins 10-13 direction control
//! - Total length: 10 bytes exactly
//!
//! Example: "car:561010" maps:
//!   - Pin 17: '5' (ASCII character)
//!   - Pin 18: '6' (ASCII character)
//!   - Pin 10: '1'
//!   - Pin 11: '0'
//!   - Pin 12: '1'
//!   - Pin 13: '0'

/// Message header that all mini car commands must start with
const CAR_HEADER: &[u8] = b"CAR:";
const HEADER_LEN: usize = 4;

/// Valid message length: exactly 10 bytes (including "CAR:" prefix)
const VALID_MSG_LEN: usize = 10;

/// Analog control characters length (pins 17-18)
const ANALOG_CHARS_LEN: usize = 2;

/// Direction bits for pins 10-13 (following the analog values)
const DIRECTION_BITS_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiniCarCommand {
    /// Pin 17 analog control as ASCII character ('0'-'9')
    pub pin_17_analog: char,
    /// Pin 18 analog control as ASCII character ('0'-'9')
    pub pin_18_analog: char,
    /// Direction bit 0 for pin 10
    pub pin_10: bool,
    /// Direction bit 1 for pin 11
    pub pin_11: bool,
    /// Direction bit 2 for pin 12
    pub pin_12: bool,
    /// Direction bit 3 for pin 13
    pub pin_13: bool,
}

impl MiniCarCommand {
    /// Creates a new mini car command with the given parameters
    pub const fn new(
        pin_17_analog: char,
        pin_18_analog: char,
        pin_10: bool,
        pin_11: bool,
        pin_12: bool,
        pin_13: bool,
    ) -> Self {
        Self {
            pin_17_analog,
            pin_18_analog,
            pin_10,
            pin_11,
            pin_12,
            pin_13,
        }
    }

    /// Checks if header matches "CAR:" (case-insensitive)
    fn is_valid_header(header: &[u8]) -> bool {
        if header.len() != HEADER_LEN {
            return false;
        }
        header.eq_ignore_ascii_case(CAR_HEADER)
    }

    /// Attempts to parse a mini car command from a UDP packet
    ///
    /// Returns `Some(MiniCarCommand)` if the packet is valid, `None` otherwise.
    ///
    /// Valid format: "CAR:AADDDD" (case-insensitive header)
    /// - Length must be exactly 10 bytes
    /// - Must start with "CAR:" (any case)
    /// - AA are analog control characters for pins 17-18 (each must be '0'-'9')
    /// - DDDD are direction bits for pins 10-13 (each must be '0' or '1')
    pub fn try_from_udp(packet: &[u8]) -> Option<Self> {
        // Check exact length
        if packet.len() != VALID_MSG_LEN {
            return None;
        }

        // Check header (case-insensitive)
        if !Self::is_valid_header(&packet[..HEADER_LEN]) {
            return None;
        }

        // Extract payload after header
        let payload = &packet[HEADER_LEN..];

        // Payload should be 6 bytes: AA + DDDD
        if payload.len() != ANALOG_CHARS_LEN + DIRECTION_BITS_LEN {
            return None;
        }

        // Parse analog characters for pins 17-18
        let pin_17_analog = payload[0] as char;
        let pin_18_analog = payload[1] as char;

        // Validate that analog characters are '0'-'9'
        if !pin_17_analog.is_ascii_digit() || !pin_18_analog.is_ascii_digit() {
            return None;
        }

        // Parse direction bits (4 bits for pins 10-13)
        let direction_bits = &payload[ANALOG_CHARS_LEN..];

        if direction_bits.len() != DIRECTION_BITS_LEN {
            return None;
        }

        let pin_10 = direction_bits[0] == b'1';
        let pin_11 = direction_bits[1] == b'1';
        let pin_12 = direction_bits[2] == b'1';
        let pin_13 = direction_bits[3] == b'1';

        // Validate that all direction bits are '0' or '1'
        if !direction_bits.iter().all(|&b| b == b'0' || b == b'1') {
            return None;
        }

        Some(Self {
            pin_17_analog,
            pin_18_analog,
            pin_10,
            pin_11,
            pin_12,
            pin_13,
        })
    }

    /// Get the analog value for pin 17 as a number (0-9)
    pub fn pin_17_value(&self) -> u8 {
        self.pin_17_analog.to_digit(10).unwrap_or(0) as u8
    }

    /// Get the analog value for pin 18 as a number (0-9)
    pub fn pin_18_value(&self) -> u8 {
        self.pin_18_analog.to_digit(10).unwrap_or(0) as u8
    }
}

impl core::fmt::Display for MiniCarCommand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MiniCar{{pin17:{}, pin18:{}, pins:{}{}{}{}}}",
            self.pin_17_analog,
            self.pin_18_analog,
            if self.pin_10 { '1' } else { '0' },
            if self.pin_11 { '1' } else { '0' },
            if self.pin_12 { '1' } else { '0' },
            if self.pin_13 { '1' } else { '0' },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_10_byte_command() {
        let packet = b"CAR:561010";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '5');
        assert_eq!(cmd.pin_18_analog, '6');
        assert_eq!(cmd.pin_17_value(), 5);
        assert_eq!(cmd.pin_18_value(), 6);
        assert!(cmd.pin_10);
        assert!(!cmd.pin_11);
        assert!(cmd.pin_12);
        assert!(!cmd.pin_13);
    }

    #[test]
    fn parses_command_with_zero_values() {
        let packet = b"CAR:000101";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '0');
        assert_eq!(cmd.pin_18_analog, '0');
        assert_eq!(cmd.pin_17_value(), 0);
        assert_eq!(cmd.pin_18_value(), 0);
        assert!(!cmd.pin_10);
        assert!(cmd.pin_11);
        assert!(!cmd.pin_12);
        assert!(cmd.pin_13);
    }

    #[test]
    fn parses_command_with_max_analog() {
        let packet = b"CAR:991111";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '9');
        assert_eq!(cmd.pin_18_analog, '9');
        assert_eq!(cmd.pin_17_value(), 9);
        assert_eq!(cmd.pin_18_value(), 9);
        assert!(cmd.pin_10);
        assert!(cmd.pin_11);
        assert!(cmd.pin_12);
        assert!(cmd.pin_13);
    }

    #[test]
    fn rejects_invalid_header() {
        let packet = b"CAX:561010";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn accepts_lowercase_header() {
        let packet = b"car:561010";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '5');
        assert_eq!(cmd.pin_18_analog, '6');
        assert!(cmd.pin_10);
    }

    #[test]
    fn accepts_mixed_case_header() {
        let packet = b"CaR:561010";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '5');
        assert_eq!(cmd.pin_18_analog, '6');
        assert!(cmd.pin_10);
    }

    #[test]
    fn accepts_uppercase_header() {
        let packet = b"CAR:561010";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_analog, '5');
        assert_eq!(cmd.pin_18_analog, '6');
        assert!(cmd.pin_10);
    }

    #[test]
    fn rejects_too_short() {
        let packet = b"CAR:56101";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn rejects_too_long() {
        let packet = b"CAR:5610101";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn rejects_invalid_direction_bits() {
        let packet = b"CAR:5610X1";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn rejects_non_digit_analog_pin_17() {
        let packet = b"CAR:A61010";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn rejects_non_digit_analog_pin_18() {
        let packet = b"CAR:5A1010";
        assert!(MiniCarCommand::try_from_udp(packet).is_none());
    }

    #[test]
    fn all_pins_low_all_directions_off() {
        let packet = b"CAR:000000";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_value(), 0);
        assert_eq!(cmd.pin_18_value(), 0);
        assert!(!cmd.pin_10);
        assert!(!cmd.pin_11);
        assert!(!cmd.pin_12);
        assert!(!cmd.pin_13);
    }

    #[test]
    fn all_pins_high_all_directions_on() {
        let packet = b"CAR:991111";
        let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
        assert_eq!(cmd.pin_17_value(), 9);
        assert_eq!(cmd.pin_18_value(), 9);
        assert!(cmd.pin_10);
        assert!(cmd.pin_11);
        assert!(cmd.pin_12);
        assert!(cmd.pin_13);
    }

    #[test]
    fn display_format() {
        let cmd = MiniCarCommand::new('5', '6', true, false, true, false);
        let display_str = format!("{}", cmd);
        assert_eq!(display_str, "MiniCar{pin17:5, pin18:6, pins:1010}");
    }
}
