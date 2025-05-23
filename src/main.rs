use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use parking_lot::RwLock;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::time;

// Game positions
const MINING_POSITION: (i32, i32) = (1855, 1335);
const UPGRADE_ICON: (i32, i32) = (570, 1315);
const UPGRADES_TAB: (i32, i32) = (200, 1200);
const SOULS_TAB: (i32, i32) = (575, 1200);
const SAFE_SCROLL_AREA: (i32, i32) = (1030, 630);

// Upgrade row positions
const UPGRADE_ROWS: [(i32, i32); 6] = [
    (830, 300),
    (830, 460),
    (830, 630),
    (830, 790),
    (830, 960),
    (830, 1050),
];

const SOULS_ROWS: [(i32, i32); 7] = [
    (830, 200),
    (830, 370),
    (830, 540),
    (830, 700),
    (830, 870),
    (830, 1040),
    (830, 1050), // Bottom row after scroll
];

// Timing constants
const MINING_DELAY_MS: u64 = 50;  // Delay between mining clicks
const CLICK_DELAY_MS: u64 = 50;
const SCROLL_DELAY_MS: u64 = 50;
const POST_SCROLL_DELAY_MS: u64 = 100;
const UPGRADE_INTERVAL_SEC: u64 = 30;
const SOULS_INTERVAL_SEC: u64 = 600;
const PRESTIGE_INTERVAL_SEC: u64 = 600;

// Prestige positions
const PRESTIGE_BUTTON: (i32, i32) = (1200, 245);
const PRESTIGE_CLAIM: (i32, i32) = (1850, 1115);
const PRESTIGE_CONFIRM: (i32, i32) = (1285, 860);

#[derive(Clone)]
struct LogEntry {
    timestamp: chrono::DateTime<Local>,
    icon: &'static str,
    message: String,
    color: Color,
}

struct Bot {
    active: Arc<AtomicBool>,
    upgrades_enabled: Arc<AtomicBool>,
    souls_enabled: Arc<AtomicBool>,
    prestige_enabled: Arc<AtomicBool>,
    stats: Arc<Stats>,
    logs: Arc<RwLock<Vec<LogEntry>>>,
    last_upgrade: Arc<RwLock<Instant>>,
    last_souls: Arc<RwLock<Instant>>,
    last_prestige: Arc<RwLock<Instant>>,
}

struct Stats {
    clicks: AtomicU64,
    session_start: RwLock<Instant>,
}

impl Stats {
    fn new() -> Self {
        Self {
            clicks: AtomicU64::new(0),
            session_start: RwLock::new(Instant::now()),
        }
    }

    fn increment_clicks(&self) {
        self.clicks.fetch_add(1, Ordering::Relaxed);
    }

    fn get_clicks(&self) -> u64 {
        self.clicks.load(Ordering::Relaxed)
    }

    fn get_cpm(&self) -> u64 {
        let elapsed = self.session_start.read().elapsed().as_secs();
        if elapsed == 0 {
            0
        } else {
            (self.get_clicks() * 60) / elapsed
        }
    }

    fn get_runtime(&self) -> Duration {
        self.session_start.read().elapsed()
    }

    fn reset(&self) {
        self.clicks.store(0, Ordering::Relaxed);
        *self.session_start.write() = Instant::now();
    }
}

