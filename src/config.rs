use crate::types::Position;
use std::time::Duration;

pub const APP_VERSION: &str = "2.0";
pub const APP_NAME: &str = "IDLE CAVE MINER BOT";

pub struct GamePositions;

impl GamePositions {
    pub const MINING: Position = Position::new(1855, 1335);
    pub const UPGRADE_ICON: Position = Position::new(570, 1315);
    pub const UPGRADES_TAB: Position = Position::new(200, 1200);
    pub const SOULS_TAB: Position = Position::new(575, 1200);
    pub const SAFE_SCROLL_AREA: Position = Position::new(1030, 630);
    pub const PRESTIGE_BUTTON: Position = Position::new(1200, 245);
    pub const PRESTIGE_CLAIM: Position = Position::new(1850, 1115);
    pub const PRESTIGE_CONFIRM: Position = Position::new(1285, 860);
}

pub struct UpgradePositions;

impl UpgradePositions {
    // Positions before scrolling - first 5 upgrade rows
    pub const BEFORE_SCROLL: [Position; 5] = [
        Position::new(830, 300),
        Position::new(830, 470),
        Position::new(830, 640),
        Position::new(830, 800),
        Position::new(830, 960),
    ];
    
    // Positions after scrolling - the Y coordinates change due to scroll offset
    pub const AFTER_SCROLL: [Position; 5] = [
        Position::new(830, 385),
        Position::new(830, 550),
        Position::new(830, 710),
        Position::new(830, 880),
        Position::new(830, 1050),
    ];
}

pub struct SoulsPositions;

impl SoulsPositions {
    // First 6 soul upgrade rows before scrolling
    pub const BEFORE_SCROLL: [Position; 6] = [
        Position::new(830, 200),
        Position::new(830, 370),
        Position::new(830, 540),
        Position::new(830, 700),
        Position::new(830, 870),
        Position::new(830, 1040),
    ];
    
    // Last row position after scrolling down
    pub const AFTER_SCROLL: Position = Position::new(830, 1050);
}

pub struct Timings;

impl Timings {
    pub const MINING_DELAY: Duration = Duration::from_millis(50);
    pub const CLICK_DELAY: Duration = Duration::from_millis(50);
    pub const SCROLL_DELAY: Duration = Duration::from_millis(50);
    pub const POST_SCROLL_DELAY: Duration = Duration::from_millis(100);
    pub const UPGRADE_INTERVAL: Duration = Duration::from_secs(30);
    pub const SOULS_INTERVAL: Duration = Duration::from_secs(600);
    pub const PRESTIGE_INTERVAL: Duration = Duration::from_secs(600);
    pub const PRESTIGE_WAIT: Duration = Duration::from_secs(1);
    pub const PRESTIGE_COMPLETE_WAIT: Duration = Duration::from_secs(3);
}

pub struct UIConfig;

impl UIConfig {
    pub const MAX_LOGS: usize = 50;
    pub const TICK_RATE: Duration = Duration::from_millis(100);
}