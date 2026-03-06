use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::symbols::border;
use ratatui::Frame;

use crate::app::App;
use crate::panel::Panel;
use crate::theme::Theme;
use crate::types::*;
use crate::viewer::ViewerMode;
use crate::tetris::{BOARD_WIDTH, BOARD_HEIGHT};

/// Render the entire application
pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Fill background
    frame.render_widget(
        Block::default().style(Theme::panel_bg()),
        size,
    );

    match &app.mode {
        AppMode::Viewer(_) => {
            draw_viewer(frame, app, size);
        }
        AppMode::Editor => {
            draw_editor(frame, app, size);
        }
        AppMode::Tetris => {
            // Draw file manager in background, then overlay tetris dialog (like DN)
            draw_file_manager(frame, app, size);
            draw_tetris(frame, app, size);
        }
        AppMode::FileFind => {
            draw_file_manager(frame, app, size);
            draw_file_find(frame, app, size);
        }
        AppMode::DirTree => {
            draw_file_manager(frame, app, size);
            draw_dir_tree(frame, app, size);
        }
        AppMode::Calculator => {
            draw_file_manager(frame, app, size);
            draw_calculator(frame, app, size);
        }
        AppMode::AsciiTable => {
            draw_file_manager(frame, app, size);
            draw_ascii_table(frame, app, size);
        }
        AppMode::DiskInfo => {
            draw_file_manager(frame, app, size);
            draw_disk_info(frame, app, size);
        }
        AppMode::SelectPattern { .. } => {
            draw_file_manager(frame, app, size);
            draw_select_pattern(frame, app, size);
        }
        AppMode::DirHistory => {
            draw_file_manager(frame, app, size);
            draw_dir_history(frame, app, size);
        }
        AppMode::FileHistory => {
            draw_file_manager(frame, app, size);
            draw_file_history(frame, app, size);
        }
        AppMode::ViewerSearch => {
            draw_viewer(frame, app, size);
            draw_viewer_search(frame, app, size);
        }
        AppMode::PanelFilter => {
            draw_file_manager(frame, app, size);
            draw_panel_filter(frame, app, size);
        }
        AppMode::DriveSelect => {
            draw_file_manager(frame, app, size);
            draw_drive_select(frame, app, size);
        }
        AppMode::UserMenu => {
            draw_file_manager(frame, app, size);
            draw_user_menu(frame, app, size);
        }
        AppMode::Menu => {
            draw_file_manager(frame, app, size);
            draw_dropdown_menu(frame, app, size);
        }
        AppMode::ArchiveView => {
            draw_archive_view(frame, app, size);
        }
        AppMode::Help => {
            draw_file_manager(frame, app, size);
            draw_help(frame, app, size);
        }
        AppMode::SystemInfo => {
            draw_file_manager(frame, app, size);
            draw_system_info(frame, app, size);
        }
        AppMode::EnvViewer => {
            draw_file_manager(frame, app, size);
            draw_env_viewer(frame, app, size);
        }
        AppMode::ScreenSaver => {
            draw_screensaver(frame, app, size);
        }
        AppMode::SplitFileDialog => {
            draw_file_manager(frame, app, size);
            draw_split_dialog(frame, app, size);
        }
        AppMode::CombineFileDialog => {
            draw_file_manager(frame, app, size);
            draw_combine_dialog(frame, app, size);
        }
        AppMode::ThemeEditor => {
            draw_file_manager(frame, app, size);
            draw_theme_editor(frame, app, size);
        }
        AppMode::DbfView => {
            draw_dbf_view(frame, app, size);
        }
        _ => {
            draw_file_manager(frame, app, size);
        }
    }
}

/// Draw the main two-panel file manager view
fn draw_file_manager(frame: &mut Frame, app: &mut App, area: Rect) {
    // DN layout: panels take the whole screen, menu bar is HIDDEN by default.
    // Bottom 2 rows: command line + function keys.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Panels (full height minus bottom 2 lines)
            Constraint::Length(1), // Command line / status
            Constraint::Length(1), // Function keys bar
        ])
        .split(area);

    // Split panels area horizontally (two equal halves)
    let panels_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    // Precompute descriptions (avoid mixed borrows during draw_panel calls)
    let left_desc: Option<String> = if app.desc_panel_visible {
        app.left_panel.current_entry()
            .map(|e| e.name.clone())
            .and_then(|n| app.get_file_description(&n).map(|s| s.to_string()))
    } else {
        None
    };
    let right_desc: Option<String> = if app.desc_panel_visible {
        app.right_panel.current_entry()
            .map(|e| e.name.clone())
            .and_then(|n| app.get_file_description(&n).map(|s| s.to_string()))
    } else {
        None
    };

    draw_panel(
        frame,
        &mut app.left_panel,
        panels_layout[0],
        app.active == ActivePanel::Left,
        matches!(&app.mode, AppMode::QuickSearch(_)) && app.active == ActivePanel::Left,
        match &app.mode {
            AppMode::QuickSearch(s) if app.active == ActivePanel::Left => Some(s.as_str()),
            _ => None,
        },
        left_desc.as_deref(),
    );

    // Quick view: if enabled, inactive panel shows file preview
    if app.quick_view && app.active == ActivePanel::Left {
        draw_quick_view(frame, app, panels_layout[1]);
    } else {
        draw_panel(
            frame,
            &mut app.right_panel,
            panels_layout[1],
            app.active == ActivePanel::Right,
            matches!(&app.mode, AppMode::QuickSearch(_)) && app.active == ActivePanel::Right,
            match &app.mode {
                AppMode::QuickSearch(s) if app.active == ActivePanel::Right => Some(s.as_str()),
                _ => None,
            },
            right_desc.as_deref(),
        );
    }

    // If quick_view and right is active, left panel is quick view
    if app.quick_view && app.active == ActivePanel::Right {
        // Left was already drawn as panel, now overlay quick view
        draw_quick_view(frame, app, panels_layout[0]);
    }

    draw_command_line(frame, app, main_layout[1]);
    draw_function_keys(frame, app, main_layout[2]);

    // Draw menu bar only when activated (overlay on top)
    if app.show_menu {
        draw_menu_bar(frame, app, Rect::new(area.x, area.y, area.width, 1));
    }

    // Draw dialogs on top
    if let AppMode::Dialog(ref kind) = app.mode {
        draw_dialog(frame, app, kind.clone(), area);
    }
}

/// Draw the menu bar
fn draw_menu_bar(frame: &mut Frame, app: &App, area: Rect) {
    let menus = App::menu_labels();
    let mut spans = Vec::new();

    for (i, menu) in menus.iter().enumerate() {
        let style = if app.show_menu && app.menu_index == i {
            Theme::menu_bar_highlight()
        } else {
            Theme::menu_bar()
        };
        spans.push(Span::styled(menu.to_string(), style));
    }

    // Pad the rest (with clock in top-right if enabled)
    let clock_str = if app.clock_visible {
        chrono::Local::now().format(" %H:%M ").to_string()
    } else {
        String::new()
    };
    let clock_len = clock_str.len();
    let used: usize = menus.iter().map(|m| m.len()).sum();
    let pad_len = (area.width as usize).saturating_sub(used + clock_len);
    if pad_len > 0 {
        spans.push(Span::styled(
            " ".repeat(pad_len),
            Theme::menu_bar(),
        ));
    }
    if !clock_str.is_empty() {
        spans.push(Span::styled(clock_str, Theme::menu_bar_highlight()));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Draw the full dropdown menu overlay (menu bar + dropdown panel)
fn draw_dropdown_menu(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::style::{Color, Style};

    // Draw menu bar at top of screen
    draw_menu_bar(frame, app, Rect::new(area.x, area.y, area.width, 1));

    // Get menu items for the current menu
    let items = App::menu_items(app.menu_index);
    if items.is_empty() {
        return;
    }

    // Calculate dropdown position
    let menus = App::menu_labels();
    let mut x_offset: u16 = 0;
    for (i, menu) in menus.iter().enumerate() {
        if i == app.menu_index {
            break;
        }
        x_offset += menu.len() as u16;
    }

    // Calculate max width for label and shortcut columns
    let max_label_width = items.iter()
        .filter(|i| !i.is_separator())
        .map(|i| i.label.len())
        .max()
        .unwrap_or(10);
    let max_shortcut_width = items.iter()
        .filter(|i| !i.is_separator())
        .map(|i| i.shortcut.len())
        .max()
        .unwrap_or(0);

    let inner_width = max_label_width + 2 + max_shortcut_width + 2; // padding
    let dropdown_width = (inner_width + 2) as u16; // + borders
    let dropdown_height = (items.len() + 2) as u16; // + borders

    // Ensure dropdown doesn't go off screen
    let drop_x = if x_offset + dropdown_width > area.width {
        area.width.saturating_sub(dropdown_width)
    } else {
        x_offset
    };
    let drop_y = area.y + 1; // right below menu bar

    let dropdown_area = Rect::new(drop_x, drop_y, dropdown_width, dropdown_height.min(area.height.saturating_sub(1)));

    // Clear the area
    frame.render_widget(Clear, dropdown_area);

    // Draw border
    let border_style = Style::default()
        .fg(Color::Rgb(0, 0, 0))
        .bg(Color::Rgb(0, 170, 170));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(Color::Rgb(0, 170, 170)));
    frame.render_widget(block, dropdown_area);

    // Draw items
    let inner = Rect::new(dropdown_area.x + 1, dropdown_area.y + 1,
                          dropdown_area.width.saturating_sub(2),
                          dropdown_area.height.saturating_sub(2));

    let normal_style = Style::default()
        .fg(Color::Rgb(0, 0, 0))
        .bg(Color::Rgb(0, 170, 170));
    let highlight_style = Style::default()
        .fg(Color::Rgb(255, 255, 255))
        .bg(Color::Rgb(0, 0, 0));
    let shortcut_style = Style::default()
        .fg(Color::Rgb(170, 0, 0))
        .bg(Color::Rgb(0, 170, 170));
    let shortcut_highlight_style = Style::default()
        .fg(Color::Rgb(170, 170, 0))
        .bg(Color::Rgb(0, 0, 0));
    let separator_style = Style::default()
        .fg(Color::Rgb(0, 0, 0))
        .bg(Color::Rgb(0, 170, 170));
    let hotkey_style = Style::default()
        .fg(Color::Rgb(255, 255, 85))
        .bg(Color::Rgb(0, 170, 170));
    let hotkey_highlight_style = Style::default()
        .fg(Color::Rgb(255, 255, 85))
        .bg(Color::Rgb(0, 0, 0));

    let mut selectable_idx = 0usize;
    for (row, item) in items.iter().enumerate() {
        if row as u16 >= inner.height {
            break;
        }
        let y = inner.y + row as u16;
        let w = inner.width as usize;

        if item.is_separator() {
            // Draw separator line
            let sep = "─".repeat(w);
            frame.render_widget(
                Paragraph::new(sep).style(separator_style),
                Rect::new(inner.x, y, inner.width, 1),
            );
        } else {
            let is_selected = selectable_idx == app.menu_item_cursor;
            let base = if is_selected { highlight_style } else { normal_style };
            let sc_style = if is_selected { shortcut_highlight_style } else { shortcut_style };
            let hk_style = if is_selected { hotkey_highlight_style } else { hotkey_style };

            // Build the line: " X<label>  <shortcut> "
            // First char is highlighted as hotkey
            let mut spans = Vec::new();
            spans.push(Span::styled(" ", base));

            // Label with first-char hotkey
            let label = &item.label;
            if let Some(first_char) = label.chars().next() {
                spans.push(Span::styled(first_char.to_string(), hk_style));
                spans.push(Span::styled(label[first_char.len_utf8()..].to_string(), base));
            }

            // Padding between label and shortcut
            let label_len = label.len() + 1; // +1 for leading space
            let shortcut_len = item.shortcut.len();
            let padding = if w > label_len + shortcut_len + 1 {
                w - label_len - shortcut_len - 1
            } else {
                1
            };
            spans.push(Span::styled(" ".repeat(padding), base));

            // Shortcut
            if !item.shortcut.is_empty() {
                spans.push(Span::styled(item.shortcut.clone(), sc_style));
            }

            // Fill remaining
            let total_used = label_len + padding + shortcut_len;
            if total_used < w {
                spans.push(Span::styled(" ".repeat(w - total_used), base));
            }

            frame.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(inner.x, y, inner.width, 1),
            );

            selectable_idx += 1;
        }
    }

    // Draw shadow (subtle effect)
    if dropdown_area.x + dropdown_width < area.width && dropdown_area.y + dropdown_height < area.height {
        let shadow_style = Style::default()
            .fg(Color::Rgb(0, 0, 0))
            .bg(Color::Rgb(85, 85, 85));
        // Right shadow
        for row in 1..dropdown_height {
            let y = dropdown_area.y + row;
            if y < area.height {
                frame.render_widget(
                    Paragraph::new(" ").style(shadow_style),
                    Rect::new(dropdown_area.x + dropdown_width, y, 1, 1),
                );
            }
        }
        // Bottom shadow
        let bottom_y = dropdown_area.y + dropdown_height;
        if bottom_y < area.height {
            let shadow_width = dropdown_width.min(area.width - dropdown_area.x - 1);
            frame.render_widget(
                Paragraph::new(" ".repeat(shadow_width as usize + 1)).style(shadow_style),
                Rect::new(dropdown_area.x + 1, bottom_y, shadow_width + 1, 1),
            );
        }
    }
}

