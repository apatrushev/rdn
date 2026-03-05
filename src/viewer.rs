use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Viewer display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerMode {
    Text,
    Hex,
}

/// File viewer state
pub struct Viewer {
    pub path: PathBuf,
    pub mode: ViewerMode,
    pub lines: Vec<String>,
    pub raw_data: Vec<u8>,
    pub scroll_offset: usize,
    pub horizontal_offset: usize,
    pub wrap: bool,
    pub file_size: u64,
    pub visible_height: usize,
    pub visible_width: usize,
    pub search_query: Option<String>,
    pub search_matches: Vec<(usize, usize)>, // (line, col)
    pub current_match: usize,
}

impl Viewer {
    pub fn open(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        let mut viewer = Viewer {
            path: path.to_path_buf(),
            mode: ViewerMode::Text,
            lines: Vec::new(),
            raw_data: Vec::new(),
            scroll_offset: 0,
            horizontal_offset: 0,
            wrap: false,
            file_size,
            visible_height: 24,
            visible_width: 80,
            search_query: None,
            search_matches: Vec::new(),
            current_match: 0,
        };

        viewer.load_text()?;
        Ok(viewer)
    }

    /// Load file as text
    fn load_text(&mut self) -> io::Result<()> {
        let content = fs::read(&self.path)?;
        self.raw_data = content.clone();

        // Detect if binary
        let is_binary = content.iter().take(8192).any(|&b| b == 0);
        if is_binary {
            self.mode = ViewerMode::Hex;
        }

        // Convert to string lines, handling both LF and CRLF
        let text = String::from_utf8_lossy(&content);
        self.lines = text.lines().map(|l| l.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        Ok(())
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ViewerMode::Text => ViewerMode::Hex,
            ViewerMode::Hex => ViewerMode::Text,
        };
        self.scroll_offset = 0;
        self.horizontal_offset = 0;
    }

    pub fn toggle_wrap(&mut self) {
        self.wrap = !self.wrap;
    }

    pub fn total_lines(&self) -> usize {
        match self.mode {
            ViewerMode::Text => self.lines.len(),
            ViewerMode::Hex => (self.raw_data.len() + 15) / 16,
        }
    }

    pub fn scroll_down(&mut self) {
        let max = self.total_lines().saturating_sub(self.visible_height);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn page_down(&mut self) {
        let page = self.visible_height.saturating_sub(1);
        let max = self.total_lines().saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + page).min(max);
    }

    pub fn page_up(&mut self) {
        let page = self.visible_height.saturating_sub(1);
        self.scroll_offset = self.scroll_offset.saturating_sub(page);
    }

    pub fn scroll_home(&mut self) {
        self.scroll_offset = 0;
        self.horizontal_offset = 0;
    }

    pub fn scroll_end(&mut self) {
        self.scroll_offset = self.total_lines().saturating_sub(self.visible_height);
    }

    pub fn scroll_left(&mut self) {
        if self.horizontal_offset > 0 {
            self.horizontal_offset -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_offset += 1;
    }

    /// Get hex dump lines for rendering
    pub fn hex_lines(&self, start: usize, count: usize) -> Vec<(String, String, String)> {
        let mut result = Vec::new();
        let bytes_per_line = 16;

        for i in 0..count {
            let offset = (start + i) * bytes_per_line;
            if offset >= self.raw_data.len() {
                break;
            }

            let end = (offset + bytes_per_line).min(self.raw_data.len());
            let chunk = &self.raw_data[offset..end];

            // Offset column
            let offset_str = format!("{:08X}", offset);

            // Hex column
            let mut hex_str = String::with_capacity(48);
            for (j, byte) in chunk.iter().enumerate() {
                if j == 8 {
                    hex_str.push(' ');
                }
                hex_str.push_str(&format!("{:02X} ", byte));
            }
            // Pad if less than 16 bytes
            let missing = bytes_per_line - chunk.len();
            for j in 0..missing {
                if chunk.len() + j == 8 {
                    hex_str.push(' ');
                }
                hex_str.push_str("   ");
            }

            // ASCII column
            let ascii_str: String = chunk
                .iter()
                .map(|&b| if (0x20..=0x7E).contains(&b) { b as char } else { '.' })
                .collect();

            result.push((offset_str, hex_str, ascii_str));
        }

        result
    }

    /// Search for text in the file
    pub fn search(&mut self, query: &str) {
        self.search_query = Some(query.to_string());
        self.search_matches.clear();
        self.current_match = 0;

        let query_lower = query.to_lowercase();
        for (line_idx, line) in self.lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                self.search_matches.push((line_idx, start + pos));
                start += pos + 1;
            }
        }

        if !self.search_matches.is_empty() {
            self.go_to_match(0);
        }
    }

    /// Navigate to next search match
    pub fn next_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.search_matches.len();
            self.go_to_match(self.current_match);
        }
    }

    /// Navigate to previous search match
    pub fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.search_matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
            self.go_to_match(self.current_match);
        }
    }

    fn go_to_match(&mut self, idx: usize) {
        if let Some(&(line, _col)) = self.search_matches.get(idx) {
            self.scroll_offset = line.saturating_sub(self.visible_height / 2);
        }
    }
}
