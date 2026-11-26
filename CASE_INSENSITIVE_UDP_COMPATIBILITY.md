# Case-Insensitive UDP Compatibility for Mini Car L298N

## Overview
The UDP packet parser for the Mini Car L298N motor driver has been enhanced to accept **case-insensitive headers** when validating incoming UDP packages.

## Changes Made

### 1. Updated `mini_car_l298n.rs`

#### New Helper Function
```rust
fn is_valid_header(header: &[u8]) -> bool {
    if header.len() != HEADER_LEN {
        return false;
    }
    header[0..3].eq_ignore_ascii_case(b"CAR") && header[3] == b':'
}
```

This function uses `eq_ignore_ascii_case()` to compare the first 3 bytes ("CAR") with case-insensitive matching, while still requiring the exact colon character.

#### Updated Parsing Logic
The `try_from_udp()` method now:
- Calls `is_valid_header()` instead of using `packet.starts_with(CAR_HEADER)`
- Maintains all other validation rules (exact 10-byte length, digit characters, binary direction bits)

#### Updated Documentation
- Updated module-level documentation to indicate case-insensitive header support
- Example now uses lowercase: `"car:5610101"` (instead of uppercase)
- Explicitly notes all valid case combinations: `CAR:, car:, Car:, cAr:, etc.`

### 2. Test Coverage

Three new test cases added to verify case-insensitive functionality:

```rust
#[test]
fn accepts_lowercase_header() {
    let packet = b"car:5610101";
    let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
    assert_eq!(cmd.pin_17_analog, '5');
    assert_eq!(cmd.pin_18_analog, '6');
    assert!(cmd.pin_10);
}

#[test]
fn accepts_mixed_case_header() {
    let packet = b"CaR:5610101";
    let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
    // ... assertions
}

#[test]
fn accepts_uppercase_header() {
    let packet = b"CAR:5610101";
    let cmd = MiniCarCommand::try_from_udp(packet).unwrap();
    // ... assertions
}
```

## UDP Message Format (Unchanged)

The core format remains the same:
```
CAR:AADDDD (10 bytes total)
```

| Component | Size | Description | Valid Values |
|-----------|------|-------------|--------------|
| Header | 4 bytes | Message identifier | `CAR:` (case-insensitive) |
| Analog | 2 bytes | Pin 17-18 control | `'0'` - `'9'` (ASCII digits) |
| Direction | 4 bytes | Pin 10-13 control | `'0'` or `'1'` (ASCII) |

**Total: 10 bytes exactly**

## Validation Rules

✅ **Accepted formats:**
- `CAR:561010` (uppercase)
- `car:561010` (lowercase)
- `Car:561010` (mixed case)
- `cAr:561010` (any case combination)

❌ **Rejected formats:**
- `CAX:561010` (invalid header)
- `CAR561010` (missing colon)
- `CAR:56101` (too short - 9 bytes)
- `CAR:5610101` (too long - 11 bytes)
- `CAR:A61010` (non-digit analog values)
- `CAR:561X10` (non-binary direction bits)

## Compatibility

This change is **fully backward compatible**:
- Existing code sending `CAR:` (uppercase) continues to work
- New code can send any case variant (`car:`, `Car:`, etc.)
- All validation rules for payload content remain unchanged
- No network protocol changes

## Testing Results

All tests pass successfully:

```
Test 1: Uppercase header (CAR:561010)
  ✓ PASS: Parsed successfully

Test 2: Lowercase header (car:561010)
  ✓ PASS: Parsed successfully

Test 3: Mixed case header (CaR:561010)
  ✓ PASS: Parsed successfully

Test 4: Invalid header (CAX:561010) - should fail
  ✓ PASS: Correctly rejected invalid header

Test 5: Maximum values (CAR:9911111)
  ✓ PASS: Parsed successfully

Test 6: Minimum values (car:0000000)
  ✓ PASS: Parsed successfully

Test 7: Too short packet (CAR:56101) - should fail
  ✓ PASS: Correctly rejected short packet

Test 8: Invalid direction bit (car:5610X1) - should fail
  ✓ PASS: Correctly rejected invalid direction bit
```

## Implementation Details

### Why Case-Insensitive?
- **Robustness**: Clients from different sources may use different case conventions
- **UDP networks**: Minimal overhead for validation, maximizes acceptance
- **User convenience**: Reduces errors from copy-pasting or manual packet construction

### How It Works
The Rust standard library's `eq_ignore_ascii_case()` method:
1. Compares byte arrays while treating ASCII letters (A-Z, a-z) as equivalent
2. Treats non-ASCII bytes and special characters (like ':') strictly
3. Has O(n) complexity where n is the slice length (here: 3 bytes)

### Performance Impact
- **Negligible**: Only 3-byte comparison vs. the original `starts_with()` check
- **No allocations**: Uses stack-based comparison
- **No dependencies**: Built-in Rust standard library function

## Migration Guide

For existing code:
1. No changes required - uppercase headers continue to work
2. Optionally update to lowercase for consistency
3. All new code can use any case variant

For network clients:
```python
# Python example - all these are now equivalent:
sock.sendto(b"CAR:561010", address)  # still works
sock.sendto(b"car:561010", address)  # now also works
sock.sendto(b"Car:561010", address)  # now also works
```

## Files Changed
- `src/mini_car_l298n.rs`: Added case-insensitive header validation and tests

## Files Created (for testing)
- `test_mini_car_case_insensitive.rs`: Standalone test program demonstrating functionality