/// Draw a single file panel (DN-style: double-line frame when active, single when inactive)
fn draw_panel(
    frame: &mut Frame,
    panel: &mut Panel,
    area: Rect,
    is_active: bool,
    show_search: bool,
    search_text: Option<&str>,
    file_desc: Option<&str>,
) {
    let border_style = if is_active {
        Theme::panel_border_active()
    } else {
        Theme::panel_border_inactive()
    };

    let title_style = if is_active {
        Theme::panel_title_active()
    } else {
        Theme::panel_title_inactive()
    };

    // Truncate path for title
    let path_str = panel.path.to_string_lossy();
    let filter_str = panel.filter.as_ref()
        .filter(|f| *f != "__branch__")
        .map(|f| format!(" [{}]", f))
        .unwrap_or_default();
    let branch_str = if panel.filter.as_deref() == Some("__branch__") { " [Branch]" } else { "" };
    let max_title = (area.width as usize).saturating_sub(4 + filter_str.len() + branch_str.len());
    let title = if path_str.len() > max_title {
        format!("…{}{}{}", &path_str[path_str.len() - max_title + 1..], filter_str, branch_str)
    } else {
        format!("{}{}{}", path_str, filter_str, branch_str)
    };

    // DN uses double-line frame for active panel, single-line for inactive
    let border_set = if is_active {
        border::DOUBLE
    } else {
        border::PLAIN
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled(format!(" {} ", title), title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 || inner.width < 10 {
        return;
    }

    // DN layout inside panel:
    //   1 line   column headers
    //   N lines  file list
    //   1 line   separator (─── with ┤ ├ at edges)
    //   1 line   current file info (name, size, date)
    //   1 line   selection/free info
    let info_lines: u16 = 3; // separator + file info + selection info

    let panel_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),             // Column headers
            Constraint::Min(1),               // File list
            Constraint::Length(info_lines),    // Info area (separator + 2 info lines)
        ])
        .split(inner);

    // Update visible_height for scrolling - in brief mode, multiply by columns
    let file_list_height = panel_layout[1].height as usize;
    let inner_width = inner.width as usize;
    let num_columns = match panel.panel_mode {
        PanelMode::Brief => 3, // DN always shows exactly 3 columns in Brief mode
        PanelMode::Full => 1,
    };
    panel.visible_height = file_list_height * num_columns;

    // Column headers
    draw_column_headers(frame, panel, panel_layout[0], num_columns);

    // File list
    draw_file_list(frame, panel, panel_layout[1], is_active, num_columns);

    // Info area
    draw_info_area(frame, panel, panel_layout[2], inner_width, is_active, border_set, file_desc);

    // Quick search overlay
    if show_search {
        if let Some(text) = search_text {
            let search_area = Rect::new(
                area.x + 1,
                area.y + area.height - 1,
                (text.len() as u16 + 10).min(area.width - 2),
                1,
            );
            let search_widget = Paragraph::new(Line::from(vec![
                Span::styled(" Search: ", Theme::quick_search()),
                Span::styled(text, Theme::quick_search()),
                Span::styled("▌", Theme::quick_search()),
            ]));
            frame.render_widget(search_widget, search_area);
        }
    }
}

/// Draw column headers (DN-style: "Name │ Name │ Name" in brief mode)
fn draw_column_headers(frame: &mut Frame, panel: &Panel, area: Rect, num_columns: usize) {
    let width = area.width as usize;

    let header = match panel.panel_mode {
        PanelMode::Full => {
            let sort_indicator = |mode: SortMode| {
                if panel.sort_mode == mode {
                    "▼"
                } else {
                    ""
                }
            };

            let name_w = width.saturating_sub(30);
            Line::from(vec![
                Span::styled(
                    format!(" {:<width$}", format!("Name{}", sort_indicator(SortMode::Name)), width = name_w),
                    Theme::column_header(),
                ),
                Span::styled(
                    format!("{:>10}", format!("Size{}", sort_indicator(SortMode::Size))),
                    Theme::column_header(),
                ),
                Span::styled(
                    format!("{:>18} ", format!("Modified{}", sort_indicator(SortMode::Date))),
                    Theme::column_header(),
                ),
            ])
        }
        PanelMode::Brief => {
            // DN brief mode: "  Name  │  Name  │  Name  "
            let mut spans = Vec::new();
            let col_w = if num_columns > 0 {
                (width.saturating_sub(num_columns - 1)) / num_columns
            } else {
                width
            };

            for c in 0..num_columns {
                if c > 0 {
                    spans.push(Span::styled("│", Theme::panel_border_active()));
                }
                let label_w = if c == num_columns - 1 {
                    // Last column takes remaining space
                    width.saturating_sub(c * (col_w + 1))
                } else {
                    col_w
                };
                spans.push(Span::styled(
                    format!("{:^width$}", "Name", width = label_w),
                    Theme::column_header(),
                ));
            }
            Line::from(spans)
        }
    };

    frame.render_widget(Paragraph::new(header), area);
}

/// Draw the file list (multi-column in brief mode, like DN)
fn draw_file_list(frame: &mut Frame, panel: &Panel, area: Rect, is_active: bool, num_columns: usize) {
    let width = area.width as usize;
    let height = area.height as usize;

    match panel.panel_mode {
        PanelMode::Brief => {
            // DN brief mode: files flow column-first (down then right)
            // Each column is col_w chars wide, separated by │
            let col_w = if num_columns > 0 {
                (width.saturating_sub(num_columns - 1)) / num_columns
            } else {
                width
            };

            for row in 0..height {
                let mut spans = Vec::new();
                for col in 0..num_columns {
                    let idx = panel.scroll_offset + col * height + row;

                    if col > 0 {
                        spans.push(Span::styled("│", Theme::panel_border_inactive()));
                    }

                    let this_col_w = if col == num_columns - 1 {
                        width.saturating_sub(col * (col_w + 1))
                    } else {
                        col_w
                    };

                    if idx < panel.entries.len() {
                        let entry = &panel.entries[idx];
                        let is_cursor = idx == panel.cursor;
                        let style = Theme::file_style(entry, is_cursor, is_active);

                        let tag = if entry.selected { "►" } else { " " };
                        let name = format!("{}{}", tag, entry.name);
                        let display = if name.len() > this_col_w {
                            name[..this_col_w].to_string()
                        } else {
                            format!("{:<width$}", name, width = this_col_w)
                        };
                        spans.push(Span::styled(display, style));
                    } else {
                        spans.push(Span::styled(
                            " ".repeat(this_col_w),
                            Theme::panel_bg(),
                        ));
                    }
                }
                let line = Line::from(spans);
                let line_area = Rect::new(area.x, area.y + row as u16, area.width, 1);
                frame.render_widget(Paragraph::new(line), line_area);
            }
        }
        PanelMode::Full => {
            let mut lines = Vec::new();
            for i in 0..height {
                let idx = panel.scroll_offset + i;
                if idx >= panel.entries.len() {
                    lines.push(Line::from(Span::styled(
                        " ".repeat(width),
                        Theme::panel_bg(),
                    )));
                    continue;
                }

                let entry = &panel.entries[idx];
                let is_cursor = idx == panel.cursor;
                let style = Theme::file_style(entry, is_cursor, is_active);

                let name_w = width.saturating_sub(30);
                let tag = if entry.selected { "►" } else { " " };
                let name = format!(
                    "{}{}",
                    tag,
                    if entry.name.len() > name_w - 1 {
                        &entry.name[..name_w - 1]
                    } else {
                        &entry.name
                    }
                );
                let size = entry.formatted_size();
                let date = entry.formatted_date();

                let line = Line::from(vec![
                    Span::styled(format!("{:<width$}", name, width = name_w), style),
                    Span::styled(format!("{:>10}", size), style),
                    Span::styled(format!("{:>18} ", date), style),
                ]);
                lines.push(line);
            }

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, area);

            // Scrollbar
            if panel.entries.len() > height {
                let mut scrollbar_state = ScrollbarState::new(panel.entries.len())
                    .position(panel.scroll_offset);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));
                frame.render_stateful_widget(
                    scrollbar,
                    area,
                    &mut scrollbar_state,
                );
            }
        }
    }
}

/// Draw panel info area (DN-style: separator line + file details + selection info)
fn draw_info_area(
    frame: &mut Frame,
    panel: &Panel,
    area: Rect,
    _inner_width: usize,
    _is_active: bool,
    border_set: border::Set,
    file_desc: Option<&str>,
) {
    if area.height < 3 {
        return;
    }
    let width = area.width as usize;

    // Line 0: separator ─── connecting to panel frame
    {
        let h_char = if border_set.horizontal_top == "═" { "═" } else { "─" };
        let sep: String = h_char.repeat(width);
        let sep_area = Rect::new(area.x, area.y, area.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(sep, Theme::panel_border_active()))),
            sep_area,
        );
    }

    // Line 1: current file info (filename, size, date/time) or description - like DN
    {
        let info = if let Some(desc) = file_desc {
            // Show file description when desc_panel_visible
            format!(" {}", desc)
        } else if let Some(entry) = panel.current_entry() {
            if entry.name == ".." {
                format!(" {:<12}    <UP--DIR>", "..")
            } else if entry.is_dir {
                let date_str = entry.formatted_date();
                format!(" {:<12}    <SUB-DIR>  {}",
                    if entry.name.len() > 12 { &entry.name[..12] } else { &entry.name },
                    date_str)
            } else {
                let date_str = entry.formatted_date();
                format!(" {:<12} {:>10}  {}",
                    if entry.name.len() > 12 { &entry.name[..12] } else { &entry.name },
                    entry.size,
                    date_str)
            }
        } else {
            String::new()
        };

        let info_area = Rect::new(area.x, area.y + 1, area.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{:<width$}", info, width = width),
                Theme::info_line(),
            ))),
            info_area,
        );
    }

    // Line 2: selection summary or total files summary
    {
        let (total_files, total_size, sel_count, sel_size) = panel.get_info();
        let summary = if sel_count > 0 {
            format!(" {} bytes in {} selected files", 
                format_with_commas(sel_size), sel_count)
        } else {
            format!(" {} files, {} bytes",
                total_files, format_with_commas(total_size))
        };

        let sum_area = Rect::new(area.x, area.y + 2, area.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{:<width$}", summary, width = width),
                Theme::info_line(),
            ))),
            sum_area,
        );
    }
}

