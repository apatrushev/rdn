use std::fs;
use std::path::{Path, PathBuf};

/// Built-in text editor (inspired by DN's EDITOR.PAS / MICROED.PAS)
#[derive(Debug)]
pub struct Editor {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub visible_height: usize,
    pub visible_width: usize,
    pub modified: bool,
    pub selection: Option<Selection>,
    pub clipboard: Vec<String>,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
    pub search_query: String,
    pub replace_query: String,
    pub auto_indent: bool,
    pub word_wrap: bool,
    pub tab_size: usize,
    pub insert_mode: bool,
    pub status_msg: Option<String>,
    pub input_mode: EditorInputMode,
    pub input_buffer: String,
    // Macro recording
    pub macro_recording: bool,
    pub macro_buffer: Vec<crossterm::event::KeyEvent>,
    pub macro_saved: Vec<crossterm::event::KeyEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    Insert { x: usize, y: usize, text: String },
    Delete { x: usize, y: usize, text: String },
    JoinLines { y: usize, col: usize },
    SplitLine { y: usize, col: usize },
    ReplaceAll { old_lines: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorInputMode {
    Normal,
    Search,
    Replace,
    ReplaceConfirm,
    SaveAs,
    GotoLine,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            path: None,
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            visible_height: 20,
            visible_width: 80,
            modified: false,
            selection: None,
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_query: String::new(),
            replace_query: String::new(),
            auto_indent: true,
            word_wrap: false,
            tab_size: 4,
            insert_mode: true,
            status_msg: None,
            input_mode: EditorInputMode::Normal,
            input_buffer: String::new(),
            macro_recording: false,
            macro_buffer: Vec::new(),
            macro_saved: Vec::new(),
        }
    }

    pub fn open(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        Ok(Editor {
            path: Some(path.to_path_buf()),
            lines,
            ..Self::new()
        })
    }

    pub fn save(&mut self) -> Result<String, String> {
        if let Some(ref path) = self.path {
            let content = self.lines.join("\n");
            fs::write(path, &content).map_err(|e| e.to_string())?;
            self.modified = false;
            Ok(format!("Saved: {}", path.display()))
        } else {
            Err("No filename".to_string())
        }
    }

    pub fn save_as(&mut self, path: &str) -> Result<String, String> {
        self.path = Some(PathBuf::from(path));
        self.save()
    }

    pub fn title(&self) -> String {
        let name = self.path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        if self.modified {
            format!("{}*", name)
        } else {
            name
        }
    }

    // ── Cursor movement ──────────────────────────────────────────

