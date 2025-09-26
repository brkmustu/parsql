//! Interactive TUI module for parsql CLI

mod app;
mod command_input;
mod components;
mod event;
mod keys;
mod migration_list;
mod migration_detail;
mod help;
mod theme;
mod output_stream;
mod database;
mod migration_creator;
mod migration_loader;
mod migration_executor;
mod migration_viewer;
mod migration_content_view;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

pub use app::App;
use event::{Event, EventHandler};

/// Run the interactive TUI
pub fn run_tui(
    database_url: Option<String>,
    config: crate::config::Config,
    verbose: bool,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(database_url, config, verbose);
    
    // Create event handler
    let event_handler = EventHandler::new(250);

    // Run the app
    let res = run_app(&mut terminal, &mut app, event_handler);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| app.draw(f))?;

        // Handle events
        match event_handler.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if app.handle_key_event(key_event)? {
                    return Ok(());
                }
                // Check if app should quit
                if app.should_quit {
                    return Ok(());
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }
}