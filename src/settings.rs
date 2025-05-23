use crate::geometry::Vec2;

// Delay in ms after each mouse click to let UI register
pub const CLICK_DELAY_MS: u64 = 50;

// Delay in ms between each scroll tick
pub const SCROLL_DELAY_MS: u64 = 50;
pub const POST_SCROLL_DELAY_MS: u64 = 100;

// Safe scroll hover position
pub const SAFE_SCROLL_AREA: Vec2 = Vec2 { x: 1000, y: 700 };

// Mining click target
pub const MINING_POSITION: Vec2 = Vec2 { x: 1855, y: 1335 };

// Example panel tab positions
pub const FORGE_ICON: Vec2 = Vec2 { x: 115, y: 1315 };
pub const MINER_ICON: Vec2 = Vec2 { x: 350, y: 1315 };
pub const UPGRADE_ICON: Vec2 = Vec2 { x: 570, y: 1315 };

// Upgrade tab buttons
pub const TAB_FORGE: Vec2 = Vec2 { x: 200, y: 1200 };
pub const TAB_UPGRADES: Vec2 = Vec2 { x: 575, y: 1200 };
pub const TAB_ALCHEMY: Vec2 = Vec2 { x: 940, y: 1200 };

// Click grid column for upgrade rows
pub const UPGRADE_COLUMN_X: i32 = 830;

// Intervals
pub const UPGRADE_INTERVAL_SEC: u64 = 30;
pub const PRESTIGE_INTERVAL_SEC: u64 = 600;