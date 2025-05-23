use crate::geometry::Vec2;
use crate::input::{SAFE_SCROLL_AREA, move_and_click, scroll_at};
use enigo::Enigo;
use std::{thread, time::Duration};

pub struct Upgrades {
    pub icon: Vec2,
    pub upgrades_tab: Vec2,
    pub souls_tab: Vec2,
    pub relics_tab: Vec2,
}

impl Upgrades {
    pub fn new() -> Self {
        Self {
            icon: Vec2::new(570, 1315),       // Upgrades icon
            upgrades_tab: Vec2::new(200, 1200),
            souls_tab:    Vec2::new(575, 1200),
            relics_tab:   Vec2::new(940, 1200),
        }
    }

    pub fn open_upgrades(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.upgrades_tab);
    }

    pub fn open_souls(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.souls_tab);
    }

    pub fn open_relics(&self, enigo: &mut Enigo) {
        move_and_click(enigo, self.icon);
        move_and_click(enigo, self.relics_tab);
    }

    pub fn do_upgrades(&self, enigo: &mut Enigo) {
        self.open_upgrades(enigo);

        let top_clicks = [
            Vec2::new(830, 300),
            Vec2::new(830, 460),
            Vec2::new(830, 630),
            Vec2::new(830, 790),
            Vec2::new(830, 960),
        ];

        for pos in top_clicks {
            move_and_click(enigo, pos);
        }

        scroll_at(enigo, SAFE_SCROLL_AREA, -8); // scroll down 8 times

        let bottom_clicks = [
            Vec2::new(830, 220),
            Vec2::new(830, 380),
            Vec2::new(830, 560),
            Vec2::new(830, 720),
            Vec2::new(830, 880),
            Vec2::new(830, 1050),
        ];

        for pos in bottom_clicks {
            move_and_click(enigo, pos);
        }

        scroll_at(enigo, SAFE_SCROLL_AREA, 8); // reset scroll
    }

    pub fn do_souls_upgrade(&self, enigo: &mut Enigo) {
        self.open_souls(enigo);

        let top_rows = [
            Vec2::new(830, 200),
            Vec2::new(830, 370),
            Vec2::new(830, 540),
            Vec2::new(830, 700),
            Vec2::new(830, 870),
            Vec2::new(830, 1040),
        ];

        for pos in top_rows {
            move_and_click(enigo, pos);
        }

        // Scroll down 2 ticks to get final row in view
        scroll_at(enigo, SAFE_SCROLL_AREA, -2);

        // Final bottom row
        move_and_click(enigo, Vec2::new(830, 1050));

        // Reset scroll position
        scroll_at(enigo, SAFE_SCROLL_AREA, 2);
    }


}
