use crate::{
    config::{GamePositions, SoulsPositions, Timings, UpgradePositions},
    logger::{LogLevel, Logger},
    stats::Stats,
    types::{Position, TaskType},
};
use anyhow::Result;
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse, Settings};
use parking_lot::RwLock;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::time;

pub struct Bot {
    state: Arc<BotState>,
    stats: Arc<Stats>,
    logger: Arc<Logger>,
    task_manager: Arc<TaskManager>,
}

struct BotState {
    active: AtomicBool,
    upgrades_enabled: AtomicBool,
    souls_enabled: AtomicBool,
    prestige_enabled: AtomicBool,
}

impl BotState {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            upgrades_enabled: AtomicBool::new(true),
            souls_enabled: AtomicBool::new(true),
            prestige_enabled: AtomicBool::new(true),
        }
    }
}

pub struct TaskManager {
    last_upgrade: RwLock<Instant>,
    last_souls: RwLock<Instant>,
    last_prestige: RwLock<Instant>,
}

impl TaskManager {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_upgrade: RwLock::new(now),
            last_souls: RwLock::new(now),
            last_prestige: RwLock::new(now),
        }
    }

    fn should_run_task(&self, task_type: TaskType) -> bool {
        let now = Instant::now();
        let elapsed = match task_type {
            TaskType::Upgrades => now.duration_since(*self.last_upgrade.read()),
            TaskType::Souls => now.duration_since(*self.last_souls.read()),
            TaskType::Prestige => now.duration_since(*self.last_prestige.read()),
        };

        let interval = match task_type {
            TaskType::Upgrades => Timings::UPGRADE_INTERVAL,
            TaskType::Souls => Timings::SOULS_INTERVAL,
            TaskType::Prestige => Timings::PRESTIGE_INTERVAL,
        };

        elapsed > interval
    }

    fn update_last_run(&self, task_type: TaskType) {
        let now = Instant::now();
        match task_type {
            TaskType::Upgrades => *self.last_upgrade.write() = now,
            TaskType::Souls => *self.last_souls.write() = now,
            TaskType::Prestige => *self.last_prestige.write() = now,
        }
    }

    pub fn get_time_until_next(&self, task_type: TaskType) -> Duration {
        let elapsed = match task_type {
            TaskType::Upgrades => self.last_upgrade.read().elapsed(),
            TaskType::Souls => self.last_souls.read().elapsed(),
            TaskType::Prestige => self.last_prestige.read().elapsed(),
        };

        let interval = match task_type {
            TaskType::Upgrades => Timings::UPGRADE_INTERVAL,
            TaskType::Souls => Timings::SOULS_INTERVAL,
            TaskType::Prestige => Timings::PRESTIGE_INTERVAL,
        };

        interval.saturating_sub(elapsed)
    }
}

impl Bot {
    pub fn new() -> Self {
        Self {
            state: Arc::new(BotState::new()),
            stats: Arc::new(Stats::new()),
            logger: Arc::new(Logger::new()),
            task_manager: Arc::new(TaskManager::new()),
        }
    }