/// Format number with comma separators (DN-style)
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let bytes: Vec<u8> = s.bytes().collect();
    let mut result = Vec::new();
    for (i, &b) in bytes.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(b',');
        }
        result.push(b);
    }
    result.reverse();
    String::from_utf8(result).unwrap_or_else(|_| s)
}

/// Draw command line
fn draw_command_line(frame: &mut Frame, app: &App, area: Rect) {
    let path = app.active_panel().path.to_string_lossy();

    let line = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::styled(format!(" {} ", msg), Theme::command_line()),
        ])
    } else if matches!(app.mode, AppMode::CommandLine) {
        Line::from(vec![
            Span::styled(
                format!(" {}> {}", path, app.command_line),
                Theme::command_line(),
            ),
            Span::styled("▌", Theme::command_line()),
        ])
    } else {
        Line::from(vec![Span::styled(
            format!(" {}> ", path),
            Theme::command_line(),
        )])
    };

    // Pad to full width
    let width = area.width as usize;
    let content_len: usize = line.spans.iter().map(|s| s.content.len()).sum();
    let mut spans = line.spans;
    if content_len < width {
        spans.push(Span::styled(
            " ".repeat(width - content_len),
            Theme::command_line(),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Draw function keys bar
fn draw_function_keys(frame: &mut Frame, _app: &App, area: Rect) {
    let keys = [
        ("1", "Help"),
        ("2", "Menu"),
        ("3", "View"),
        ("4", "Edit"),
        ("5", "Copy"),
        ("6", "Move"),
        ("7", "MkDir"),
        ("8", "Delete"),
        ("9", "Sort"),
        ("10", "Quit"),
    ];

    let width = area.width as usize;
    let key_width = width / 10;

    let mut spans = Vec::new();
    for (num, label) in &keys {
        spans.push(Span::styled(
            format!("{}", num),
            Theme::fn_key_number(),
        ));
        let label_w = key_width.saturating_sub(num.len());
        spans.push(Span::styled(
            format!("{:<width$}", label, width = label_w),
            Theme::fn_key_label(),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Draw a dialog
fn draw_dialog(frame: &mut Frame, app: &App, kind: DialogKind, area: Rect) {
    // Per-dialog widths (capped to screen minus 4 for margin)
    let max_w = area.width.saturating_sub(4);
    let dialog_width = match &kind {
        DialogKind::Input { title, value, .. } => {
            // Wide enough for title + 4 padding, or 60, whichever is larger
            let need = (title.len() + 6)
                .max(value.len() + 6)
                .max(60) as u16;
            need.min(max_w)
        }
        DialogKind::Confirm { .. } => 54u16.min(max_w),
        DialogKind::Error(_) => 54u16.min(max_w),
        DialogKind::CompareResult(_) => 54u16.min(max_w),
        DialogKind::FileInfo => 60u16.min(max_w),
        DialogKind::Attributes => 54u16.min(max_w),
        DialogKind::AttributesEdit { .. } => 54u16.min(max_w),
        DialogKind::SortMenu => 36u16.min(max_w),
        DialogKind::ConfirmSettings { .. } => 40u16.min(max_w),
    };
    let dialog_height = match &kind {
        DialogKind::Confirm { .. } => 7,
        DialogKind::Input { .. } => 7,
        DialogKind::Error(_) => 6,
        DialogKind::FileInfo => 12,
        DialogKind::Attributes => 10,
        DialogKind::AttributesEdit { .. } => 16,
        DialogKind::CompareResult(_) => 7,
        DialogKind::SortMenu => 10,
        DialogKind::ConfirmSettings { .. } => 9,
    };

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    match kind {
        DialogKind::Confirm { title, message, .. } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
                .title(Span::styled(
                    format!(" {} ", title),
                    Theme::dialog_border().add_modifier(Modifier::BOLD),
                ));

            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);

            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" {}", message),
                    Theme::dialog_text(),
                ))),
                content_layout[1],
            );

            let buttons = Line::from(vec![
                Span::styled("  ", Theme::dialog_text()),
                Span::styled(" Yes (Enter) ", Theme::dialog_button_focused()),
                Span::styled("   ", Theme::dialog_text()),
                Span::styled(" No (Esc) ", Theme::dialog_button()),
            ]);
            frame.render_widget(Paragraph::new(buttons), content_layout[3]);
        }
        DialogKind::Input {
            title,
            prompt,
            value,
            ..
        } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
                .title(Span::styled(
                    format!(" {} ", title),
                    Theme::dialog_border().add_modifier(Modifier::BOLD),
                ));

            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);

            // inner height = 5: [0] spacer | [1] prompt | [2] input | [3] spacer | [4] buttons
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" {}", prompt),
                    Theme::dialog_text(),
                ))),
                content_layout[1],
            );

            // Input field occupies inner width minus 2 margin chars (1 each side).
            // Of that field, 1 column is reserved for the cursor '▌'.
            let field_w = inner.width.saturating_sub(2) as usize; // total field width
            let text_w  = field_w.saturating_sub(1);               // visible value chars
            let display_value = if value.len() > text_w {
                &value[value.len() - text_w..]
            } else {
                &value
            };
            // Left-pad value to text_w, then append cursor — exactly field_w chars
            let field_text = format!("{:<width$}▌", display_value, width = text_w);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" ", Theme::dialog_text()),
                    Span::styled(field_text, Theme::dialog_input()),
                    Span::styled(" ", Theme::dialog_text()),
                ])),
                content_layout[2],
            );

            let buttons = Line::from(vec![
                Span::styled("  ", Theme::dialog_text()),
                Span::styled(" OK (Enter) ", Theme::dialog_button_focused()),
                Span::styled("   ", Theme::dialog_text()),
                Span::styled(" Cancel (Esc) ", Theme::dialog_button()),
            ]);
            frame.render_widget(Paragraph::new(buttons), content_layout[4]);
        }
        DialogKind::Error(msg) => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::error())
                .title(Span::styled(" Error ", Theme::error()));

            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);

            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(&*msg, Theme::dialog_text()))),
                content_layout[1],
            );
        }
        DialogKind::FileInfo => {
            draw_file_info_dialog(frame, app, dialog_area);
        }
        DialogKind::Attributes => {
            draw_attributes_dialog(frame, app, dialog_area);
        }
        DialogKind::AttributesEdit { ref path, mode, readonly, cursor } => {
            draw_attributes_edit_dialog(frame, path, mode, readonly, cursor, dialog_area);
        }
        DialogKind::CompareResult(msg) => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
                .title(Span::styled(
                    " Compare Directories ",
                    Theme::dialog_border().add_modifier(Modifier::BOLD),
                ));
            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(&*msg, Theme::dialog_text()))),
                content_layout[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " Press Enter to close ",
                    Theme::dialog_button_focused(),
                ))),
                content_layout[3],
            );
        }
        DialogKind::SortMenu => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
                .title(Span::styled(
                    " Sort by ",
                    Theme::dialog_border().add_modifier(Modifier::BOLD),
                ));
            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);
            let items = vec![
                Line::from(Span::styled("  N - Name", Theme::dialog_text())),
                Line::from(Span::styled("  E - Extension", Theme::dialog_text())),
                Line::from(Span::styled("  S - Size", Theme::dialog_text())),
                Line::from(Span::styled("  D - Date", Theme::dialog_text())),
                Line::from(Span::styled("", Theme::dialog_text())),
                Line::from(Span::styled("  Esc - Cancel", Theme::dialog_text())),
            ];
            let content_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(1));
            frame.render_widget(Paragraph::new(items), content_area);
        }
        DialogKind::ConfirmSettings { cursor } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
                .title(Span::styled(
                    " Confirmation settings ",
                    Theme::dialog_border().add_modifier(Modifier::BOLD),
                ));
            let inner = block.inner(dialog_area);
            frame.render_widget(block, dialog_area);

            let checkboxes = [
                ("Confirm delete",    app.confirm_delete),
                ("Confirm overwrite", app.confirm_overwrite),
                ("Confirm on exit",   app.confirm_exit),
            ];

            let mut lines: Vec<Line> = Vec::new();
            for (i, (label, checked)) in checkboxes.iter().enumerate() {
                let box_char = if *checked { "[X] " } else { "[ ] " };
                let style = if cursor as usize == i {
                    Theme::cursor_active()
                } else {
                    Theme::dialog_text()
                };
                lines.push(Line::from(Span::styled(format!("  {}{}", box_char, label), style)));
            }
            lines.push(Line::from(Span::raw("")));
            lines.push(Line::from(Span::styled(
                "  Enter=toggle  Esc/F10=close",
                Theme::dialog_text(),
            )));

            let content_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(1));
            frame.render_widget(Paragraph::new(lines), content_area);
        }
    }
}

/// Draw file info dialog
fn draw_file_info_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " File Info ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(entry) = app.active_panel().current_entry() {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Name: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(entry.name.clone(), Theme::dialog_text()),
            ]),
            Line::from(vec![
                Span::styled(" Path: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(
                    entry.path.to_string_lossy().to_string(),
                    Theme::dialog_text(),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Type: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(
                    if entry.is_dir {
                        "Directory"
                    } else if entry.is_symlink {
                        "Symlink"
                    } else {
                        "File"
                    },
                    Theme::dialog_text(),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Size: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} bytes ({})", entry.size, entry.formatted_size()),
                    Theme::dialog_text(),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Modified: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(entry.formatted_date(), Theme::dialog_text()),
            ]),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled(" Attrs: ", Theme::dialog_text().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(
                        "{}{}{}",
                        if entry.is_readonly { "R " } else { "" },
                        if entry.is_hidden { "H " } else { "" },
                        if entry.is_executable { "X" } else { "" },
                    ),
                    Theme::dialog_text(),
                ),
            ]),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("       ", Theme::dialog_text()),
                Span::styled(" OK (Enter/Esc) ", Theme::dialog_button_focused()),
            ]),
        ];

        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// Draw the file viewer
