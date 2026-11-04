//! Xbox 360 wired controller (XInput) USB identity and input report encoding.

use crate::controller::{button, ControllerState, Direction};

pub const VENDOR_ID: u16 = 0x045E;
pub const PRODUCT_ID: u16 = 0x028E;
pub const DEVICE_RELEASE: u16 = 0x0114;

/// Interface class/subclass/protocol used by the Xbox 360 controller.
pub const IFACE_CLASS: u8 = 0xFF;
pub const IFACE_SUBCLASS: u8 = 0x5D;
pub const IFACE_PROTOCOL: u8 = 0x01;

/// Vendor specific class descriptor (type 0x21) found on genuine pads.
pub const CLASS_DESCRIPTOR: [u8; 15] = [
    0x00, 0x01, 0x01, 0x25, 0x81, 0x14, 0x00, 0x00, 0x00, 0x00, 0x13, 0x01, 0x08, 0x00, 0x00,
];

pub const REPORT_LEN: usize = 20;

/// Reply to vendor request 0x01, wValue 0x0100 (input capabilities).
pub const CAPABILITIES: [u8; 20] = [
    0x00, 0x14, 0xFF, 0xF7, 0xFF, 0xFF, 0xC0, 0xFF, 0xC0, 0xFF, 0xC0, 0xFF, 0xC0, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];
/// Reply to vendor request 0x01, wValue 0x0000.
pub const CAPABILITIES_SHORT: [u8; 8] = [0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub const AUDIO_DESCRIPTOR: [u8; 25] = [
    0x00, 0x01, 0x01, 0x01, 0x82, 0x40, 0x01, 0x02, 0x20, 0x16, 0x83, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x16, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
pub const PLUGIN_DESCRIPTOR: [u8; 7] = [0x00, 0x01, 0x01, 0x22, 0x84, 0x07, 0x00];
pub const SECURITY_DESCRIPTOR: [u8; 4] = [0x00, 0x01, 0x01, 0x03];
pub const XSM3_STRING: &str =
    "Xbox Security Method 3, Version 1.00, \u{a9} 2005 Microsoft Corporation. All rights reserved.";

const DPAD_UP: u8 = 0x01;
const DPAD_DOWN: u8 = 0x02;
const DPAD_LEFT: u8 = 0x04;
const DPAD_RIGHT: u8 = 0x08;
const START: u8 = 0x10;
const BACK: u8 = 0x20;
const L3: u8 = 0x40;
const R3: u8 = 0x80;

const LB: u8 = 0x01;
const RB: u8 = 0x02;
const GUIDE: u8 = 0x04;
const A: u8 = 0x10;
const B: u8 = 0x20;
const X: u8 = 0x40;
const Y: u8 = 0x80;

fn axis(value: f32) -> [u8; 2] {
    ((value.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()
}

fn trigger(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0) as u8
}

pub fn encode(state: &ControllerState) -> [u8; REPORT_LEN] {
    let mut low = 0u8;
    let mut high = 0u8;

    let b = state.buttons;
    let map = |mask: u16, target: &mut u8, bit: u8| {
        if b & mask != 0 {
            *target |= bit;
        }
    };
    map(button::MENU_RIGHT, &mut low, START);
    map(button::MENU_LEFT, &mut low, BACK);
    map(button::LS, &mut low, L3);
    map(button::RS, &mut low, R3);
    map(button::LB, &mut high, LB);
    map(button::RB, &mut high, RB);
    map(button::GUIDE, &mut high, GUIDE);
    map(button::A, &mut high, A);
    map(button::B, &mut high, B);
    map(button::X, &mut high, X);
    map(button::Y, &mut high, Y);

    low |= match state.dpad {
        Direction::Neutral => 0,
        Direction::North => DPAD_UP,
        Direction::NorthEast => DPAD_UP | DPAD_RIGHT,
        Direction::East => DPAD_RIGHT,
        Direction::SouthEast => DPAD_DOWN | DPAD_RIGHT,
        Direction::South => DPAD_DOWN,
        Direction::SouthWest => DPAD_DOWN | DPAD_LEFT,
        Direction::West => DPAD_LEFT,
        Direction::NorthWest => DPAD_UP | DPAD_LEFT,
    };

    let mut report = [0u8; REPORT_LEN];
    report[1] = REPORT_LEN as u8;
    report[2] = low;
    report[3] = high;
    report[4] = trigger(state.left_trigger);
    report[5] = trigger(state.right_trigger);
    report[6..8].copy_from_slice(&axis(state.left_x));
    report[8..10].copy_from_slice(&axis(state.left_y));
    report[10..12].copy_from_slice(&axis(state.right_x));
    report[12..14].copy_from_slice(&axis(state.right_y));
    report
}
