# Mini Car L298N UDP Examples

## Format Reference
```
CAR:AADDDD (10 bytes, case-insensitive)
├─ CAR: = 4 bytes (header, any case)
├─ AA = 2 bytes (analog control pins 17-18, '0'-'9')
└─ DDDD = 4 bytes (direction pins 10-13, '0' or '1')
```

## Valid Examples

### Forward at Full Speed
```
CAR:9911111
car:9911111
Car:9911111
```
- **Pin 17 (analog)**: 9 (max speed)
- **Pin 18 (analog)**: 9 (max speed)
- **Pin 10 (direction)**: 1 (forward)
- **Pin 11 (direction)**: 1 (forward)
- **Pin 12 (direction)**: 1 (forward)
- **Pin 13 (direction)**: 1 (forward)

### Idle/Stop
```
CAR:0000000
car:0000000
Car:0000000
```
- **All pins**: 0 (stopped)

### Half Speed Forward
```
CAR:5500001
```
- **Pin 17**: 5 (half speed)
- **Pin 18**: 5 (half speed)
- **Pins 10-13**: 0001 (slight forward tilt)

### Mixed Speed with Different Directions
```
car:7301010
```
- **Pin 17**: 7 (high speed)
- **Pin 18**: 3 (low speed)
- **Pins 10-13**: 1010 (alternating directions)

### Reverse at Low Speed
```
CAR:1100000
```
- **Pin 17**: 1 (low speed)
- **Pin 18**: 1 (low speed)
- **Pins 10-13**: 0000 (reversed direction)

## Invalid Examples (Will Be Rejected)

### Wrong Header
```
CAX:561010  ❌ Invalid header (not CAR)
XAR:561010  ❌ Invalid header
CAR561010   ❌ Missing colon
CAR :56101  ❌ Space before colon
```

### Wrong Length
```
CAR:56101   ❌ Too short (9 bytes)
CAR:5610101 ❌ Too long (11 bytes)
CAR:56      ❌ Too short
```

### Invalid Characters
```
CAR:A61010  ❌ Non-digit analog value (A is not 0-9)
CAR:5A1010  ❌ Non-digit analog value
CAR:561X10  ❌ Non-binary direction bit (X is not 0-1)
CAR:56101A  ❌ Non-binary direction bit
CAR:5610 10 ❌ Space in direction bits
CAR:56-1010 ❌ Invalid character
```

## Python Client Examples

### Send Raw Bytes
```python
import socket

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
address = ("192.168.1.100", 8888)

# All these work now (case-insensitive)
sock.sendto(b"CAR:561010", address)  # Uppercase
sock.sendto(b"car:561010", address)  # Lowercase
sock.sendto(b"Car:561010", address)  # Mixed case
sock.sendto(b"cAr:561010", address)  # Any combination
```

### Send with Variable Values
```python
def send_car_command(sock, address, pin17, pin18, pin10, pin11, pin12, pin13):
    """
    Send a mini car command.
    
    Args:
        pin17, pin18: Speed values 0-9
        pin10, pin11, pin12, pin13: Direction bits 0-1
    """
    cmd = f"car:{pin17}{pin18}{pin10}{pin11}{pin12}{pin13}".encode()
    sock.sendto(cmd, address)

# Example: Forward at half speed
send_car_command(sock, address, 5, 5, 1, 1, 1, 1)

# Example: Stopped
send_car_command(sock, address, 0, 0, 0, 0, 0, 0)

# Example: Reverse at high speed
send_car_command(sock, address, 9, 9, 0, 0, 0, 0)
```

### Safe Validation Before Sending
```python
def is_valid_mini_car_command(cmd_str):
    """Validate a mini car command string."""
    
    # Check length
    if len(cmd_str) != 10:
        return False
    
    # Check header (now case-insensitive in firmware)
    if not cmd_str[:3].upper() == "CAR" or cmd_str[3] != ':':
        return False
    
    # Check analog values (pins 17-18)
    if not all(c in '0123456789' for c in cmd_str[4:6]):
        return False
    
    # Check direction bits (pins 10-13)
    if not all(c in '01' for c in cmd_str[6:10]):
        return False
    
    return True

# Usage
valid = is_valid_mini_car_command("car:561010")  # True
valid = is_valid_mini_car_command("CAR:9A1010")  # False (A not allowed)
valid = is_valid_mini_car_command("car:561010extra")  # False (too long)
```

