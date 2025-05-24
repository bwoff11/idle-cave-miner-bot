use chrono::{DateTime, Local};
use parking_lot::RwLock;
use ratatui::style::Color;
use crate::config::UIConfig;

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Task,
}

impl LogLevel {
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Info => Color::Blue,
            LogLevel::Success => Color::Green,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::Red,
            LogLevel::Task => Color::Cyan,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "🚀",
            LogLevel::Success => "✅",
            LogLevel::Warning => "⚡",
            LogLevel::Error => "❌",
            LogLevel::Task => "🔧",
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

pub struct Logger {
    entries: RwLock<Vec<LogEntry>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let mut entries = self.entries.write();
        entries.push(LogEntry {
            timestamp: Local::now(),
            level,
            message: message.to_string(),
        });

        // Keep only the last N entries
        if entries.len() > UIConfig::MAX_LOGS {
            let excess = entries.len().saturating_sub(UIConfig::MAX_LOGS);
            entries.drain(0..excess);
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.read().clone()
    }
}