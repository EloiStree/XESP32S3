import socket
import struct
import time


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

def wheel(pos):
    pos = pos % 256
    if pos < 85:
        return (pos * 3, 255 - pos * 3, 0)
    elif pos < 170:
        pos -= 85
        return (255 - pos * 3, 0, pos * 3)
    else:
        pos -= 170
        return (0, pos * 3, 255 - pos * 3)

num_leds = 100
array_of_bytes = b"".join([
    bytes(wheel(int(i * 256 / num_leds)))
    for i in range(num_leds)
])



time_between_step = 0.1
number_of_steps = 10

# Send UDP packet to the target IP and port.
while True:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.sendto(array_of_bytes, (target_ip, port))
        #print(f"Sending packet to {target_ip}:{port} size {len(array_of_bytes)}")
    for i in range(number_of_steps):
        array_of_bytes = array_of_bytes[3:] + array_of_bytes[:3]
    time.sleep(time_between_step)
    

