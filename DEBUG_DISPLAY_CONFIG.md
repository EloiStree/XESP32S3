# Xbox Input Debug Display Configuration

## Overview
You can now disable the Xbox input debug display on the OLED screen to reduce display update latency and improve message throughput.

## Configuration

The debug display can be controlled via the `DISPLAY_DEBUG_INFO` constant in [src/config.rs](src/config.rs):

```rust
/// Enable debug display of Xbox input on the OLED screen.
/// Set to false to disable input debug display and reduce display update latency.
pub const DISPLAY_DEBUG_INFO: bool = true;
```

## How to Disable

1. Open `src/config.rs`
2. Change `pub const DISPLAY_DEBUG_INFO: bool = true;` to `pub const DISPLAY_DEBUG_INFO: bool = false;`
3. Recompile and deploy

## What Gets Disabled

When `DISPLAY_DEBUG_INFO` is set to `false`:

- **Display**: The two lines showing Xbox input debug info are hidden:
  - Line 22 (button and D-pad state): `B:XXX D:X`
  - Line 33 (analog stick values): `LX,Y RX,Y`

- **Processing**: The `DEBUG.signal()` call in the `udp_task` is skipped, eliminating the overhead of building and signaling debug information on each UDP packet received

## Display Layout

### When `DISPLAY_DEBUG_INFO = true` (Connected):
```
W:SSID M:xbox.local:3615
IP: 192.168.1.100
B:001 D:-
L+0.5,+0.5 R-0.2,-0.2
                      [BUILD_INFO]
```

### When `DISPLAY_DEBUG_INFO = false` (Connected):
```
W:SSID M:xbox.local:3615
IP: 192.168.1.100


                      [BUILD_INFO]
```

## Performance Impact

Disabling debug display eliminates:
- Signal overhead for each UDP packet
- Display rendering of debug information
- I2C traffic for updating those display lines

This should reduce latency for overall message processing.