    pub async fn run_loop(&self) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())?;
        let mut mining_interval = time::interval(Timings::MINING_DELAY);
        
        self.logger.log(LogLevel::Info, "Bot loop started");

        loop {
            mining_interval.tick().await;
            
            if !self.is_active() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            self.perform_mining_click(&mut enigo);
            self.check_and_run_tasks(&mut enigo).await;
        }
    }

    fn perform_mining_click(&self, enigo: &mut Enigo) {
        let _ = enigo.move_mouse(GamePositions::MINING.x, GamePositions::MINING.y, Coordinate::Abs);
        let _ = enigo.button(Button::Left, Direction::Click);
        self.stats.increment_clicks();
    }

    async fn check_and_run_tasks(&self, enigo: &mut Enigo) {
        if self.state.upgrades_enabled.load(Ordering::Relaxed) 
            && self.task_manager.should_run_task(TaskType::Upgrades) {
            self.perform_upgrades(enigo).await;
            self.task_manager.update_last_run(TaskType::Upgrades);
        }
        
        if self.state.souls_enabled.load(Ordering::Relaxed) 
            && self.task_manager.should_run_task(TaskType::Souls) {
            self.perform_souls_upgrade(enigo).await;
            self.task_manager.update_last_run(TaskType::Souls);
        }
        
        if self.state.prestige_enabled.load(Ordering::Relaxed) 
            && self.task_manager.should_run_task(TaskType::Prestige) {
            self.perform_prestige(enigo).await;
            self.task_manager.update_last_run(TaskType::Prestige);
        }
    }

    async fn perform_upgrades(&self, enigo: &mut Enigo) {
        self.logger.log(LogLevel::Task, "Running upgrades...");
        
        // Open upgrades panel
        self.click_at(enigo, GamePositions::UPGRADE_ICON).await;
        self.click_at(enigo, GamePositions::UPGRADES_TAB).await;
        
        // Click first 5 rows before scrolling
        for (i, pos) in UpgradePositions::BEFORE_SCROLL.iter().enumerate() {
            self.click_at(enigo, *pos).await;
            if i == 2 {
                // Small pause mid-way to ensure clicks register
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Scroll down by 8 units to reveal more upgrades
        self.scroll_at(enigo, GamePositions::SAFE_SCROLL_AREA, -8).await;
        
        // Click all rows after scrolling (positions have changed due to scroll)
        for pos in &UpgradePositions::AFTER_SCROLL {
            self.click_at(enigo, *pos).await;
        }
        
        // Reset scroll to original position
        self.scroll_at(enigo, GamePositions::SAFE_SCROLL_AREA, 8).await;
        
        self.logger.log(LogLevel::Success, "Upgrades complete");
    }

    async fn perform_souls_upgrade(&self, enigo: &mut Enigo) {
        self.logger.log(LogLevel::Task, "Running souls upgrade...");
        
        // Open souls panel
        self.click_at(enigo, GamePositions::UPGRADE_ICON).await;
        self.click_at(enigo, GamePositions::SOULS_TAB).await;
        
        // Click first 6 rows
        for pos in &SoulsPositions::BEFORE_SCROLL {
            self.click_at(enigo, *pos).await;
        }
        
        // Scroll down and click last row
        self.scroll_at(enigo, GamePositions::SAFE_SCROLL_AREA, -2).await;
        self.click_at(enigo, SoulsPositions::AFTER_SCROLL).await;
        
        // Reset scroll
        self.scroll_at(enigo, GamePositions::SAFE_SCROLL_AREA, 2).await;
        
        self.logger.log(LogLevel::Success, "Souls upgrade complete");
    }

    async fn perform_prestige(&self, enigo: &mut Enigo) {
        self.logger.log(LogLevel::Task, "Running prestige...");
        
        self.click_at(enigo, GamePositions::PRESTIGE_BUTTON).await;
        tokio::time::sleep(Timings::PRESTIGE_WAIT).await;
        
        self.click_at(enigo, GamePositions::PRESTIGE_CLAIM).await;
        tokio::time::sleep(Timings::PRESTIGE_WAIT).await;
        
        self.click_at(enigo, GamePositions::PRESTIGE_CONFIRM).await;
        tokio::time::sleep(Timings::PRESTIGE_COMPLETE_WAIT).await;
        
        self.logger.log(LogLevel::Success, "Prestige complete");
    }

    async fn click_at(&self, enigo: &mut Enigo, pos: Position) {
        let _ = enigo.move_mouse(pos.x, pos.y, Coordinate::Abs);
        tokio::time::sleep(Timings::CLICK_DELAY).await;
        let _ = enigo.button(Button::Left, Direction::Click);
        tokio::time::sleep(Timings::CLICK_DELAY).await;
    }

    async fn scroll_at(&self, enigo: &mut Enigo, pos: Position, amount: i32) {
        let _ = enigo.move_mouse(pos.x, pos.y, Coordinate::Abs);
        tokio::time::sleep(Timings::SCROLL_DELAY).await;
        
        for _ in 0..amount.abs() {
            let _ = enigo.scroll(if amount > 0 { -1 } else { 1 }, Axis::Vertical);
            tokio::time::sleep(Timings::POST_SCROLL_DELAY).await;
        }
    }

    // Public interface methods
    pub fn toggle(&self) {
        let was_active = self.state.active.fetch_xor(true, Ordering::Relaxed);
        let (status, level) = if !was_active {
            self.stats.reset();
            ("ACTIVATED", LogLevel::Success)
        } else {
            ("PAUSED", LogLevel::Warning)
        };
        self.logger.log(level, &format!("Bot {}", status));
    }

    pub fn toggle_upgrades(&self) {
        self.toggle_task(TaskType::Upgrades, &self.state.upgrades_enabled);
    }

    pub fn toggle_souls(&self) {
        self.toggle_task(TaskType::Souls, &self.state.souls_enabled);
    }

    pub fn toggle_prestige(&self) {
        self.toggle_task(TaskType::Prestige, &self.state.prestige_enabled);
    }

    fn toggle_task(&self, task_type: TaskType, enabled: &AtomicBool) {
        let was_enabled = enabled.fetch_xor(true, Ordering::Relaxed);
        let (status, level) = if !was_enabled {
            ("ENABLED", LogLevel::Success)
        } else {
            ("DISABLED", LogLevel::Error)
        };
        self.logger.log(level, &format!("{} {}", task_type.name(), status));
    }

    pub fn is_active(&self) -> bool {
        self.state.active.load(Ordering::Relaxed)
    }

    pub fn is_task_enabled(&self, task_type: TaskType) -> bool {
        match task_type {
            TaskType::Upgrades => self.state.upgrades_enabled.load(Ordering::Relaxed),
            TaskType::Souls => self.state.souls_enabled.load(Ordering::Relaxed),
            TaskType::Prestige => self.state.prestige_enabled.load(Ordering::Relaxed),
        }
    }

    pub fn get_stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    pub fn get_logger(&self) -> Arc<Logger> {
        self.logger.clone()
    }

    pub fn get_task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }
}