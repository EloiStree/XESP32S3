use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use crate::commands::execute;
use crate::compact_xbox_int_parser;
use crate::controller::{ControllerState, Direction};
use crate::integer_action_to_led_strip::{is_led_command, RGB_PACKET_BYTES};
use crate::screen::SCREEN_FRAME_BYTES;

pub static CONTROLLER: Mutex<CriticalSectionRawMutex, ControllerState> =
    Mutex::new(ControllerState {
        buttons: 0,
        dpad: Direction::Neutral,

        left_x: 0.0,
        left_y: 0.0,

        right_x: 0.0,
        right_y: 0.0,

        left_trigger: 0.0,
        right_trigger: 0.0,

        random_axis: false,
        recording: false,
    });

/// Applies every command code found in a UDP payload (e.g. `"1300 2300"`). Returns how many were valid.
pub fn process_udp(state: &mut ControllerState, packet: &[u8]) -> usize {
    udp_integers(packet)
        .filter(|&code| {
            u16::try_from(code)
                .map(|command| execute(state, command))
                .unwrap_or_else(|_| compact_xbox_int_parser::execute(state, code))
        })
        .count()
}

/// Parses the integer tokens shared by controller and LED UDP commands.
pub fn udp_integers(packet: &[u8]) -> impl Iterator<Item = u32> + '_ {
    core::str::from_utf8(packet)
        .ok()
        .into_iter()
        .flat_map(|text| text.split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';'))
        .filter_map(|token| token.parse::<u32>().ok())
}

/// Returns LED commands in the order they appear in a UDP payload.
pub fn udp_led_commands(packet: &[u8]) -> impl Iterator<Item = u32> + '_ {
    udp_integers(packet).filter(|&value| is_led_command(value))
}

/// Identifies a complete 100-LED RGB packet.
pub fn udp_rgb_packet(packet: &[u8]) -> Option<&[u8; RGB_PACKET_BYTES]> {
    packet.try_into().ok()
}

/// Whether a UDP datagram should be forwarded to the UART outputs.
pub fn should_forward_udp(packet: &[u8]) -> bool {
    packet.len() != RGB_PACKET_BYTES && packet.len() != SCREEN_FRAME_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::button;
    use crate::integer_action_to_led_strip::IntegerActionToLedStrip;
    use smart_leds_trait::RGB8;

    #[test]
    fn parses_mixed_udp_integer_commands() {
        let packet = b"1300,200990099; 201990099\ninvalid 1800000000";
        assert!(
            udp_integers(packet).eq([1300, 200_990_099, 201_990_099, 1_800_000_000])
        );
        assert_eq!(udp_integers(b"\xff 200990099").count(), 0);
    }

    #[test]
    fn identifies_only_exactly_300_byte_rgb_packets() {
        assert!(udp_rgb_packet(&[0; 299]).is_none());
        assert_eq!(udp_rgb_packet(&[255; 300]), Some(&[255; 300]));
        assert!(udp_rgb_packet(&[0; 301]).is_none());
    }

    #[test]
    fn excludes_rgb_and_screen_packets_from_uart_forwarding() {
        assert!(!should_forward_udp(&[0; 300]));
        assert!(!should_forward_udp(&[0; 1024]));
        assert!(should_forward_udp(&[0; 299]));
        assert!(should_forward_udp(&[0; 301]));
        assert!(should_forward_udp(b"1300"));
    }

    #[test]
    fn routes_mixed_commands_to_controller_and_strip() {
        let packet = b"1300,200990099; 201990099\n200000000";
        let mut controller = ControllerState::default();
        let mut strip = IntegerActionToLedStrip::new(RGB8::new(0, 0, 255));

        assert_eq!(process_udp(&mut controller, packet), 1);
        assert_ne!(controller.buttons & button::A, 0);
        for command in udp_led_commands(packet) {
            assert_eq!(strip.handle_integer(command), Ok(true));
        }
        assert_eq!(strip.state()[0], RGB8::new(0, 0, 0));
        assert_eq!(strip.state()[1], RGB8::new(99, 0, 99));
        assert_eq!(strip.state()[2], RGB8::new(0, 0, 255));
    }
}
