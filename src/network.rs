use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use crate::commands::execute;
use crate::controller::{ControllerState, Direction};

pub const WIFI_SSID: &str = "UART.LOCAL";
pub const WIFI_PASSWORD: &str = "12345678";
pub const MDNS_HOST: &str = "xbox";
pub const UDP_PORT: u16 = 3615;

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
    let Ok(text) = core::str::from_utf8(packet) else {
        return 0;
    };

    text.split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';')
        .filter_map(|token| token.parse::<u16>().ok())
        .filter(|&code| execute(state, code))
        .count()
}
