use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::App;
use crate::types::*;

/// Poll and handle events, returns true if event was handled
pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // Clear status message on any keypress
            app.status_message = None;

            match &app.mode {
                AppMode::Normal => handle_normal_mode(app, key),
                AppMode::CommandLine => handle_command_line(app, key),
                AppMode::QuickSearch(_) => handle_quick_search(app, key),
                AppMode::Dialog(_) => handle_dialog(app, key),
                AppMode::Viewer(_) => handle_viewer(app, key),
                AppMode::Tetris => handle_tetris(app, key),
                AppMode::Help => {
                    app.mode = AppMode::Normal;
                }
            }
            return Ok(true);
        }

        if let Event::Resize(_, _) = event::read()? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Handle keys in normal (panel) mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigation
        KeyCode::Up => app.active_panel_mut().cursor_up(),
        KeyCode::Down => app.active_panel_mut().cursor_down(),
        KeyCode::Home => app.active_panel_mut().cursor_home(),
        KeyCode::End => app.active_panel_mut().cursor_end(),
        KeyCode::PageUp => app.active_panel_mut().page_up(),
        KeyCode::PageDown => app.active_panel_mut().page_down(),

        // Enter directory or open file
        KeyCode::Enter => app.enter(),

        // Go up / Backspace
        KeyCode::Backspace => app.active_panel_mut().go_up(),

        // Tab - switch panel
        KeyCode::Tab => app.switch_panel(),

        // Selection
        KeyCode::Insert => {
            app.active_panel_mut().toggle_select();
            app.active_panel_mut().cursor_down();
        }

        // Function keys
        KeyCode::F(1) => {
            app.mode = AppMode::Help;
        }
        KeyCode::F(3) => {
            // View
            if let Some(entry) = app.active_panel().current_entry() {
                if !entry.is_dir {
                    let path = entry.path.clone();
                    app.open_viewer(&path);
                }
            }
        }
        KeyCode::F(4) => {
            // Edit - open with system editor
            app.open_file_external();
        }
        KeyCode::F(5) => app.start_copy(),
        KeyCode::F(6) => app.start_move(),
        KeyCode::F(7) => app.start_mkdir(),
        KeyCode::F(8) => app.start_delete(),
        KeyCode::F(9) => app.toggle_sort(),
        KeyCode::F(10) | KeyCode::Esc => {
            if app.show_menu {
                app.show_menu = false;
            } else if key.code == KeyCode::F(10) {
                app.should_quit = true;
            }
        }

        // Char keys
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                match c {
                    'x' | 'X' => app.should_quit = true,
                    'q' | 'Q' => app.should_quit = true,

                    // Quick search with Alt
                    _ => {
                        if c.is_alphabetic() {
                            app.mode = AppMode::QuickSearch(c.to_string());
                            app.active_panel_mut().quick_search(&c.to_string());
                        }
                    }
                }
            } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'q' => app.should_quit = true,
                    'g' => app.start_tetris(),
                    'r' => app.refresh_panels(),
                    'l' => app.refresh_panels(),
                    'o' => {
                        // Toggle panel mode
                        let panel = app.active_panel_mut();
                        panel.panel_mode = match panel.panel_mode {
                            PanelMode::Brief => PanelMode::Full,
                            PanelMode::Full => PanelMode::Brief,
                        };
                    }
                    'h' => app.toggle_hidden(),
                    'u' => app.swap_panels(),
                    's' => app.sync_panels(),
                    'i' => app.show_file_info(),
                    _ => {}
                }
            } else {
                match c {
                    '+' => app.active_panel_mut().select_by_pattern("*"),
                    '-' => app.active_panel_mut().deselect_by_pattern("*"),
                    '*' => app.active_panel_mut().invert_selection(),
                    _ => {
                        // Quick search
                        if c.is_alphanumeric() || c == '.' || c == '_' {
                            app.mode = AppMode::QuickSearch(c.to_string());
                            app.active_panel_mut().quick_search(&c.to_string());
                        }
                    }
                }
            }
        }

        _ => {}
    }
}

/// Handle keys in command line mode
fn handle_command_line(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.command_line.clear();
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            // Execute command (placeholder - just show it ran)
            let cmd = app.command_line.clone();
            if !cmd.is_empty() {
                app.status_message = Some(format!("Executed: {}", cmd));
            }
            app.command_line.clear();
            app.mode = AppMode::Normal;
        }
        KeyCode::Backspace => {
            app.command_line.pop();
            if app.command_line.is_empty() {
                app.mode = AppMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.command_line.push(c);
        }
        _ => {}
    }
}

