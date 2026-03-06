use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader};

/// File find dialog state (inspired by DN's FILEFIND.PAS)
#[derive(Debug)]
pub struct FileFinder {
    pub pattern: String,
    pub content_query: String,
    pub search_root: PathBuf,
    pub results: Vec<FindResult>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub searching: bool,
    pub search_complete: bool,
    pub visible_height: usize,
    pub case_sensitive: bool,
    pub search_subdirs: bool,
}

#[derive(Debug, Clone)]
pub struct FindResult {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub match_line: Option<(usize, String)>, // line number + content
}

impl FileFinder {
    pub fn new(root: &Path) -> Self {
        FileFinder {
            pattern: "*".to_string(),
            content_query: String::new(),
            search_root: root.to_path_buf(),
            results: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            searching: false,
            search_complete: false,
            visible_height: 15,
            case_sensitive: false,
            search_subdirs: true,
        }
    }

    /// Execute the search synchronously (blocking)
    pub fn execute(&mut self) {
        self.results.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.searching = true;
        self.search_complete = false;

        self.search_dir(&self.search_root.clone());

        self.searching = false;
        self.search_complete = true;
    }

    fn search_dir(&mut self, dir: &Path) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata();

            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

            // Check name pattern match
            if self.matches_pattern(&name) {
                if !self.content_query.is_empty() && !is_dir {
                    // Search file content
                    if let Some(match_info) = self.search_file_content(&path) {
                        self.results.push(FindResult {
                            path: path.clone(),
                            name: name.clone(),
                            size,
                            is_dir,
                            match_line: Some(match_info),
                        });
                    }
                } else {
                    self.results.push(FindResult {
                        path: path.clone(),
                        name,
                        size,
                        is_dir,
                        match_line: None,
                    });
                }
            }

            // Recurse into subdirectories
            if is_dir && self.search_subdirs {
                self.search_dir(&path);
            }
        }
    }

    fn search_file_content(&self, path: &Path) -> Option<(usize, String)> {
        let file = fs::File::open(path).ok()?;
        let reader = BufReader::new(file);
        let query = if self.case_sensitive {
            self.content_query.clone()
        } else {
            self.content_query.to_lowercase()
        };

        for (line_num, line_result) in reader.lines().enumerate() {
            if let Ok(line) = line_result {
                let hay = if self.case_sensitive {
                    line.clone()
                } else {
                    line.to_lowercase()
                };
                if hay.contains(&query) {
                    let preview = if line.len() > 80 {
                        format!("{}...", &line[..77])
                    } else {
                        line
                    };
                    return Some((line_num + 1, preview));
                }
            }
            // Don't read beyond 10MB of text
            if line_num > 100_000 {
                break;
            }
        }
        None
    }

    fn matches_pattern(&self, name: &str) -> bool {
        let pattern = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };
        let name_check = if self.case_sensitive {
            name.to_string()
        } else {
            name.to_lowercase()
        };
        wildcard_match(&name_check, &pattern)
    }

    // Navigation
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.ensure_visible();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.results.len() {
            self.cursor += 1;
            self.ensure_visible();
        }
    }

    pub fn page_up(&mut self) {
        let page = self.visible_height.saturating_sub(1).max(1);
        self.cursor = self.cursor.saturating_sub(page);
        self.ensure_visible();
    }

    pub fn page_down(&mut self) {
        let page = self.visible_height.saturating_sub(1).max(1);
        self.cursor = (self.cursor + page).min(self.results.len().saturating_sub(1));
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
        if self.cursor >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.cursor - self.visible_height + 1;
        }
    }

    pub fn current_result(&self) -> Option<&FindResult> {
        self.results.get(self.cursor)
    }
}

/// Simple wildcard matching (* and ?)
fn wildcard_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    wm_recursive(&t, &p)
}

fn wm_recursive(text: &[char], pattern: &[char]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if pattern[0] == '*' {
        for i in 0..=text.len() {
            if wm_recursive(&text[i..], &pattern[1..]) {
                return true;
            }
        }
        false
    } else if text.is_empty() {
        false
    } else if pattern[0] == '?' || pattern[0] == text[0] {
        wm_recursive(&text[1..], &pattern[1..])
    } else {
        false
    }
}
