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
        AppMode::Tetris => {
            // Draw file manager in background, then overlay tetris dialog (like DN)
            draw_file_manager(frame, app, size);
            draw_tetris(frame, app, size);
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
    );

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
    );

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
    let menus = vec![" Left ", " Files ", " Commands ", " Options ", " Right "];
    let mut spans = Vec::new();

    for (i, menu) in menus.iter().enumerate() {
        let style = if app.show_menu && app.menu_index == i {
            Theme::menu_bar_highlight()
        } else {
            Theme::menu_bar()
        };
        spans.push(Span::styled(menu.to_string(), style));
    }

    // Pad the rest
    let used: usize = menus.iter().map(|m| m.len()).sum();
    if (used as u16) < area.width {
        spans.push(Span::styled(
            " ".repeat((area.width as usize).saturating_sub(used)),
            Theme::menu_bar(),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Draw a single file panel (DN-style: double-line frame when active, single when inactive)
fn draw_panel(
    frame: &mut Frame,
    panel: &mut Panel,
    area: Rect,
    is_active: bool,
    show_search: bool,
    search_text: Option<&str>,
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
    let max_title = (area.width as usize).saturating_sub(4);
    let title = if path_str.len() > max_title {
        format!("…{}", &path_str[path_str.len() - max_title + 1..])
    } else {
        path_str.to_string()
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
    draw_info_area(frame, panel, panel_layout[2], inner_width, is_active, border_set);

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

    // Line 1: current file info (filename, size, date/time) - like DN
    {
        let info = if let Some(entry) = panel.current_entry() {
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
    let dialog_width = 50u16.min(area.width - 4);
    let dialog_height = match &kind {
        DialogKind::Confirm { .. } => 7,
        DialogKind::Input { .. } => 8,
        DialogKind::Error(_) => 6,
        DialogKind::FileInfo => 12,
    };

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    match kind {
        DialogKind::Confirm { title, message, .. } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::dialog_border())
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
                Paragraph::new(Line::from(Span::styled(&*message, Theme::dialog_text()))),
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
                    Constraint::Length(1),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(&*prompt, Theme::dialog_text()))),
                content_layout[1],
            );

            let input_width = inner.width.saturating_sub(2) as usize;
            let display_value = if value.len() > input_width {
                &value[value.len() - input_width..]
            } else {
                &value
            };
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" ", Theme::dialog_text()),
                    Span::styled(
                        format!("{:<width$}", format!("{}▌", display_value), width = input_width),
                        Theme::dialog_input(),
                    ),
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
    }
}

/// Draw file info dialog
fn draw_file_info_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::dialog_border())
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
    match ((color_idx - 1) % 7) {
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
