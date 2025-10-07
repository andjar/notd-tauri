use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use outliner_tui::{App, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new("outliner.db")?;
    
    // Initialize with sample data if needed
    app.initialize_sample_data()?;
    
    // Load the first note
    app.load_first_note()?;

    // Create event handler
    let event_handler = EventHandler::new(250); // 250ms tick rate

    // Main loop
    let result = run_app(&mut terminal, &mut app, &event_handler);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print result
    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| outliner_tui::ui::render(f, app))?;

        // Handle events
        let event = event_handler.next()?;
        match event {
            outliner_tui::Event::Key(key) => {
                outliner_tui::event::handle_key_event(key, app);
            }
            outliner_tui::Event::Tick => {
                app.tick();
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
