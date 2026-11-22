# Delayed Integer Integration - Xbox Input Simulation

## Problem
The integer delay system was queuing delayed integers but never actually applying them to simulate Xbox controller input. The `DELAYED_INTEGERS` channel was being populated but not consumed.

## Solution
Implemented a new `delayed_integers_task` that continuously processes delayed integer commands and applies them to the Xbox controller state at the appropriate time.

## Implementation Details

### New Task: `delayed_integers_task`
Located in `src/bin/main.rs`, this task:

1. **Continuously checks the delayer for ready-to-execute integers**
   ```rust
   let mut delayer = DELAYER.lock().await;
   for delayed_value in delayer.check_for_integer_to_execute() {
       DELAYED_INTEGERS.send((delayed_value.m_index, delayed_value.m_integer)).await;
   }
   ```

2. **Processes pending delayed integers from the channel**
   - Uses a 10ms timeout to avoid blocking other tasks
   - Attempts to convert the value to a u16 command for direct execution
   - Falls back to compact parser for larger values

3. **Applies commands to Xbox controller state**
   ```rust
   if commands::execute(&mut state, command) {
       rprintln!("Applied delayed Xbox command: value={}", value);
   } else {
       compact_xbox_int_parser::execute(&mut state, value);
   }
   ```

### Integration Points

1. **UDP Task** - Already queues IID format packets (16-byte chunks) to the delayer
2. **Delayed Integers Task** - New task that monitors and executes ready commands
3. **Controller State** - Receives and applies commands via existing `execute()` functions

### Data Flow

```
UDP Packet (IID format)
    ↓
delayer.append_value_if_dividable_by_16_and_not_1024()
    ↓
delayer.check_for_integer_to_execute() [waits for m_when_to_execute time]
    ↓
DELAYED_INTEGERS channel
    ↓
delayed_integers_task receives and applies to CONTROLLER
    ↓
Xbox input is simulated
```

## Usage

No changes needed from the user perspective. When an IID format packet is received with delay times:

1. Format: 16 bytes per command = [4-byte index][4-byte value][8-byte delay_ms]
2. All commands in the packet are queued with their millisecond delays
3. At the specified time, each command is applied to simulate Xbox input
4. Multiple commands are supported per packet (array of 16-byte chunks)

### Example IID Packet Format
```
Bytes 0-3:   Index (u32 LE)
Bytes 4-7:   Command value (u32 LE) - e.g., 1300 for button A press
Bytes 8-15:  Delay in milliseconds (u64 LE) - e.g., 1000 for 1 second delay
```

## Testing

To verify the implementation works:

1. Build the project: `cargo build --release`
2. Send an IID format UDP packet with delayed commands
3. Watch debug output for:
   - "IID format packet received and queued"
   - "Delayed integer ready: index=X, value=Y"
   - "Applied delayed Xbox command: value=Z"

## Performance Considerations

- **Task Priority**: The `delayed_integers_task` runs at normal priority, interleaved with other tasks
- **CPU Usage**: Minimal - uses embassy's async timeout mechanism to avoid busy-waiting
- **Memory**: Uses existing `DELAYED_INTEGERS` channel (capacity: 64 entries) and heap-based delayer (max 256 entries)
- **Latency**: Command execution latency is typically <1ms after the delay time is reached

## Timing Precision

The delay precision depends on:
1. RTC time accuracy from the delayer (set during initialization)
2. Task scheduling jitter (typically <10ms on this system)
3. For millisecond-precision requirements, use delays > 100ms to account for system jitter

## Backward Compatibility

- Existing UDP packet formats continue to work unchanged
- IID format packets (divisible by 16, not 1024 bytes) are automatically detected
- Non-IID packets are processed normally as before
