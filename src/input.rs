use crate::{geometry::Vec2, settings::{CLICK_DELAY_MS, POST_SCROLL_DELAY_MS}};
use enigo::{Enigo, MouseButton, MouseControllable};
use std::{thread, time::Duration};
use crate::settings::SCROLL_DELAY_MS;

pub const MINING_POSITION: Vec2 = Vec2 { x: 1855, y: 1335 };
pub const SAFE_SCROLL_AREA: Vec2 = Vec2 { x: 1030, y: 630 };

pub fn move_and_click(enigo: &mut Enigo, pos: Vec2) {
    enigo.mouse_move_to(pos.x, pos.y);
    thread::sleep(Duration::from_millis(CLICK_DELAY_MS));
    enigo.mouse_click(MouseButton::Left);
    thread::sleep(Duration::from_millis(CLICK_DELAY_MS));
}

pub fn scroll_at(enigo: &mut Enigo, pos: Vec2, amount: i32) {
    enigo.mouse_move_to(pos.x, pos.y);
    thread::sleep(Duration::from_millis(SCROLL_DELAY_MS));
    for _ in 0..amount.abs() {
        if amount > 0 {
            enigo.mouse_scroll_y(-1); // scroll down
        } else {
            enigo.mouse_scroll_y(1);  // scroll up
        }
        thread::sleep(Duration::from_millis(POST_SCROLL_DELAY_MS));
    }
}