    pub fn cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_y].len();
        if self.cursor_x < line_len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.clamp_cursor_x();
        }
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn cursor_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.clamp_cursor_x();
        }
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn cursor_home(&mut self) {
        self.cursor_x = 0;
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn cursor_end(&mut self) {
        self.cursor_x = self.lines[self.cursor_y].len();
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn page_up(&mut self) {
        let page = self.visible_height.saturating_sub(1).max(1);
        self.cursor_y = self.cursor_y.saturating_sub(page);
        self.clamp_cursor_x();
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn page_down(&mut self) {
        let page = self.visible_height.saturating_sub(1).max(1);
        self.cursor_y = (self.cursor_y + page).min(self.lines.len() - 1);
        self.clamp_cursor_x();
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn goto_top(&mut self) {
        self.cursor_y = 0;
        self.cursor_x = 0;
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn goto_bottom(&mut self) {
        self.cursor_y = self.lines.len() - 1;
        self.cursor_x = self.lines[self.cursor_y].len();
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn word_left(&mut self) {
        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.lines[self.cursor_y].len();
            }
            self.ensure_visible();
            return;
        }
        let line = &self.lines[self.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let mut x = self.cursor_x.min(chars.len());
        // Skip spaces
        while x > 0 && !chars[x - 1].is_alphanumeric() {
            x -= 1;
        }
        // Skip word
        while x > 0 && chars[x - 1].is_alphanumeric() {
            x -= 1;
        }
        self.cursor_x = x;
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn word_right(&mut self) {
        let line = &self.lines[self.cursor_y];
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if self.cursor_x >= len {
            if self.cursor_y + 1 < self.lines.len() {
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            self.ensure_visible();
            return;
        }
        let mut x = self.cursor_x;
        // Skip word
        while x < len && chars[x].is_alphanumeric() {
            x += 1;
        }
        // Skip spaces
        while x < len && !chars[x].is_alphanumeric() {
            x += 1;
        }
        self.cursor_x = x;
        self.clear_selection();
        self.ensure_visible();
    }

    pub fn goto_line(&mut self, line_num: usize) {
        let target = line_num.saturating_sub(1).min(self.lines.len() - 1);
        self.cursor_y = target;
        self.cursor_x = 0;
        self.clear_selection();
        self.ensure_visible();
    }

    // ── Text editing ─────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        if c == '\t' {
            let spaces = self.tab_size - (self.cursor_x % self.tab_size);
            let text: String = " ".repeat(spaces);
            self.push_undo(UndoAction::Insert {
                x: self.cursor_x,
                y: self.cursor_y,
                text: text.clone(),
            });
            let line = &mut self.lines[self.cursor_y];
            for _ in 0..spaces {
                if self.cursor_x <= line.len() {
                    line.insert(self.cursor_x, ' ');
                    self.cursor_x += 1;
                }
            }
        } else {
            self.push_undo(UndoAction::Insert {
                x: self.cursor_x,
                y: self.cursor_y,
                text: c.to_string(),
            });
            let line = &mut self.lines[self.cursor_y];
            if self.insert_mode || self.cursor_x >= line.len() {
                if self.cursor_x <= line.len() {
                    line.insert(self.cursor_x, c);
                } else {
                    line.push(c);
                    self.cursor_x = line.len() - 1;
                }
            } else {
                // Overwrite mode
                let mut chars: Vec<char> = line.chars().collect();
                chars[self.cursor_x] = c;
                *line = chars.into_iter().collect();
            }
            self.cursor_x += 1;
        }
        self.modified = true;
        self.redo_stack.clear();
        self.ensure_visible();
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let ch = self.lines[self.cursor_y]
                .chars()
                .nth(self.cursor_x - 1)
                .unwrap_or(' ');
            self.push_undo(UndoAction::Delete {
                x: self.cursor_x - 1,
                y: self.cursor_y,
                text: ch.to_string(),
            });
            self.lines[self.cursor_y].remove(self.cursor_x - 1);
            self.cursor_x -= 1;
            self.modified = true;
            self.redo_stack.clear();
        } else if self.cursor_y > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            let join_col = self.lines[self.cursor_y].len();
            self.push_undo(UndoAction::JoinLines {
                y: self.cursor_y,
                col: join_col,
            });
            self.lines[self.cursor_y].push_str(&current_line);
            self.cursor_x = join_col;
            self.modified = true;
            self.redo_stack.clear();
        }
        self.ensure_visible();
    }

    pub fn delete_char(&mut self) {
        let line_len = self.lines[self.cursor_y].len();
        if self.cursor_x < line_len {
            let ch = self.lines[self.cursor_y]
                .chars()
                .nth(self.cursor_x)
                .unwrap_or(' ');
            self.push_undo(UndoAction::Delete {
                x: self.cursor_x,
                y: self.cursor_y,
                text: ch.to_string(),
            });
            self.lines[self.cursor_y].remove(self.cursor_x);
            self.modified = true;
            self.redo_stack.clear();
        } else if self.cursor_y + 1 < self.lines.len() {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_y + 1);
            self.push_undo(UndoAction::JoinLines {
                y: self.cursor_y,
                col: self.lines[self.cursor_y].len(),
            });
            self.lines[self.cursor_y].push_str(&next_line);
            self.modified = true;
            self.redo_stack.clear();
        }
    }

    pub fn enter_key(&mut self) {
        let indent = if self.auto_indent {
            let line = &self.lines[self.cursor_y];
            let spaces: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            spaces
        } else {
            String::new()
        };

        self.push_undo(UndoAction::SplitLine {
            y: self.cursor_y,
            col: self.cursor_x,
        });

        let rest = self.lines[self.cursor_y][self.cursor_x..].to_string();
        self.lines[self.cursor_y].truncate(self.cursor_x);
        self.cursor_y += 1;
        let new_line = format!("{}{}", indent, rest);
        self.cursor_x = indent.len();
        self.lines.insert(self.cursor_y, new_line);
        self.modified = true;
        self.redo_stack.clear();
        self.ensure_visible();
    }

    pub fn delete_line(&mut self) {
        if self.lines.len() > 1 {
            let removed = self.lines.remove(self.cursor_y);
            self.push_undo(UndoAction::Delete {
                x: 0,
                y: self.cursor_y,
                text: removed + "\n",
            });
            if self.cursor_y >= self.lines.len() {
                self.cursor_y = self.lines.len() - 1;
            }
            self.clamp_cursor_x();
            self.modified = true;
            self.redo_stack.clear();
        }
    }

    // ── Selection ────────────────────────────────────────────────

    pub fn start_selection(&mut self) {
        self.selection = Some(Selection {
            start_x: self.cursor_x,
            start_y: self.cursor_y,
            end_x: self.cursor_x,
            end_y: self.cursor_y,
        });
    }

    pub fn update_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.end_x = self.cursor_x;
            sel.end_y = self.cursor_y;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn select_all(&mut self) {
        self.selection = Some(Selection {
            start_x: 0,
            start_y: 0,
            end_x: self.lines.last().map(|l| l.len()).unwrap_or(0),
            end_y: self.lines.len() - 1,
        });
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    fn normalized_selection(&self) -> Option<(usize, usize, usize, usize)> {
        self.selection.map(|s| {
            if s.start_y < s.end_y || (s.start_y == s.end_y && s.start_x <= s.end_x) {
                (s.start_x, s.start_y, s.end_x, s.end_y)
            } else {
                (s.end_x, s.end_y, s.start_x, s.start_y)
            }
        })
    }

    pub fn is_in_selection(&self, x: usize, y: usize) -> bool {
        if let Some((sx, sy, ex, ey)) = self.normalized_selection() {
            if y < sy || y > ey {
                return false;
            }
            if y == sy && y == ey {
                return x >= sx && x < ex;
            }
            if y == sy {
                return x >= sx;
            }
            if y == ey {
                return x < ex;
            }
            true
        } else {
            false
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some((sx, sy, ex, ey)) = self.normalized_selection() {
            let mut result = String::new();
            for y in sy..=ey {
                if y >= self.lines.len() {
                    break;
                }
                let line = &self.lines[y];
                if y == sy && y == ey {
                    let start = sx.min(line.len());
                    let end = ex.min(line.len());
                    result.push_str(&line[start..end]);
                } else if y == sy {
                    let start = sx.min(line.len());
                    result.push_str(&line[start..]);
                    result.push('\n');
                } else if y == ey {
                    let end = ex.min(line.len());
                    result.push_str(&line[..end]);
                } else {
                    result.push_str(line);
                    result.push('\n');
                }
            }
            result
        } else {
            String::new()
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some((sx, sy, ex, ey)) = self.normalized_selection() {
            // Save for undo
            let old_lines = self.lines.clone();
            self.push_undo(UndoAction::ReplaceAll { old_lines });

            if sy == ey {
                let start = sx.min(self.lines[sy].len());
                let end = ex.min(self.lines[sy].len());
                self.lines[sy] = format!(
                    "{}{}",
                    &self.lines[sy][..start],
                    &self.lines[sy][end..]
                );
            } else {
                let first_part = self.lines[sy][..sx.min(self.lines[sy].len())].to_string();
                let last_part = if ey < self.lines.len() {
                    self.lines[ey][ex.min(self.lines[ey].len())..].to_string()
                } else {
                    String::new()
                };
                // Remove lines sy+1..=ey
                let end = (ey + 1).min(self.lines.len());
                self.lines.drain((sy + 1)..end);
                self.lines[sy] = format!("{}{}", first_part, last_part);
            }

            self.cursor_x = sx;
            self.cursor_y = sy;
            self.selection = None;
            self.modified = true;
            self.redo_stack.clear();
            self.ensure_visible();
        }
    }

    // ── Clipboard ────────────────────────────────────────────────

    pub fn copy_selection(&mut self) {
        let text = self.get_selected_text();
        if !text.is_empty() {
            self.clipboard = text.lines().map(|l| l.to_string()).collect();
            // Sync to OS clipboard
            if let Ok(mut ctx) = arboard::Clipboard::new() {
                let _ = ctx.set_text(&text);
            }
            self.status_msg = Some("Block copied".to_string());
        }
    }

    pub fn cut_selection(&mut self) {
        self.copy_selection();
        self.delete_selected();
    }

    pub fn paste(&mut self) {
        // Try to get from OS clipboard first
        if let Ok(mut ctx) = arboard::Clipboard::new() {
            if let Ok(text) = ctx.get_text() {
                if !text.is_empty() {
                    self.clipboard = text.lines().map(|l| l.to_string()).collect();
                }
            }
        }

        if self.clipboard.is_empty() {
            return;
        }
        if self.has_selection() {
            self.delete_selected();
        }

        let old_lines = self.lines.clone();
        self.push_undo(UndoAction::ReplaceAll { old_lines });

        let text = self.clipboard.join("\n");
        let cx = self.cursor_x.min(self.lines[self.cursor_y].len());
        let before = self.lines[self.cursor_y][..cx].to_string();
        let after = self.lines[self.cursor_y][cx..].to_string();

        let combined = format!("{}{}{}", before, text, after);
        let new_lines: Vec<String> = combined.lines().map(|l| l.to_string()).collect();
        let num_new = new_lines.len();
        let after_len = after.len();

        self.lines.splice(self.cursor_y..=self.cursor_y, new_lines);

        // Move cursor to end of pasted text
        if num_new == 1 {
            self.cursor_x += text.len();
        } else {
            self.cursor_y += num_new - 1;
            self.cursor_x = self.lines[self.cursor_y].len() - after_len;
        }

        self.modified = true;
        self.redo_stack.clear();
        self.ensure_visible();
    }

    // ── Macros ───────────────────────────────────────────────────

    pub fn toggle_macro_recording(&mut self) {
        if self.macro_recording {
            // Stop recording
            self.macro_recording = false;
            self.macro_saved = self.macro_buffer.clone();
            self.macro_buffer.clear();
            let count = self.macro_saved.len();
            self.status_msg = Some(format!("Macro recorded ({} keys)", count));
        } else {
            // Start recording
            self.macro_recording = true;
            self.macro_buffer.clear();
            self.status_msg = Some("Recording macro...".to_string());
        }
    }

    pub fn record_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.macro_recording {
            // Don't record the macro toggle key itself
            if !(key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                 && key.code == crossterm::event::KeyCode::Char('m'))
            {
                self.macro_buffer.push(key);
            }
        }
    }

    pub fn get_macro_keys(&self) -> Vec<crossterm::event::KeyEvent> {
        self.macro_saved.clone()
    }

    // ── Search & Replace ─────────────────────────────────────────

    pub fn find_next(&mut self) -> bool {
        if self.search_query.is_empty() {
            return false;
        }
        let query = self.search_query.to_lowercase();
        // Search from current position forward
        let start_y = self.cursor_y;
        let start_x = self.cursor_x + 1;

        for y in start_y..self.lines.len() {
            let line_lower = self.lines[y].to_lowercase();
            let from = if y == start_y { start_x } else { 0 };
            if let Some(pos) = line_lower[from..].find(&query) {
                self.cursor_y = y;
                self.cursor_x = from + pos;
                self.selection = Some(Selection {
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x + self.search_query.len(),
                    end_y: self.cursor_y,
                });
                self.ensure_visible();
                return true;
            }
        }
        // Wrap around
        for y in 0..=start_y.min(self.lines.len() - 1) {
            let line_lower = self.lines[y].to_lowercase();
            let to = if y == start_y { start_x } else { line_lower.len() };
            if let Some(pos) = line_lower[..to].find(&query) {
                self.cursor_y = y;
                self.cursor_x = pos;
                self.selection = Some(Selection {
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x + self.search_query.len(),
                    end_y: self.cursor_y,
                });
                self.ensure_visible();
                return true;
            }
        }
        self.status_msg = Some("Not found".to_string());
        false
    }

    pub fn find_prev(&mut self) -> bool {
        if self.search_query.is_empty() {
            return false;
        }
        let query = self.search_query.to_lowercase();
        let start_y = self.cursor_y;
        let start_x = self.cursor_x;

        for y in (0..=start_y).rev() {
            let line_lower = self.lines[y].to_lowercase();
            let to = if y == start_y { start_x } else { line_lower.len() };
            if let Some(pos) = line_lower[..to].rfind(&query) {
                self.cursor_y = y;
                self.cursor_x = pos;
                self.selection = Some(Selection {
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x + self.search_query.len(),
                    end_y: self.cursor_y,
                });
                self.ensure_visible();
                return true;
            }
        }
        self.status_msg = Some("Not found".to_string());
        false
    }

    pub fn replace_current(&mut self) {
        if let Some((sx, sy, ex, _ey)) = self.normalized_selection() {
            let line = &mut self.lines[sy];
            let start = sx.min(line.len());
            let end = ex.min(line.len());
            *line = format!("{}{}{}", &line[..start], self.replace_query, &line[end..]);
            self.cursor_x = start + self.replace_query.len();
            self.modified = true;
            self.selection = None;
        }
    }

    pub fn replace_all(&mut self) -> usize {
        if self.search_query.is_empty() {
            return 0;
        }
        let old_lines = self.lines.clone();
        self.push_undo(UndoAction::ReplaceAll { old_lines });

        let mut count = 0;
        let query = &self.search_query;
        let replacement = &self.replace_query.clone();
        for line in &mut self.lines {
            let occurrences = line.matches(query.as_str()).count();
            if occurrences > 0 {
                count += occurrences;
                *line = line.replace(query.as_str(), replacement);
            }
        }
        if count > 0 {
            self.modified = true;
            self.redo_stack.clear();
        }
        self.status_msg = Some(format!("Replaced {} occurrences", count));
        count
    }

    // ── Undo / Redo ──────────────────────────────────────────────

    fn push_undo(&mut self, action: UndoAction) {
        if self.undo_stack.len() > 1000 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(action);
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Insert { x, y, ref text } => {
                    let len = text.len();
                    self.lines[y] = format!(
                        "{}{}",
                        &self.lines[y][..x],
                        &self.lines[y][x + len..]
                    );
                    self.cursor_x = x;
                    self.cursor_y = y;
                    self.redo_stack.push(action);
                }
                UndoAction::Delete { x, y, ref text } => {
                    self.lines[y].insert_str(x, text.trim_end_matches('\n'));
                    if text.ends_with('\n') {
                        // Was a full line delete — re-insert
                        self.lines.insert(y, text.trim_end_matches('\n').to_string());
                    }
                    self.cursor_x = x;
                    self.cursor_y = y;
                    self.redo_stack.push(action);
                }
                UndoAction::JoinLines { y, col } => {
                    let rest = self.lines[y][col..].to_string();
                    self.lines[y].truncate(col);
                    self.lines.insert(y + 1, rest);
                    self.cursor_x = 0;
                    self.cursor_y = y + 1;
                    self.redo_stack.push(action);
                }
                UndoAction::SplitLine { y, col } => {
                    if y + 1 < self.lines.len() {
                        let next = self.lines.remove(y + 1);
                        self.lines[y].push_str(next.trim_start());
                    }
                    self.cursor_x = col;
                    self.cursor_y = y;
                    self.redo_stack.push(action);
                }
                UndoAction::ReplaceAll { ref old_lines } => {
                    let current = self.lines.clone();
                    self.lines = old_lines.clone();
                    self.cursor_x = 0;
                    self.cursor_y = 0;
                    self.redo_stack.push(UndoAction::ReplaceAll {
                        old_lines: current,
                    });
                }
            }
            self.modified = true;
            self.ensure_visible();
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            match action {
                UndoAction::Insert { x, y, ref text } => {
                    self.lines[y].insert_str(x, text);
                    self.cursor_x = x + text.len();
                    self.cursor_y = y;
                    self.undo_stack.push(action);
                }
                UndoAction::Delete { x, y, ref text } => {
                    if text.ends_with('\n') && y < self.lines.len() {
                        self.lines.remove(y);
                    } else {
                        let end = (x + text.len()).min(self.lines[y].len());
                        self.lines[y] = format!(
                            "{}{}",
                            &self.lines[y][..x],
                            &self.lines[y][end..]
                        );
                    }
                    self.cursor_x = x;
                    self.cursor_y = y;
                    self.undo_stack.push(action);
                }
                UndoAction::JoinLines { y, col } => {
                    if y + 1 < self.lines.len() {
                        let next = self.lines.remove(y + 1);
                        self.lines[y].push_str(&next);
                    }
                    self.cursor_x = col;
                    self.cursor_y = y;
                    self.undo_stack.push(action);
                }
                UndoAction::SplitLine { y, col } => {
                    let rest = self.lines[y][col..].to_string();
                    self.lines[y].truncate(col);
                    self.lines.insert(y + 1, rest);
                    self.cursor_y = y + 1;
                    self.cursor_x = 0;
                    self.undo_stack.push(action);
                }
                UndoAction::ReplaceAll { ref old_lines } => {
                    let current = self.lines.clone();
                    self.lines = old_lines.clone();
                    self.undo_stack.push(UndoAction::ReplaceAll {
                        old_lines: current,
                    });
                }
            }
            self.modified = true;
            self.ensure_visible();
        }
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn clamp_cursor_x(&mut self) {
        let line_len = self.lines[self.cursor_y].len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    fn ensure_visible(&mut self) {
        if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        }
        if self.cursor_y >= self.scroll_y + self.visible_height {
            self.scroll_y = self.cursor_y - self.visible_height + 1;
        }
        if self.cursor_x < self.scroll_x {
            self.scroll_x = self.cursor_x;
        }
        if self.cursor_x >= self.scroll_x + self.visible_width {
            self.scroll_x = self.cursor_x - self.visible_width + 1;
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}
