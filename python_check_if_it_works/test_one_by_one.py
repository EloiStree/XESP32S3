"""Send test commands to the ESP32-S3 Xbox 360 pad over UDP (xbox.local:3615).

Usage: python test_one_by_one.py [host]
(default host: xbox.local, with fallback IP 192.168.137.143)
Watch the OLED / UART log, or a gamepad tester, while it runs.
"""
import random
import socket
import sys
import time

PORT = 3615
DEFAULT_HOST = "xbox.local"
FALLBACK_IP = "192.168.137.143"
RELEASE_ALL = 1390



XBOX_A = 1300
XBOX_B = 1302
XBOX_X = 1301
XBOX_Y = 1303
XBOX_LB = 1304
XBOX_RB = 1305
XBOX_LEFT_STICK_CLICK = 1306
XBOX_RIGHT_STICK_CLICK = 1307
XBOX_START = 1308
XBOX_BACK = 1309
XBOX_DPAD_UP = 1311
XBOX_DPAD_RIGHT = 1313
XBOX_DPAD_DOWN = 1315
XBOX_DPAD_LEFT = 1317
XBOX_LEFT_STICK_RIGHT = 1350
XBOX_LEFT_STICK_LEFT = 1351
XBOX_LEFT_STICK_UP = 1352
XBOX_LEFT_STICK_DOWN = 1353
XBOX_RIGHT_STICK_RIGHT = 1354
XBOX_RIGHT_STICK_LEFT = 1355
XBOX_LEFT_TRIGGER = 1358
XBOX_RIGHT_TRIGGER = 1359




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

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)


def resolve(host):
    try:
        return socket.gethostbyname(host)
    except OSError:
        print(f"Could not resolve {host}, using {FALLBACK_IP}")
        return FALLBACK_IP


host = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_HOST
ip = resolve(host)
print(f"Target: {ip}:{PORT}")


def send_integer_as_utf_8(integer: int):
    as_text = str(integer) + "\n"
    sock.sendto(as_text.encode("utf-8"), (ip, PORT))


def press(int_action):
    send_integer_as_utf_8(int_action)


def release(int_action):
    send_integer_as_utf_8(int_action + 1000)


def press_and_release(press_time, between_press_time, n_time, press_code):
    for _ in range(n_time):
        press(press_code)
        time.sleep(press_time)
        release(press_code)
        time.sleep(between_press_time)

def double_press_and_release(press_time,time_between_the_two, between_double_press_time,n_times, press_code):
        for _ in range(n_times):
            press(press_code)
            time.sleep(press_time)
            release(press_code)
        time.sleep(time_between_the_two)
        for _ in range(n_times):
            press(press_code)
            time.sleep(press_time)
            release(press_code)
        time.sleep(between_double_press_time)



# Set left stick horizontal to 1.0	1350	2350
# Set left stick horizontal to -1.0	1351	2351
# Set left stick vertical to 1.0	1352	2352
# Set left stick vertical to -1.0	1353	2353
# Set right stick horizontal to 1.0	1354	2354
# Set right stick horizontal to -1.0	1355	2355
# Set right stick vertical to 1.0	1356	2356
# Set right stick vertical to -1.0	1357	2357
# Set left trigger to 100%	1358	2358
# Set right trigger to 100%	1359	2359


JOYSTICK_LEFT_FORWARD = 1352
JOYSTICK_LEFT_BACK = 1353
JOYSTICK_LEFT_RIGHT = 1350
JOYSTICK_LEFT_LEFT = 1351
JOYSTICK_RIGHT_FORWARD = 1356
JOYSTICK_RIGHT_BACK = 1357
JOYSTICK_RIGHT_RIGHT = 1354
JOYSTICK_RIGHT_LEFT = 1355

TRIGGER_LEFT = 1358
TRIGGER_RIGHT = 1359


def main():
    time.sleep(1)
    send_integer_as_utf_8(RELEASE_ALL)    

    color_number=200000000
    color_array = [990000, 9900, 99]
    color_selected_randomly = random.choice(color_array)
    color = color_number + color_selected_randomly
    time_between_commands = 0.25
    for i in range(8):
        add_index = i*1000000
        send_integer_as_utf_8(color + add_index)
        time.sleep(time_between_commands)
        #press_and_release(0.1, 0.3, 32, XBOX_DPAD_UP)  # D-pad up 8 times with 0.5s interval
    double_press_and_release(0.15, 0.15, 1.0, 4, XBOX_DPAD_UP)  # D-pad up double press with 0.1s press time and 0.3s between press time


    # press_and_release(1, 1,1, JOYSTICK_LEFT_FORWARD)
    # press_and_release(1, 1,1, JOYSTICK_LEFT_BACK)
    # press_and_release(1, 1,1, JOYSTICK_LEFT_RIGHT)
    # press_and_release(1, 1,1, JOYSTICK_LEFT_LEFT)

    # press_and_release(1, 1,1, JOYSTICK_RIGHT_FORWARD)
    # press_and_release(1, 1,1, JOYSTICK_RIGHT_BACK)
    # press_and_release(1, 1,1, JOYSTICK_RIGHT_RIGHT)
    # press_and_release(1, 1,1, JOYSTICK_RIGHT_LEFT)


    # press_and_release(1, 1,1,  TRIGGER_LEFT)

    # press_and_release(1, 1,1,  TRIGGER_RIGHT)


    # for name, press_code in BUTTONS:
    #     print(f"{name}: press {press_code}")
    #     press(press_code)
    #     time.sleep(0.6)
    #     release(press_code)
    #     time.sleep(0.2)

    # print("Combo: A + D-pad up for 1 s")
    # press(1300)
    # press(1311)
    # time.sleep(1)
    # release(1300)
    # release(1311)     
    # send_integer_as_utf_8(RELEASE_ALL)
    # print("Done.")


if __name__ == "__main__":
    main()
