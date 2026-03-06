#![allow(dead_code, unused_imports)]

mod app;
mod archive;
mod calculator;
mod config;
mod dbf;
mod dirtree;
mod editor;
mod file_ops;
mod filefind;
mod help;
mod keys;
mod panel;
mod screensaver;
mod splitfile;
mod tetris;
mod theme;
mod types;
mod ui;
mod usermenu;
mod viewer;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use keys::handle_events;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app (load saved config if available)
    let mut app = App::from_config();

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Handle pending shell commands (leave TUI, run, return)
        if let Some(cmd) = app.pending_command.take() {
            // Leave alternate screen
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            // Run the command
            let cwd = app.active_panel().path.clone();
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .current_dir(&cwd)
                .status();

            match output {
                Ok(status) => {
                    app.last_command_output = format!("Command: {}\nExit status: {}", cmd, status);
                    app.status_message = Some(format!("[{}] {}", status, cmd));
                }
                Err(e) => {
                    app.last_command_output = format!("Error running '{}': {}", cmd, e);
                    app.status_message = Some(format!("Error: {}", e));
                }
            }

            // Prompt to return
            println!("\n--- Press any key to return to RDN ---");
            enable_raw_mode()?;
            // Wait for a keypress
            loop {
                if crossterm::event::poll(std::time::Duration::from_secs(60))? {
                    if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
                        break;
                    }
                }
            }

            // Re-enter alternate screen
            execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
            terminal.hide_cursor()?;
            terminal.clear()?;
            app.refresh_panels();
            continue;
        }

        // Ctrl+O user screen mode: just show blank screen with last output
        if app.show_user_screen {
            // Leave alternate screen temporarily
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
            )?;
            terminal.show_cursor()?;

            // Print last command output
            if !app.last_command_output.is_empty() {
                print!("{}", app.last_command_output);
            }

            // Wait for any key, specifically Ctrl+O to return
            enable_raw_mode()?;
            loop {
                if crossterm::event::poll(std::time::Duration::from_secs(60))? {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                        // Any key returns
                        let _ = key;
                        break;
                    }
                }
            }

            // Return to alternate screen
            execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
            terminal.hide_cursor()?;
            terminal.clear()?;
            app.show_user_screen = false;
            continue;
        }

        terminal.draw(|frame| {
            ui::draw(frame, app);
        })?;

        handle_events(app)?;

        if app.should_quit {
            return Ok(());
        }
    }
}
