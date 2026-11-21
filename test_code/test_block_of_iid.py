import socket
import struct
import time


target_ip = "xbox.local"
target_ip = "192.168.137.111"
port = 3615
    

# def resolve_target(host):
#     try:
#         return socket.gethostbyname(host)
#     except OSError:
#          return "192.168.137.111"
#target_ip = resolve_target(target_ip)



print(f"Resolved target IP: {target_ip}")
array_of_bytes = b"".join([
    struct.pack("<iiQ", 0, 1311, 100),  # index=1, value=2, delay=16
    struct.pack("<iiQ", 0, 2311, 200),  # index=3, value=4, delay=32
    struct.pack("<iiQ", 0, 1311, 300),  # index=1, value=2, delay=16
    struct.pack("<iiQ", 0, 2311, 400),  # index=3, value=4, delay=32
    struct.pack("<iiQ", 0, 1311, 500),  # index=1, value=2, delay=16
    struct.pack("<iiQ", 0, 2311, 600),  # index=3, value=4, delay=32
    struct.pack("<iiQ", 0, 1311, 700),  # index=1, value=2, delay=16
    struct.pack("<iiQ", 0, 2311, 800),  # index=3, value=4, delay=32
])


# Send UDP packet to the target IP and port.
while True:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.sendto(array_of_bytes, (target_ip, port))
        print(f"Sending packet to {target_ip}:{port} size {len(array_of_bytes)}")
    time.sleep(2)