fn draw_viewer(frame: &mut Frame, app: &mut App, area: Rect) {
    let _viewer = match &app.viewer {
        Some(v) => v,
        None => return,
    };

    // Layout: header | content | footer
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),   // Content
            Constraint::Length(1), // Footer / keys
        ])
        .split(area);

    // Update viewer dimensions
    if let Some(ref mut v) = app.viewer {
        v.visible_height = layout[1].height as usize;
        v.visible_width = layout[1].width as usize;
    }
    let viewer = app.viewer.as_ref().unwrap();

    // Header
    let mode_str = match viewer.mode {
        ViewerMode::Text => "Text",
        ViewerMode::Hex => "Hex",
    };
    let filename = viewer.path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let header = Line::from(vec![
        Span::styled(
            format!(" {} - {} [{}] ({} bytes) ",
                filename,
                mode_str,
                if viewer.wrap { "Wrap" } else { "NoWrap" },
                viewer.file_size,
            ),
            Theme::viewer_header(),
        ),
        Span::styled(
            format!(
                "{:>width$}",
                format!(
                    "Line {}/{} ",
                    viewer.scroll_offset + 1,
                    viewer.total_lines()
                ),
                width = (area.width as usize).saturating_sub(filename.len() + mode_str.len() + 30)
            ),
            Theme::viewer_header(),
        ),
    ]);
    frame.render_widget(Paragraph::new(header), layout[0]);

    // Content
    let content_area = layout[1];
    let height = content_area.height as usize;
    let width = content_area.width as usize;

    match viewer.mode {
        ViewerMode::Text => {
            let mut lines = Vec::new();
            for i in 0..height {
                let line_idx = viewer.scroll_offset + i;
                if line_idx >= viewer.lines.len() {
                    lines.push(Line::from(Span::styled(
                        " ".repeat(width),
                        Theme::viewer_text(),
                    )));
                    continue;
                }

                let line_text = &viewer.lines[line_idx];
                let display = if viewer.horizontal_offset < line_text.len() {
                    &line_text[viewer.horizontal_offset..]
                } else {
                    ""
                };

                // Highlight search matches
                if let Some(ref query) = viewer.search_query {
                    let display_lower = display.to_lowercase();
                    let query_lower = query.to_lowercase();
                    let mut spans = Vec::new();
                    let mut last = 0;

                    for (pos, _) in display_lower.match_indices(&query_lower) {
                        if pos > last {
                            spans.push(Span::styled(
                                &display[last..pos],
                                Theme::viewer_text(),
                            ));
                        }
                        let end = pos + query.len();
                        spans.push(Span::styled(
                            &display[pos..end.min(display.len())],
                            Theme::quick_search(),
                        ));
                        last = end;
                    }
                    if last < display.len() {
                        spans.push(Span::styled(
                            &display[last..],
                            Theme::viewer_text(),
                        ));
                    }
                    if spans.is_empty() {
                        spans.push(Span::styled(
                            format!("{:<width$}", display, width = width),
                            Theme::viewer_text(),
                        ));
                    }
                    lines.push(Line::from(spans));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("{:<width$}", display, width = width),
                        Theme::viewer_text(),
                    )));
                }
            }
            frame.render_widget(Paragraph::new(lines), content_area);
        }
        ViewerMode::Hex => {
            let hex_lines = viewer.hex_lines(viewer.scroll_offset, height);
            let mut lines = Vec::new();

            for (offset, hex, ascii) in &hex_lines {
                lines.push(Line::from(vec![
                    Span::styled(format!(" {} ", offset), Theme::viewer_hex_offset()),
                    Span::styled(format!("│ {} ", hex), Theme::viewer_hex_bytes()),
                    Span::styled(format!("│ {} ", ascii), Theme::viewer_hex_ascii()),
                ]));
            }

            // Pad remaining lines
            while lines.len() < height {
                lines.push(Line::from(Span::styled(
                    " ".repeat(width),
                    Theme::viewer_text(),
                )));
            }

            frame.render_widget(Paragraph::new(lines), content_area);
        }
    }

    // Footer / keys
    let viewer_keys = [
        ("1", "Help"),
        ("2", "Wrap"),
        ("3", "Quit"),
        ("4", "Hex"),
        ("5", ""),
        ("6", ""),
        ("7", "Search"),
        ("8", ""),
        ("9", ""),
        ("10", "Quit"),
    ];

    let key_width = area.width as usize / 10;
    let mut spans = Vec::new();
    for (num, label) in &viewer_keys {
        spans.push(Span::styled(format!("{}", num), Theme::fn_key_number()));
        let label_w = key_width.saturating_sub(num.len());
        spans.push(Span::styled(
            format!("{:<width$}", label, width = label_w),
            Theme::fn_key_label(),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), layout[2]);
}

/// Format a byte size for display
fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// DOS color for a Tetris piece (from DN: color = 15 - fig mod 7)
fn piece_color(color_idx: u8) -> ratatui::style::Color {
    use ratatui::style::Color;
    match (color_idx - 1) % 7 {
        0 => Color::Rgb(255, 255, 255), // white (15)
        1 => Color::Rgb(255, 255, 85),  // yellow (14)
        2 => Color::Rgb(255, 85, 255),  // light magenta (13)
        3 => Color::Rgb(255, 85, 85),   // light red (12)
        4 => Color::Rgb(85, 255, 255),  // light cyan (11)
        5 => Color::Rgb(85, 255, 85),   // light green (10)
        6 => Color::Rgb(85, 85, 255),   // light blue (9)
        _ => Color::Rgb(85, 85, 85),
    }
}

/// Draw the Tetris game as a dialog overlaying the file manager (like DN)
fn draw_tetris(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    // Tick
    if let Some(ref mut tetris) = app.tetris {
        tetris.tick();
    }
    let tetris = match &app.tetris {
        Some(t) => t,
        None => return,
    };

    // DOS palette colours
    let gray_bg  = Color::Rgb(170, 170, 170);
    let black    = Color::Rgb(0, 0, 0);
    let white    = Color::Rgb(255, 255, 255);
    let yellow   = Color::Rgb(255, 255, 85);
    let dark_gray = Color::Rgb(85, 85, 85);
    let cyan     = Color::Rgb(0, 170, 170);

    // ── Dialog dimensions ───────────────────────────────────────────
    let board_inner_w = (BOARD_WIDTH as u16) * 2;   // 20
    let board_inner_h = BOARD_HEIGHT as u16;         // 20
    let board_w = board_inner_w + 2;                 // 22 (incl border)
    let board_h = board_inner_h + 2;                 // 22
    let sidebar_w = 22u16;
    let dialog_inner_w = board_w + 1 + sidebar_w;    // 45
    let dialog_inner_h = board_h;                     // 22
    let dialog_w = dialog_inner_w + 2;                // 47
    let dialog_h = dialog_inner_h + 2;                // 24

    let dx = area.x + area.width.saturating_sub(dialog_w) / 2;
    let dy = area.y + area.height.saturating_sub(dialog_h) / 2;
    let dialog_area = Rect::new(
        dx, dy,
        dialog_w.min(area.width),
        dialog_h.min(area.height),
    );

    // ── Dialog frame (gray background, white double border) ─────────
    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);
    let dialog_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(white).bg(gray_bg))
        .title(Span::styled(
            " Navigator's game ",
            Style::default().fg(white).bg(gray_bg).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(gray_bg));
    let di = dialog_block.inner(dialog_area);
    frame.render_widget(dialog_block, dialog_area);

    // ── Game board (black interior, framed) ─────────────────────────
    let board_area = Rect::new(
        di.x, di.y,
        board_w.min(di.width),
        board_h.min(di.height),
    );
    let board_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(white).bg(gray_bg))
        .style(Style::default().bg(black));
    let bi = board_block.inner(board_area);
    frame.render_widget(board_block, board_area);

    // Draw board cells
    for row in 0..BOARD_HEIGHT {
        if row as u16 >= bi.height { break; }
        let mut spans = Vec::new();
        for col in 0..BOARD_WIDTH {
            let cell = tetris.board[row][col];
            let mut is_current = false;
            let mut cur_color = 0u8;

            if !tetris.game_over && !tetris.paused {
                let shape = tetris.current.shape();
                for r in 0..4 {
                    for c in 0..4 {
                        if shape[r][c]
                            && tetris.current.y + r as i32 == row as i32
                            && tetris.current.x + c as i32 == col as i32
                        {
                            is_current = true;
                            cur_color = tetris.current.piece.color_index();
                        }
                    }
                }
            }

            if is_current {
                spans.push(Span::styled("██", Style::default().fg(piece_color(cur_color)).bg(black)));
            } else if cell != 0 {
                spans.push(Span::styled("██", Style::default().fg(piece_color(cell)).bg(black)));
            } else {
                spans.push(Span::styled("  ", Style::default().bg(black)));
            }
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(bi.x, bi.y + row as u16, bi.width, 1),
        );
    }

    // ── Sidebar ─────────────────────────────────────────────────────
    let side_x = board_area.x + board_area.width + 1;
    let side_y = di.y;
    if side_x >= di.x + di.width { return; }
    let side_w = (di.x + di.width).saturating_sub(side_x);

    let text_s = Style::default().fg(black).bg(gray_bg);
    let head_s = Style::default().fg(white).bg(gray_bg).add_modifier(Modifier::BOLD);

    // ── Info box ─────────────────────────────────────────────────────
    let info_h = 5u16;
    if info_h <= di.height {
        let info_area = Rect::new(side_x, side_y, side_w, info_h);
        let info_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(black).bg(gray_bg))
            .title(Span::styled(" Info ", head_s))
            .style(Style::default().bg(gray_bg));
        let ii = info_block.inner(info_area);
        frame.render_widget(info_block, info_area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(format!("Score: {}", tetris.score), text_s)),
                Line::from(Span::styled(format!("Lines: {}", tetris.lines_cleared), text_s)),
                Line::from(Span::styled(format!("Level: {}", tetris.level), text_s)),
            ]),
            ii,
        );
    }

    // ── Next piece box ──────────────────────────────────────────────
    let next_y = side_y + info_h;
    let next_h = 8u16;
    if next_y + next_h <= di.y + di.height {
        let next_area = Rect::new(side_x, next_y, side_w, next_h);
        let next_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(black).bg(gray_bg))
            .title(Span::styled(" Next ", head_s))
            .style(Style::default().bg(black));
        let ni = next_block.inner(next_area);
        frame.render_widget(next_block, next_area);

        if !tetris.paused {
            let nshape = tetris.next.shape(0);
            let nc = piece_color(tetris.next.color_index());
            for r in 0..4 {
                if r as u16 >= ni.height { break; }
                let mut sp = Vec::new();
                for c in 0..4 {
                    if nshape[r][c] {
                        sp.push(Span::styled("██", Style::default().fg(nc).bg(black)));
                    } else {
                        sp.push(Span::styled("  ", Style::default().bg(black)));
                    }
                }
                frame.render_widget(
                    Paragraph::new(Line::from(sp)),
                    Rect::new(
                        ni.x + (ni.width.saturating_sub(8)) / 2,
                        ni.y + 1 + r as u16,
                        8, 1,
                    ),
                );
            }
        }
    }

    // ── Best section (yellow border) ────────────────────────────────
    let best_y = next_y + next_h;
    let best_h = 4u16;
    if best_y + best_h <= di.y + di.height {
        let best_area = Rect::new(side_x, best_y, side_w, best_h);
        let best_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(yellow).bg(gray_bg))
            .title(Span::styled(
                " Best ",
                Style::default().fg(yellow).bg(gray_bg).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(gray_bg));
        let bsi = best_block.inner(best_area);
        frame.render_widget(best_block, best_area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("Name: Anonymous", text_s)),
                Line::from(Span::styled(format!("Score: {}", tetris.score), text_s)),
            ]),
            bsi,
        );
    }

    // ── Buttons ──────────────────────────────────────────────────────
    let btn_y = best_y + best_h;
    let btn_s = Style::default().fg(white).bg(dark_gray).add_modifier(Modifier::BOLD);
    let half = side_w / 2;
    if btn_y + 4 <= di.y + di.height {
        // Row 1: New  Setup
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("  New   ", btn_s))),
            Rect::new(side_x, btn_y + 1, half, 1),
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(" Setup  ", btn_s))),
            Rect::new(side_x + half + 1, btn_y + 1, half, 1),
        );
        // Row 2: Top 10  Pause
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(" Top 10 ", btn_s))),
            Rect::new(side_x, btn_y + 3, half, 1),
        );
        let plbl = if tetris.paused { " Resume " } else { " Pause  " };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(plbl, btn_s))),
            Rect::new(side_x + half + 1, btn_y + 3, half, 1),
        );
    }

    // ── Tetris function-key bar (overwrites the normal one) ─────────
    let fn_y = area.y + area.height - 1;
    let num_s = Style::default().fg(white).bg(black);
    let lbl_s = Style::default().fg(black).bg(cyan);
    let fn_spans = vec![
        Span::styled("F1", num_s),
        Span::styled(" Help  ", lbl_s),
        Span::styled("F2", num_s),
        Span::styled(" New   ", lbl_s),
        Span::styled("F3", num_s),
        Span::styled(" Pause ", lbl_s),
        Span::styled(" + ", num_s),
        Span::styled(" Level ", lbl_s),
        Span::styled(" * ", num_s),
        Span::styled(" Prev  ", lbl_s),
        Span::styled("       ", lbl_s),
        Span::styled("       ", lbl_s),
        Span::styled("Esc", num_s),
        Span::styled(" Quit", lbl_s),
    ];
    // Fill the bar
    frame.render_widget(
        Paragraph::new(Line::from(fn_spans)),
        Rect::new(area.x, fn_y, area.width, 1),
    );

    // ── Game Over overlay ───────────────────────────────────────────
    if tetris.game_over {
        let red = Color::Rgb(255, 85, 85);
        let ow = 22u16;
        let oh = 5u16;
        let ox = board_area.x + board_area.width.saturating_sub(ow) / 2;
        let oy = board_area.y + board_area.height.saturating_sub(oh) / 2;
        let oa = Rect::new(ox, oy, ow, oh);
        frame.render_widget(Clear, oa);
        let ob = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(red).bg(black))
            .style(Style::default().bg(black));
        let oi = ob.inner(oa);
        frame.render_widget(ob, oa);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "    GAME OVER    ",
                    Style::default().fg(red).bg(black).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  Score: {}  ", tetris.score),
                    Style::default().fg(yellow).bg(black),
                )),
                Line::from(Span::styled(
                    " F2=New   Esc=Quit",
                    Style::default().fg(Color::Rgb(170, 170, 170)).bg(black),
                )),
            ]),
            oi,
        );
    } else if tetris.paused {
        let ow = 14u16;
        let oh = 3u16;
        let ox = board_area.x + board_area.width.saturating_sub(ow) / 2;
        let oy = board_area.y + board_area.height.saturating_sub(oh) / 2;
        let pa = Rect::new(ox, oy, ow, oh);
        frame.render_widget(Clear, pa);
        let pb = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(yellow).bg(black))
            .style(Style::default().bg(black));
        let pi = pb.inner(pa);
        frame.render_widget(pb, pa);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  PAUSED  ",
                Style::default().fg(yellow).bg(black).add_modifier(Modifier::BOLD),
            ))),
            pi,
        );
    }
}
// ═══════════════════════════════════════════════════════════════════
// New mode draw functions
// ═══════════════════════════════════════════════════════════════════

