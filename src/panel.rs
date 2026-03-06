use std::fs;
use std::path::{Path, PathBuf};

use crate::types::*;

/// Represents a single file panel (left or right)
#[derive(Debug)]
pub struct Panel {
    pub path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub sort_mode: SortMode,
    pub panel_mode: PanelMode,
    pub show_hidden: bool,
    pub visible_height: usize,
    pub filter: Option<String>, // wildcard filter e.g. "*.rs"
}

impl Panel {
    pub fn new(path: PathBuf) -> Self {
        let mut panel = Panel {
            path: path.clone(),
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            sort_mode: SortMode::Name,
            panel_mode: PanelMode::Brief,
            show_hidden: false,
            visible_height: 20,
            filter: None,
        };
        panel.load_directory();
        panel
    }

    /// Read the current directory and populate entries
    pub fn load_directory(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.path.parent() {
            if parent != self.path {
                self.entries.push(FileEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    is_dir: true,
                    is_symlink: false,
                    size: 0,
                    modified: None,
                    is_readonly: false,
                    is_hidden: false,
                    is_executable: false,
                    selected: false,
                });
            }
        }

        if let Ok(read_dir) = fs::read_dir(&self.path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }

                let metadata = entry.metadata();
                let (size, modified, is_readonly, is_dir, is_symlink) = match &metadata {
                    Ok(m) => (
                        m.len(),
                        m.modified().ok(),
                        m.permissions().readonly(),
                        m.is_dir(),
                        m.is_symlink(),
                    ),
                    Err(_) => (0, None, false, false, false),
                };

                // Apply filter (only to files, dirs always shown)
                if !is_dir {
                    if let Some(ref filter) = self.filter {
                        if !Self::matches_wildcard(&name, filter) {
                            continue;
                        }
                    }
                }

                let is_executable = Self::check_executable(&path, &metadata);

