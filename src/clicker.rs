use device_query::Keycode;
use enigo::{Enigo, MouseButton, MouseControllable};
use std::{thread, time::Duration};

use crate::input::MINING_POSITION;

pub struct Clicker {
    enigo: Enigo,
    active: bool,
    last_toggle: bool,
    delay: Duration,
}

impl Clicker {
    pub fn new(clicks_per_second: u64) -> Self {
        Self {
            enigo: Enigo::new(),
            active: false,
            last_toggle: false,
            delay: Duration::from_millis(1000 / clicks_per_second),
        }
    }

    pub fn handle_input(&mut self, keys: &[Keycode]) -> bool {
        self.toggle(keys);
        if keys.contains(&Keycode::Escape) {
            println!("Exiting.");
            return false;
        }
        true
    }

    pub fn tick(&mut self) {
        if self.active {
            self.enigo.mouse_move_to(MINING_POSITION.x, MINING_POSITION.y);
            self.enigo.mouse_click(MouseButton::Left);
        }
        thread::sleep(self.delay);
    }

    fn toggle(&mut self, keys: &[Keycode]) {
        let f1_pressed = keys.contains(&Keycode::F1);
        if f1_pressed && !self.last_toggle {
            self.active = !self.active;
            println!("Mining toggled: {}", self.active);
        }
        self.last_toggle = f1_pressed;
    }
}
