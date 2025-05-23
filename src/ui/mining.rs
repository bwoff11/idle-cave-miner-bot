use crate::input::MINING_POSITION;
use enigo::{Enigo, MouseButton, MouseControllable};

pub struct Miner {
    enigo: Enigo,
}

impl Miner {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }

    pub fn tick(&mut self) {
        self.enigo.mouse_move_to(MINING_POSITION.x, MINING_POSITION.y);
        self.enigo.mouse_click(MouseButton::Left);
    }
}
