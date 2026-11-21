# USB composite_with_iads and I2C Display Conflict

## Problem
When setting `usb_config.composite_with_iads = true;` (line 216 in main.rs), the OLED screen (I2C) and device functionality stop working.

## Root Cause
The `composite_with_iads` configuration changes how USB handles descriptor reporting and device enumeration, which can:
1. Increase USB interrupt handler overhead
2. Cause I2C bus timing to become unreliable during USB enumeration
3. Create scheduling conflicts between USB and I2C tasks
4. Lead to I2C bus lockup if the timing becomes too tight

The ESP32-S3 has limited interrupt handling capacity, and USB interrupts can starve the I2C communication timing.

## Current Configuration
- **Screen I2C**: GPIO 8 (SDA), GPIO 9 (SCL)
- **USB**: GPIO 19 (D-), GPIO 20 (D+), native USB FS
- **I2C Frequency**: Default (typically 100 kHz)

## Solutions

### Option 1: Keep composite_with_iads = false (Recommended)
This is the safest option and maintains compatibility with the display.

```rust
usb_config.composite_with_iads = false;  // ✓ Works with I2C display
```

### Option 2: Increase I2C Speed
If you need `composite_with_iads = true`, try increasing the I2C frequency to reduce communication time and avoid timing conflicts.

Modify `src/bin/main.rs` around line 196:

**Before:**
```rust
let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
    .expect("I2C config")
    .with_sda(screen_sda)
    .with_scl(screen_scl);
```

**After (attempt 400 kHz):**
```rust
let i2c = I2c::new(
    peripherals.I2C0,
    I2cConfig::default()
        .with_frequency(Rate::new(400_000))  // 400 kHz instead of default
)
.expect("I2C config")
.with_sda(screen_sda)
.with_scl(screen_scl);
```

### Option 3: Use Different I2C Port
If available, you could use I2C1 or separate the display from the main I2C bus.

### Option 4: Disable Display Task
If you don't need the display, you can disable the display task entirely:

```rust
// spawner.spawn(display_task(i2c, stack).unwrap());  // Comment out
```

## Why composite_with_iads is Needed
The `composite_with_iads` flag is typically needed when:
- Supporting multiple USB interfaces from a single device
- Ensuring Windows/macOS compatibility with complex USB descriptors
- Creating proper IAD (Interface Association Descriptor) for interface grouping

However, for a simple Xbox controller emulation, it may not be necessary.

## Testing Recommendations

1. **Test with composite_with_iads = false** (current default - should work)
2. **If you need it = true**, try gradually increasing I2C frequency:
   - 100 kHz → 200 kHz → 400 kHz → 1 MHz
   - Find the minimum working frequency

3. **Monitor for I2C errors** by adding error logging in the display_task

## Verification
To verify the I2C works properly when re-enabling composite_with_iads, check:
- Display initializes successfully on startup
- QR splash screen appears
- Wi-Fi status updates correctly
- No "SSD1306 init failed" messages in UART output
