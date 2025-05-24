use crate::bot::Bot;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{sync::Arc, time::Duration};

pub struct InputHandler {
    bot: Arc<Bot>,
    device: DeviceState,
}

impl InputHandler {
    pub fn new(bot: Arc<Bot>) -> Self {
        Self {
            bot,
            device: DeviceState::new(),
        }
    }

    pub async fn run(&self) {
        let mut key_states = KeyStates::default();

        loop {
            let keys = self.device.get_keys();
            
            self.handle_key(&keys, Keycode::F1, &mut key_states.f1, || self.bot.toggle());
            self.handle_key(&keys, Keycode::F2, &mut key_states.f2, || self.bot.toggle_upgrades());
            self.handle_key(&keys, Keycode::F3, &mut key_states.f3, || self.bot.toggle_souls());
            self.handle_key(&keys, Keycode::F4, &mut key_states.f4, || self.bot.toggle_prestige());
            
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    fn handle_key<F>(&self, keys: &Vec<Keycode>, key: Keycode, state: &mut bool, action: F)
    where
        F: FnOnce(),
    {
        let pressed = keys.contains(&key);
        if pressed && !*state {
            action();
        }
        *state = pressed;
    }
}

#[derive(Default)]
struct KeyStates {
    f1: bool,
    f2: bool,
    f3: bool,
    f4: bool,
}