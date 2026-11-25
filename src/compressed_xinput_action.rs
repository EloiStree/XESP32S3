




/// Gamepad state used by the compressed action format.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadState {
    pub button_y_up: bool,
    pub button_b_right: bool,
    pub button_a_down: bool,
    pub button_x_left: bool,
    pub arrow_up: bool,
    pub arrow_right: bool,
    pub arrow_down: bool,
    pub arrow_left: bool,
    pub side_button_left: bool,
    pub side_button_right: bool,
    pub stick_joystick_left: bool,
    pub stick_joystick_right: bool,
    pub menu_left: bool,
    pub menu_center: bool,
    pub menu_right: bool,
    pub kill_switch_is_connected: bool,
    pub trigger_left_axis_01_percent: f32,
    pub trigger_right_axis_01_percent: f32,
    pub left_horizontal_percent: f32,
    pub left_vertical_percent: f32,
    pub right_horizontal_percent: f32,
    pub right_vertical_percent: f32,
}

const ACTION_BASE: i32 = 1_700_000_000;
const STICK_BASE: i32 = 1_800_000_000;

fn set_bit(value: &mut i32, bit: u32, enabled: bool) {
    if enabled {
        *value |= 1_i32 << bit;
    }
}

fn trigger_bits(value: f32) -> i32 {
    let value = value.clamp(0.0, 1.0);
    let mut bits = 0;
    if value > 0.14 {
        bits |= 1 << 17;
    }
    if value > 0.24 {
        bits |= 1 << 18;
    }
    if value > 0.49 {
        bits |= 1 << 19;
    }
    if value > 0.74 {
        bits |= 1 << 18;
        bits |= 1 << 19;
    }
    if value > 0.99 {
        bits |= 1 << 16;
        bits |= 1 << 17;
        bits |= 1 << 18;
        bits |= 1 << 19;
    }
    bits
}

/// Encode buttons and triggers into the 1_700_000_000 action format.
pub fn gamepad_to_action(gamepad: &GamepadState) -> i32 {
    let mut result = 0;
    let buttons = [
        gamepad.button_y_up,
        gamepad.button_b_right,
        gamepad.button_a_down,
        gamepad.button_x_left,
        gamepad.arrow_up,
        gamepad.arrow_right,
        gamepad.arrow_down,
        gamepad.arrow_left,
        gamepad.side_button_left,
        gamepad.side_button_right,
        gamepad.stick_joystick_left,
        gamepad.stick_joystick_right,
        gamepad.menu_left,
        gamepad.menu_center,
        gamepad.menu_right,
        gamepad.kill_switch_is_connected,
    ];
    for (bit, enabled) in buttons.into_iter().enumerate() {
        set_bit(&mut result, bit as u32, enabled);
    }
    result |= trigger_bits(gamepad.trigger_left_axis_01_percent);
    result |= trigger_bits(gamepad.trigger_right_axis_01_percent) << 4;
    ACTION_BASE + result
}

fn decode_trigger(value: i32, shift: u32) -> f32 {
    let mut result = 0.0;
    if value & (1 << shift) != 0 {
        result += 0.10;
    }
    if value & (1 << (shift + 1)) != 0 {
        result += 0.15;
    }
    if value & (1 << (shift + 2)) != 0 {
        result += 0.25;
    }
    if value & (1 << (shift + 3)) != 0 {
        result += 0.50;
    }
    result.min(1.0)
}

/// Decode the 1_700_000_000 action format.
pub fn action_to_gamepad(value: i32) -> Option<GamepadState> {
    if !(ACTION_BASE..ACTION_BASE + 100_000_000).contains(&value) {
        return None;
    }
    let value = value - ACTION_BASE;
    let bit = |index: u32| value & (1 << index) != 0;
    Some(GamepadState {
        button_y_up: bit(0),
        button_b_right: bit(1),
        button_a_down: bit(2),
        button_x_left: bit(3),
        arrow_up: bit(4),
        arrow_right: bit(5),
        arrow_down: bit(6),
        arrow_left: bit(7),
        side_button_left: bit(8),
        side_button_right: bit(9),
        stick_joystick_left: bit(10),
        stick_joystick_right: bit(11),
        menu_left: bit(12),
        menu_center: bit(13),
        menu_right: bit(14),
        kill_switch_is_connected: bit(15),
        trigger_left_axis_01_percent: decode_trigger(value, 16),
        trigger_right_axis_01_percent: decode_trigger(value, 20),
        ..GamepadState::default()
    })
}

fn percent_to_1_99(value: f32) -> i32 {
    (value.clamp(-1.0, 1.0) * 99.0).round() as i32
}

