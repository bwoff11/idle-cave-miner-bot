use crate::geometry::Vec2;
use crate::input::move_and_click;
use enigo::Enigo;
use std::{thread, time::Duration};

pub struct Prestige {
    pub button: Vec2,
    pub claim_button: Vec2,
    pub confirm_button: Vec2,
}

impl Prestige {
    pub fn new() -> Self {
        Self {
            button: Vec2::new(1200, 245),
            claim_button: Vec2::new(1850, 1115),
            confirm_button: Vec2::new(1285, 860),
        }
    }

    pub fn do_prestige(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.button);
        thread::sleep(Duration::from_secs(1));
        move_and_click(enigo, self.claim_button);
        thread::sleep(Duration::from_secs(1));
        move_and_click(enigo, self.confirm_button);
        thread::sleep(Duration::from_secs(3));
    }
}
