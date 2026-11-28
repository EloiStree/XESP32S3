import socket
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError


target_ip = "192.168.137.187"
port = 3615


print(f"Resolved target IP: {target_ip}")

width = 200
height = 200
border_width = 20

# 1 bit per pixel, with the most-significant bit representing the leftmost pixel.
array_of_bytes = bytearray([0xff] * (width * height // 8))

for y in range(height):
    for x in range(width):
        if x < border_width or x >= width - border_width or y < border_width or y >= height - border_width:
            byte_index = (y * width + x) // 8
            bit_index = 7 - ((y * width + x) % 8)
            array_of_bytes[byte_index] &= ~(1 << bit_index)


# Draw a centered happy face by clearing its 1-bit pixels.
time_between_fresh = 60

# Compact 3x5 bitmap font for the time display.
font = {
    "0": ("111", "101", "101", "101", "111"),
    "1": ("010", "110", "010", "010", "111"),
    "2": ("111", "001", "111", "100", "111"),
    "3": ("111", "001", "111", "001", "111"),
    "4": ("101", "101", "111", "001", "001"),
    "5": ("111", "100", "111", "001", "111"),
    "6": ("111", "100", "111", "101", "111"),
    "7": ("111", "001", "010", "010", "010"),
    "8": ("111", "101", "111", "101", "111"),
    "9": ("111", "101", "111", "001", "111"),
    "U": ("101", "101", "101", "101", "111"),
    "T": ("111", "010", "010", "010", "010"),
    "C": ("111", "100", "100", "100", "111"),
    "S": ("111", "100", "111", "001", "111"),
    "A": ("010", "101", "111", "101", "101"),
    "B": ("110", "101", "110", "101", "110"),
    "E": ("111", "100", "110", "100", "111"),
    "L": ("100", "100", "100", "100", "111"),
    "J": ("001", "001", "001", "101", "111"),
    "P": ("110", "101", "110", "100", "100"),
    "N": ("101", "111", "111", "111", "101"),
    ":": ("000", "010", "000", "010", "000"),
    " ": ("000", "000", "000", "000", "000"),
}
base_image = bytes(array_of_bytes)


def draw_text(image, x, y, text):
    for character in text:
        glyph = font[character]
        for glyph_y, row in enumerate(glyph):
            for glyph_x, pixel in enumerate(row):
                if pixel == "1":
                    for scale_y in range(2):
                        for scale_x in range(2):
                            pixel_x = x + glyph_x * 2 + scale_x
                            pixel_y = y + glyph_y * 2 + scale_y
                            pixel_index = pixel_y * width + pixel_x
                            image[pixel_index // 8] &= ~(1 << (7 - pixel_index % 8))
        x += 8

# Windows may not have an IANA time zone database; install tzdata if needed.
try:
    time_zones = (
        ("UTC", timezone.utc),
        ("USA", ZoneInfo("America/New_York")),  # Eastern Time
        ("BEL", ZoneInfo("Europe/Brussels")),
        ("JPN", ZoneInfo("Asia/Tokyo")),
    )
except ZoneInfoNotFoundError as exc:
    raise SystemExit(
        "Time zone data is missing. Install it with: python -m pip install tzdata"
    ) from exc

# Send UDP packet to the target IP and port.
while True:

    # Send a random pixel array before the clock frame.
    random_frame = bytearray(__import__("random").getrandbits(8 * (width * height // 8)).to_bytes(width * height // 8, "big"))
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.sendto(bytes(random_frame), (target_ip, port))

    time.sleep(5  )
    frame = bytearray(base_image)
    for row, (label, zone) in enumerate(time_zones):
        current_time = datetime.now(zone).strftime("%H:%M")
        draw_text(frame, 24, 24 + row * 16, f"{label} {current_time}")

    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.sendto(bytes(frame), (target_ip, port))
    time.sleep(time_between_fresh   )

    

