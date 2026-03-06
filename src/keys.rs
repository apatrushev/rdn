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
                AppMode::Editor => handle_editor(app, key),
                AppMode::FileFind => handle_file_find(app, key),
                AppMode::DirTree => handle_dir_tree(app, key),
                AppMode::Calculator => handle_calculator(app, key),
                AppMode::AsciiTable => handle_ascii_table(app, key),
                AppMode::DiskInfo => handle_disk_info(app, key),
                AppMode::SelectPattern { .. } => handle_select_pattern(app, key),
                AppMode::DirHistory => handle_dir_history(app, key),
                AppMode::FileHistory => handle_file_history(app, key),
                AppMode::ViewerSearch => handle_viewer_search(app, key),
                AppMode::PanelFilter => handle_panel_filter(app, key),
                AppMode::DriveSelect => handle_drive_select(app, key),
                AppMode::UserMenu => handle_user_menu(app, key),
                AppMode::Menu => handle_menu(app, key),
                AppMode::ArchiveView => handle_archive(app, key),
                AppMode::Help => handle_help(app, key),
                AppMode::SystemInfo => {
                    app.mode = AppMode::Normal;
                }
                AppMode::EnvViewer => handle_env_viewer(app, key),
                AppMode::ScreenSaver => {
                    app.deactivate_screensaver();
                }
                AppMode::SplitFileDialog => handle_split_dialog(app, key),
                AppMode::CombineFileDialog => handle_combine_dialog(app, key),
                AppMode::ThemeEditor => handle_theme_editor(app, key),
                AppMode::DbfView => handle_dbf_view(app, key),
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
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F1 = drive/bookmark select for active panel
                app.start_drive_select();
            } else {
                app.start_help();
            }
        }
        KeyCode::F(2) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F2 = drive/bookmark select (same as Alt+F1)
                app.start_drive_select();
            } else {
                // F2 = User menu
                app.start_user_menu();
            }
        }
        KeyCode::F(3) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+F3 = split file
                app.start_split_file();
            } else {
                // View
                if let Some(entry) = app.active_panel().current_entry() {
                    if !entry.is_dir {
                        let ext = entry.path.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_ascii_lowercase())
                            .unwrap_or_default();
                        let path = entry.path.clone();
                        if ext == "dbf" || ext == "csv" {
                            app.open_dbf_viewer();
                        } else if key.modifiers.contains(KeyModifiers::ALT) {
                            // Alt+F3 = new viewer window (push current)
                            app.open_viewer(&path);
                        } else {
                            app.open_viewer(&path);
                        }
                    }
                }
            }
        }
        KeyCode::F(4) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+F4 = new file in editor
                app.new_editor();
            } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+F4 = combine files
                app.start_combine_file();
            } else {
                // F4 = Edit file in built-in editor
                app.open_editor();
            }
        }
        KeyCode::F(5) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+F5 = count dir sizes
                app.count_dir_sizes();
            } else {
                app.start_copy();
            }
        }
        KeyCode::F(6) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+F6 = quick rename
                app.start_quick_rename();
            } else {
                app.start_move();
            }
        }
        KeyCode::F(7) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F7 = file find
                app.start_file_find();
            } else {
                app.start_mkdir();
            }
        }
        KeyCode::F(8) => app.start_delete(),
        KeyCode::F(9) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F9 = ASCII table
                app.start_ascii_table();
            } else {
                // F9 = Open menu bar
                app.open_menu();
            }
        }
        KeyCode::F(10) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F10 = directory tree
                app.start_dir_tree();
            } else {
                app.request_quit();
            }
        }
        KeyCode::Esc => {
            // Esc in normal mode - nothing
        }
        KeyCode::F(11) => {
            // F11 = panel filter
            app.start_panel_filter();
        }
        KeyCode::F(12) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+F12 = directory history
                app.start_dir_history();
            } else {
                // F12 = panel filter
                app.start_panel_filter();
            }
        }

        // Char keys
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                match c {
                    'x' | 'X' => app.request_quit(),
                    'q' | 'Q' => app.request_quit(),
                    'c' | 'C' => app.compare_directories(),
                    'i' | 'I' => app.start_disk_info(),

                    // Alt+1..9 = goto bookmarks
                    '1'..='9' => {
                        let idx = (c as u8 - b'1') as usize;
                        app.goto_bookmark(idx);
                    }

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
                    'q' => app.request_quit(),
                    'g' => app.start_tetris(),
                    'r' => app.refresh_panels(),
                    'l' => app.refresh_panels(),
                    'o' => {
                        // Ctrl+O = toggle user screen (show terminal output)
                        app.show_user_screen = !app.show_user_screen;
                    }
                    'h' => app.toggle_hidden(),
                    'u' => app.swap_panels(),
                    's' => app.sync_panels(),
                    'i' => app.start_system_info(),
                    'k' => app.start_calculator(),
                    't' => app.touch_file(),
                    'f' => app.make_file_list(),
                    'y' => app.start_file_history(),
                    'w' => app.start_theme_editor(),
                    'b' => app.toggle_dir_branch(), // Ctrl+B = branch
                    'p' => app.toggle_quick_view(),
                    'e' => app.start_env_viewer(),
                    // Ctrl+1..9 = set bookmarks
                    '1'..='9' => {
                        let idx = (c as u8 - b'1') as usize;
                        app.set_bookmark(idx);
                    }
                    _ => {}
                }
            } else {
                match c {
                    '+' => app.start_select_pattern(true),
                    '-' => app.start_select_pattern(false),
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
            let cmd = app.command_line.clone();
            if !cmd.is_empty() {
                app.execute_command(&cmd);
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
        AppMode::Dialog(DialogKind::Confirm { op, value, .. }) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.execute_op(op, value);
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
        AppMode::Dialog(DialogKind::Attributes) => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Dialog(DialogKind::AttributesEdit { ref path, mode, readonly, cursor }) => {
            match key.code {
                KeyCode::Esc => app.mode = AppMode::Normal,
                KeyCode::Enter => {
                    let p = path.clone();
                    let m = mode;
                    let r = readonly;
                    app.apply_attributes(&p, m, r);
                }
                KeyCode::Up => {
                    let c = cursor;
                    if c > 0 {
                        if let AppMode::Dialog(DialogKind::AttributesEdit { cursor: ref mut cur, .. }) = app.mode {
                            *cur -= 1;
                        }
                    }
                }
                KeyCode::Down => {
                    let c = cursor;
                    if c < 9 { // 0=readonly, 1-9=rwxrwxrwx
                        if let AppMode::Dialog(DialogKind::AttributesEdit { cursor: ref mut cur, .. }) = app.mode {
                            *cur += 1;
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    let c = cursor;
                    if c == 0 {
                        // Toggle readonly
                        if let AppMode::Dialog(DialogKind::AttributesEdit { readonly: ref mut ro, .. }) = app.mode {
                            *ro = !*ro;
                        }
                    } else {
                        // Toggle permission bit (1=owner_r, 2=owner_w, 3=owner_x, etc.)
                        let bit_index = 9 - c as u32; // bit 8..0
                        let bit = 1u32 << bit_index;
                        if let AppMode::Dialog(DialogKind::AttributesEdit { mode: ref mut m, .. }) = app.mode {
                            *m ^= bit;
                        }
                    }
                }
                _ => {}
            }
        }
        AppMode::Dialog(DialogKind::CompareResult(_)) => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Dialog(DialogKind::SortMenu) => {
            match key.code {
                KeyCode::Esc => app.mode = AppMode::Normal,
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    app.active_panel_mut().sort_mode = SortMode::Name;
                    app.active_panel_mut().sort_entries();
                    app.mode = AppMode::Normal;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    app.active_panel_mut().sort_mode = SortMode::Extension;
                    app.active_panel_mut().sort_entries();
                    app.mode = AppMode::Normal;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    app.active_panel_mut().sort_mode = SortMode::Size;
                    app.active_panel_mut().sort_entries();
                    app.mode = AppMode::Normal;
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    app.active_panel_mut().sort_mode = SortMode::Date;
                    app.active_panel_mut().sort_entries();
                    app.mode = AppMode::Normal;
                }
                _ => {}
            }
        }
        AppMode::Dialog(DialogKind::ConfirmSettings { cursor }) => {
            match key.code {
                KeyCode::Esc | KeyCode::F(10) => app.mode = AppMode::Normal,
                KeyCode::Up => {
                    let new_cursor = if cursor > 0 { cursor - 1 } else { 0 };
                    app.mode = AppMode::Dialog(DialogKind::ConfirmSettings { cursor: new_cursor });
                }
                KeyCode::Down => {
                    let new_cursor = if cursor < 2 { cursor + 1 } else { 2 };
                    app.mode = AppMode::Dialog(DialogKind::ConfirmSettings { cursor: new_cursor });
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    match cursor {
                        0 => app.confirm_delete = !app.confirm_delete,
                        1 => app.confirm_overwrite = !app.confirm_overwrite,
                        2 => app.confirm_exit = !app.confirm_exit,
                        _ => {}
                    }
                    app.save_desktop();
                    // restore dialog mode so user can keep toggling
                    app.mode = AppMode::Dialog(DialogKind::ConfirmSettings { cursor });
                }
                _ => {}
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
            // Search in viewer
            app.start_viewer_search();
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
/// Handle keys in editor mode
fn handle_editor(app: &mut App, key: KeyEvent) {
    use crate::editor::EditorInputMode;

    let editor = match app.editor.as_mut() {
        Some(e) => e,
        None => return,
    };

    // Handle sub-modes (search/replace/saveas/gotoline input)
    match &editor.input_mode {
        EditorInputMode::Search => {
            match key.code {
                KeyCode::Esc => editor.input_mode = EditorInputMode::Normal,
                KeyCode::Enter => {
                    editor.search_query = editor.input_buffer.clone();
                    editor.input_mode = EditorInputMode::Normal;
                    editor.find_next();
                }
                KeyCode::Backspace => { editor.input_buffer.pop(); }
                KeyCode::Char(c) => editor.input_buffer.push(c),
                _ => {}
            }
            return;
        }
        EditorInputMode::Replace => {
            match key.code {
                KeyCode::Esc => editor.input_mode = EditorInputMode::Normal,
                KeyCode::Enter => {
                    editor.replace_query = editor.input_buffer.clone();
                    editor.input_mode = EditorInputMode::ReplaceConfirm;
                    editor.input_buffer.clear();
                }
                KeyCode::Backspace => { editor.input_buffer.pop(); }
                KeyCode::Char(c) => editor.input_buffer.push(c),
                _ => {}
            }
            return;
        }
        EditorInputMode::ReplaceConfirm => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    editor.replace_current();
                    editor.find_next();
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    editor.find_next();
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let count = editor.replace_all();
                    editor.status_msg = Some(format!("Replaced {} occurrences", count));
                    editor.input_mode = EditorInputMode::Normal;
                }
                KeyCode::Esc => editor.input_mode = EditorInputMode::Normal,
                _ => {}
            }
            return;
        }
        EditorInputMode::SaveAs => {
            match key.code {
                KeyCode::Esc => editor.input_mode = EditorInputMode::Normal,
                KeyCode::Enter => {
                    let path = editor.input_buffer.clone();
                    match editor.save_as(&path) {
                        Ok(msg) => editor.status_msg = Some(msg),
                        Err(e) => editor.status_msg = Some(e),
                    }
                    editor.input_mode = EditorInputMode::Normal;
                }
                KeyCode::Backspace => { editor.input_buffer.pop(); }
                KeyCode::Char(c) => editor.input_buffer.push(c),
                _ => {}
            }
            return;
        }
        EditorInputMode::GotoLine => {
            match key.code {
                KeyCode::Esc => editor.input_mode = EditorInputMode::Normal,
                KeyCode::Enter => {
                    if let Ok(n) = editor.input_buffer.parse::<usize>() {
                        editor.goto_line(n);
                    }
                    editor.input_mode = EditorInputMode::Normal;
                }
                KeyCode::Backspace => { editor.input_buffer.pop(); }
                KeyCode::Char(c) if c.is_ascii_digit() => editor.input_buffer.push(c),
                _ => {}
            }
            return;
        }
        EditorInputMode::Normal => {} // fall through to main editor keys
    }

    // Record key press for macros
    editor.record_key(key);

    // Main editor keys
    match key.code {
        KeyCode::Esc | KeyCode::F(10) => {
            if editor.modified {
                editor.status_msg = Some("Modified! F2=Save, Esc again=discard".to_string());
                // Simple: second Esc closes
                app.close_editor();
            } else {
                app.close_editor();
            }
        }
        KeyCode::F(2) => {
            if let Some(ref mut e) = app.editor {
                match e.save() {
                    Ok(msg) => e.status_msg = Some(msg),
                    Err(err) => {
                        // No path? Prompt Save As
                        if e.path.is_none() {
                            e.input_mode = EditorInputMode::SaveAs;
                            e.input_buffer.clear();
                        } else {
                            e.status_msg = Some(err);
                        }
                    }
                }
            }
        }
        KeyCode::F(7) => {
            // Search
            if let Some(ref mut e) = app.editor {
                e.input_mode = EditorInputMode::Search;
                e.input_buffer = e.search_query.clone();
            }
        }
        KeyCode::F(8) => {
            // Delete line
            if let Some(ref mut e) = app.editor {
                e.delete_line();
            }
        }
        KeyCode::F(5) => {
            // Goto line
            if let Some(ref mut e) = app.editor {
                e.input_mode = EditorInputMode::GotoLine;
                e.input_buffer.clear();
            }
        }

        // Navigation
        KeyCode::Up => editor.cursor_up(),
        KeyCode::Down => editor.cursor_down(),
        KeyCode::Left => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                editor.word_left();
            } else {
                editor.cursor_left();
            }
        }
        KeyCode::Right => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                editor.word_right();
            } else {
                editor.cursor_right();
            }
        }
        KeyCode::Home => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                editor.goto_top();
            } else {
                editor.cursor_home();
            }
        }
        KeyCode::End => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                editor.goto_bottom();
            } else {
                editor.cursor_end();
            }
        }
        KeyCode::PageUp => editor.page_up(),
        KeyCode::PageDown => editor.page_down(),

        // Editing
        KeyCode::Enter => editor.enter_key(),
        KeyCode::Backspace => editor.backspace(),
        KeyCode::Delete => editor.delete_char(),
        KeyCode::Insert => editor.insert_mode = !editor.insert_mode,
        KeyCode::Tab => editor.insert_char('\t'),

        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'c' => editor.copy_selection(),
                    'x' => editor.cut_selection(),
                    'v' => editor.paste(),
                    'z' => editor.undo(),
                    'y' => editor.redo(),
                    'a' => editor.select_all(),
                    'k' => editor.delete_line(),
                    'r' => {
                        // Replace
                        editor.input_mode = EditorInputMode::Replace;
                        editor.input_buffer = editor.replace_query.clone();
                    }
                    'g' => {
                        // Goto line
                        editor.input_mode = EditorInputMode::GotoLine;
                        editor.input_buffer.clear();
                    }
                    'n' => { editor.find_next(); }
                    'm' => { editor.toggle_macro_recording(); }
                    'p' => {
                        // Play macro
                        let keys = editor.get_macro_keys();
                        if keys.is_empty() {
                            editor.status_msg = Some("No macro recorded".to_string());
                        } else {
                            // We need to replay without re-recording
                            let was_recording = editor.macro_recording;
                            editor.macro_recording = false;
                            for macro_key in keys {
                                // Process each key — simplified dispatch
                                match macro_key.code {
                                    KeyCode::Char(ch) if !macro_key.modifiers.contains(KeyModifiers::CONTROL) => {
                                        editor.insert_char(ch);
                                    }
                                    KeyCode::Enter => editor.enter_key(),
                                    KeyCode::Backspace => editor.backspace(),
                                    KeyCode::Delete => editor.delete_char(),
                                    KeyCode::Tab => editor.insert_char('\t'),
                                    KeyCode::Up => editor.cursor_up(),
                                    KeyCode::Down => editor.cursor_down(),
                                    KeyCode::Left => editor.cursor_left(),
                                    KeyCode::Right => editor.cursor_right(),
                                    KeyCode::Home => editor.cursor_home(),
                                    KeyCode::End => editor.cursor_end(),
                                    _ => {}
                                }
                            }
                            editor.macro_recording = was_recording;
                            editor.status_msg = Some("Macro played".to_string());
                        }
                    }
                    _ => {}
                }
            } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+char - handle selection start
                if editor.selection.is_none() {
                    editor.start_selection();
                }
                editor.insert_char(c);
                editor.update_selection();
            } else {
                editor.insert_char(c);
            }
        }
        _ => {}
    }
}