/// Encode the four stick axes into the 1_800_000_000 format.
pub fn sticks_to_action(gamepad: &GamepadState) -> i32 {
    let values = [
        percent_to_1_99(gamepad.left_horizontal_percent),
        percent_to_1_99(gamepad.left_vertical_percent),
        percent_to_1_99(gamepad.right_horizontal_percent),
        percent_to_1_99(gamepad.right_vertical_percent),
    ];
    STICK_BASE + values[0] * 1_000_000 + values[1] * 10_000 + values[2] * 100 + values[3]
}

/// Decode the 1_800_000_000 stick format.
pub fn action_to_sticks(value: i32) -> Option<GamepadState> {
    if !(STICK_BASE..STICK_BASE + 100_000_000).contains(&value) {
        return None;
    }
    let value = value - STICK_BASE;
    let decode = |divisor: i32| (value / divisor % 100) as f32 / 99.0;
    Some(GamepadState {
        left_horizontal_percent: decode(1_000_000),
        left_vertical_percent: decode(10_000),
        right_horizontal_percent: decode(100),
        right_vertical_percent: decode(1),
        ..GamepadState::default()
    })
}







/*** 
static func gamepad_resource_to_1700000000_integer(gamepad: S2W_Data_Gamepad1817) -> int:

    else if(value>=1700000000 && value<=1799999999){
		var result: int = 0
		
		# Set button bits
		if gamepad.button_y_up:
			result |= (1 << 0)
		if gamepad.button_b_right:
			result |= (1 << 1)
		if gamepad.button_a_down:
			result |= (1 << 2)
		if gamepad.button_x_left:
			result |= (1 << 3)

		if gamepad.arrow_up:
			result |= (1 << 4)
		if gamepad.arrow_right:
			result |= (1 << 5)
		if gamepad.arrow_down:
			result |= (1 << 6)
		if gamepad.arrow_left:
			result |= (1 << 7)

		if gamepad.side_button_left:
			result |= (1 << 8)
		if gamepad.side_button_right:
			result |= (1 << 9)
			
		if gamepad.stick_joystick_left:
			result |= (1 << 10)
		if gamepad.stick_joystick_right:
			result |= (1 << 11)

		if gamepad.menu_left:
			result |= (1 << 12)
		if gamepad.menu_center:
			result |= (1 << 13)
		if gamepad.menu_right:
			result |= (1 << 14)
		if gamepad.kill_switch_is_connected:
			result |= (1 << 15)
		

		#binary order
		#0.10  16 20
		#0.15  17 21
		#0.25  18 22
		#0.50  19 23

		var tl = gamepad.trigger_left_axis_01_percent
		if tl > 0.99:
			result |= (1 << 16)
			result |= (1 << 17)
			result |= (1 << 18)
			result |= (1 << 19)
		if tl > 0.74:
			result |= (1 << 18)
			result |= (1 << 19)
		elif tl > 0.49:
			result |= (1 << 19)
		elif tl > 0.24:
			result |= (1 << 18)
		elif tl > 0.14:
			result |= (1 << 17)
		elif tl > 0.9:
			result |= (1 << 16)
		
		var tr = gamepad.trigger_right_axis_01_percent
		if tr > 0.99:
			result |= (1 << 20)
			result |= (1 << 21)
			result |= (1 << 22)
			result |= (1 << 23)
		if tr > 0.74:
			result |= (1 << 22)
			result |= (1 << 23)
		elif tr > 0.49:
			result |= (1 << 23)
		elif tr > 0.24:
			result |= (1 << 22)
		elif tr > 0.14:
			result |= (1 << 21)
		elif tr > 0.9:
			result |= (1 << 20)

		return result + 1700000000



    
    else if(value>=1800000000 && value<=1899999999){

      //18 50 20 00 10
      //1850200010
      //4 bytes because integer
      int left_horizontal_from_1_to_99 =   (value/1000000)%100;
      int left_vertical_from_1_to_99 =     (value/10000)%100;
      int right_horizontal_from_1_to_99 =  (value/100)%100;
      int right_vertical_from_1_to_99 =    (value/1)%100;
      float left_horizontal_percent= IntAndBinaryUtility::turn_from_1_to_99_as_percent(left_horizontal_from_1_to_99);
      float left_vertical_percent= IntAndBinaryUtility::turn_from_1_to_99_as_percent(left_vertical_from_1_to_99);
      float right_horizontal_percent= IntAndBinaryUtility::turn_from_1_to_99_as_percent(right_horizontal_from_1_to_99);
      float right_vertical_percent= IntAndBinaryUtility::turn_from_1_to_99_as_percent(right_vertical_from_1_to_99);
      gamepad()->set_left_horizontal_percent(left_horizontal_percent);
      gamepad()->set_left_vertical_percent(left_vertical_percent);
      gamepad()->set_right_horizontal_percent(right_horizontal_percent);
      gamepad()->set_right_vertical_percent(right_vertical_percent );
    }
    else if(value>=1700000000 && value<=1799999999){
            m_binaryBufferOfInteger[33]; // Buffer to store the binary representation (32 bits + null terminator)
            //IntAndBinaryUtility::int_to_binary_buffer(value, m_binaryBufferOfInteger, 33);
            //Serial.println(m_binaryBufferOfInteger);
            value=value-1700000000;
            IntAndBinaryUtility::int_to_binary_buffer(value,m_binaryBufferOfInteger,33);
            //Serial.println(m_binaryBufferOfInteger);


            float triggerLeft=0.0;
            float triggerRight=0.0;
            float arrowHorizontal=0;
            float arrowVertical =0;
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 0)) gamepad()->press_y(true); else gamepad()->press_y(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 1)) gamepad()->press_b(true); else gamepad()->press_b(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 2)) gamepad()->press_a(true); else gamepad()->press_a(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 3)) gamepad()->press_x(true); else gamepad()->press_x(false);

            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 4)) arrowVertical+=1; // CLOCK WISE N
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 5)) arrowHorizontal+=1; // CLOCK WISE E
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 6)) arrowVertical+=-1; // CLOCK WISE S
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 7)) arrowHorizontal+=-1; //// CLOCK WISE W

            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 8)) gamepad()->press_left_side_button(true);            else gamepad()->press_left_side_button(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 9)) gamepad()->press_right_side_button(true);            else gamepad()->press_right_side_button(false);

            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 10)) gamepad()->press_left_stick(true);  else gamepad()->press_left_stick(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 11)) gamepad()->press_right_stick(true);  else gamepad()->press_right_stick(false);


            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 12)) gamepad()->press_menu_left(true);        else gamepad()->press_menu_left(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 13)) gamepad()->press_home_xbox_button(true);    else gamepad()->press_home_xbox_button(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 14)) gamepad()->press_menu_right(true);        else gamepad()->press_menu_right(false);
            if(IntAndBinaryUtility::is_integer_bit_right_to_left_true (value, 15)) {/*KILL SWITCH ON*/}        else /*KILL SWITCH OFF*/;

          //Decode trigger values from bits
          //Left trigger: bits 16-19
          //Right trigger: bits 20-23
          //Pattern: 0.10 (16/20), 0.15 (17/21), 0.25 (18/22), 0.50 (19/23)          
          //This is the decoding logic (inverse of encoding above)
          // For reference when implementing the decode function:
           var trigger_left = 0.0
           var trigger_right = 0.0
           if is_integer_bit_right_to_left_true(value, 16): trigger_left += 0.10
           if is_integer_bit_right_to_left_true(value, 17): trigger_left += 0.15
           if is_integer_bit_right_to_left_true(value, 18): trigger_left += 0.25
           if is_integer_bit_right_to_left_true(value, 19): trigger_left += 0.50
           if is_integer_bit_right_to_left_true(value, 20): trigger_right += 0.10
           if is_integer_bit_right_to_left_true(value, 21): trigger_right += 0.15
           if is_integer_bit_right_to_left_true(value, 22): trigger_right += 0.25
           if is_integer_bit_right_to_left_true(value, 23): trigger_right += 0.50

            gamepad()->set_trigger_left_percent(triggerLeft);
            gamepad()->set_trigger_right_percent(triggerRight);

            if(arrowVertical==1 && arrowHorizontal==0)
                 gamepad()->press_arrow_n();
            else if(arrowVertical==1 && arrowHorizontal==1)
                gamepad()->press_arrow_ne();
            else if(arrowVertical==0 && arrowHorizontal==1)
                gamepad()->press_arrow_e();
            else if(arrowVertical==-1 && arrowHorizontal==1)
                gamepad()->press_arrow_se();
            else if(arrowVertical==-1 && arrowHorizontal==0)
                gamepad()->press_arrow_s();
            else if(arrowVertical==-1 && arrowHorizontal==-1)
                gamepad()->press_arrow_sw();
            else if(arrowVertical==0 && arrowHorizontal==-1)
                gamepad()->press_arrow_w();
            else if(arrowVertical==1 && arrowHorizontal==-1)
                gamepad()->press_arrow_nw();
            else
                gamepad()->release_dpad();
          }
      }



      */