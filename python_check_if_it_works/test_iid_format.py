#!/usr/bin/env python3
"""
Test script for IID (Index-Integer-Delay) UDP format.

This script sends UDP packets in the binary IID format to test the integer delayer.
Each 16-byte chunk contains:
  - 4 bytes: Index (u32, little-endian)
  - 4 bytes: Value (u32, little-endian) 
  - 8 bytes: Delay in milliseconds (u64, little-endian)

Expected device output:
  [UDP] ✓ Packet received: X bytes
  [IID] Processing X bytes = Y chunks
  [IID] Chunk 0: index=X, value=X, delay_ms=X
  [DELAYED] ✓ Sent to channel: index=X, value=X
  [DELAYED] Received from channel: index=X, value=X
  [DELAYED] ✓ Buttons changed: 0x00000000 → 0x00001000
"""

import struct
import socket
import time
import sys

def send_iid_packet(host, port, commands):
    """
    Send an IID format UDP packet.
    
    Args:
        host: IP address or hostname
        port: UDP port
        commands: List of tuples (index, value, delay_ms)
    
    Example:
        commands = [
            (0, 1300, 0),      # index=0, button A (1300), execute immediately
            (1, 1301, 100),    # index=1, button B (1301), execute after 100ms
        ]
    """
    packet = b''
    
    for index, value, delay_ms in commands:
        # Pack as: index(u32) + value(u32) + delay_ms(u64), all little-endian
        chunk = struct.pack('<IIQQ', index, value, delay_ms)
        packet += chunk
        print(f"  Added: index={index}, value={value}, delay_ms={delay_ms}ms")
    
    print(f"\nSending {len(packet)} bytes to {host}:{port}")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.sendto(packet, (host, port))
    sock.close()
    
    print("✓ Packet sent!")
    return len(packet)

def test_immediate_button_press():
    """Test: Press button A immediately (0ms delay)"""
    print("\n" + "="*60)
    print("TEST 1: Immediate button press (button A)")
    print("="*60)
    print("Expected: Xbox button A pressed immediately")
    print("Expected log: [DELAYED] ✓ Buttons changed: 0x00000000 → 0x00001000")
    
    send_iid_packet(
        '192.168.1.100',  # Change to your ESP32 IP
        8888,
        [
            (0, 1300, 0),  # Button A, immediate
        ]
    )

def test_delayed_button_press():
    """Test: Press button B with 500ms delay"""
    print("\n" + "="*60)
    print("TEST 2: Delayed button press (button B, 500ms)")
    print("="*60)
    print("Expected: Nothing happens for 500ms, then Xbox button B pressed")
    print("Expected log:")
    print("  [DELAYED] Queue has 1 items")
    print("  [DELAYED] Found 1 ready commands")
    print("  [DELAYED] ✓ Buttons changed: 0x00000000 → 0x00002000")
    
    send_iid_packet(
        '192.168.1.100',  # Change to your ESP32 IP
        8888,
        [
            (0, 1301, 500),  # Button B, delay 500ms
        ]
    )

def test_multiple_commands_same_packet():
    """Test: Multiple commands in one packet with different delays"""
    print("\n" + "="*60)
    print("TEST 3: Multiple commands in one packet")
    print("="*60)
    print("Expected: 32 bytes = 2 chunks, two commands queued")
    print("Expected log:")
    print("  [IID] Processing 32 bytes = 2 chunks")
    print("  [IID] Chunk 0: index=0, value=1300, delay_ms=0")
    print("  [IID] Chunk 1: index=1, value=1301, delay_ms=100")
    print("  [DELAYED] Found 1 ready commands (immediate)")
    print("  [DELAYED] ✓ Buttons changed: 0x00000000 → 0x00001000")
    print("  (after 100ms)")
    print("  [DELAYED] Found 1 ready commands (button B)")
    print("  [DELAYED] ✓ Buttons changed: 0x00001000 → 0x00003000")
    
    send_iid_packet(
        '192.168.1.100',  # Change to your ESP32 IP
        8888,
        [
            (0, 1300, 0),    # Button A, immediate
            (1, 1301, 100),  # Button B, delay 100ms
        ]
    )

def test_invalid_delay_rejected():
    """Test: Delay > 36,000,000ms is rejected"""
    print("\n" + "="*60)
    print("TEST 4: Invalid delay (> 36,000,000ms) rejected")
    print("="*60)
    print("Expected: Command with delay > 36000000ms is REJECTED")
    print("Expected log:")
    print("  [IID] Chunk 0: index=0, value=1300, delay_ms=50000000 - REJECTED")
    
    send_iid_packet(
        '192.168.1.100',  # Change to your ESP32 IP
        8888,
        [
            (0, 1300, 50_000_000),  # Too large delay, should be rejected
        ]
    )

def test_button_hold_and_release():
    """Test: Press button, hold for time, then release"""
    print("\n" + "="*60)
    print("TEST 5: Button hold and release")
    print("="*60)
    print("Expected: Button A pressed, held for 300ms, then released")
    
    # 1300 is button A pressed
    # 1300 sent twice with second at delay = release time
    # To release: send a command that clears buttons
    send_iid_packet(
        '192.168.1.100',  # Change to your ESP32 IP
        8888,
        [
            (0, 1300, 0),     # Press button A, immediate
            (1, 1300, 300),   # Release button A (or send 1200 for explicit release), delay 300ms
        ]
    )

def main():
    if len(sys.argv) > 1:
        # Allow passing custom host as argument
        host = sys.argv[1]
    else:
        host = '192.168.1.100'  # Change to your ESP32 IP
    
    print("IID Format UDP Test Suite")
    print("=" * 60)
    print(f"Target: {host}:8888")
    print("\nMake sure the device is connected and listening on UDP 8888")
    print("Watch the serial output for confirmation messages\n")
    
    # Run tests
    try:
        # Uncomment the test you want to run, or run all:
        test_immediate_button_press()
        time.sleep(1)
        
        # test_delayed_button_press()
        # time.sleep(1)
        
        # test_multiple_commands_same_packet()
        # time.sleep(1)
        
        # test_invalid_delay_rejected()
        # time.sleep(1)
        
        # test_button_hold_and_release()
        # time.sleep(1)
        
        print("\n" + "=" * 60)
        print("Tests complete! Check device serial output for results.")
        print("=" * 60)
        
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n✗ Error: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