/// Handle keys in file find mode
fn handle_file_find(app: &mut App, key: KeyEvent) {
    let finder = match app.finder.as_mut() {
        Some(f) => f,
        None => return,
    };

    if !finder.search_complete {
        // Still in input mode - editing pattern/content
        match key.code {
            KeyCode::Esc => {
                app.finder = None;
                app.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                finder.execute();
            }
            KeyCode::Tab => {
                // Toggle between pattern and content fields
                // (swap which is being edited)
            }
            KeyCode::Backspace => {
                finder.pattern.pop();
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        's' => finder.search_subdirs = !finder.search_subdirs,
                        'c' => finder.case_sensitive = !finder.case_sensitive,
                        _ => {}
                    }
                } else {
                    finder.pattern.push(c);
                }
            }
            _ => {}
        }
    } else {
        // Results mode
        match key.code {
            KeyCode::Esc => {
                app.finder = None;
                app.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                app.close_file_find();
            }
            KeyCode::Up => finder.cursor_up(),
            KeyCode::Down => finder.cursor_down(),
            KeyCode::PageUp => finder.page_up(),
            KeyCode::PageDown => finder.page_down(),
            KeyCode::F(7) => {
                // New search
                finder.search_complete = false;
                finder.results.clear();
                finder.pattern = "*".to_string();
            }
            _ => {}
        }
    }
}

