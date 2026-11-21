# Testing Guide: Xbox 360 Emulation + Display

## Current Configuration
- ✓ `composite_with_iads = true` (line 216)
- ✓ I2C Frequency: 400 kHz (line 201)
- ✓ Compiles successfully

## Testing Checklist

### 1. Device Recognition (Windows)
```
Expected Behavior:
  □ Connect USB to Windows PC
  □ Device Manager should show: "Xbox 360 Controller" or similar
  □ No unknown devices or error codes
  □ Single device entry (not multiple)
  
If NOT working:
  → Check USB cable (should work with native USB C, not through hub)
  → Check USB VID/PID match Xbox 360 (see xinput.rs)
  → Try disconnecting/reconnecting
```

### 2. UART Output Check
Connect to UART to monitor initialization:
```
Expected messages (in order):
  1. "IP xxx.xxx.xxx.xxx - UDP 3615"       (network ready)
  2. "Wi-Fi connected to UART.LOCAL"       (Wi-Fi connected)
  3. No "SSD1306 init failed" message      (display working)

If you see "SSD1306 init failed":
  → I2C communication failed
  → Try different frequency (200 kHz or 1000 kHz)
  → Check GPIO 8/9 wiring
```

### 3. Display Testing
```
Expected Display Output:
  
  Startup:
    - QR code splash screen for 4 seconds
  
  After Wi-Fi connected:
    - Line 1: "W:UART.LOCAL M:xbox.local:3615"
    - Line 2: "IP: 192.168.x.x"
    - (Lines 3-4: Empty if DISPLAY_DEBUG_INFO = false)
    - (Or controller data if DISPLAY_DEBUG_INFO = true)
    - Line 7: Build date and git commit

If display is NOT showing:
  → Increase I2C frequency to 1000 kHz
  → Or decrease to 200 kHz
  → Check OLED contrast (try adjusting in SSD1306 init)
```

### 4. Controller Input (Windows)
```
Test with Windows Game Controller:
  1. Settings → Devices → Game controllers
  2. Click "Properties"
  3. Should show controller tests (buttons, sticks, triggers)
  4. Move sticks, press buttons → Should respond in real-time

If NOT working:
  → Check UDP port (default 3615)
  → Send test packets to verify responsiveness
```

### 5. I2C Frequency Testing

If display doesn't work at 400 kHz, try:

**Option A: 200 kHz (Slower)**
```rust
// Line 201 in src/bin/main.rs
.with_frequency(Rate::from_khz(200))
```
- Safer if OLED has timing issues
- May be slightly slower to update

**Option B: 1000 kHz (Faster)**
```rust
// Line 201 in src/bin/main.rs
.with_frequency(Rate::from_khz(1000))
```
- May work better if display is nearby
- Some OLEDs don't support this

**Testing Process:**
1. Change frequency
2. Run `cargo build --release`
3. Flash to device
4. Check UART for "SSD1306 init failed"
5. Check if display shows content
6. If works, keep that frequency; if not, try next option

## Success Criteria

✅ **Minimum Success:**
- Device recognized as Xbox controller (Device Manager shows it)
- UART output shows no errors
- Either display works OR you disabled it intentionally

✅ **Full Success:**
- Device recognized as Xbox controller
- Display initializes without errors
- Display shows Wi-Fi status and IP
- Controller responds to input
- No timeout or communication errors in UART

## Common Issues & Fixes

### "SSD1306 init failed" in UART
**Problem:** I2C communication failed
**Solutions:**
1. Try `Rate::from_khz(200)` (slower)
2. Check GPIO 8/9 soldering
3. Check I2C pull-up resistors (should have 4.7kΩ or 10kΩ to 3.3V)
4. Try different SSD1306 module

### Device not recognized as Xbox controller
**Problem:** USB descriptor issue
**Solutions:**
1. Keep `composite_with_iads = true` (required)
2. Try different USB cable
3. Try different USB port on PC
4. Check UART for USB errors

### Display shows but no updates
**Problem:** Display initialized but data not updating
**Solutions:**
1. Check UDP packets arriving (listen on port 3615)
2. Check Wi-Fi connection
3. Check if DISPLAY_DEBUG_INFO is enabled
4. Verify display refresh task is running

### USB and I2C both failing
**Problem:** Likely resource conflict still present
**Solutions:**
1. Try `Rate::from_khz(1000)` (fastest)
2. Try disabling debug display: `DISPLAY_DEBUG_INFO = false` in config.rs
3. Consider reducing USB endpoint buffer size
4. Last resort: disable I2C display entirely

## Monitoring UART Output

Connect to UART at 115200 baud to see:
- Startup messages
- Wi-Fi connection status
- UDP packet statistics
- I2C initialization status
- Any error messages

```
Example good output:
  SSD1306 init succeeded
  IP 192.168.1.100 - UDP 3615
  Wi-Fi connected to UART.LOCAL
```

```
Example bad output:
  SSD1306 init failed
  (I2C communication error)
```

## Next Steps If Everything Works

1. **Fine-tune** controller behavior in src/network.rs
2. **Add custom** USB strings or behavior as needed
3. **Optimize** display refresh rate if needed
4. **Test** with actual games requiring Xbox 360 controller

## Roll-Back If Needed

If 400 kHz doesn't work, go back to default:

```rust
let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
    .expect("I2C config")
    .with_sda(screen_sda)
    .with_scl(screen_scl);
```

Then either:
- Set `composite_with_iads = false` (display works, manual driver)
- Or disable display entirely
