
use crate::controller::{
    button, Axis, ControllerState, Direction,
    Stick, Trigger,
};

pub fn execute(state: &mut ControllerState, code: u16) -> bool {
    let pressed = match code {
        1300..=1399 => true,
        2300..=2399 => false,
        _ => return false,
    };

    let id = if pressed { code } else { code - 1000 };

    match id {
        // Buttons
        1300 => state.set_button(button::A, pressed),
        1301 => state.set_button(button::X, pressed),
        1302 => state.set_button(button::B, pressed),
        1303 => state.set_button(button::Y, pressed),
        1304 => state.set_button(button::LB, pressed),
        1305 => state.set_button(button::RB, pressed),
        1306 => state.set_button(button::LS, pressed),
        1307 => state.set_button(button::RS, pressed),
        1308 => state.set_button(button::MENU_RIGHT, pressed),
        1309 => state.set_button(button::MENU_LEFT, pressed),

        1310 => state.dpad = Direction::Neutral,

        // D-pad
        1311 => set_dpad(state, Direction::North, pressed),
        1312 => set_dpad(state, Direction::NorthEast, pressed),
        1313 => set_dpad(state, Direction::East, pressed),
        1314 => set_dpad(state, Direction::SouthEast, pressed),
        1315 => set_dpad(state, Direction::South, pressed),
        1316 => set_dpad(state, Direction::SouthWest, pressed),
        1317 => set_dpad(state, Direction::West, pressed),
        1318 => set_dpad(state, Direction::NorthWest, pressed),

        1319 => state.set_button(button::GUIDE, pressed),

        1320 => state.random_axis = pressed,
        1321 => state.recording = pressed,

        // Left stick directions
        1330 => state.set_stick(Stick::Left, Direction::Neutral),
        1331 => set_stick(state, Stick::Left, Direction::North, pressed),
        1332 => set_stick(state, Stick::Left, Direction::NorthEast, pressed),
        1333 => set_stick(state, Stick::Left, Direction::East, pressed),
        1334 => set_stick(state, Stick::Left, Direction::SouthEast, pressed),
        1335 => set_stick(state, Stick::Left, Direction::South, pressed),
        1336 => set_stick(state, Stick::Left, Direction::SouthWest, pressed),
        1337 => set_stick(state, Stick::Left, Direction::West, pressed),
        1338 => set_stick(state, Stick::Left, Direction::NorthWest, pressed),

        // Right stick directions
        1340 => state.set_stick(Stick::Right, Direction::Neutral),
        1341 => set_stick(state, Stick::Right, Direction::North, pressed),
        1342 => set_stick(state, Stick::Right, Direction::NorthEast, pressed),
        1343 => set_stick(state, Stick::Right, Direction::East, pressed),
        1344 => set_stick(state, Stick::Right, Direction::SouthEast, pressed),
        1345 => set_stick(state, Stick::Right, Direction::South, pressed),
        1346 => set_stick(state, Stick::Right, Direction::SouthWest, pressed),
        1347 => set_stick(state, Stick::Right, Direction::West, pressed),
        1348 => set_stick(state, Stick::Right, Direction::NorthWest, pressed),

        // Full axis values
        1350 => set_axis(state, Axis::LeftX, 1.0, pressed),
        1351 => set_axis(state, Axis::LeftX, -1.0, pressed),
        1352 => set_axis(state, Axis::LeftY, 1.0, pressed),
        1353 => set_axis(state, Axis::LeftY, -1.0, pressed),
        1354 => set_axis(state, Axis::RightX, 1.0, pressed),
        1355 => set_axis(state, Axis::RightX, -1.0, pressed),
        1356 => set_axis(state, Axis::RightY, 1.0, pressed),
        1357 => set_axis(state, Axis::RightY, -1.0, pressed),
        1358 => set_trigger(state, Trigger::Left, 1.0, pressed),
        1359 => set_trigger(state, Trigger::Right, 1.0, pressed),

        // 75%
        1360 => set_axis(state, Axis::LeftX, 0.75, pressed),
        1361 => set_axis(state, Axis::LeftX, -0.75, pressed),
        1362 => set_axis(state, Axis::LeftY, 0.75, pressed),
        1363 => set_axis(state, Axis::LeftY, -0.75, pressed),
        1364 => set_axis(state, Axis::RightX, 0.75, pressed),
        1365 => set_axis(state, Axis::RightX, -0.75, pressed),
        1366 => set_axis(state, Axis::RightY, 0.75, pressed),
        1367 => set_axis(state, Axis::RightY, -0.75, pressed),
        1368 => set_trigger(state, Trigger::Left, 0.75, pressed),
        1369 => set_trigger(state, Trigger::Right, 0.75, pressed),

        // 50%
        1370 => set_axis(state, Axis::LeftX, 0.50, pressed),
        1371 => set_axis(state, Axis::LeftX, -0.50, pressed),
        1372 => set_axis(state, Axis::LeftY, 0.50, pressed),
        1373 => set_axis(state, Axis::LeftY, -0.50, pressed),
        1374 => set_axis(state, Axis::RightX, 0.50, pressed),
        1375 => set_axis(state, Axis::RightX, -0.50, pressed),
        1376 => set_axis(state, Axis::RightY, 0.50, pressed),
        1377 => set_axis(state, Axis::RightY, -0.50, pressed),
        1378 => set_trigger(state, Trigger::Left, 0.50, pressed),
        1379 => set_trigger(state, Trigger::Right, 0.50, pressed),

        // 25%
        1380 => set_axis(state, Axis::LeftX, 0.25, pressed),
        1381 => set_axis(state, Axis::LeftX, -0.25, pressed),
        1382 => set_axis(state, Axis::LeftY, 0.25, pressed),
        1383 => set_axis(state, Axis::LeftY, -0.25, pressed),
        1384 => set_axis(state, Axis::RightX, 0.25, pressed),
        1385 => set_axis(state, Axis::RightX, -0.25, pressed),
        1386 => set_axis(state, Axis::RightY, 0.25, pressed),
        1387 => set_axis(state, Axis::RightY, -0.25, pressed),
        1388 => set_trigger(state, Trigger::Left, 0.25, pressed),
        1389 => set_trigger(state, Trigger::Right, 0.25, pressed),

        // Global release operations
        1390 => state.release_all(),
        1391 => state.release_all_except_menu(),

        _ => return false,
    }

    true
}

fn set_dpad(
    state: &mut ControllerState,
    direction: Direction,
    pressed: bool,
) {
    if pressed {
        state.dpad = direction;
    } else if state.dpad == direction {
        state.dpad = Direction::Neutral;
    }
}

fn set_stick(
    state: &mut ControllerState,
    stick: Stick,
    direction: Direction,
    pressed: bool,
) {
    if pressed {
        state.set_stick(stick, direction);
    } else {
        state.set_stick(stick, Direction::Neutral);
    }
}

fn set_axis(
    state: &mut ControllerState,
    axis: Axis,
    value: f32,
    pressed: bool,
) {
    state.set_axis(axis, if pressed { value } else { 0.0 });
}

fn set_trigger(
    state: &mut ControllerState,
    trigger: Trigger,
    value: f32,
    pressed: bool,
) {
    state.set_trigger(trigger, if pressed { value } else { 0.0 });
}