/// Handle keys in directory tree mode
fn handle_dir_tree(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_dir_tree(false),
        KeyCode::Enter => app.close_dir_tree(true),
        KeyCode::Up => {
            if let Some(ref mut t) = app.dir_tree {
                t.cursor_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut t) = app.dir_tree {
                t.cursor_down();
            }
        }
        KeyCode::PageUp => {
            if let Some(ref mut t) = app.dir_tree {
                t.page_up();
            }
        }
        KeyCode::PageDown => {
            if let Some(ref mut t) = app.dir_tree {
                t.page_down();
            }
        }
        KeyCode::Right | KeyCode::Char('+') => {
            if let Some(ref mut t) = app.dir_tree {
                let idx = t.cursor;
                t.expand_node(idx);
            }
        }
        KeyCode::Left | KeyCode::Char('-') => {
            if let Some(ref mut t) = app.dir_tree {
                let idx = t.cursor;
                t.collapse_node(idx);
            }
        }
        KeyCode::Char(' ') => {
            if let Some(ref mut t) = app.dir_tree {
                t.toggle_expand();
            }
        }
        _ => {}
    }
}

/// Handle keys in calculator mode
fn handle_calculator(app: &mut App, key: KeyEvent) {
    let calc = match app.calculator.as_mut() {
        Some(c) => c,
        None => return,
    };

    match key.code {
        KeyCode::Esc => app.close_calculator(),
        KeyCode::Char(c) => match c {
            '0'..='9' | '.' => calc.press_digit(c),
            '+' | '-' | '*' | '/' => calc.press_op(c),
            '=' => calc.press_equals(),
            '%' => calc.press_percent(),
            'n' | 'N' => calc.press_negate(),
            's' | 'S' => calc.press_sqrt(),
            'i' | 'I' => calc.press_inverse(),
            'c' | 'C' => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    calc.clear_entry();
                } else {
                    calc.clear();
                }
            }
            'm' | 'M' => calc.memory_store(),
            'r' | 'R' => calc.memory_recall(),
            'p' | 'P' => calc.memory_add(),
            _ => {}
        },
        KeyCode::Enter => calc.press_equals(),
        KeyCode::Backspace => calc.backspace(),
        KeyCode::Delete => calc.clear_entry(),
        _ => {}
    }
}

