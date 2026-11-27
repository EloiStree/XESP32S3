use embedded_graphics::framebuffer::Framebuffer;
use core::fmt::Write;
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::{raw::{BigEndian, RawU1}, BinaryColor};
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};

pub const SCREEN_FRAME_BYTES: usize = 128 * 64 / 8;
pub const EINK_FRAME_BYTES: usize = 200 * 200 / 8;

pub type EinkFramebuffer = Framebuffer<BinaryColor, RawU1, BigEndian, 200, 200, EINK_FRAME_BYTES>;

const XINPUT_COMMANDS: [(&str, u16); 28] = [
    ("A", 1300), ("X", 1301), ("B", 1302), ("Y", 1303),
    ("LB", 1304), ("RB", 1305), ("LS", 1306), ("RS", 1307),
    ("MENU>", 1308), ("MENU<", 1309), ("D0", 1310), ("DN", 1311),
    ("DNE", 1312), ("DE", 1313), ("DSE", 1314), ("DS", 1315),
    ("DSW", 1316), ("DW", 1317), ("DNW", 1318), ("HOME", 1319),
    ("LX+", 1350), ("LX-", 1351), ("LY+", 1352), ("LY-", 1353),
    ("RX+", 1354), ("RX-", 1355), ("RY+", 1356), ("RY-", 1357),
];

/// Draws the boot status using the e-paper's row-major, MSB-first pixel format.
pub fn eink_startup_image(ip: &str, utc: &str) -> EinkFramebuffer {
    let mut frame = EinkFramebuffer::new();
    frame.clear(BinaryColor::On).unwrap();
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    for (label, text, y) in [("IP", ip, 0), ("UTC", utc, 10)] {
        let mut line = heapless::String::<19>::new();
        write!(&mut line, "{label} {text}").expect("status must fit on the display");
        Text::with_baseline(&line, Point::new(0, y), style, Baseline::Top)
            .draw(&mut frame).unwrap();
    }
    Text::with_baseline("D=DPAD P/R=ON/OFF", Point::new(0, 20), style, Baseline::Top)
        .draw(&mut frame).unwrap();
    for (index, &(label, code)) in XINPUT_COMMANDS.iter().enumerate() {
        let mut line = heapless::String::<15>::new();
        write!(&mut line, "{label:<5} {code}/{}", code + 1000)
            .expect("command must fit on the display");
        let x = if index < 14 { 0 } else { 104 };
        let y = 32 + (index % 14) as i32 * 12;
        Text::with_baseline(&line, Point::new(x, y), style, Baseline::Top)
            .draw(&mut frame).unwrap();
    }
    frame
}

/// Identifies a complete SSD1306 128x64 framebuffer.
pub fn udp_screen_packet(packet: &[u8]) -> Option<&[u8; SCREEN_FRAME_BYTES]> {
    packet.try_into().ok()
}

/// Identifies a complete 200x200 row-major e-paper framebuffer.
pub fn udp_eink_screen_packet(packet: &[u8]) -> Option<&[u8; EINK_FRAME_BYTES]> {
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

    #[test]
    fn identifies_complete_eink_frames_without_reordering_bytes() {
        assert!(udp_eink_screen_packet(&[0; 4999]).is_none());
        assert!(udp_eink_screen_packet(&[0; 5001]).is_none());
        let mut frame = [0xff; EINK_FRAME_BYTES];
        frame[0] = 0x7f;
        frame[25] = 0xfe;
        frame[4999] = 0x00;
        let packet = udp_eink_screen_packet(&frame).unwrap();
        assert_eq!(packet[0], 0x7f);
        assert_eq!(packet[25], 0xfe);
        assert_eq!(packet[4999], 0x00);
    }

    #[test]
    fn startup_image_shows_ip_time_and_all_command_pairs() {
        let frame = eink_startup_image("192.168.137.187", "12:34:56");
        let different_ip = eink_startup_image("10.0.0.1", "12:34:56");
        let different_time = eink_startup_image("192.168.137.187", "12:34:57");
        assert_ne!(&frame.data()[..10 * 25], &different_ip.data()[..10 * 25]);
        assert_ne!(&frame.data()[10 * 25..20 * 25], &different_time.data()[10 * 25..20 * 25]);

        let expected = [
            "A     1300/2300", "X     1301/2301", "B     1302/2302", "Y     1303/2303",
            "LB    1304/2304", "RB    1305/2305", "LS    1306/2306", "RS    1307/2307",
            "MENU> 1308/2308", "MENU< 1309/2309", "D0    1310/2310", "DN    1311/2311",
            "DNE   1312/2312", "DE    1313/2313", "DSE   1314/2314", "DS    1315/2315",
            "DSW   1316/2316", "DW    1317/2317", "DNW   1318/2318", "HOME  1319/2319",
            "LX+   1350/2350", "LX-   1351/2351", "LY+   1352/2352", "LY-   1353/2353",
            "RX+   1354/2354", "RX-   1355/2355", "RY+   1356/2356", "RY-   1357/2357",
        ];
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
        for (index, text) in expected.iter().enumerate() {
            let x = if index < 14 { 0 } else { 104 };
            let y = 32 + (index % 14) * 12;
            let mut reference = EinkFramebuffer::new();
            reference.clear(BinaryColor::On).unwrap();
            let rendered = Text::with_baseline(text, Point::new(x, y as i32), style, Baseline::Top);
            assert!(rendered.bounding_box().bottom_right().unwrap().x < 200);
            assert!(rendered.bounding_box().bottom_right().unwrap().y < 200);
            rendered.draw(&mut reference).unwrap();
            let byte_column = x as usize / 8;
            for row in y..y + 10 {
                let start = row * 25 + byte_column;
                assert_eq!(
                    &frame.data()[start..start + 12],
                    &reference.data()[start..start + 12],
                    "command {text} at row {row}"
                );
            }
        }
    }
}
