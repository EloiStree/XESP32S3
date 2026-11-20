use crate::controller::{Axis, ControllerState, Direction, Trigger, button};

const AXIS_CODE_START: u32 = 1_800_000_000;
const AXIS_CODE_END: u32 = 1_899_999_999;
const BUTTON_CODE_START: u32 = 1_700_000_000;
const BUTTON_CODE_END: u32 = 1_799_999_999;

/// Applies a compact Xbox input integer to the controller state.
///
/// Values starting with 18 encode four two-digit stick axes. Values starting
/// with 17 encode controller buttons and triggers as bits.
pub fn execute(state: &mut ControllerState, value: u32) -> bool {
    match value {
        AXIS_CODE_START..=AXIS_CODE_END => {
            state.set_axis(Axis::LeftX, decode_axis((value / 1_000_000) % 100));
            state.set_axis(Axis::LeftY, decode_axis((value / 10_000) % 100));
            state.set_axis(Axis::RightX, decode_axis((value / 100) % 100));
            state.set_axis(Axis::RightY, decode_axis(value % 100));
        }
        BUTTON_CODE_START..=BUTTON_CODE_END => {
            decode_buttons(state, value - BUTTON_CODE_START);
        }
        _ => return false,
    }

    true
}

fn decode_axis(value: u32) -> f32 {
    if value == 0 {
        0.0
    } else {
        (value as f32 - 50.0) / 49.0
    }
}

fn decode_buttons(state: &mut ControllerState, bits: u32) {
    state.set_button(button::Y, bits & (1 << 0) != 0);
    state.set_button(button::B, bits & (1 << 1) != 0);
    state.set_button(button::A, bits & (1 << 2) != 0);
    state.set_button(button::X, bits & (1 << 3) != 0);
    state.set_button(button::LB, bits & (1 << 8) != 0);
    state.set_button(button::RB, bits & (1 << 9) != 0);
    state.set_button(button::LS, bits & (1 << 10) != 0);
    state.set_button(button::RS, bits & (1 << 11) != 0);
    state.set_button(button::MENU_LEFT, bits & (1 << 12) != 0);
    state.set_button(button::GUIDE, bits & (1 << 13) != 0);
    state.set_button(button::MENU_RIGHT, bits & (1 << 14) != 0);

    let vertical = ((bits & (1 << 4) != 0) as i8) - ((bits & (1 << 6) != 0) as i8);
    let horizontal = ((bits & (1 << 5) != 0) as i8) - ((bits & (1 << 7) != 0) as i8);
    state.dpad = match (vertical, horizontal) {
        (1, 0) => Direction::North,
        (1, 1) => Direction::NorthEast,
        (0, 1) => Direction::East,
        (-1, 1) => Direction::SouthEast,
        (-1, 0) => Direction::South,
        (-1, -1) => Direction::SouthWest,
        (0, -1) => Direction::West,
        (1, -1) => Direction::NorthWest,
        _ => Direction::Neutral,
    };

    state.set_trigger(Trigger::Left, decode_trigger(bits, 16));
    state.set_trigger(Trigger::Right, decode_trigger(bits, 20));
}

fn decode_trigger(bits: u32, first_bit: u32) -> f32 {
    let weights = [0.10, 0.15, 0.25, 0.50];
    (0..4).fold(0.0, |trigger, offset| {
        if bits & (1u32 << (first_bit + offset)) != 0 {
            trigger + weights[offset as usize]
        } else {
            trigger
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_stick_axes_from_18_code() {
        let mut state = ControllerState::default();

        assert!(execute(&mut state, 1_850_200_010));

        assert_eq!(state.left_x, 0.0);
        assert!((state.left_y - (-30.0 / 49.0)).abs() < f32::EPSILON);
        assert_eq!(state.right_x, 0.0);
        assert!((state.right_y - (-40.0 / 49.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn maps_axis_endpoints_and_zero_to_neutral() {
        let mut state = ControllerState::default();

        assert!(execute(&mut state, 1_801_509_900));

        assert_eq!(state.left_x, -1.0);
        assert_eq!(state.left_y, 0.0);
        assert_eq!(state.right_x, 1.0);
        assert_eq!(state.right_y, 0.0);
    }

    #[test]
    fn decodes_buttons_dpad_and_triggers_from_17_code() {
        let mut state = ControllerState::default();
        let bits = (1 << 0)
            | (1 << 5)
            | (1 << 8)
            | (1 << 12)
            | (1 << 16)
            | (1 << 19)
            | (1 << 20)
            | (1 << 23);

        assert!(execute(&mut state, BUTTON_CODE_START + bits));

        assert_ne!(state.buttons & button::Y, 0);
        assert_ne!(state.buttons & button::LB, 0);
        assert_ne!(state.buttons & button::MENU_LEFT, 0);
        assert_eq!(state.dpad, Direction::East);
        assert!((state.left_trigger - 0.6).abs() < 0.001);
        assert!((state.right_trigger - 0.6).abs() < 0.001);
    }

    #[test]
    fn rejects_values_outside_compact_ranges_without_changing_state() {
        let mut state = ControllerState::default();
        let before = state;

        assert!(!execute(&mut state, 1_699_999_999));
        assert_eq!(state.buttons, before.buttons);
        assert_eq!(state.left_x, before.left_x);
    }

    
}



/*** KEEP AS DOC 
 * ## THIS IS THE ARDUINO VERSION I AM USING AND THAT I CODE MYSELF.
 * ## II KNOW THIS CODE WORK ON ARDUINO C++
 * 
 * 
 * static func gamepad_resource_to_1700000000_integer(gamepad: S2W_Data_Gamepad1817) -> int:
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
 * 
 * 
 * 
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
 * 
 * 
 */