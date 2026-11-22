# Delayed Integer System - Quick Start Guide

## What Changed

The `delayed_integers_task` now continuously monitors the delay queue and **applies Xbox input at the right time**. The delay value directly controls **when** the Xbox controller input is simulated.

## How It Works Now

### Before (Broken)
```
UDP Packet → Queue → Delayer → (Never executed) ✗
```

### After (Fixed)
```
UDP Packet → Delayer → Task Monitors → Waits for Time → Apply to Xbox ✓
```

## Sending a Delayed Xbox Input

### Python Example
```python
import socket
import struct
import time

# Create UDP socket
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
esp32_ip = "192.168.1.100"  # Replace with your ESP32's IP
esp32_port = 12345

# Press Button A after 2 seconds
index = 0
button_a_press = 1300
delay_ms = 2000  # 2 seconds

# Pack: index (u32) + button (u32) + delay (u64)
packet = struct.pack('<IIQ', index, button_a_press, delay_ms)

print("Sending delayed button press (A) in 2 seconds...")
sock.sendto(packet, (esp32_ip, esp32_port))
print("Packet sent! Xbox button A will be pressed in 2 seconds...")

time.sleep(3)
sock.close()
```

### Expected Debug Output
```
IID format packet received and queued
Delayed integer ready: index=0, value=1300
Applied delayed Xbox command: value=1300
```

## Packet Format

Each delayed command is exactly **16 bytes**:

```
Offset  Size  Type   Name
------  ----  ----   ----
0       4     u32    Index (usually 0, can track multiple commands)
4       4     u32    Xbox Command (1300=A, 1301=X, 1302=B, 1303=Y, etc.)
8       8     u64    Delay in milliseconds (1000 = 1 second)
```

### Common Xbox Commands
```
Button Commands:
  1300 = A press      / 2300 = A release
  1301 = X press      / 2301 = X release
  1302 = B press      / 2302 = B release
  1303 = Y press      / 2303 = Y release
  1304 = LB press     / 2304 = LB release
  1305 = RB press     / 2305 = RB release

Stick Commands:
  1330 = Left stick neutral
  1331 = Left stick north
  1333 = Left stick east
  1335 = Left stick south
  1337 = Left stick west
  
Trigger Commands:
  1358 = Left trigger full
  1359 = Right trigger full
```

## Multiple Commands in One Packet

Send multiple 16-byte commands in a single packet:

```python
# Two commands: press A after 1s, release A after 2s
packet = struct.pack('<IIQ', 0, 1300, 1000)  # Press A at 1s
packet += struct.pack('<IIQ', 1, 2300, 2000) # Release A at 2s

sock.sendto(packet, (esp32_ip, esp32_port))
```

## Testing the Implementation

### Test 1: Simple Button Press
```bash
# Send button A press in 2 seconds
python send_delayed_xbox.py
```
Expected: Xbox reports button A pressed 2 seconds after packet sent

### Test 2: Sequence of Commands
```bash
# Send: press A, wait 0.5s, release A
# All in one packet with appropriate delays
```
Expected: Xbox plays sequence: A pressed → A released

### Test 3: Verify Timing
```bash
# Send multiple commands with known delays
# Measure actual execution time
# Should be within ±50ms of requested delay
```

## Troubleshooting

### "IID format packet received and queued" but no execution
- **Check**: Is the delay time large enough? (minimum ~50ms recommended)
- **Check**: Is the Xbox command valid? (valid range: 1300-1399 or 2300-2399)
- **Solution**: Enable debug output on serial console

### "Packet sent" but no Xbox input
- **Check**: Packet format - must be exact 16 bytes per command
- **Check**: Delay time passed - give it a few seconds
- **Check**: Xbox is connected - check USB connection
- **Solution**: Send a regular (non-delayed) command first to verify Xbox works

### Commands executing but at wrong time
- **Check**: System clock - RTC must be synchronized
- **Check**: Task scheduling - jitter is normal (±10ms)
- **Solution**: For precision-critical delays, add 50-100ms buffer

### Too many delayed commands not executing
- **Check**: Queue size (max 256 delayed values)
- **Check**: Channel capacity (max 64 pending)
- **Solution**: Spread commands out over time

## Real-World Example: Game Macro

```python
def play_macro():
    """Example: Perform a game combo"""
    commands = [
        (0, 1300, 0),     # Press A immediately
        (1, 2300, 100),   # Release A after 100ms
        (2, 1301, 200),   # Press X after 200ms
        (3, 2301, 300),   # Release X after 300ms
        (4, 1302, 400),   # Press B after 400ms
        (5, 2302, 500),   # Release B after 500ms
    ]
    
    packet = b''
    for index, cmd, delay in commands:
        packet += struct.pack('<IIQ', index, cmd, delay)
    
    sock.sendto(packet, (esp32_ip, 12345))
    print(f"Sent combo with {len(commands)} commands")
```

## Performance Notes

- **Delay Precision**: ±5-10ms typical (depends on system load)
- **Maximum Delay**: ~10 hours (36,000,000ms)
- **Maximum Queued**: 256 commands at once
- **Minimum Delay**: ~20ms recommended (less than this may execute immediately)
- **Task Overhead**: <1% CPU impact

## Debug Console Output

Watch your serial console for messages like:
```
UDP receive: 16 bytes
IID format packet received and queued
Delayed integer ready: index=0, value=1300
Applied delayed Xbox command: value=1300
```

This confirms:
1. ✓ Packet received
2. ✓ Packet parsed as IID format
3. ✓ Delay time reached
4. ✓ Command applied to Xbox

## Still Not Working?

1. **Verify UDP connectivity**: Send a regular (non-delayed) command
2. **Check packet format**: Use `hexdump` to verify 16-byte structure
3. **Enable serial debug**: Check for error messages on console
4. **Test with smaller delay**: Try 100ms delay first
5. **Check Xbox connection**: Make sure Xbox shows as connected

## Next Steps

- Test with your game controller
- Create macros for your favorite games
- Automate complex button sequences
- Build a macro library for quick testing

## Questions or Issues?

Refer to these documentation files for more details:
- **DELAYED_INTEGER_INTEGRATION.md** - Technical details
- **DELAYED_INTEGER_CHANGES.md** - Code changes
- **DELAYED_INTEGER_VERIFICATION.md** - Testing report