/// Handle keys in ASCII table mode
fn handle_ascii_table(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.mode = AppMode::Normal,
        KeyCode::Up => {
            app.ascii_cursor = app.ascii_cursor.wrapping_sub(16);
        }
        KeyCode::Down => {
            app.ascii_cursor = app.ascii_cursor.wrapping_add(16);
        }
        KeyCode::Left => {
            app.ascii_cursor = app.ascii_cursor.wrapping_sub(1);
        }
        KeyCode::Right => {
            app.ascii_cursor = app.ascii_cursor.wrapping_add(1);
        }
        _ => {}
    }
}

/// Handle keys in disk info mode
fn handle_disk_info(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.mode = AppMode::Normal,
        _ => {}
    }
}

/// Handle keys in select pattern mode
fn handle_select_pattern(app: &mut App, key: KeyEvent) {
    let selecting = matches!(app.mode, AppMode::SelectPattern { selecting: true });
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Normal,
        KeyCode::Enter => app.apply_select_pattern(selecting),
        KeyCode::Backspace => { app.select_pattern_buf.pop(); }
        KeyCode::Char(c) => app.select_pattern_buf.push(c),
        _ => {}
    }
}

/// Handle keys in directory history mode
fn handle_dir_history(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_dir_history(false),
        KeyCode::Enter => app.close_dir_history(true),
        KeyCode::Up => {
            if app.dir_history_cursor > 0 {
                app.dir_history_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.dir_history_cursor + 1 < app.dir_history.len() {
                app.dir_history_cursor += 1;
            }
        }
        _ => {}
    }
}

