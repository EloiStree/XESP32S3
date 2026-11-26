// Simple verification of mini_car_l298n parsing (not a Rust test, just for documentation)
// This can be tested with the following inputs:

// Valid 10-byte commands:
// "CAR:501010"  -> speed=50, pins=1010
// "CAR:991111"  -> speed=99, pins=1111
// "CAR:000000"  -> speed=0, pins=0000
// "CAR:100000"  -> speed=10, pins=0000

// Valid 11-byte commands:
// "CAR:1001111" -> speed=100, pins=1111
// "CAR:2550000" -> speed=255, pins=0000
// "CAR:1500101" -> speed=150, pins=0101

// Invalid - too short:
// "CAR:5010" (only 8 bytes)
// "CAR:50" (only 6 bytes)

// Invalid - too long:
// "CAR:50101010" (12 bytes)

// Invalid - wrong header:
// "CAX:501010"
// "BAR:501010"

// Invalid - bad direction bits:
// "CAR:50102X"
// "CAR:5010A0"

// Invalid - non-numeric speed:
// "CAR:5A1010"
// "CAR:AB1010"