/// Handle keys in quick search mode
fn handle_quick_search(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Tab => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Backspace => {
            if let AppMode::QuickSearch(ref mut s) = app.mode {
                s.pop();
                if s.is_empty() {
                    app.mode = AppMode::Normal;
                    return;
                }
                let search = s.clone();
                app.active_panel_mut().quick_search(&search);
            }
        }
        KeyCode::Up => {
            app.mode = AppMode::Normal;
            app.active_panel_mut().cursor_up();
        }
        KeyCode::Down => {
            app.mode = AppMode::Normal;
            app.active_panel_mut().cursor_down();
        }
        KeyCode::Char(c) => {
            if let AppMode::QuickSearch(ref mut s) = app.mode {
                s.push(c);
                let search = s.clone();
                app.active_panel_mut().quick_search(&search);
            }
        }
        _ => {
            app.mode = AppMode::Normal;
        }
    }
}

/// Handle keys in dialog mode
fn handle_dialog(app: &mut App, key: KeyEvent) {
    let mode = app.mode.clone();
    match mode {
        AppMode::Dialog(DialogKind::Confirm { op, .. }) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.execute_op(op, None);
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.mode = AppMode::Normal;
            }
            _ => {}
        },
        AppMode::Dialog(DialogKind::Input {
            op, mut value, title, prompt,
        }) => match key.code {
            KeyCode::Enter => {
                app.execute_op(op, Some(value));
            }
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                value.pop();
                app.mode = AppMode::Dialog(DialogKind::Input {
                    title,
                    prompt,
                    value,
                    op,
                });
            }
            KeyCode::Char(c) => {
                value.push(c);
                app.mode = AppMode::Dialog(DialogKind::Input {
                    title,
                    prompt,
                    value,
                    op,
                });
            }
            _ => {}
        },
        AppMode::Dialog(DialogKind::Error(_)) => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Dialog(DialogKind::FileInfo) => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                app.mode = AppMode::Normal;
            }
        }
        _ => {}
    }
}

/// Handle keys in viewer mode
fn handle_viewer(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(3) | KeyCode::F(10) | KeyCode::Char('q') => {
            app.close_viewer();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_down();
            }
        }
        KeyCode::PageUp => {
            if let Some(ref mut v) = app.viewer {
                v.page_up();
            }
        }
        KeyCode::PageDown | KeyCode::Char(' ') => {
            if let Some(ref mut v) = app.viewer {
                v.page_down();
            }
        }
        KeyCode::Home => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_home();
            }
        }
        KeyCode::End => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_end();
            }
        }
        KeyCode::Left => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_left();
            }
        }
        KeyCode::Right => {
            if let Some(ref mut v) = app.viewer {
                v.scroll_right();
            }
        }
        KeyCode::F(2) => {
            if let Some(ref mut v) = app.viewer {
                v.toggle_wrap();
            }
        }
        KeyCode::F(4) => {
            if let Some(ref mut v) = app.viewer {
                v.toggle_mode();
            }
        }
        KeyCode::F(7) | KeyCode::Char('/') => {
            // Search - simple implementation
            // In a real app, we'd show an input dialog
            app.status_message = Some("Search: use '/' to search (TODO: input dialog)".to_string());
        }
        KeyCode::Char('n') => {
            if let Some(ref mut v) = app.viewer {
                v.next_match();
            }
        }
        KeyCode::Char('N') => {
            if let Some(ref mut v) = app.viewer {
                v.prev_match();
            }
        }
        _ => {}
    }
}

/// Handle keys in Tetris mode
fn handle_tetris(app: &mut App, key: KeyEvent) {
    // Check for game tick first
    if let Some(ref mut tetris) = app.tetris {
        tetris.tick();
    }

    match key.code {
        KeyCode::Esc => {
            app.close_tetris();
        }
        KeyCode::F(2) | KeyCode::Char('r') | KeyCode::Char('R') => {
            // F2 = New game (matches DN)
            if let Some(ref mut t) = app.tetris {
                t.restart();
            }
        }
        KeyCode::F(3) | KeyCode::Char('p') | KeyCode::Char('P') => {
            // F3 = Pause (matches DN)
            if let Some(ref mut t) = app.tetris {
                t.toggle_pause();
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            // + = Increase level (matches DN)
            if let Some(ref mut t) = app.tetris {
                if t.level < 10 {
                    t.level += 1;
                    let ms = (800.0 * 0.85f64.powi((t.level - 1) as i32)) as u64;
                    t.drop_interval = std::time::Duration::from_millis(ms.max(50));
                }
            }
        }
        KeyCode::Char('*') => {
            // * = Toggle preview (matches DN) — currently always on
        }
        KeyCode::Left => {
            if let Some(ref mut t) = app.tetris {
                t.move_left();
            }
        }
        KeyCode::Right => {
            if let Some(ref mut t) = app.tetris {
                t.move_right();
            }
        }
        KeyCode::Down | KeyCode::Char(' ') => {
            // Down / Space = hard drop (matches DN: space mapped to kbDown)
            if let Some(ref mut t) = app.tetris {
                while t.move_down() {}
            }
        }
        KeyCode::Up => {
            if let Some(ref mut t) = app.tetris {
                t.rotate();
            }
        }
        _ => {}
    }
}