fn handle_file_history(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_file_history(false),
        KeyCode::Enter => app.close_file_history(true),
        KeyCode::Up => {
            if app.file_history_cursor > 0 {
                app.file_history_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.file_history_cursor + 1 < app.file_history.len() {
                app.file_history_cursor += 1;
            }
        }
        _ => {}
    }
}

/// Handle keys in viewer search mode
fn handle_viewer_search(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            // Return to viewer
            app.mode = if let Some(ref v) = app.viewer {
                AppMode::Viewer(v.path.clone())
            } else {
                AppMode::Normal
            };
        }
        KeyCode::Enter => app.apply_viewer_search(),
        KeyCode::Backspace => { app.viewer_search_buf.pop(); }
        KeyCode::Char(c) => app.viewer_search_buf.push(c),
        _ => {}
    }
}

/// Handle keys in panel filter mode
fn handle_panel_filter(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Normal,
        KeyCode::Enter => app.apply_panel_filter(),
        KeyCode::Backspace => { app.filter_buf.pop(); }
        KeyCode::Char(c) => app.filter_buf.push(c),
        _ => {}
    }
}

/// Handle keys in drive select mode
fn handle_drive_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_drive_select(false),
        KeyCode::Enter => app.close_drive_select(true),
        KeyCode::Up => {
            if app.drive_cursor > 0 {
                app.drive_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.drive_cursor + 1 < app.drive_list.len() {
                app.drive_cursor += 1;
            }
        }
        _ => {}
    }
}