/// Draw the built-in text editor (full screen)
fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::editor::EditorInputMode;
    use ratatui::style::{Color, Style};

    let editor = match &app.editor {
        Some(e) => e,
        None => return,
    };

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);
    let white = Color::Rgb(255, 255, 255);
    let yellow = Color::Rgb(255, 255, 85);
    let bg = Color::Rgb(0, 0, 170);

    // Layout: title bar + editing area + status bar + input bar
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // title bar
            Constraint::Min(3),    // editor content
            Constraint::Length(1), // status bar
            Constraint::Length(1), // input / function keys
        ])
        .split(area);

    // Title bar
    let title = editor.title();
    let modified_mark = if editor.modified { " [*]" } else { "" };
    let title_text = format!(" {}{} ", title, modified_mark);
    let title_line = Line::from(Span::styled(
        format!("{:<width$}", title_text, width = area.width as usize),
        Style::default().fg(black).bg(cyan),
    ));
    frame.render_widget(Paragraph::new(title_line), layout[0]);

    // Editor content
    let content_area = layout[1];
    let visible_h = content_area.height as usize;
    let visible_w = content_area.width as usize;

    let mut lines_display: Vec<Line> = Vec::new();
    for row in 0..visible_h {
        let line_idx = editor.scroll_y + row;
        if line_idx < editor.lines.len() {
            let line_text = &editor.lines[line_idx];
            let display_start = editor.scroll_x;
            let display_end = (display_start + visible_w).min(line_text.len());
            let visible_part = if display_start < line_text.len() {
                &line_text[display_start..display_end]
            } else {
                ""
            };

            let mut spans = Vec::new();
            for (ci, ch) in visible_part.chars().enumerate() {
                let col = display_start + ci;
                let in_sel = editor.is_in_selection(col, line_idx);
                let is_cursor = line_idx == editor.cursor_y && col == editor.cursor_x;
                let style = if is_cursor {
                    Style::default().fg(black).bg(yellow)
                } else if in_sel {
                    Style::default().fg(white).bg(cyan)
                } else {
                    Style::default().fg(yellow).bg(bg)
                };
                spans.push(Span::styled(ch.to_string(), style));
            }

            // Cursor at end of line
            let vis_len = visible_part.len();
            if line_idx == editor.cursor_y && editor.cursor_x >= display_start + vis_len && editor.cursor_x < display_start + visible_w {
                // Pad to cursor position
                let pad = editor.cursor_x - display_start - vis_len;
                if pad > 0 {
                    spans.push(Span::styled(" ".repeat(pad), Style::default().fg(yellow).bg(bg)));
                }
                spans.push(Span::styled(" ", Style::default().fg(black).bg(yellow)));
            }

            // Fill remaining width
            let used: usize = spans.iter().map(|s| s.content.len()).sum();
            if used < visible_w {
                spans.push(Span::styled(
                    " ".repeat(visible_w - used),
                    Style::default().fg(yellow).bg(bg),
                ));
            }

            lines_display.push(Line::from(spans));
        } else {
            lines_display.push(Line::from(Span::styled(
                " ".repeat(visible_w),
                Style::default().fg(yellow).bg(bg),
            )));
        }
    }
    frame.render_widget(Paragraph::new(lines_display), content_area);

    // Status bar
    let cursor_info = format!(
        " Ln {}, Col {} | {} lines | {} ",
        editor.cursor_y + 1,
        editor.cursor_x + 1,
        editor.lines.len(),
        if editor.insert_mode { "INS" } else { "OVR" }
    );
    let status_msg = editor.status_msg.as_deref().unwrap_or("");
    let status_text = format!(
        "{}{:<width$}",
        status_msg,
        cursor_info,
        width = area.width as usize - status_msg.len()
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status_text,
            Style::default().fg(black).bg(cyan),
        ))),
        layout[2],
    );

    // Input/function keys bar
    let input_line = match &editor.input_mode {
        EditorInputMode::Search => {
            Line::from(vec![
                Span::styled(" Search: ", Style::default().fg(white).bg(bg)),
                Span::styled(
                    format!("{}▌", editor.input_buffer),
                    Style::default().fg(yellow).bg(bg),
                ),
            ])
        }
        EditorInputMode::Replace => {
            Line::from(vec![
                Span::styled(" Replace with: ", Style::default().fg(white).bg(bg)),
                Span::styled(
                    format!("{}▌", editor.input_buffer),
                    Style::default().fg(yellow).bg(bg),
                ),
            ])
        }
        EditorInputMode::ReplaceConfirm => {
            Line::from(Span::styled(
                " Replace? (Y)es (N)o (A)ll (Esc)Cancel ",
                Style::default().fg(white).bg(bg),
            ))
        }
        EditorInputMode::SaveAs => {
            Line::from(vec![
                Span::styled(" Save as: ", Style::default().fg(white).bg(bg)),
                Span::styled(
                    format!("{}▌", editor.input_buffer),
                    Style::default().fg(yellow).bg(bg),
                ),
            ])
        }
        EditorInputMode::GotoLine => {
            Line::from(vec![
                Span::styled(" Go to line: ", Style::default().fg(white).bg(bg)),
                Span::styled(
                    format!("{}▌", editor.input_buffer),
                    Style::default().fg(yellow).bg(bg),
                ),
            ])
        }
        EditorInputMode::Normal => {
            // Show function key hints
            let keys = [
                ("F2", "Save"), ("F5", "Goto"), ("F7", "Search"),
                ("F8", "DelLn"), ("F10", "Quit"),
            ];
            let mut spans = Vec::new();
            for (num, label) in &keys {
                spans.push(Span::styled(
                    format!(" {} ", num),
                    Style::default().fg(black).bg(white),
                ));
                spans.push(Span::styled(
                    format!("{} ", label),
                    Style::default().fg(white).bg(bg),
                ));
            }
            Line::from(spans)
        }
    };
    frame.render_widget(
        Paragraph::new(input_line)
            .style(Style::default().bg(bg)),
        layout[3],
    );
}

/// Draw file find dialog overlay
fn draw_file_find(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let finder = match &app.finder {
        Some(f) => f,
        None => return,
    };

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);
    let yellow = Color::Rgb(255, 255, 85);

    let dialog_w = 60u16.min(area.width.saturating_sub(4));
    let dialog_h = 20u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Find File ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    if !finder.search_complete {
        // Input mode: show pattern field
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // label
                Constraint::Length(1), // pattern input
                Constraint::Length(1), // spacer
                Constraint::Length(1), // options
                Constraint::Length(1), // spacer
                Constraint::Length(1), // buttons
                Constraint::Min(0),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(" File mask:", Theme::dialog_text()))),
            layout[0],
        );
        let input_w = inner.width.saturating_sub(2) as usize;
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" ", Theme::dialog_text()),
                Span::styled(
                    format!("{:<width$}", format!("{}▌", finder.pattern), width = input_w),
                    Theme::dialog_input(),
                ),
            ])),
            layout[1],
        );

        let opts = format!(
            " [{}] Subdirs  [{}] Case sensitive",
            if finder.search_subdirs { "x" } else { " " },
            if finder.case_sensitive { "x" } else { " " },
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(opts, Theme::dialog_text()))),
            layout[3],
        );

        let buttons = Line::from(vec![
            Span::styled("  ", Theme::dialog_text()),
            Span::styled(" Search (Enter) ", Theme::dialog_button_focused()),
            Span::styled("   ", Theme::dialog_text()),
            Span::styled(" Cancel (Esc) ", Theme::dialog_button()),
        ]);
        frame.render_widget(Paragraph::new(buttons), layout[5]);
    } else {
        // Results mode
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // header
                Constraint::Min(2),   // result list
                Constraint::Length(1), // status
            ])
            .split(inner);

        let header = format!(
            " Found: {} files (pattern: {})",
            finder.results.len(),
            finder.pattern
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(header, Theme::dialog_text()))),
            layout[0],
        );

        // Results list
        let list_h = layout[1].height as usize;
        let mut result_lines: Vec<Line> = Vec::new();
        for i in 0..list_h {
            let idx = finder.scroll_offset + i;
            if idx < finder.results.len() {
                let r = &finder.results[idx];
                let is_cur = idx == finder.cursor;
                let style = if is_cur {
                    Style::default().fg(black).bg(cyan)
                } else {
                    Theme::dialog_text()
                };
                let text = format!(
                    " {:<width$} {:>8}",
                    r.name,
                    format_size(r.size),
                    width = (inner.width as usize).saturating_sub(12),
                );
                result_lines.push(Line::from(Span::styled(text, style)));
            }
        }
        frame.render_widget(Paragraph::new(result_lines), layout[1]);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" Enter", Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170))),
                Span::styled("=Go  ", Theme::dialog_text()),
                Span::styled("F7", Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170))),
                Span::styled("=New  ", Theme::dialog_text()),
                Span::styled("Esc", Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170))),
                Span::styled("=Close", Theme::dialog_text()),
            ])),
            layout[2],
        );
    }
}

