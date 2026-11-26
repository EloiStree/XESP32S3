"""Send repeating L298N test commands: python try_if_l298n_motor_works.py [host]."""

import socket
import sys
import time


HOST = "192.168.137.106"
PORT = 3615
STEP_SECONDS = 0.1
STOP = b"CAR:000000"

# Exercise each direction input separately at half speed.
STEPS = (
    ("GPIO10", b"CAR:501000"),
    ("GPIO11", b"car:500100"),
    ("GPIO12", b"CaR:050010"),
    ("GPIO13", b"cAr:050001"),
)


def main():
    host = sys.argv[1] if len(sys.argv) > 1 else HOST
    address = (host, PORT)
    print(f"Sending L298N commands to {host}:{PORT}. Press Ctrl+C to stop.")
    print("UDP has no confirmation; observe the motors and device logs.")
    print("The motors stop if no valid car command arrives for one second.")

    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        try:
            while True:
                for name, packet in STEPS:
                    print(f"{name}: {packet.decode('utf-8')}", flush=True)
                    sock.sendto(packet, address)
                    time.sleep(STEP_SECONDS)
                    sock.sendto(STOP, address)
                    time.sleep(STEP_SECONDS)
        except KeyboardInterrupt:
            print("\nStopping.")
        finally:
            sock.sendto(STOP, address)
            print("Stop command sent; UDP delivery is not guaranteed.")


if __name__ == "__main__":
    main()