/// Handle keys in archive view mode
fn handle_archive(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(10) => app.close_archive(),
        KeyCode::Enter => {
            let is_dir = app.archive_browser.as_ref()
                .and_then(|b| {
                    let entries = b.visible_entries();
                    entries.get(b.cursor).map(|e| e.is_dir)
                })
                .unwrap_or(false);

            if is_dir {
                if let Some(ref mut browser) = app.archive_browser {
                    browser.enter_dir();
                }
            } else {
                app.archive_view_file();
            }
        }
        KeyCode::Up => {
            if let Some(ref mut b) = app.archive_browser {
                b.cursor_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut b) = app.archive_browser {
                b.cursor_down();
            }
        }
        KeyCode::PageUp => {
            if let Some(ref mut b) = app.archive_browser {
                b.page_up();
            }
        }
        KeyCode::PageDown => {
            if let Some(ref mut b) = app.archive_browser {
                b.page_down();
            }
        }
        KeyCode::F(3) => {
            // View file within archive
            app.archive_view_file();
        }
        KeyCode::F(5) => {
            // Extract file to disk (to inactive panel directory)
            let result = {
                let browser = match &app.archive_browser {
                    Some(b) => b,
                    None => return,
                };
                let entries = browser.visible_entries();
                let entry = match entries.get(browser.cursor) {
                    Some(e) => e,
                    None => return,
                };
                if entry.is_dir {
                    app.status_message = Some("Cannot extract directory".to_string());
                    return;
                }
                let entry_path = entry.path.to_string_lossy().to_string();
                browser.extract_file(&entry_path)
            };
            match result {
                Ok(temp_path) => {
                    let dest = app.inactive_panel().path.join(
                        temp_path.file_name().unwrap_or_default()
                    );
                    match std::fs::copy(&temp_path, &dest) {
                        Ok(_) => {
                            app.status_message = Some(format!("Extracted to {}", dest.display()));
                            app.inactive_panel_mut().load_directory();
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Extract error: {}", e));
                        }
                    }
                    let _ = std::fs::remove_file(temp_path);
                }
                Err(e) => {
                    app.status_message = Some(format!("Extract error: {}", e));
                }
            }
        }
        KeyCode::Backspace => {
            // Go up in archive directory
            if let Some(ref mut browser) = app.archive_browser {
                if !browser.current_dir.is_empty() {
                    if let Some(pos) = browser.current_dir[..browser.current_dir.len().saturating_sub(1)].rfind('/') {
                        browser.current_dir = browser.current_dir[..=pos].to_string();
                    } else {
                        browser.current_dir = String::new();
                    }
                    browser.cursor = 0;
                    browser.scroll_offset = 0;
                }
            }
        }
        _ => {}
    }
}