/// Draw directory tree overlay
fn draw_dir_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let tree = match &app.dir_tree {
        Some(t) => t,
        None => return,
    };

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);

    let dialog_w = 50u16.min(area.width.saturating_sub(4));
    let dialog_h = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Directory Tree ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let vis_h = inner.height as usize;
    let mut tree_lines: Vec<Line> = Vec::new();
    for i in 0..vis_h {
        let idx = tree.scroll_offset + i;
        if idx < tree.nodes.len() {
            let node = &tree.nodes[idx];
            let is_cur = idx == tree.cursor;
            let indent = "  ".repeat(node.depth);
            let icon = if node.has_children {
                if node.expanded { "▼ " } else { "► " }
            } else {
                "  "
            };

            let style = if is_cur {
                Style::default().fg(black).bg(cyan)
            } else {
                Theme::dialog_text()
            };

            let text = format!(
                "{}{}{:<width$}",
                indent,
                icon,
                node.name,
                width = (inner.width as usize).saturating_sub(indent.len() + 2),
            );
            tree_lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(tree_lines), inner);
}

/// Draw calculator overlay
fn draw_calculator(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let calc = match &app.calculator {
        Some(c) => c,
        None => return,
    };

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);
    let white = Color::Rgb(255, 255, 255);
    let yellow = Color::Rgb(255, 255, 85);

    let dialog_w = 32u16;
    let dialog_h = 14u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Calculator ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // display
            Constraint::Length(1), // spacer
            Constraint::Length(1), // memory indicator
            Constraint::Length(1), // spacer
            Constraint::Min(1),   // history
            Constraint::Length(1), // help
        ])
        .split(inner);

    // Display
    let display_w = inner.width.saturating_sub(2) as usize;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Theme::dialog_text()),
            Span::styled(
                format!("{:>width$}", calc.display, width = display_w),
                Style::default().fg(white).bg(black),
            ),
        ])),
        layout[0],
    );

    // Memory indicator
    let mem_text = if calc.memory != 0.0 {
        format!(" M={}", calc.memory)
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(mem_text, Theme::dialog_text()))),
        layout[2],
    );

    // History
    let hist_h = layout[4].height as usize;
    let hist_start = calc.history.len().saturating_sub(hist_h);
    let mut hist_lines: Vec<Line> = Vec::new();
    for entry in &calc.history[hist_start..] {
        hist_lines.push(Line::from(Span::styled(
            format!(" {}", entry),
            Style::default().fg(cyan).bg(Color::Rgb(0, 0, 170)),
        )));
    }
    frame.render_widget(Paragraph::new(hist_lines), layout[4]);

    // Help line
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " 0-9 +-*/= S√ C=clr Esc",
            Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170)),
        ))),
        layout[5],
    );
}

/// Draw ASCII table overlay
fn draw_ascii_table(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let cursor = app.ascii_cursor;
    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);
    let yellow = Color::Rgb(255, 255, 85);

    let dialog_w = 54u16;
    let dialog_h = 22u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " ASCII Table ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Header row
    let mut header_spans = vec![Span::styled("    ", Theme::dialog_text())];
    for col in 0..16u8 {
        header_spans.push(Span::styled(
            format!("{:2X} ", col),
            Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170)),
        ));
    }
    if inner.height > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(header_spans)),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
    }

    // 16 rows of 16 characters
    for row in 0..16u8 {
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!(" {:X}_ ", row),
            Style::default().fg(yellow).bg(Color::Rgb(0, 0, 170)),
        ));
        for col in 0..16u8 {
            let ch_val = row * 16 + col;
            let ch = if ch_val >= 32 && ch_val < 127 {
                (ch_val as char).to_string()
            } else {
                ".".to_string()
            };
            let is_cur = ch_val == cursor;
            let style = if is_cur {
                Style::default().fg(black).bg(cyan)
            } else {
                Theme::dialog_text()
            };
            spans.push(Span::styled(format!(" {} ", ch), style));
        }

        let row_y = inner.y + 1 + row as u16;
        if row_y < inner.y + inner.height {
            frame.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(inner.x, row_y, inner.width, 1),
            );
        }
    }

    // Info line at bottom
    let info_y = inner.y + 18;
    if info_y < inner.y + inner.height {
        let ch_display = if cursor >= 32 && cursor < 127 {
            format!("'{}'", cursor as char)
        } else {
            format!("#{}", cursor)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" Char: {} | Dec: {} | Hex: {:02X} | Oct: {:03o}",
                    ch_display, cursor, cursor, cursor
                ),
                Theme::dialog_text(),
            ))),
            Rect::new(inner.x, info_y, inner.width, 1),
        );
    }
}

/// Draw disk info overlay
fn draw_disk_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let path = app.active_panel().path.clone();

    let dialog_w = 46u16;
    let dialog_h = 10u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Disk Information ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Try to get filesystem info using sys-info style approach
    // On macOS/Linux, parse from `df`-like data or use std APIs
    let display_path = path.to_string_lossy();
    let mut info_lines = vec![
        Line::from(Span::styled(
            format!(" Path: {}", display_path),
            Theme::dialog_text(),
        )),
    ];

    // statvfs info via nix or fallback
    #[cfg(unix)]
    {
        use std::ffi::CString;
        if let Ok(cpath) = CString::new(path.to_string_lossy().as_bytes()) {
            unsafe {
                let mut stat: libc::statvfs = std::mem::zeroed();
                if libc::statvfs(cpath.as_ptr(), &mut stat) == 0 {
                    let total = stat.f_blocks as u64 * stat.f_frsize as u64;
                    let free = stat.f_bavail as u64 * stat.f_frsize as u64;
                    let used = total.saturating_sub(free);
                    info_lines.push(Line::from(Span::styled(
                        format!(" Total:  {}", format_size_long(total)),
                        Theme::dialog_text(),
                    )));
                    info_lines.push(Line::from(Span::styled(
                        format!(" Free:   {}", format_size_long(free)),
                        Theme::dialog_text(),
                    )));
                    info_lines.push(Line::from(Span::styled(
                        format!(" Used:   {}", format_size_long(used)),
                        Theme::dialog_text(),
                    )));
                    let pct = if total > 0 {
                        (used as f64 / total as f64 * 100.0) as u64
                    } else {
                        0
                    };
                    info_lines.push(Line::from(Span::styled(
                        format!(" Usage:  {}%", pct),
                        Theme::dialog_text(),
                    )));
                }
            }
        }
    }

    info_lines.push(Line::from(Span::styled("", Theme::dialog_text())));
    info_lines.push(Line::from(Span::styled(
        " Press Enter or Esc to close",
        Theme::dialog_text(),
    )));

    frame.render_widget(Paragraph::new(info_lines), inner);
}

/// Draw select pattern input overlay
fn draw_select_pattern(frame: &mut Frame, app: &mut App, area: Rect) {
    let selecting = matches!(app.mode, AppMode::SelectPattern { selecting: true });
    let title = if selecting { " Select " } else { " Unselect " };

    let dialog_w = 40u16;
    let dialog_h = 6u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            title,
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(" Pattern:", Theme::dialog_text()))),
        layout[0],
    );

    let input_w = inner.width.saturating_sub(2) as usize;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Theme::dialog_text()),
            Span::styled(
                format!("{:<width$}", format!("{}▌", app.select_pattern_buf), width = input_w),
                Theme::dialog_input(),
            ),
        ])),
        layout[1],
    );
}

/// Draw file attributes dialog
fn draw_attributes_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " File Attributes ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(entry) = app.active_panel().current_entry() {
        let meta = std::fs::metadata(&entry.path);
        let mut lines = vec![
            Line::from(Span::styled(
                format!(" Name: {}", entry.name),
                Theme::dialog_text(),
            )),
            Line::from(Span::styled(
                format!(" Size: {}", format_size(entry.size)),
                Theme::dialog_text(),
            )),
        ];

        if let Ok(m) = &meta {
            let readonly = m.permissions().readonly();
            lines.push(Line::from(Span::styled(
                format!(" Read-only: {}", if readonly { "Yes" } else { "No" }),
                Theme::dialog_text(),
            )));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = m.permissions().mode();
                lines.push(Line::from(Span::styled(
                    format!(" Unix mode: {:o}", mode & 0o7777),
                    Theme::dialog_text(),
                )));
            }
        }

        lines.push(Line::from(Span::styled("", Theme::dialog_text())));
        lines.push(Line::from(Span::styled(
            " Press Enter or Esc to close",
            Theme::dialog_text(),
        )));

        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// Draw editable file attributes dialog
fn draw_attributes_edit_dialog(
    frame: &mut Frame,
    path: &std::path::PathBuf,
    mode: u32,
    readonly: bool,
    cursor: u8,
    area: Rect,
) {
    use ratatui::style::{Color, Style};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Edit Attributes ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let normal = Theme::dialog_text();
    let highlight = Style::default()
        .fg(Color::Rgb(255, 255, 255))
        .bg(Color::Rgb(0, 0, 170));

    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let mut lines = vec![
        Line::from(Span::styled(format!(" File: {}", name), normal)),
        Line::from(Span::styled("", normal)),
    ];

    // Readonly toggle
    let ro_marker = if readonly { "[x]" } else { "[ ]" };
    let ro_style = if cursor == 0 { highlight } else { normal };
    lines.push(Line::from(Span::styled(
        format!(" {} Read-only", ro_marker),
        ro_style,
    )));

    lines.push(Line::from(Span::styled("", normal)));
    lines.push(Line::from(Span::styled(" Unix permissions:", normal)));

    // Permission bits display: rwxrwxrwx
    let perm_labels = [
        ("Owner Read",    8),
        ("Owner Write",   7),
        ("Owner Execute", 6),
        ("Group Read",    5),
        ("Group Write",   4),
        ("Group Execute", 3),
        ("Other Read",    2),
        ("Other Write",   1),
        ("Other Execute", 0),
    ];

    for (i, (label, bit_pos)) in perm_labels.iter().enumerate() {
        let is_set = (mode >> bit_pos) & 1 == 1;
        let marker = if is_set { "[x]" } else { "[ ]" };
        let style = if cursor == (i as u8 + 1) { highlight } else { normal };
        lines.push(Line::from(Span::styled(
            format!("  {} {}", marker, label),
            style,
        )));
    }

    lines.push(Line::from(Span::styled("", normal)));
    lines.push(Line::from(Span::styled(
        format!(" Mode: {:o}  Enter=Apply  Esc=Cancel", mode),
        normal,
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Format byte size in human-readable form (long)
fn format_size_long(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB ({} bytes)", bytes as f64 / 1024.0, bytes)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB ({} bytes)", bytes as f64 / (1024.0 * 1024.0), bytes)
    } else {
        format!("{:.1} GB ({} bytes)", bytes as f64 / (1024.0 * 1024.0 * 1024.0), bytes)
    }
}

/// Draw directory history overlay
fn draw_dir_history(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);

    let dialog_w = 60u16.min(area.width.saturating_sub(4));
    let dialog_h = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Directory History ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let vis_h = inner.height as usize;
    let total = app.dir_history.len();
    let scroll = if app.dir_history_cursor >= vis_h {
        app.dir_history_cursor - vis_h + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for i in 0..vis_h {
        let idx = scroll + i;
        if idx < total {
            let path = &app.dir_history[idx];
            let is_cur = idx == app.dir_history_cursor;
            let style = if is_cur {
                Style::default().fg(black).bg(cyan)
            } else {
                Theme::dialog_text()
            };
            let text = format!(
                " {:<width$}",
                path.to_string_lossy(),
                width = inner.width.saturating_sub(1) as usize
            );
            lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_file_history(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);

    let dialog_w = 72u16.min(area.width.saturating_sub(4));
    let dialog_h = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " File History ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let vis_h = inner.height as usize;
    let total = app.file_history.len();
    let scroll = if app.file_history_cursor >= vis_h {
        app.file_history_cursor - vis_h + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for i in 0..vis_h {
        let idx = scroll + i;
        if idx < total {
            let path = &app.file_history[idx];
            let is_cur = idx == app.file_history_cursor;
            let style = if is_cur {
                Style::default().fg(black).bg(cyan)
            } else {
                Theme::dialog_text()
            };
            let text = format!(
                " {:<width$}",
                path.to_string_lossy(),
                width = inner.width.saturating_sub(1) as usize
            );
            lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Draw viewer search input overlay
fn draw_viewer_search(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let dialog_w = 50u16.min(area.width.saturating_sub(4));
    let dialog_h = 5u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Search ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(" Search for:", Theme::dialog_text()))),
        layout[0],
    );

    let input_w = inner.width.saturating_sub(2) as usize;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Theme::dialog_text()),
            Span::styled(
                format!("{:<width$}", format!("{}▌", app.viewer_search_buf), width = input_w),
                Theme::dialog_input(),
            ),
        ])),
        layout[1],
    );
}

