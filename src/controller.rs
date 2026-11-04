
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Neutral,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stick {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trigger {
    Left,
    Right,
}

pub mod button {
    pub const A: u16          = 1 << 0;
    pub const B: u16          = 1 << 1;
    pub const X: u16          = 1 << 2;
    pub const Y: u16          = 1 << 3;
    pub const LB: u16         = 1 << 4;
    pub const RB: u16         = 1 << 5;
    pub const LS: u16         = 1 << 6;
    pub const RS: u16         = 1 << 7;
    pub const MENU_RIGHT: u16 = 1 << 8;
    pub const MENU_LEFT: u16  = 1 << 9;
    pub const GUIDE: u16      = 1 << 10;

    pub const MENU_MASK: u16 =
        MENU_RIGHT | MENU_LEFT | GUIDE;
}

#[derive(Debug, Clone, Copy)]
pub struct ControllerState {
    pub buttons: u16,

    pub dpad: Direction,

    pub left_x: f32,
    pub left_y: f32,

    pub right_x: f32,
    pub right_y: f32,

    pub left_trigger: f32,
    pub right_trigger: f32,

    pub random_axis: bool,
    pub recording: bool,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            buttons: 0,
            dpad: Direction::Neutral,

            left_x: 0.0,
            left_y: 0.0,

            right_x: 0.0,
            right_y: 0.0,

            left_trigger: 0.0,
            right_trigger: 0.0,

            random_axis: false,
            recording: false,
        }
    }
}

impl ControllerState {
    pub fn set_button(&mut self, mask: u16, pressed: bool) {
        if pressed {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }
    }

    pub fn set_axis(&mut self, axis: Axis, value: f32) {
        let value = value.clamp(-1.0, 1.0);

        match axis {
            Axis::LeftX => self.left_x = value,
            Axis::LeftY => self.left_y = value,
            Axis::RightX => self.right_x = value,
            Axis::RightY => self.right_y = value,
        }
    }

    pub fn set_trigger(&mut self, trigger: Trigger, value: f32) {
        let value = value.clamp(0.0, 1.0);

        match trigger {
            Trigger::Left => self.left_trigger = value,
            Trigger::Right => self.right_trigger = value,
        }
    }

    pub fn set_stick(&mut self, stick: Stick, direction: Direction) {
        let (x, y) = match direction {
            Direction::Neutral => (0.0, 0.0),
            Direction::North => (0.0, 1.0),
            Direction::NorthEast => (0.7071, 0.7071),
            Direction::East => (1.0, 0.0),
            Direction::SouthEast => (0.7071, -0.7071),
            Direction::South => (0.0, -1.0),
            Direction::SouthWest => (-0.7071, -0.7071),
            Direction::West => (-1.0, 0.0),
            Direction::NorthWest => (-0.7071, 0.7071),
        };

        match stick {
            Stick::Left => {
                self.left_x = x;
                self.left_y = y;
            }

            Stick::Right => {
                self.right_x = x;
                self.right_y = y;
            }
        }
    }

    pub fn release_all(&mut self) {
        *self = Self::default();
    }

    pub fn release_all_except_menu(&mut self) {
        self.buttons &= button::MENU_MASK;

        self.dpad = Direction::Neutral;

        self.left_x = 0.0;
        self.left_y = 0.0;

        self.right_x = 0.0;
        self.right_y = 0.0;

        self.left_trigger = 0.0;
        self.right_trigger = 0.0;

        self.random_axis = false;
    }
}