impl Bot {
    fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            upgrades_enabled: Arc::new(AtomicBool::new(true)),
            souls_enabled: Arc::new(AtomicBool::new(true)),
            prestige_enabled: Arc::new(AtomicBool::new(true)),
            stats: Arc::new(Stats::new()),
            logs: Arc::new(RwLock::new(Vec::new())),
            last_upgrade: Arc::new(RwLock::new(Instant::now())),
            last_souls: Arc::new(RwLock::new(Instant::now())),
            last_prestige: Arc::new(RwLock::new(Instant::now())),
        }
    }

    fn toggle(&self) {
        let was_active = self.active.fetch_xor(true, Ordering::Relaxed);
        let status = if !was_active { "ACTIVATED" } else { "PAUSED" };
        let color = if !was_active { Color::Green } else { Color::Yellow };
        self.log("⚡", format!("Bot {}", status), color);
        
        if !was_active {
            self.stats.reset();
        }
    }

    fn toggle_upgrades(&self) {
        let enabled = self.upgrades_enabled.fetch_xor(true, Ordering::Relaxed);
        let status = if !enabled { "ENABLED" } else { "DISABLED" };
        let color = if !enabled { Color::Green } else { Color::Red };
        self.log("🔧", format!("Upgrades {}", status), color);
    }

    fn toggle_souls(&self) {
        let enabled = self.souls_enabled.fetch_xor(true, Ordering::Relaxed);
        let status = if !enabled { "ENABLED" } else { "DISABLED" };
        let color = if !enabled { Color::Green } else { Color::Red };
        self.log("👻", format!("Souls {}", status), color);
    }

    fn toggle_prestige(&self) {
        let enabled = self.prestige_enabled.fetch_xor(true, Ordering::Relaxed);
        let status = if !enabled { "ENABLED" } else { "DISABLED" };
        let color = if !enabled { Color::Green } else { Color::Red };
        self.log("⭐", format!("Prestige {}", status), color);
    }

    fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn is_upgrades_enabled(&self) -> bool {
        self.upgrades_enabled.load(Ordering::Relaxed)
    }

    fn is_souls_enabled(&self) -> bool {
        self.souls_enabled.load(Ordering::Relaxed)
    }

    fn is_prestige_enabled(&self) -> bool {
        self.prestige_enabled.load(Ordering::Relaxed)
    }

    fn log(&self, icon: &'static str, message: String, color: Color) {
        let mut logs = self.logs.write();
        logs.push(LogEntry {
            timestamp: Local::now(),
            icon,
            message,
            color,
        });
        
        // Keep last 50 logs
        let logs_len = logs.len();
        if logs_len > 50 {
            logs.drain(0..logs_len - 50);
        }
    }

    async fn run_bot_loop(&self) -> Result<()> {
        let mut enigo = Enigo::new();
        let mut mining_interval = time::interval(Duration::from_millis(MINING_DELAY_MS));
        
        self.log("🚀", "Bot loop started".to_string(), Color::Blue);

        loop {
            mining_interval.tick().await;
            
            if !self.is_active() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Mining clicks
            self.perform_click(&mut enigo);
            
            // Check for scheduled tasks
            self.check_scheduled_tasks(&mut enigo).await;
        }
    }

    fn perform_click(&self, enigo: &mut Enigo) {
        enigo.mouse_move_to(MINING_POSITION.0, MINING_POSITION.1);
        enigo.mouse_click(MouseButton::Left);
        self.stats.increment_clicks();
    }

    async fn check_scheduled_tasks(&self, enigo: &mut Enigo) {
        let now = Instant::now();
        
        // Check upgrades
        if self.is_upgrades_enabled() && 
           now.duration_since(*self.last_upgrade.read()) > Duration::from_secs(UPGRADE_INTERVAL_SEC) {
            self.perform_upgrades(enigo).await;
            *self.last_upgrade.write() = now;
        }
        
        // Check souls upgrade
        if self.is_souls_enabled() && 
           now.duration_since(*self.last_souls.read()) > Duration::from_secs(SOULS_INTERVAL_SEC) {
            self.perform_souls_upgrade(enigo).await;
            *self.last_souls.write() = now;
        }
        
        // Check prestige
        if self.is_prestige_enabled() && 
           now.duration_since(*self.last_prestige.read()) > Duration::from_secs(PRESTIGE_INTERVAL_SEC) {
            self.perform_prestige(enigo).await;
            *self.last_prestige.write() = now;
        }
    }

    async fn perform_upgrades(&self, enigo: &mut Enigo) {
        self.log("🔧", "Running upgrades...".to_string(), Color::Cyan);
        
        // Open upgrades panel
        self.click_at(enigo, UPGRADE_ICON).await;
        self.click_at(enigo, UPGRADES_TAB).await;
        
        // Click all visible upgrade rows
        for (i, pos) in UPGRADE_ROWS[..5].iter().enumerate() {
            self.click_at(enigo, *pos).await;
            if i == 2 {
                // Small pause mid-way
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Scroll down and click bottom rows
        self.scroll_at(enigo, SAFE_SCROLL_AREA, -8).await;
        
        for pos in &UPGRADE_ROWS {
            self.click_at(enigo, *pos).await;
        }
        
        // Reset scroll
        self.scroll_at(enigo, SAFE_SCROLL_AREA, 8).await;
        
        self.log("✅", "Upgrades complete".to_string(), Color::Green);
    }

    async fn perform_souls_upgrade(&self, enigo: &mut Enigo) {
        self.log("👻", "Running souls upgrade...".to_string(), Color::Magenta);
        
        // Open souls panel
        self.click_at(enigo, UPGRADE_ICON).await;
        self.click_at(enigo, SOULS_TAB).await;
        
        // Click first 6 rows
        for pos in &SOULS_ROWS[..6] {
            self.click_at(enigo, *pos).await;
        }
        
        // Scroll down for last row
        self.scroll_at(enigo, SAFE_SCROLL_AREA, -2).await;
        self.click_at(enigo, SOULS_ROWS[6]).await;
        
        // Reset scroll
        self.scroll_at(enigo, SAFE_SCROLL_AREA, 2).await;
        
        self.log("✅", "Souls upgrade complete".to_string(), Color::Green);
    }

    async fn perform_prestige(&self, enigo: &mut Enigo) {
        self.log("⭐", "Running prestige...".to_string(), Color::Cyan);
        
        // Click prestige button
        self.click_at(enigo, PRESTIGE_BUTTON).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Click claim button
        self.click_at(enigo, PRESTIGE_CLAIM).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Click confirm button
        self.click_at(enigo, PRESTIGE_CONFIRM).await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        self.log("✅", "Prestige complete".to_string(), Color::Green);
    }

    async fn click_at(&self, enigo: &mut Enigo, pos: (i32, i32)) {
        enigo.mouse_move_to(pos.0, pos.1);
        tokio::time::sleep(Duration::from_millis(CLICK_DELAY_MS)).await;
        enigo.mouse_click(MouseButton::Left);
        tokio::time::sleep(Duration::from_millis(CLICK_DELAY_MS)).await;
    }

    async fn scroll_at(&self, enigo: &mut Enigo, pos: (i32, i32), amount: i32) {
        enigo.mouse_move_to(pos.0, pos.1);
        tokio::time::sleep(Duration::from_millis(SCROLL_DELAY_MS)).await;
        
        for _ in 0..amount.abs() {
            if amount > 0 {
                enigo.mouse_scroll_y(-1);
            } else {
                enigo.mouse_scroll_y(1);
            }
            tokio::time::sleep(Duration::from_millis(POST_SCROLL_DELAY_MS)).await;
        }
    }
}

struct App {
    bot: Arc<Bot>,
    should_quit: AtomicBool,
}

impl App {
    fn new() -> Self {
        Self {
            bot: Arc::new(Bot::new()),
            should_quit: AtomicBool::new(false),
        }
    }

    async fn run(&self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Start bot loop
        let bot = self.bot.clone();
        tokio::spawn(async move {
            if let Err(e) = bot.run_bot_loop().await {
                eprintln!("Bot error: {}", e);
            }
        });

        // Start input handler
        let bot = self.bot.clone();
        tokio::spawn(async move {
            let device = DeviceState::new();
            let mut last_f1 = false;
            let mut last_f2 = false;
            let mut last_f3 = false;
            let mut last_f4 = false;
            
            loop {
                let keys = device.get_keys();
                
                let f1_pressed = keys.contains(&Keycode::F1);
                if f1_pressed && !last_f1 {
                    bot.toggle();
                }
                last_f1 = f1_pressed;
                
                let f2_pressed = keys.contains(&Keycode::F2);
                if f2_pressed && !last_f2 {
                    bot.toggle_upgrades();
                }
                last_f2 = f2_pressed;
                
                let f3_pressed = keys.contains(&Keycode::F3);
                if f3_pressed && !last_f3 {
                    bot.toggle_souls();
                }
                last_f3 = f3_pressed;
                
                let f4_pressed = keys.contains(&Keycode::F4);
                if f4_pressed && !last_f4 {
                    bot.toggle_prestige();
                }
                last_f4 = f4_pressed;
                
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        // Run UI
        let res = self.run_ui(&mut terminal).await;

        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        
        res
    }

    async fn run_ui(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            terminal.draw(|f| self.draw_ui(f))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => {
                            self.should_quit.store(true, Ordering::Relaxed);
                            break;
                        }
                        KeyCode::F(1) => self.bot.toggle(),
                        KeyCode::F(2) => self.bot.toggle_upgrades(),
                        KeyCode::F(3) => self.bot.toggle_souls(),
                        KeyCode::F(4) => self.bot.toggle_prestige(),
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn draw_ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(3),  // Status
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Footer
            ])
            .split(f.area());

        self.draw_header(f, chunks[0]);
        self.draw_status(f, chunks[1]);
        self.draw_content(f, chunks[2]);
        self.draw_footer(f, chunks[3]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new("⛏️  IDLE CAVE MINER BOT v2.0")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, area);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // Status indicator
        let active = self.bot.is_active();
        let status = if active { "● ACTIVE" } else { "● PAUSED" };
        let color = if active { Color::Green } else { Color::Yellow };
        
        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_widget, chunks[0]);

        // Runtime
        let runtime = format_duration(self.bot.stats.get_runtime());
        let runtime_widget = Paragraph::new(format!("Runtime: {}", runtime))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(runtime_widget, chunks[1]);

        // Total clicks
        let clicks = self.bot.stats.get_clicks();
        let clicks_widget = Paragraph::new(format!("Clicks: {}", format_number(clicks)))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(clicks_widget, chunks[2]);

        // CPM
        let cpm = self.bot.stats.get_cpm();
        let cpm_widget = Paragraph::new(format!("{} CPM", cpm))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(cpm_widget, chunks[3]);
    }

    fn draw_content(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Task timers
        self.draw_timers(f, chunks[0]);
        
        // Logs
        self.draw_logs(f, chunks[1]);
    }

    fn draw_timers(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("⏱️  Task Timers")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(inner);

        // Upgrade timer
        let upgrade_enabled = self.bot.is_upgrades_enabled();
        let upgrade_remaining = Duration::from_secs(UPGRADE_INTERVAL_SEC)
            .saturating_sub(self.bot.last_upgrade.read().elapsed());
        let upgrade_percent = ((UPGRADE_INTERVAL_SEC - upgrade_remaining.as_secs()) * 100 
            / UPGRADE_INTERVAL_SEC) as u16;
        
        let upgrade_gauge = Gauge::default()
            .block(Block::default().title(format!("Upgrades [{}]", if upgrade_enabled { "ON" } else { "OFF" })).borders(Borders::NONE))
            .gauge_style(Style::default().fg(if upgrade_enabled { Color::Cyan } else { Color::DarkGray }))
            .percent(if upgrade_enabled { upgrade_percent } else { 0 })
            .label(if upgrade_enabled { format!("Next in: {}", format_duration(upgrade_remaining)) } else { "DISABLED".to_string() });
        f.render_widget(upgrade_gauge, chunks[0]);

        // Souls timer
        let souls_enabled = self.bot.is_souls_enabled();
        let souls_remaining = Duration::from_secs(SOULS_INTERVAL_SEC)
            .saturating_sub(self.bot.last_souls.read().elapsed());
        let souls_percent = ((SOULS_INTERVAL_SEC - souls_remaining.as_secs()) * 100 
            / SOULS_INTERVAL_SEC) as u16;
        
        let souls_gauge = Gauge::default()
            .block(Block::default().title(format!("Souls [{}]", if souls_enabled { "ON" } else { "OFF" })).borders(Borders::NONE))
            .gauge_style(Style::default().fg(if souls_enabled { Color::Magenta } else { Color::DarkGray }))
            .percent(if souls_enabled { souls_percent } else { 0 })
            .label(if souls_enabled { format!("Next in: {}", format_duration(souls_remaining)) } else { "DISABLED".to_string() });
        f.render_widget(souls_gauge, chunks[1]);

        // Prestige timer
        let prestige_enabled = self.bot.is_prestige_enabled();
        let prestige_remaining = Duration::from_secs(PRESTIGE_INTERVAL_SEC)
            .saturating_sub(self.bot.last_prestige.read().elapsed());
        let prestige_percent = ((PRESTIGE_INTERVAL_SEC - prestige_remaining.as_secs()) * 100 
            / PRESTIGE_INTERVAL_SEC) as u16;
        
        let prestige_gauge = Gauge::default()
            .block(Block::default().title(format!("Prestige [{}]", if prestige_enabled { "ON" } else { "OFF" })).borders(Borders::NONE))
            .gauge_style(Style::default().fg(if prestige_enabled { Color::Yellow } else { Color::DarkGray }))
            .percent(if prestige_enabled { prestige_percent } else { 0 })
            .label(if prestige_enabled { format!("Next in: {}", format_duration(prestige_remaining)) } else { "DISABLED".to_string() });
        f.render_widget(prestige_gauge, chunks[2]);
    }

    fn draw_logs(&self, f: &mut Frame, area: Rect) {
        let logs = self.bot.logs.read();
        let log_items: Vec<ListItem> = logs
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|entry| {
                let timestamp = entry.timestamp.format("%H:%M:%S");
                let text = format!("[{}] {} {}", timestamp, entry.icon, entry.message);
                ListItem::new(text).style(Style::default().fg(entry.color))
            })
            .collect();

        let logs_list = List::new(log_items)
            .block(Block::default().borders(Borders::ALL).title("📋 Activity Log"));
        f.render_widget(logs_list, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let help = Paragraph::new("[F1] Toggle │ [F2] Upgrades │ [F3] Souls │ [F4] Prestige │ [ESC] Exit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(help, area);
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n⛏️  IDLE CAVE MINER BOT v2.0\n");
    println!("Starting up...\n");

    let app = App::new();
    app.run().await?;

    println!("\nGoodbye!");
    Ok(())
}