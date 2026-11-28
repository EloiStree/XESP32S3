# SSD1306 OLED Screen GPIO Configuration

## Configuration Summary
The SSD1306 128x64 OLED screen is properly configured to use the following GPIO pins for I2C communication:

### GPIO Pin Assignment
- **SDA (Serial Data Line)**: GPIO 20
- **SCL (Serial Clock Line)**: GPIO 19

### Configuration Source
These constants are defined in `src/config.rs`:

```rust
/// GPIO for OLED I2C data.
pub const SCREEN_SDA_GPIO: u8 = 20;

/// GPIO for OLED I2C clock.
pub const SCREEN_SCL_GPIO: u8 = 19;
```

## Implementation Details

### I2C Configuration
Located in `src/bin/main.rs`, the I2C bus is configured with:
- **Frequency**: 400 kHz (chosen to overcome USB interrupt latency)
- **SDA Pin**: GPIO 20 (screen_sda)
- **SCL Pin**: GPIO 19 (screen_scl)

```rust
let i2c = I2c::new(
    peripherals.I2C0,
    I2cConfig::default()
        .with_frequency(Rate::from_khz(400))
)
    .expect("I2C config")
    .with_sda(screen_sda)
    .with_scl(screen_scl);
```

### SSD1306 Initialization
The display is initialized in the `display_task` function with:
- **Display Size**: 128x64 pixels
- **Rotation**: Rotate0 (default orientation)
- **Mode**: Buffered graphics mode

```rust
let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
display.init().expect("SSD1306 initialization");
```

## Features Supported

1. **Wi-Fi Connection Status Display**
   - Shows SSID, password, and connection state
   - Displays IP address once connected

2. **Debug Information Display**
   - Shows Xbox controller input states
   - Displays button/axis information (when DISPLAY_DEBUG_INFO is enabled)

3. **UDP Frame Updates**
   - Receives 128x64 pixel frames (1024 bytes) via UDP
   - Updates display with remote framebuffer content

4. **QR Code Splash Screen**
   - Shows a QR code on startup for 4 seconds

## Validation

The GPIO assignments are validated at compile time in `src/config.rs` to ensure:
- All GPIO pins are within valid ranges
- No duplicate GPIO assignments
- Proper separation from reserved pins

## Connection Checklist

✓ SDA on GPIO 20  
✓ SCL on GPIO 19  
✓ I2C clock frequency: 400 kHz  
✓ Display size: 128x64  
✓ I2C0 peripheral configured  
✓ Configuration validated at compile time
