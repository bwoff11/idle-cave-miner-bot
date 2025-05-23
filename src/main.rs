mod geometry;
mod input;
mod ui;
mod scheduler;
mod settings;

use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::Enigo;
use scheduler::{Scheduler, TimedTask};
use std::thread;
use std::time::Duration;
use ui::mining::Miner;
use ui::upgrades::Upgrades;
use std::sync::{Arc, Mutex};
use crate::settings::{UPGRADE_INTERVAL_SEC, PRESTIGE_INTERVAL_SEC};

fn main() {
    let device = DeviceState::new();
    let mut miner = Miner::new();
    let mut scheduler = Scheduler::new();

    let upgrades = Arc::new(Mutex::new(Upgrades::new()));
    let prestige = Arc::new(Mutex::new(ui::prestige::Prestige::new()));

    let mut enabled = false;
    let mut last_f1 = false;

    {
        let upgrades = upgrades.clone();
        scheduler.add_task(TimedTask::new(UPGRADE_INTERVAL_SEC, move || {
            println!("Running upgrades...");
            upgrades.lock().unwrap().do_upgrades(&mut Enigo::new());
        }));
    }

    /*{
        let prestige = prestige.clone();
        scheduler.add_task(TimedTask::new(PRESTIGE_INTERVAL_SEC, move || {
            println!("Running prestige...");
            prestige.lock().unwrap().do_prestige(&mut Enigo::new());
        }));
    }*/

    {
        let upgrades = upgrades.clone();
        scheduler.add_task(TimedTask::new(PRESTIGE_INTERVAL_SEC, move || {
            println!("Running souls upgrade...");
            upgrades.lock().unwrap().do_souls_upgrade(&mut Enigo::new());
        }));
    }

    println!("Press F1 to toggle automation. Press Esc to quit.");

    loop {
        let keys = device.get_keys();

        let f1_pressed = keys.contains(&Keycode::F1);
        if f1_pressed && !last_f1 {
            enabled = !enabled;
            println!("Automation toggled: {}", enabled);
        }
        last_f1 = f1_pressed;

        if keys.contains(&Keycode::Escape) {
            println!("Exiting.");
            break;
        }

        if enabled {
            miner.tick();
            scheduler.tick();
        }

        thread::sleep(Duration::from_millis(1));
    }
}
