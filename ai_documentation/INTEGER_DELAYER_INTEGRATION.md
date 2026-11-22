# Integer Delayer Integration Guide

## Overview
The `IndexValueIntegerTimeMillisecondsDelayer` has been successfully integrated into the UDP packet processing pipeline. When UDP packets are received that are **divisible by 16** (and not 1024 bytes), they are automatically validated and processed as IID (Index-Integer-Delay) format packets.

## How It Works

### 1. UDP Packet Reception
When a UDP packet is received in the `udp_task()`:

```rust
let (len, _from) = socket.recv_from(&mut buf).await;
let packet = &buf[..len];
```

### 2. Divisible-by-16 Check
Before regular UDP processing, the packet is validated:

```rust
{
    let mut delayer = DELAYER.lock().await;
    if delayer.append_value_if_dividable_by_16_and_not_1024(packet) {
        rprintln!("IID format packet received and queued");
        // Check for any delayed integers ready to execute
        for delayed_value in delayer.check_for_integer_to_execute() {
            rprintln!("Executing delayed integer: index={}, value={}", 
                      delayed_value.m_index, delayed_value.m_integer);
            DELAYED_INTEGERS.send((delayed_value.m_index, delayed_value.m_integer)).await;
        }
    }
}
```

### 3. Validation Criteria
A packet is processed as IID format if:
- ✅ Length is divisible by 16
- ❌ Length is NOT 1024 bytes (screen updates)
- ✅ Parsed delay value is < 36,000,000 ms (~10 hours)

### 4. IID Packet Structure
Each 16-byte chunk contains:
```
Byte 0-3:   u32 - Index (little-endian)
Byte 4-7:   u32 - Value (little-endian)
Byte 8-15:  u64 - Delay in milliseconds (little-endian)
```

**Example payload (32 bytes = 2 IID packets):**
```
Index1=1, Value1=255, Delay1=100ms, Index2=2, Value2=128, Delay2=200ms
```

## Integration Points

### Static Resources

**`src/bin/main.rs` line ~79:**
```rust
static DELAYER: embassy_sync::mutex::Mutex<CriticalSectionRawMutex, IndexValueIntegerTimeMillisecondsDelayer> =
    embassy_sync::mutex::Mutex::new(IndexValueIntegerTimeMillisecondsDelayer::new(0.0, 0.0));
```

**`src/bin/main.rs` line ~77:**
```rust
static DELAYED_INTEGERS: Channel<CriticalSectionRawMutex, (u32, u32), 64> = Channel::new();
```

### UDP Task Integration
**`src/bin/main.rs` lines 668-680:**
The delayer check happens immediately after receiving a UDP packet and before regular command processing.

## Processing Flow

```
UDP Packet Received
        ↓
    Divisible by 16?
        ↙         ↘
      YES          NO
        ↓          ↓
  IID Format      Regular UDP
  Processing      Processing
        ↓
  Check Ready
  Delayed Values
        ↓
  Send via
  DELAYED_INTEGERS
  Channel
```

## Usage Examples

### Sending IID Format Packets (Binary)

**Python Example:**
```python
import socket
import struct

def send_delayed_integer(sock, index, value, delay_ms):
    # Create 16-byte IID packet
    packet = struct.pack('<IIQ', index, value, delay_ms)
    sock.sendto(packet, ('ESP_IP', 8888))

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
send_delayed_integer(sock, 1, 255, 100)      # Execute index=1, value=255 in 100ms
send_delayed_integer(sock, 2, 128, 200)      # Execute index=2, value=128 in 200ms
```

**Sending multiple packets in one UDP datagram:**
```python
def send_multiple_delayed_integers(sock, commands):
    # commands = [(index, value, delay_ms), ...]
    packet = b''
    for index, value, delay_ms in commands:
        packet += struct.pack('<IIQ', index, value, delay_ms)
    sock.sendto(packet, ('ESP_IP', 8888))

# Send 2 commands in one 32-byte UDP packet
send_multiple_delayed_integers(sock, [
    (1, 255, 100),
    (2, 128, 200)
])
```

### Receiving Delayed Integers

Delayed integers are sent via the `DELAYED_INTEGERS` channel and can be consumed by any task:

```rust
// In any async task:
loop {
    let (index, value) = DELAYED_INTEGERS.recv().await;
    rprintln!("Delayed value received: index={}, value={}", index, value);
    // Process the delayed value
}
```

## Debug Output

When an IID format packet is received, you'll see:
```
IID format packet received and queued
Executing delayed integer: index=1, value=255
Executing delayed integer: index=2, value=128
```

## Configuration

### Maximum Queued Values
Current limit: **256 delayed integers** (see `MAX_DELAYED_VALUES` in `src/integer_delayer.rs`)

### Maximum Delayed Channel Capacity
Current limit: **64 delayed integers** in transit (see `DELAYED_INTEGERS` channel size in `src/bin/main.rs`)

To modify these limits:
1. For delayer queue: Edit `MAX_DELAYED_VALUES` in `src/integer_delayer.rs`
2. For channel capacity: Edit the channel size in `src/bin/main.rs` line ~77

## Thread Safety
- The `DELAYER` instance is wrapped in an `embassy_sync::mutex::Mutex`
- All access is async-safe with proper await/lock handling
- Multiple tasks can safely access the delayer sequentially

## Packet Filtering
The delayer check acts as an **IID format filter**:
- ✅ 16-byte packets → Processed as IID
- ✅ 32-byte packets → 2 IID packets
- ✅ 48-byte packets → 3 IID packets
- ❌ 1024-byte packets → Skipped (screen updates)
- ❌ Other sizes → Processed normally as UDP commands

## Logging
The integration includes debug logging via `rprintln!()`:
- When IID packets are received and queued
- When delayed integers are ready to execute

To disable these logs, comment out or remove the `rprintln!()` calls in `udp_task()`.

## Performance Considerations
- Each UDP packet check is O(n) where n = number of 16-byte chunks
- Delayed value checking is O(m) where m = queued delayed values
- Negligible overhead for non-IID packets (single modulo check)

## Next Steps
1. Create tasks to consume `DELAYED_INTEGERS` channel
2. Implement application-specific logic for delayed values
3. Consider integrating with controller state or LED strip
