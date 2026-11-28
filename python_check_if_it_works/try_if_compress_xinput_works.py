

import socket
import struct
import time


int_release_joystick = 1800000000
int_joystick_up_right_joystick = 1899999999
int_joystick_down_left_joystick = 1801010101



button_y = True
button_b = True
button_a = True 
button_x = True
arrow_up = True
arrow_right = True
arrow_down = False
arrow_left = False
button_side_left = True
button_side_right = True
button_stick_left = True
button_stick_right = True
menu_left = False
menu_center = False
menu_right = False
kill_switch_is_connected = False

trigger_left_010= True
trigger_left_015= True
trigger_left_025= False
trigger_left_050= True

trigger_right_010= True
trigger_right_015= True
trigger_right_025= False
trigger_right_050= True


host = "192.168.137.187"
port = 3615

def compress_binary_to_17_integer():
    result = 0
    if button_y:
        result |= (1 << 0)
    if button_b:
        result |= (1 << 1)
    if button_a:
        result |= (1 << 2)
    if button_x:
        result |= (1 << 3)
    if arrow_up:
        result |= (1 << 4)
    if arrow_right:
        result |= (1 << 5)
    if arrow_down:
        result |= (1 << 6)
    if arrow_left:
        result |= (1 << 7)
    if button_side_left:
        result |= (1 << 8)
    if button_side_right:
        result |= (1 << 9)
    if button_stick_left:
        result |= (1 << 10)
    if button_stick_right:
        result |= (1 << 11)
    if menu_left:
        result |= (1 << 12)
    if menu_center:
        result |= (1 << 13)
    if menu_right:
        result |= (1 << 14)
    if kill_switch_is_connected:
        result |= (1 << 15)
    if trigger_left_010:
        result |= (1 << 16)
    if trigger_left_015:
        result |= (1 << 17)
    if trigger_left_025:
        result |= (1 << 18)
    if trigger_left_050:
        result |= (1 << 19)
    if trigger_right_010:
        result |= (1 << 20)
    if trigger_right_015:
        result |= (1 << 21)
    if trigger_right_025:
        result |= (1 << 22)
    if trigger_right_050:
        result |= (1 << 23)
    return result + 1700000000


def send_integer(value):
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.sendto(struct.pack("<iiQ", 0, value,0 ), (host, port))
        sock.close()
        


while True:
    value = compress_binary_to_17_integer()
    print("Sending compress value of button states")
    send_integer(value)
    time.sleep(1)
    print("Sending reset value")
    send_integer(1700000000)
    time.sleep(1)
    print("Sending joystick up right value")
    send_integer(int_joystick_up_right_joystick)
    time.sleep(1)
    print("Sending joystick down left value")
    send_integer(int_joystick_down_left_joystick)
    time.sleep(1)
    print("Sending joystick release value")
    send_integer(int_release_joystick)
    time.sleep(1)
    






	
# 		# Set button bits
# 		if gamepad.button_y_up:
# 			result |= (1 << 0)
# 		if gamepad.button_b_right:
# 			result |= (1 << 1)
# 		if gamepad.button_a_down:
# 			result |= (1 << 2)
# 		if gamepad.button_x_left:
# 			result |= (1 << 3)

# 		if gamepad.arrow_up:
# 			result |= (1 << 4)
# 		if gamepad.arrow_right:
# 			result |= (1 << 5)
# 		if gamepad.arrow_down:
# 			result |= (1 << 6)
# 		if gamepad.arrow_left:
# 			result |= (1 << 7)

# 		if gamepad.side_button_left:
# 			result |= (1 << 8)
# 		if gamepad.side_button_right:
# 			result |= (1 << 9)
			
# 		if gamepad.stick_joystick_left:
# 			result |= (1 << 10)
# 		if gamepad.stick_joystick_right:
# 			result |= (1 << 11)

# 		if gamepad.menu_left:
# 			result |= (1 << 12)
# 		if gamepad.menu_center:
# 			result |= (1 << 13)
# 		if gamepad.menu_right:
# 			result |= (1 << 14)
# 		if gamepad.kill_switch_is_connected:
# 			result |= (1 << 15)
		

# 		#binary order
# 		#0.10  16 20
# 		#0.15  17 21
# 		#0.25  18 22
# 		#0.50  19 23

# 		var tl = gamepad.trigger_left_axis_01_percent
# 		if tl > 0.99:
# 			result |= (1 << 16)
# 			result |= (1 << 17)
# 			result |= (1 << 18)
# 			result |= (1 << 19)
# 		if tl > 0.74:
# 			result |= (1 << 18)
# 			result |= (1 << 19)
# 		elif tl > 0.49:
# 			result |= (1 << 19)
# 		elif tl > 0.24:
# 			result |= (1 << 18)
# 		elif tl > 0.14:
# 			result |= (1 << 17)
# 		elif tl > 0.9:
# 			result |= (1 << 16)
		
# 		var tr = gamepad.trigger_right_axis_01_percent
# 		if tr > 0.99:
# 			result |= (1 << 20)
# 			result |= (1 << 21)
# 			result |= (1 << 22)
# 			result |= (1 << 23)
# 		if tr > 0.74:
# 			result |= (1 << 22)
# 			result |= (1 << 23)
# 		elif tr > 0.49:
# 			result |= (1 << 23)
# 		elif tr > 0.24:
# 			result |= (1 << 22)
# 		elif tr > 0.14:
# 			result |= (1 << 21)
# 		elif tr > 0.9:
# 			result |= (1 << 20)

# 		return result + 1700000000
#  * 
#  * 