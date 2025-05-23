use crate::geometry::Vec2;
use crate::input::{SAFE_SCROLL_AREA, move_and_click, scroll_at};
use enigo::Enigo;
use std::{thread, time::Duration};

pub struct Forge {
    pub icon: Vec2,
    pub forge_button: Vec2,
    pub upgrade_button: Vec2,
    pub alchemy_button: Vec2,
}

impl Forge {
    pub fn new() -> Self {
        Self {
            icon: Vec2::new(115, 1315),
            forge_button: Vec2::new(200, 1200),
            upgrade_button: Vec2::new(575, 1200),
            alchemy_button: Vec2::new(940, 1200),
        }
    }

    pub fn open_forge(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.forge_button);
    }

    pub fn open_upgrades(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.upgrade_button);
    }

    pub fn open_alchemy(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.alchemy_button);
    }

}
