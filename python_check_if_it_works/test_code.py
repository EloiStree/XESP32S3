"""Send test commands to the ESP32-S3 Xbox 360 pad over UDP (xbox.local:3615).

Usage: python test_code.py [host]   (default host: xbox.local, falls back to 192.168.137.143)
Watch the OLED / UART log, or a gamepad tester, while it runs.
"""
import socket
import sys
import time

PORT = 3615
FALLBACK_IP = "192.168.137.143"
RELEASE_ALL = 1390

# (description, press code); release code is press + 1000
BUTTONS = [
    ("A", 1300), ("X", 1301), ("B", 1302), ("Y", 1303),
    ("LB", 1304), ("RB", 1305), ("Left stick click", 1306),
    ("Right stick click", 1307), ("Start", 1308), ("Back", 1309),
    ("D-pad up", 1311), ("D-pad right", 1313),
    ("D-pad down", 1315), ("D-pad left", 1317),
    ("Left stick right", 1350), ("Left stick left", 1351),
    ("Left stick up", 1352), ("Left stick down", 1353),
    ("Right stick right", 1354), ("Right stick left", 1355),
    ("Left trigger", 1358), ("Right trigger", 1359),
]


def resolve(host):
    try:
        return socket.gethostbyname(host)
    except OSError:
        print(f"Could not resolve {host}, using {FALLBACK_IP}")
        return FALLBACK_IP


def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "xbox.local"
    ip = resolve(host)
    print(f"Target: {host} -> {ip}:{PORT}")
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    def send(text):
        sock.sendto(str(text).encode("ascii"), (ip, PORT))

    send(RELEASE_ALL)
    for name, press in BUTTONS:
        print(f"{name}: press {press}")
        send(press)
        time.sleep(0.6)
        send(press + 1000)
        time.sleep(0.2)

    print("Combo: A + D-pad up for 1 s")
    send("1300 1311")
    time.sleep(1)
    send("2300 2311")
    send(RELEASE_ALL)
    print("Done.")


if __name__ == "__main__":
    main()
