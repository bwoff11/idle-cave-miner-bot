use crate::{
    bot::Bot,
    config::{APP_NAME, APP_VERSION, Timings},
    types::TaskType,
};
use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::Stdout;

pub struct UI {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn new(stdout: Stdout) -> Result<Self> {
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw(&mut self, bot: &Bot) -> Result<()> {
        self.terminal.draw(|f| render_ui(f, bot))?;
        Ok(())
    }
}

fn render_ui(f: &mut Frame, bot: &Bot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Status
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    render_status(f, chunks[1], bot);
    render_content(f, chunks[2], bot);
    render_footer(f, chunks[3]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(format!("⛏️  {} v{}", APP_NAME, APP_VERSION))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_status(f: &mut Frame, area: Rect, bot: &Bot) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let stats = bot.get_stats();
    
    // Status indicator
    let active = bot.is_active();
    let status = if active { "● ACTIVE" } else { "● PAUSED" };
    let color = if active { Color::Green } else { Color::Yellow };
    
    let status_widget = Paragraph::new(status)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_widget, chunks[0]);

    // Runtime
    let runtime = format_duration(stats.get_runtime());
    let runtime_widget = Paragraph::new(format!("Runtime: {}", runtime))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(runtime_widget, chunks[1]);

    // Total clicks
    let clicks = stats.get_clicks();
    let clicks_widget = Paragraph::new(format!("Clicks: {}", format_number(clicks)))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(clicks_widget, chunks[2]);

    // CPM
    let cpm = stats.get_cpm();
    let cpm_widget = Paragraph::new(format!("{} CPM", cpm))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(cpm_widget, chunks[3]);
}

fn render_content(f: &mut Frame, area: Rect, bot: &Bot) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_timers(f, chunks[0], bot);
    render_logs(f, chunks[1], bot);
}

fn render_timers(f: &mut Frame, area: Rect, bot: &Bot) {
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

    let task_manager = bot.get_task_manager();

    render_task_timer(f, chunks[0], bot, TaskType::Upgrades, &task_manager);
    render_task_timer(f, chunks[1], bot, TaskType::Souls, &task_manager);
    render_task_timer(f, chunks[2], bot, TaskType::Prestige, &task_manager);
}

fn render_task_timer(
    f: &mut Frame,
    area: Rect,
    bot: &Bot,
    task_type: TaskType,
    task_manager: &crate::bot::TaskManager,
) {
    let enabled = bot.is_task_enabled(task_type);
    let remaining = task_manager.get_time_until_next(task_type);
    
    let total_secs = match task_type {
        TaskType::Upgrades => Timings::UPGRADE_INTERVAL.as_secs(),
        TaskType::Souls => Timings::SOULS_INTERVAL.as_secs(),
        TaskType::Prestige => Timings::PRESTIGE_INTERVAL.as_secs(),
    };
    
    let percent = ((total_secs - remaining.as_secs()) * 100 / total_secs) as u16;
    let color = match task_type {
        TaskType::Upgrades => Color::Cyan,
        TaskType::Souls => Color::Magenta,
        TaskType::Prestige => Color::Yellow,
    };

    let gauge = Gauge::default()
        .block(Block::default()
            .title(format!("{} [{}]", task_type.name(), if enabled { "ON" } else { "OFF" }))
            .borders(Borders::NONE))
        .gauge_style(Style::default().fg(if enabled { color } else { Color::DarkGray }))
        .percent(if enabled { percent } else { 0 })
        .label(if enabled {
            format!("Next in: {}", format_duration(remaining))
        } else {
            "DISABLED".to_string()
        });
    f.render_widget(gauge, area);
}

fn render_logs(f: &mut Frame, area: Rect, bot: &Bot) {
    let logger = bot.get_logger();
    let entries = logger.get_entries();
    
    let log_items: Vec<ListItem> = entries
        .iter()
        .rev()
        .take(area.height as usize - 2)
        .map(|entry| {
            let timestamp = entry.timestamp.format("%H:%M:%S");
            let text = format!(
                "[{}] {} {}",
                timestamp,
                entry.level.icon(),
                entry.message
            );
            ListItem::new(text).style(Style::default().fg(entry.level.color()))
        })
        .collect();

    let logs_list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title("📋 Activity Log"));
    f.render_widget(logs_list, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let help = Paragraph::new("[F1] Toggle │ [F2] Upgrades │ [F3] Souls │ [F4] Prestige │ [ESC] Exit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, area);
}

// Utility functions
fn format_duration(d: std::time::Duration) -> String {
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