/// Handle keys in menu mode
fn handle_menu(app: &mut App, key: KeyEvent) {
    let menu_count = App::menu_labels().len();

    match key.code {
        KeyCode::Esc | KeyCode::F(10) => {
            app.show_menu = false;
            app.mode = AppMode::Normal;
        }
        KeyCode::Left => {
            if app.menu_index == 0 {
                app.menu_index = menu_count - 1;
            } else {
                app.menu_index -= 1;
            }
            app.menu_item_cursor = 0;
        }
        KeyCode::Right => {
            app.menu_index = (app.menu_index + 1) % menu_count;
            app.menu_item_cursor = 0;
        }
        KeyCode::Up => {
            let max = app.menu_selectable_count();
            if max > 0 {
                if app.menu_item_cursor == 0 {
                    app.menu_item_cursor = max - 1;
                } else {
                    app.menu_item_cursor -= 1;
                }
            }
        }
        KeyCode::Down => {
            let max = app.menu_selectable_count();
            if max > 0 {
                app.menu_item_cursor = (app.menu_item_cursor + 1) % max;
            }
        }
        KeyCode::Home => {
            app.menu_item_cursor = 0;
        }
        KeyCode::End => {
            let max = app.menu_selectable_count();
            if max > 0 {
                app.menu_item_cursor = max - 1;
            }
        }
        KeyCode::Enter => {
            let items = App::menu_items(app.menu_index);
            let idx = app.menu_cursor_to_index();
            if let Some(item) = items.get(idx) {
                let action = item.action;
                app.execute_menu_action(action);
            }
        }
        KeyCode::F(9) => {
            // F9 again closes menu
            app.show_menu = false;
            app.mode = AppMode::Normal;
        }
        // Hotkey: first letter of menu item
        KeyCode::Char(c) => {
            let items = App::menu_items(app.menu_index);
            let c_lower = c.to_ascii_lowercase();
            // Find selectable item starting with this character
            let mut selectable_idx = 0usize;
            for item in &items {
                if !item.is_separator() {
                    if let Some(first) = item.label.chars().next() {
                        if first.to_ascii_lowercase() == c_lower {
                            app.menu_item_cursor = selectable_idx;
                            let action = item.action;
                            app.execute_menu_action(action);
                            return;
                        }
                    }
                    selectable_idx += 1;
                }
            }
        }
        _ => {}
    }
}

/// Handle keys in user menu mode (placeholder)
fn handle_user_menu(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_user_menu(),
        KeyCode::Up => {
            if let Some(ref mut data) = app.user_menu_data {
                data.cursor_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut data) = app.user_menu_data {
                data.cursor_down();
            }
        }
        KeyCode::Enter => app.execute_user_menu_item(),
        _ => {}
    }
}

