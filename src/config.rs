/// Change to the Wi-Fi SSID you want to connect to.
pub const WIFI_SSID: &str = "UART.LOCAL";

/// Change to the Wi-Fi password for the network you want to connect to.
pub const WIFI_PASSWORD: &str = "12345678";

/// Change to the mDNS host name for the device.
/// `ping xbox.local`` to test mDNS resolution.
pub const MDNS_HOST: &str = "xbox";

/// Change to the UDP port the device should listen on.
pub const UDP_PORT: u16 = 3615;

/// Baud rate for both UDP-to-UART outputs.
pub const UART_BAUD_RATE: u32 = 9_600;

/// Number of WS2812B LEDs connected to the strip output.
pub const STRIP_LED_COUNT: usize = 100;

/// GPIO for the LED strip data input.
pub const STRIP_GPIO: u8 = 14;
/// GPIO for UART1 transmit.
pub const UART1_TX_GPIO: u8 = 4;
/// GPIO for UART2 transmit.
pub const UART2_TX_GPIO: u8 = 5;
/// GPIO for OLED I2C data.
pub const SCREEN_SDA_GPIO: u8 = 8;
/// GPIO for OLED I2C clock.
pub const SCREEN_SCL_GPIO: u8 = 9;

const fn valid_configurable_gpio(pin: u8) -> bool {
    matches!(pin, 1..=18 | 21 | 38..=42 | 45..=47)
}

const _: () = {
    let pins = [
        STRIP_GPIO,
        UART1_TX_GPIO,
        UART2_TX_GPIO,
        SCREEN_SDA_GPIO,
        SCREEN_SCL_GPIO,
    ];
    let mut i = 0;
    while i < pins.len() {
        assert!(
            valid_configurable_gpio(pins[i]),
            "unsupported or reserved GPIO"
        );
        let mut j = 0;
        while j < i {
            assert!(pins[i] != pins[j], "GPIO assignments must be unique");
            j += 1;
        }
        i += 1;
    }
};