/// Draw panel filter input overlay
fn draw_panel_filter(frame: &mut Frame, app: &mut App, area: Rect) {
    let dialog_w = 40u16;
    let dialog_h = 6u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Filter ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(" File mask (* for all):", Theme::dialog_text()))),
        layout[0],
    );

    let input_w = inner.width.saturating_sub(2) as usize;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Theme::dialog_text()),
            Span::styled(
                format!("{:<width$}", format!("{}▌", app.filter_buf), width = input_w),
                Theme::dialog_input(),
            ),
        ])),
        layout[1],
    );
}

/// Draw drive/bookmark select overlay  
fn draw_drive_select(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let cyan = Color::Rgb(0, 170, 170);
    let black = Color::Rgb(0, 0, 0);

    let dialog_w = 50u16.min(area.width.saturating_sub(4));
    let dialog_h = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Drives / Bookmarks ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let vis_h = inner.height as usize;
    let total = app.drive_list.len();
    let scroll = if app.drive_cursor >= vis_h {
        app.drive_cursor - vis_h + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for i in 0..vis_h {
        let idx = scroll + i;
        if idx < total {
            let path = &app.drive_list[idx];
            let is_cur = idx == app.drive_cursor;
            let style = if is_cur {
                Style::default().fg(black).bg(cyan)
            } else {
                Theme::dialog_text()
            };
            let text = format!(
                " {:<width$}",
                path.to_string_lossy(),
                width = inner.width.saturating_sub(1) as usize
            );
            lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Draw quick view panel (file preview in inactive panel)
fn draw_quick_view(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::style::{Color, Style};

    let bg = Color::Rgb(0, 0, 170);
    let yellow = Color::Rgb(255, 255, 85);
    let white = Color::Rgb(255, 255, 255);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(white).bg(bg))
        .title(Span::styled(
            " Quick View ",
            Style::default().fg(white).bg(bg).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Get current file from active panel
    if let Some(entry) = app.active_panel().current_entry() {
        if !entry.is_dir {
            // Try to read file preview
            if let Ok(content) = std::fs::read_to_string(&entry.path) {
                let vis_h = inner.height as usize;
                let vis_w = inner.width as usize;
                let mut lines: Vec<Line> = Vec::new();
                for line in content.lines().take(vis_h) {
                    let truncated = if line.len() > vis_w {
                        &line[..vis_w]
                    } else {
                        line
                    };
                    lines.push(Line::from(Span::styled(
                        format!("{:<width$}", truncated, width = vis_w),
                        Style::default().fg(yellow).bg(bg),
                    )));
                }
                frame.render_widget(Paragraph::new(lines), inner);
            } else {
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        " [Binary file]",
                        Style::default().fg(yellow).bg(bg),
                    ))),
                    inner,
                );
            }
        } else {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" [Directory: {}]", entry.name),
                    Style::default().fg(yellow).bg(bg),
                ))),
                inner,
            );
        }
    }
}

/// Draw user menu (placeholder)
fn draw_user_menu(frame: &mut Frame, app: &mut App, area: Rect) {
    let data = match &app.user_menu_data {
        Some(d) => d,
        None => return,
    };

    let max_label = data.items.iter()
        .filter(|i| !i.is_separator)
        .map(|i| i.label.len())
        .max()
        .unwrap_or(20);

    let dialog_w = (max_label as u16 + 6).max(30).min(area.width - 4);
    let item_count = data.items.len() as u16;
    let dialog_h = (item_count + 2).min(area.height - 2);
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " User Menu (F2) ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let mut selectable_idx = 0usize;
    let mut lines = Vec::new();
    for item in &data.items {
        if item.is_separator {
            lines.push(Line::from(Span::styled(
                "─".repeat(inner.width as usize),
                Theme::dialog_text(),
            )));
        } else {
            let is_cursor = selectable_idx == data.cursor;
            let style = if is_cursor {
                Theme::cursor_bar()
            } else {
                Theme::dialog_text()
            };
            let label = format!(" {} ", item.label);
            lines.push(Line::from(Span::styled(
                format!("{:<width$}", label, width = inner.width as usize),
                style,
            )));
            selectable_idx += 1;
        }
    }

    // Scroll if needed
    let vis_h = inner.height as usize;
    let scroll = if data.cursor >= vis_h {
        data.cursor - vis_h + 1
    } else {
        0
    };
    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).take(vis_h).collect();
    frame.render_widget(Paragraph::new(visible_lines), inner);
}

/// Draw archive browser (full screen, like file panels)
fn draw_archive_view(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let browser = match &app.archive_browser {
        Some(b) => b,
        None => return,
    };

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let title = browser.title();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(85, 85, 85)))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(Color::Rgb(85, 255, 255))
                .bg(Color::Rgb(85, 85, 85))
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(main_layout[0]);
    frame.render_widget(block, main_layout[0]);

    let entries = browser.visible_entries();
    let vis_h = inner.height as usize;

    // Column header
    let header_style = Style::default()
        .fg(Color::Rgb(255, 255, 85))
        .bg(Color::Rgb(85, 85, 85));

    if vis_h > 1 {
        let header = Line::from(vec![
            Span::styled(format!(" {:<30}", "Name"), header_style),
            Span::styled(format!("{:>12}", "Size"), header_style),
            Span::styled(format!("  {:<16}", "Date/Time"), header_style),
        ]);
        frame.render_widget(
            Paragraph::new(header),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
    }

    let list_start = 1usize;
    let list_h = vis_h.saturating_sub(1);
    let scroll = browser.scroll_offset;

    for i in 0..list_h {
        let idx = scroll + i;
        if idx >= entries.len() {
            break;
        }
        let entry = &entries[idx];
        let is_cursor = idx == browser.cursor;

        let style = if is_cursor {
            Style::default().fg(Color::Rgb(0, 0, 0)).bg(Color::Rgb(0, 170, 170))
        } else if entry.is_dir {
            Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(85, 85, 85))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(170, 170, 170)).bg(Color::Rgb(85, 85, 85))
        };

        let size_str = if entry.is_dir {
            "<DIR>".to_string()
        } else {
            format_size(entry.size)
        };

        let date_str = entry.formatted_date();
        let name = &entry.name;
        let w = inner.width as usize;
        let line_text = format!(" {:<30} {:>12}  {:<16}",
            if name.len() > 30 { &name[..30] } else { name },
            size_str,
            date_str,
        );
        let display = if line_text.len() > w {
            &line_text[..w]
        } else {
            &line_text
        };

        let y = inner.y + (list_start + i) as u16;
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{:<width$}", display, width = w),
                style,
            ))),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }

    // Status bar
    let status = format!(
        " Archive: {} | {} entries | Esc=Close  F3=View  F5=Extract  Enter=Open",
        browser.archive_path.display(),
        entries.len(),
    );
    frame.render_widget(
        Paragraph::new(status).style(Theme::status_bar()),
        main_layout[1],
    );

    // Function keys
    let fn_keys = vec![
        ("1", ""),
        ("2", ""),
        ("3", "View"),
        ("4", ""),
        ("5", "Extr"),
        ("6", ""),
        ("7", ""),
        ("8", ""),
        ("9", ""),
        ("10", "Quit"),
    ];
    let mut spans = Vec::new();
    for (num, label) in fn_keys {
        spans.push(Span::styled(num, Theme::fn_key_number()));
        spans.push(Span::styled(
            format!("{:<6}", label),
            Theme::fn_key_label(),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)),
        main_layout[2],
    );
}

/// Draw help viewer (full-screen overlay with multiple topics)
fn draw_help(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let viewer = match &mut app.help_viewer {
        Some(v) => v,
        None => return,
    };
    viewer.visible_height = area.height.saturating_sub(4) as usize;

    let dialog_w = area.width.min(60);
    let dialog_h = area.height.saturating_sub(2);
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let topic = &viewer.topics[viewer.current_topic];
    let title = format!(" {} [{}/{}] ",
        topic.title,
        viewer.current_topic + 1,
        viewer.topics.len(),
    );

    let help_bg = Color::Rgb(0, 0, 170);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default()
            .fg(Color::Rgb(255, 255, 85))
            .bg(help_bg))
        .style(Style::default().bg(help_bg))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(help_bg)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let text_style = Style::default()
        .fg(Color::Rgb(255, 255, 255))
        .bg(help_bg);
    let highlight_style = Style::default()
        .fg(Color::Rgb(255, 255, 85))
        .bg(Color::Rgb(0, 0, 170));

    let vis_h = inner.height.saturating_sub(1) as usize; // reserve 1 for status
    let lines = viewer.visible_lines();
    let mut render_lines = Vec::new();

    for line in lines.iter().take(vis_h) {
        // Highlight special characters in help text
        if line.starts_with(" ─") || line.starts_with(" ═") || line.starts_with(" ┌") || line.starts_with(" └") {
            render_lines.push(Line::from(Span::styled(
                format!("{:<width$}", line, width = inner.width as usize),
                highlight_style,
            )));
        } else {
            render_lines.push(Line::from(Span::styled(
                format!("{:<width$}", line, width = inner.width as usize),
                text_style,
            )));
        }
    }

    // Pad remaining lines
    for _ in render_lines.len()..vis_h {
        render_lines.push(Line::from(Span::styled(
            " ".repeat(inner.width as usize),
            text_style,
        )));
    }

    // Status line
    let status = " ←→Topics  ↑↓Scroll  PgUp/PgDn  Backspace=Back  Esc=Close";
    render_lines.push(Line::from(Span::styled(
        format!("{:<width$}", status, width = inner.width as usize),
        highlight_style,
    )));

    frame.render_widget(Paragraph::new(render_lines), inner);
}