## C/C++ Client Examples

### Arduino/ESP32
```cpp
#include <WiFiUdp.h>

WiFiUDP udp;
IPAddress server(192, 168, 1, 100);
uint16_t port = 8888;

void send_car_command(int pin17, int pin18, int pin10, int pin11, int pin12, int pin13) {
    char cmd[11];
    snprintf(cmd, sizeof(cmd), "car:%d%d%d%d%d%d", 
             pin17, pin18, pin10, pin11, pin12, pin13);
    
    udp.beginPacket(server, port);
    udp.write((uint8_t*)cmd, 10);  // Exactly 10 bytes
    udp.endPacket();
}

// Forward at full speed
send_car_command(9, 9, 1, 1, 1, 1);

// Stop
send_car_command(0, 0, 0, 0, 0, 0);
```

## Rust Client Examples

### Send UDP Packet
```rust
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let addr = "192.168.1.100:8888";
    
    // Send with uppercase (original style)
    socket.send_to(b"CAR:561010", addr).unwrap();
    
    // Send with lowercase (now supported)
    socket.send_to(b"car:561010", addr).unwrap();
    
    // Send with dynamic values
    let pin17 = 5;
    let pin18 = 6;
    let cmd = format!("car:{}{}{}{}{}{}", 
                      pin17, pin18, 1, 0, 1, 0);
    socket.send_to(cmd.as_bytes(), addr).unwrap();
}
```

## JavaScript/Node.js Client Examples

### UDP Client
```javascript
const dgram = require('dgram');

const client = dgram.createSocket('udp4');
const serverAddress = '192.168.1.100';
const serverPort = 8888;

function sendCarCommand(pin17, pin18, pin10, pin11, pin12, pin13) {
    const cmd = `car:${pin17}${pin18}${pin10}${pin11}${pin12}${pin13}`;
    const buffer = Buffer.from(cmd, 'utf8');
    
    client.send(buffer, 0, buffer.length, serverPort, serverAddress, (err) => {
        if (err) console.error(err);
        else console.log('Sent:', cmd);
    });
}

// Forward at half speed
sendCarCommand(5, 5, 1, 1, 1, 1);

// Reverse at high speed
sendCarCommand(9, 9, 0, 0, 0, 0);
```

## Testing Checklist

- [ ] Uppercase header `CAR:` works
- [ ] Lowercase header `car:` works
- [ ] Mixed case headers work (Car:, cAr:, etc.)
- [ ] Invalid header `CAX:` is rejected
- [ ] Analog values must be 0-9
- [ ] Direction bits must be 0-1
- [ ] Exactly 10 bytes required
- [ ] Shorter packets rejected
- [ ] Longer packets rejected

## Common Issues & Troubleshooting

### "Packet Rejected - Invalid Header"
- Ensure header is exactly "CAR:" (now case-insensitive)
- Check that colon is present after the first 3 characters

### "Packet Rejected - Invalid Length"
- Count bytes: CAR:AADDDD = exactly 10 bytes
- Remove any spaces, newlines, or extra characters

### "Analog Values Not Recognized"
- Use only '0'-'9' for pins 17-18 (positions 4-5)
- Don't use letters (A-F, etc.)

### "Direction Bits Not Recognized"
- Use only '0' or '1' for pins 10-13 (positions 6-9)
- Don't use other digits or letters

## Performance Notes

The case-insensitive comparison adds minimal overhead:
- **3-byte comparison** (`eq_ignore_ascii_case`)
- **No memory allocation**
- **Constant time O(3)**
- **Negligible impact** on ESP32S3 performance

All validation happens before motor control, so it won't affect real-time performance.
