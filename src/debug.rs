//! Debug info shared between the UDP task and the OLED display task.

use core::fmt::Write;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::String;

use crate::controller::{ControllerState, Direction};

pub type Line = String<24>;

#[derive(Clone, Default)]
pub struct DebugInfo {
    /// Raw UDP payload (lossy, truncated).
    pub received: Line,
    /// How many commands in the packet were applied.
    pub applied: usize,
    /// Resulting controller state summary.
    pub state: Line,
    pub axes: Line,
}

pub static DEBUG: Signal<CriticalSectionRawMutex, DebugInfo> = Signal::new();
pub static IP: Signal<CriticalSectionRawMutex, Line> = Signal::new();

fn dpad_name(d: Direction) -> &'static str {
    match d {
        Direction::Neutral => "-",
        Direction::North => "N",
        Direction::NorthEast => "NE",
        Direction::East => "E",
        Direction::SouthEast => "SE",
        Direction::South => "S",
        Direction::SouthWest => "SW",
        Direction::West => "W",
        Direction::NorthWest => "NW",
    }
}

pub fn describe(packet: &[u8], applied: usize, state: &ControllerState) -> DebugInfo {
    let mut info = DebugInfo::default();

    for &b in packet.iter().take(info.received.capacity()) {
        let c = if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' };
        let _ = info.received.push(c);
    }
    info.applied = applied;

    let _ = write!(info.state, "B:{:03X} D:{}", state.buttons, dpad_name(state.dpad));
    let _ = write!(
        info.axes,
        "L{:+.1},{:+.1} R{:+.1},{:+.1}",
        state.left_x, state.left_y, state.right_x, state.right_y
    );
    info
}