/// Draw system info dialog
fn draw_system_info(frame: &mut Frame, _app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let dialog_w = 50u16;
    let dialog_h = 16u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " System Info ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Gather system info
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".to_string());
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let home = std::env::var("HOME").unwrap_or_else(|_| "?".to_string());
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "?".to_string());
    let term = std::env::var("TERM").unwrap_or_else(|_| "?".to_string());
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let pid = std::process::id();

    let info_lines = vec![
        format!(" OS:       {} {}", os, arch),
        format!(" Host:     {}", hostname),
        format!(" User:     {}", user),
        format!(" Home:     {}", home),
        format!(" Shell:    {}", shell),
        format!(" Terminal: {}", term),
        format!(" CWD:      {}", cwd),
        format!(" PID:      {}", pid),
        format!(" Screen:   {}x{}", area.width, area.height),
        String::new(),
        " Press any key to close".to_string(),
    ];

    let lines: Vec<Line> = info_lines.iter()
        .map(|l| Line::from(Span::styled(l.clone(), Theme::dialog_text())))
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Draw environment variables viewer
fn draw_env_viewer(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    let dialog_w = area.width.min(70);
    let dialog_h = area.height.saturating_sub(4);
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            format!(" Environment Variables ({}) ", app.env_vars.len()),
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let vis_h = inner.height.saturating_sub(1) as usize; // 1 for status
    let scroll = if app.env_cursor >= vis_h {
        app.env_cursor - vis_h + 1
    } else {
        0
    };

    let w = inner.width as usize;
    let mut lines = Vec::new();

    for i in 0..vis_h {
        let idx = scroll + i;
        if idx >= app.env_vars.len() {
            break;
        }
        let (name, val) = &app.env_vars[idx];
        let is_cursor = idx == app.env_cursor;
        let style = if is_cursor {
            Theme::cursor_bar()
        } else {
            Theme::dialog_text()
        };
        let text = format!(" {}={}", name, val);
        let display = if text.len() > w {
            format!("{}", &text[..w])
        } else {
            format!("{:<width$}", text, width = w)
        };
        lines.push(Line::from(Span::styled(display, style)));
    }

    // Status
    let status = format!(" {}/{} | ↑↓PgUp/PgDn Esc=Close",
        app.env_cursor + 1, app.env_vars.len());
    lines.push(Line::from(Span::styled(
        format!("{:<width$}", status, width = w),
        Style::default()
            .fg(Color::Rgb(255, 255, 85))
            .bg(Color::Rgb(0, 0, 170)),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Draw screen saver (starfield)
fn draw_screensaver(frame: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::style::{Color, Style};

    // Black background
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    app.screensaver.width = area.width;
    app.screensaver.height = area.height;

    for (x, y, ch, brightness) in app.screensaver.render_stars() {
        if x < area.width && y < area.height {
            let color = match brightness {
                2 => Color::White,
                1 => Color::Gray,
                _ => Color::DarkGray,
            };
            frame.render_widget(
                Paragraph::new(ch.to_string())
                    .style(Style::default().fg(color).bg(Color::Black)),
                Rect::new(x, y, 1, 1),
            );
        }
    }
}

/// Draw split file dialog
fn draw_split_dialog(frame: &mut Frame, app: &mut App, area: Rect) {
    let dialog_w = 45u16;
    let dialog_h = 10u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Split File ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let filename = app.active_panel().current_entry()
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let filesize = app.active_panel().current_entry()
        .map(|e| e.size)
        .unwrap_or(0);

    let chunk_size: u64 = app.split_size_buf.parse().unwrap_or(0);
    let chunks = if chunk_size > 0 { (filesize + chunk_size - 1) / chunk_size } else { 0 };

    let lines = vec![
        Line::from(Span::styled(format!(" File: {}", filename), Theme::dialog_text())),
        Line::from(Span::styled(format!(" Size: {} bytes", filesize), Theme::dialog_text())),
        Line::from(Span::styled("", Theme::dialog_text())),
        Line::from(Span::styled(
            format!(" Chunk size: {} bytes", app.split_size_buf),
            Theme::input_text(),
        )),
        Line::from(Span::styled(format!(" Will create {} chunks", chunks), Theme::dialog_text())),
        Line::from(Span::styled("", Theme::dialog_text())),
        Line::from(Span::styled(" Enter=Split  Esc=Cancel", Theme::dialog_text())),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Draw combine file dialog
fn draw_combine_dialog(frame: &mut Frame, app: &mut App, area: Rect) {
    let dialog_w = 45u16;
    let dialog_h = 9u16;
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Combine Files ",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let filename = app.active_panel().current_entry()
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let path = app.active_panel().current_entry()
        .map(|e| e.path.clone())
        .unwrap_or_default();
    let chunk_count = crate::splitfile::count_chunks(&path);
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("combined");

    let lines = vec![
        Line::from(Span::styled(format!(" First chunk: {}", filename), Theme::dialog_text())),
        Line::from(Span::styled(format!(" Chunks found: {}", chunk_count), Theme::dialog_text())),
        Line::from(Span::styled("", Theme::dialog_text())),
        Line::from(Span::styled(format!(" Output: {}", stem), Theme::dialog_text())),
        Line::from(Span::styled("", Theme::dialog_text())),
        Line::from(Span::styled(" Enter=Combine  Esc=Cancel", Theme::dialog_text())),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

// ── Theme Editor ─────────────────────────────────────────────────────

fn draw_theme_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::theme::{DOS_PALETTE, PALETTE_NAMES, SLOT_NAMES, NUM_SLOTS};
    use ratatui::style::{Color, Style};

    let dialog_w = 64u16.min(area.width.saturating_sub(2));
    let rows = NUM_SLOTS as u16 + 6;
    let dialog_h = rows.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(dialog_w)) / 2;
    let y = (area.height.saturating_sub(dialog_h)) / 2;
    let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(Block::default().style(Theme::dialog_text()), dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
        .style(Theme::dialog_text())
        .title(Span::styled(
            " Color Theme Editor  \u{2191}\u{2193}=Move  \u{2190}\u{2192}=FG  F3/F4=BG  Del=Reset  Enter=Apply",
            Theme::dialog_border().add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Header
    let header = Line::from(vec![
        Span::styled(
            format!("{:<20} {:>3} {:>3}  {}", "Slot name", "FG", "BG", "Preview"),
            Theme::column_header(),
        ),
    ]);

    let mut lines: Vec<Line> = vec![header];

    for i in 0..NUM_SLOTS as usize {
        let is_cur = i == app.theme_editor_cursor as usize;
        let row_style = if is_cur {
            Theme::cursor_active()
        } else {
            Theme::dialog_text()
        };

        let fi = app.theme_editor_fg[i] as usize % 16;
        let bi = app.theme_editor_bg[i] as usize % 16;
        let fg_col = DOS_PALETTE[fi];
        let bg_col = DOS_PALETTE[bi];

        // Name column
        let name_text = format!("{:<20}", SLOT_NAMES[i]);
        // Color indices
        let idx_text = format!("{:>3} {:>3}", fi, bi);
        // Preview swatch: fg name on bg color
        let preview = format!("  {} on {}  ", PALETTE_NAMES[fi], PALETTE_NAMES[bi]);

        let line = Line::from(vec![
            Span::styled(name_text, row_style),
            Span::styled(format!(" {}", idx_text), row_style),
            Span::styled("  ", row_style),
            Span::styled(preview, Style::default().fg(fg_col).bg(bg_col)),
        ]);
        lines.push(line);
    }

    lines.push(Line::from(Span::styled("", Theme::dialog_text())));
    lines.push(Line::from(Span::styled(
        " \u{2190}\u{2192} change FG   F3/F4 or b change BG   Del reset default",
        Theme::dialog_text(),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

// ── DBF / CSV Viewer ─────────────────────────────────────────────────

fn draw_dbf_view(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::dbf::DbfData;
    use ratatui::style::Style;

    // Fill background
    frame.render_widget(Block::default().style(Theme::panel_bg()), area);

    let data = match &app.dbf_data {
        Some(d) => d,
        None => return,
    };

    // Layout: header row + data rows + status bar
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // title + column header
            Constraint::Min(5),    // data rows
            Constraint::Length(1), // status
        ])
        .split(area);

    // ── Title ──
    let title = format!(
        " {} — {} columns, {} records ",
        data.filename,
        data.num_cols(),
        data.num_rows()
    );
    frame.render_widget(
        Paragraph::new(Span::styled(title, Theme::menu_bar())),
        layout[0],
    );

    // ── Calculate visible columns ──
    let content_w = layout[1].width as usize;
    let mut col_widths: Vec<usize> = Vec::new();
    let mut total_w = 0usize;
    for ci in data.col_scroll..data.num_cols() {
        let w = data.col_width(ci) + 1; // +1 for separator
        if total_w + w > content_w {
            break;
        }
        col_widths.push(w.saturating_sub(1));
        total_w += w;
    }

    // ── Column headers ──
    let col_scroll = data.col_scroll;
    let col_cursor = data.col_cursor;
    let mut header_spans: Vec<Span> = Vec::new();
    for (j, &cw) in col_widths.iter().enumerate() {
        let ci = col_scroll + j;
        let name = data.fields.get(ci).map(|f| f.name.as_str()).unwrap_or("");
        let is_cur_col = ci == col_cursor;
        let sty = if is_cur_col { Theme::cursor_active() } else { Theme::column_header() };
        header_spans.push(Span::styled(format!(" {:<width$}", name, width = cw.saturating_sub(1)), sty));
        header_spans.push(Span::styled("|", Theme::panel_border_active()));
    }
    // Column header rendered inside layout[1] top line
    let col_header_area = Rect::new(layout[1].x, layout[1].y, layout[1].width, 1);
    frame.render_widget(Paragraph::new(Line::from(header_spans)), col_header_area);

    // ── Data rows ──
    let vis_rows = (layout[1].height as usize).saturating_sub(1); // minus header
    let data_area = Rect::new(
        layout[1].x,
        layout[1].y + 1,
        layout[1].width,
        layout[1].height.saturating_sub(1),
    );
    let row_scroll = data.row_scroll;
    let row_cursor = data.row_cursor;

    let mut row_lines: Vec<Line> = Vec::new();
    for ri in 0..vis_rows {
        let row_idx = row_scroll + ri;
        if row_idx >= data.num_rows() {
            break;
        }
        let is_cur_row = row_idx == row_cursor;
        let mut spans: Vec<Span> = Vec::new();
        for (j, &cw) in col_widths.iter().enumerate() {
            let ci = col_scroll + j;
            let cell = data.records.get(row_idx)
                .and_then(|r| r.get(ci))
                .map(|s| s.as_str())
                .unwrap_or("");
            let is_cur = is_cur_row && ci == col_cursor;
            let sty = if is_cur {
                Theme::cursor_active()
            } else if is_cur_row {
                Theme::cursor_inactive()
            } else {
                Theme::file_normal()
            };
            spans.push(Span::styled(
                format!(" {:<width$}", &cell[..cell.len().min(cw)], width = cw.saturating_sub(1)),
                sty,
            ));
            spans.push(Span::styled("|", Theme::panel_border_inactive()));
        }
        row_lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(row_lines), data_area);

    // ── Status bar ──
    let status = format!(
        " Row {}/{}, Col {}/{}  \u{2191}\u{2193}\u{2190}\u{2192}=Navigate  Esc=Close",
        row_cursor + 1,
        data.num_rows(),
        col_cursor + 1,
        data.num_cols()
    );
    frame.render_widget(
        Paragraph::new(Span::styled(status, Theme::status_bar())),
        layout[2],
    );
}