                self.entries.push(FileEntry {
                    name,
                    path,
                    is_dir,
                    is_symlink,
                    size,
                    modified,
                    is_readonly,
                    is_hidden: false,
                    is_executable,
                    selected: false,
                });
            }
        }

        self.sort_entries();

        // Reset cursor if out of bounds
        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }
    }

    #[cfg(unix)]
    fn check_executable(path: &Path, metadata: &Result<std::fs::Metadata, std::io::Error>) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if path.is_dir() {
            return false;
        }
        match metadata {
            Ok(m) => m.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }

    #[cfg(not(unix))]
    fn check_executable(path: &Path, _metadata: &Result<std::fs::Metadata, std::io::Error>) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com")
    }

    /// Sort entries, keeping ".." at top, directories before files
    pub fn sort_entries(&mut self) {
        let sort_mode = self.sort_mode;
        self.entries.sort_by(|a, b| {
            // ".." always first
            if a.name == ".." {
                return std::cmp::Ordering::Less;
            }
            if b.name == ".." {
                return std::cmp::Ordering::Greater;
            }
            // Directories before files
            if a.is_dir != b.is_dir {
                return if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            // Then sort by selected mode
            match sort_mode {
                SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortMode::Extension => {
                    let ea = a.extension().to_lowercase();
                    let eb = b.extension().to_lowercase();
                    ea.cmp(&eb)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                }
                SortMode::Size => a.size.cmp(&b.size)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortMode::Date => {
                    let da = a.modified.unwrap_or(std::time::UNIX_EPOCH);
                    let db = b.modified.unwrap_or(std::time::UNIX_EPOCH);
                    da.cmp(&db)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                }
                SortMode::Unsorted => std::cmp::Ordering::Equal,
            }
        });
    }

    /// Navigate up one directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.path.parent() {
            let old_name = self.path.file_name().map(|n| n.to_string_lossy().to_string());
            self.path = parent.to_path_buf();
            self.load_directory();
            // Try to select the directory we came from
            if let Some(name) = old_name {
                if let Some(pos) = self.entries.iter().position(|e| e.name == name) {
                    self.cursor = pos;
                    self.ensure_visible();
                }
            }
        }
    }

    /// Enter the directory under cursor
    pub fn enter_dir(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                if entry.name == ".." {
                    self.go_up();
                } else {
                    self.path = entry.path.clone();
                    self.cursor = 0;
                    self.scroll_offset = 0;
                    self.load_directory();
                }
                return true;
            }
        }
        false
    }

    /// Move cursor down
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
            self.ensure_visible();
        }
    }

    /// Move cursor up
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.ensure_visible();
        }
    }

    /// Move cursor to first entry
    pub fn cursor_home(&mut self) {
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    /// Move cursor to last entry
    pub fn cursor_end(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
            self.ensure_visible();
        }
    }

    /// Page down
    pub fn page_down(&mut self) {
        let page = self.visible_height.saturating_sub(1);
        self.cursor = (self.cursor + page).min(self.entries.len().saturating_sub(1));
        self.ensure_visible();
    }

    /// Page up
    pub fn page_up(&mut self) {
        let page = self.visible_height.saturating_sub(1);
        self.cursor = self.cursor.saturating_sub(page);
        self.ensure_visible();
    }

    /// Ensure cursor is visible in the viewport
    pub fn ensure_visible(&mut self) {
        if self.visible_height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
        if self.cursor >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.cursor - self.visible_height + 1;
        }
    }

    /// Toggle selection on current entry
    pub fn toggle_select(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            if entry.name != ".." {
                entry.selected = !entry.selected;
            }
        }
    }

    /// Select all files matching a simple wildcard pattern
    pub fn select_by_pattern(&mut self, pattern: &str) {
        let pattern = pattern.to_lowercase();
        for entry in &mut self.entries {
            if entry.name != ".." && !entry.is_dir {
                if Self::matches_wildcard(&entry.name.to_lowercase(), &pattern) {
                    entry.selected = true;
                }
            }
        }
    }

    /// Deselect all files matching a simple wildcard pattern
    pub fn deselect_by_pattern(&mut self, pattern: &str) {
        let pattern = pattern.to_lowercase();
        for entry in &mut self.entries {
            if Self::matches_wildcard(&entry.name.to_lowercase(), &pattern) {
                entry.selected = false;
            }
        }
    }

    /// Invert selection
    pub fn invert_selection(&mut self) {
        for entry in &mut self.entries {
            if entry.name != ".." {
                entry.selected = !entry.selected;
            }
        }
    }

    /// Deselect all
    pub fn clear_selection(&mut self) {
        for entry in &mut self.entries {
            entry.selected = false;
        }
    }

    /// Get selected entries, or current entry if none selected
    pub fn get_selected_or_current(&self) -> Vec<&FileEntry> {
        let selected: Vec<&FileEntry> = self.entries.iter().filter(|e| e.selected).collect();
        if selected.is_empty() {
            if let Some(entry) = self.entries.get(self.cursor) {
                if entry.name != ".." {
                    return vec![entry];
                }
            }
            Vec::new()
        } else {
            selected
        }
    }

    /// Get total and selected size info for status line
    pub fn get_info(&self) -> (usize, u64, usize, u64) {
        let total_files = self.entries.iter().filter(|e| !e.is_dir && e.name != "..").count();
        let total_size: u64 = self.entries.iter().filter(|e| !e.is_dir).map(|e| e.size).sum();
        let selected_count = self.entries.iter().filter(|e| e.selected).count();
        let selected_size: u64 = self.entries.iter().filter(|e| e.selected).map(|e| e.size).sum();
        (total_files, total_size, selected_count, selected_size)
    }

    /// Current file entry
    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// Quick search - find first entry matching prefix
    pub fn quick_search(&mut self, prefix: &str) -> bool {
        let prefix_lower = prefix.to_lowercase();
        if let Some(pos) = self.entries.iter().position(|e| {
            e.name.to_lowercase().starts_with(&prefix_lower)
        }) {
            self.cursor = pos;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Simple wildcard matching (* and ?)
    fn matches_wildcard(text: &str, pattern: &str) -> bool {
        if pattern == "*" || pattern == "*.*" {
            return true;
        }
        let t_chars: Vec<char> = text.chars().collect();
        let p_chars: Vec<char> = pattern.chars().collect();
        Self::wildcard_match(&t_chars, &p_chars)
    }

    fn wildcard_match(text: &[char], pattern: &[char]) -> bool {
        if pattern.is_empty() {
            return text.is_empty();
        }
        if pattern[0] == '*' {
            // Try matching rest of pattern at every position
            for i in 0..=text.len() {
                if Self::wildcard_match(&text[i..], &pattern[1..]) {
                    return true;
                }
            }
            false
        } else if text.is_empty() {
            false
        } else if pattern[0] == '?' || pattern[0] == text[0] {
            Self::wildcard_match(&text[1..], &pattern[1..])
        } else {
            false
        }
    }
}
