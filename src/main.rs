mod config;
mod bot;
mod ui;
mod stats;
mod logger;
mod input;
mod types;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::bot::Bot;
use crate::ui::UI;
use crate::input::InputHandler;

pub struct App {
    bot: Arc<Bot>,
    should_quit: AtomicBool,
}

impl App {
    pub fn new() -> Self {
        Self {
            bot: Arc::new(Bot::new()),
            should_quit: AtomicBool::new(false),
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // Start bot loop
        let bot = self.bot.clone();
        tokio::spawn(async move {
            if let Err(e) = bot.run_loop().await {
                eprintln!("Bot error: {}", e);
            }
        });

        // Start input handler
        let bot = self.bot.clone();
        let input_handler = InputHandler::new(bot);
        tokio::spawn(async move {
            input_handler.run().await;
        });

        // Run UI
        let mut ui = UI::new(stdout)?;
        let res = self.run_ui(&mut ui).await;

        // Cleanup
        disable_raw_mode()?;
        execute!(ui.terminal.backend_mut(), LeaveAlternateScreen)?;
        
        res
    }

    async fn run_ui(&self, ui: &mut UI) -> Result<()> {
        let mut last_tick = tokio::time::Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            ui.draw(&self.bot)?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if crossterm::event::poll(timeout)? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    use crossterm::event::KeyCode;
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
                last_tick = tokio::time::Instant::now();
            }
        }

        Ok(())
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
