import socket
import time
import random


target_ip = "xbox.local"
target_ip = "192.168.137.148"
port = 3615
    

# def resolve_target(host):
#     try:
#         return socket.gethostbyname(host)
#     except OSError:
#          return "192.168.137.111"
#target_ip = resolve_target(target_ip)



print(f"Resolved target IP: {target_ip}")

array_of_bytes = bytes(random.getrandbits(8) for _ in range(1024))



time_between_step = 0.05
number_of_steps = 1

# Send UDP packet to the target IP and port.
while True:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.sendto(array_of_bytes, (target_ip, port))
        #print(f"Sending packet to {target_ip}:{port} size {len(array_of_bytes)}")
    array_of_bytes = bytes(random.getrandbits(8) for _ in range(1024))
    time.sleep(time_between_step)
    