fn handle_help(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(10) => app.close_help(),
        KeyCode::Up => {
            if let Some(ref mut h) = app.help_viewer {
                h.scroll_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut h) = app.help_viewer {
                h.scroll_down();
            }
        }
        KeyCode::PageUp => {
            if let Some(ref mut h) = app.help_viewer {
                h.page_up();
            }
        }
        KeyCode::PageDown => {
            if let Some(ref mut h) = app.help_viewer {
                h.page_down();
            }
        }
        KeyCode::Left => {
            if let Some(ref mut h) = app.help_viewer {
                h.prev_topic();
            }
        }
        KeyCode::Right => {
            if let Some(ref mut h) = app.help_viewer {
                h.next_topic();
            }
        }
        KeyCode::Backspace => {
            if let Some(ref mut h) = app.help_viewer {
                h.go_back();
            }
        }
        KeyCode::Home => {
            if let Some(ref mut h) = app.help_viewer {
                h.scroll = 0;
            }
        }
        KeyCode::End => {
            if let Some(ref mut h) = app.help_viewer {
                let max = h.current().content.len().saturating_sub(h.visible_height);
                h.scroll = max;
            }
        }
        _ => {}
    }
}

fn handle_env_viewer(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(10) => app.close_env_viewer(),
        KeyCode::Up => {
            if app.env_cursor > 0 {
                app.env_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.env_cursor + 1 < app.env_vars.len() {
                app.env_cursor += 1;
            }
        }
        KeyCode::PageUp => {
            app.env_cursor = app.env_cursor.saturating_sub(20);
        }
        KeyCode::PageDown => {
            app.env_cursor = (app.env_cursor + 20).min(app.env_vars.len().saturating_sub(1));
        }
        KeyCode::Home => app.env_cursor = 0,
        KeyCode::End => app.env_cursor = app.env_vars.len().saturating_sub(1),
        _ => {}
    }
}

fn handle_split_dialog(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Normal,
        KeyCode::Enter => app.execute_split_file(),
        KeyCode::Backspace => { app.split_size_buf.pop(); }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            app.split_size_buf.push(c);
        }
        _ => {}
    }
}

fn handle_combine_dialog(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Normal,
        KeyCode::Enter => app.execute_combine_file(),
        _ => {}
    }
}

/// Handle keys in the color theme editor
fn handle_theme_editor(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Normal,
        KeyCode::Enter | KeyCode::F(10) => app.apply_theme_editor(),
        KeyCode::Up => app.theme_editor_up(),
        KeyCode::Down => app.theme_editor_down(),
        // Left/Right cycle fg color
        KeyCode::Left => app.theme_editor_fg_prev(),
        KeyCode::Right => app.theme_editor_fg_next(),
        // Shift+Left/Right cycle bg color
        KeyCode::Char('b') | KeyCode::F(4) => app.theme_editor_bg_next(),
        KeyCode::F(3) => app.theme_editor_bg_prev(),
        // Home/End reset slot to default
        KeyCode::Delete => {
            let i = app.theme_editor_cursor as usize;
            let (df, db) = crate::theme::SLOT_DEFAULTS[i];
            app.theme_editor_fg[i] = df;
            app.theme_editor_bg[i] = db;
        }
        _ => {}
    }
}

/// Handle keys in the DBF/CSV viewer
fn handle_dbf_view(app: &mut App, key: KeyEvent) {
    let vis_rows: usize = 20; // approximate; draw fn handles clamping
    let vis_cols: usize = 6;

    if let Some(ref mut data) = app.dbf_data {
        match key.code {
            KeyCode::Esc | KeyCode::F(10) | KeyCode::Char('q') => {
                app.dbf_data = None;
                app.mode = AppMode::Normal;
                return;
            }
            KeyCode::Up => data.cursor_up(),
            KeyCode::Down => data.cursor_down(vis_rows),
            KeyCode::Left => data.cursor_left(),
            KeyCode::Right => data.cursor_right(vis_cols),
            KeyCode::PageUp => data.page_up(vis_rows),
            KeyCode::PageDown => data.page_down(vis_rows),
            KeyCode::Home => data.home(),
            KeyCode::End => data.end(),
            _ => {}
        }
    } else {
        app.mode = AppMode::Normal;
    }
}