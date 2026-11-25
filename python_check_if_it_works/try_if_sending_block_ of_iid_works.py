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
    packet = commands
    
    print(f"\nSending {len(packet)} bytes to {host}:{port}")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.sendto(packet, (host, port))
    sock.close()
    
    print("✓ Packet sent!")
    return len(packet)


blocks_of_actions = b''.join([
            struct.pack('<iiQ', 0, 1300, 0),     # Press button A, immediate
            struct.pack('<iiQ', 0, 2300, 300),   # Release button A (or send 1200 for explicit release), delay 300ms
            struct.pack('<iiQ', 0, 1300, 1000),     # Press button A, immediate
            struct.pack('<iiQ', 0, 2300, 1300),   # Release button A (or send 1200 for explicit release), delay 300ms
        ])

host = '192.168.137.106'  # Change to your ESP32 IP

def main():
    while True:
        try:
            send_iid_packet(host, 3615, blocks_of_actions)
            print("Packet sent successfully.")
            print("Size of packet:", len(blocks_of_actions), "bytes")
            time.sleep(3)  # Wait 10 seconds before sending the next packet
        except Exception as e:
            print(f"\n✗ Error: {e}")
            time.sleep(1)

if __name__ == '__main__':
    main()
