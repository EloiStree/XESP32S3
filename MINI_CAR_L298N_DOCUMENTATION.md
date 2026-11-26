# Mini Car L298N Motor Driver Module - UPDATED

## Overview
This module provides a Rust implementation for controlling a mini car equipped with an L298N motor driver via UDP commands. **Updated to support ASCII digit input ('0'-'9') for analog control on pins 17-18.**

## GPIO Pin Configuration
- **Pins 17-18**: Analog PWM speed control via ASCII characters '0'-'9'
  - Pin 17: Analog control character ('0' = 0, '1' = 1, ... '9' = 9)
  - Pin 18: Analog control character ('0' = 0, '1' = 1, ... '9' = 9)
  
- **Pins 10-13**: Boolean direction control
  - Pin 10: Direction bit 0 (IN1)
  - Pin 11: Direction bit 1 (IN2)
  - Pin 12: Direction bit 2 (IN3)
  - Pin 13: Direction bit 3 (IN4)

## UDP Message Format

### Valid Message Structure
Messages must follow this format:
- **Fixed 10-byte format**: `CAR:AADDDD`

Where:
- `CAR`: Fixed header (required, 4 bytes)
- `AA`: Two ASCII digit characters for pins 17-18 (each '0'-'9', 2 bytes)
- `DDDD`: 4 binary digits for pins 10-13 (each '0' or '1', 4 bytes)
- **Total: exactly 10 bytes**

### Examples

#### Valid Commands
| Packet | Pin 17 | Pin 18 | Pins (10,11,12,13) | Description |
|--------|--------|--------|-------------------|-------------|
| `CAR:5610101` | '5' | '6' | 1,0,1,0 | Pin17=5, Pin18=6, alternating direction |
| `CAR:9911111` | '9' | '9' | 1,1,1,1 | Maximum analog (9,9), all pins high |
| `CAR:0000000` | '0' | '0' | 0,0,0,0 | Minimum analog (0,0), all pins low |
| `CAR:0010101` | '0' | '0' | 1,0,1,0 | No analog speed, alternating pattern |
| `CAR:5500000` | '5' | '5' | 0,0,0,0 | Speed 5 on both pins, no direction |

#### Invalid Commands
| Packet | Reason |
|--------|--------|
| `CAR:561010` | Too short (9 bytes) |
| `CAR:56101010` | Too long (11 bytes) |
| `CAX:5610101` | Invalid header |
| `CAR:56101X1` | Invalid direction bit (X is not 0 or 1) |
| `CAR:A610101` | Non-digit analog value for pin 17 |
| `CAR:5A10101` | Non-digit analog value for pin 18 |

## Usage in Rust

### Basic Parsing
```rust
use xesp32s3::mini_car_l298n::MiniCarCommand;

let packet = b"CAR:5610101";
if let Some(cmd) = MiniCarCommand::try_from_udp(packet) {
    println!("Pin 17: {}", cmd.pin_17_analog);  // '5'
    println!("Pin 18: {}", cmd.pin_18_analog);  // '6'
    println!("Pin 17 value: {}", cmd.pin_17_value());  // 5
    println!("Pin 18 value: {}", cmd.pin_18_value());  // 6
    println!("Pin 10: {}", cmd.pin_10);  // true
    println!("Pin 11: {}", cmd.pin_11);  // false
    println!("Pin 12: {}", cmd.pin_12);  // true
    println!("Pin 13: {}", cmd.pin_13);  // false
}
```

### Network Integration
```rust
use xesp32s3::network::udp_mini_car_command;

let packet = b"CAR:5610101";
if let Some(cmd) = udp_mini_car_command(packet) {
    // Process the command
    apply_pwm_pin_17(cmd.pin_17_value());  // 0-9
    apply_pwm_pin_18(cmd.pin_18_value());  // 0-9
    set_pin_10(cmd.pin_10);
    set_pin_11(cmd.pin_11);
    set_pin_12(cmd.pin_12);
    set_pin_13(cmd.pin_13);
}
```

### Creating Commands Programmatically
```rust
use xesp32s3::mini_car_l298n::MiniCarCommand;

let cmd = MiniCarCommand::new(
    '5',   // pin_17_analog
    '6',   // pin_18_analog
    true,  // pin_10
    false, // pin_11
    true,  // pin_12
    false, // pin_13
);

println!("{}", cmd);  // Output: MiniCar{pin17:5, pin18:6, pins:1010}
```

## Validation Rules

1. **Message Length**: Must be exactly 10 bytes
2. **Header**: Must start with "CAR:" (4 bytes)
3. **Analog Characters**: 2 ASCII digits (each '0'-'9')
4. **Direction Bits**: Exactly 4 characters (each '0' or '1')
5. **UTF-8 Encoding**: Message must be valid UTF-8
6. **No variable length**: Unlike the previous version, this is strictly 10 bytes

## Module Structure

### `MiniCarCommand` Struct
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiniCarCommand {
    pub pin_17_analog: char,  // '0'-'9'
    pub pin_18_analog: char,  // '0'-'9'
    pub pin_10: bool,         // Direction control
    pub pin_11: bool,         // Direction control
    pub pin_12: bool,         // Direction control
    pub pin_13: bool,         // Direction control
}
```

### Key Methods
- `new()`: Create a command manually
- `try_from_udp()`: Parse from UDP packet bytes
- `pin_17_value()`: Convert pin 17 character to 0-9 number
- `pin_18_value()`: Convert pin 18 character to 0-9 number
- `fmt()` via `Display`: Pretty-print the command

## Integration with main.rs

The module is designed to be integrated into the UDP packet processing loop in `main.rs`:

```rust
// In the UDP processing section
if let Some(cmd) = udp_mini_car_command(&packet) {
    // Apply motor control
    // Set analog PWM to pins 17-18 based on cmd.pin_17_value() and cmd.pin_18_value() (0-9)
    // Set digital outputs to pins 10-13 based on pin_10/11/12/13
}
```

## Testing

The module includes comprehensive unit tests covering:
- Valid 10-byte commands with various analog values (0-9)
- Direction bit combinations
- Zero values on analog pins
- Maximum values on analog pins (9,9)
- Invalid headers
- Too-short and too-long packets
- Invalid direction bits
- Non-digit analog values on pin 17
- Non-digit analog values on pin 18
- All pins on/off combinations
- Display format validation

All 11 tests pass with the current implementation.

## Constants

- `CAR_HEADER`: Fixed header bytes "CAR:"
- `HEADER_LEN`: 4 bytes
- `VALID_MSG_LEN`: 10 bytes (exactly)
- `ANALOG_CHARS_LEN`: 2 bytes (pins 17-18)
- `DIRECTION_BITS_LEN`: 4 bytes (pins 10-13)

## Implementation Notes

- The module uses `no_std` compatible code
- All parsing uses only UTF-8 conversion and character validation
- No dynamic memory allocation
- Zero-copy design where possible
- Display trait for debugging output
- Provides helper methods to convert analog characters to numeric values (0-9)
- ASCII digit validation using `is_ascii_digit()`
- Strictly enforces 10-byte message length

## Changes from Previous Version

✅ **Fixed-length format**: Now exactly 10 bytes instead of 10-11 bytes
✅ **Analog support**: Pins 17-18 use ASCII digit characters ('0'-'9') instead of multi-digit numbers
✅ **Cleaner parsing**: No variable-length parsing, simpler validation
✅ **Helper methods**: Added `pin_17_value()` and `pin_18_value()` for numeric conversion
✅ **Better structure**: Clear separation between analog (chars) and digital (bools) components
