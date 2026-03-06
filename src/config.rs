use std::path::PathBuf;
use crate::types::{SortMode, PanelMode};

/// Application configuration that can be saved/loaded
#[derive(Debug, Clone)]
pub struct Config {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub left_sort: SortMode,
    pub right_sort: SortMode,
    pub left_mode: PanelMode,
    pub right_mode: PanelMode,
    pub show_hidden: bool,
    pub editor_tab_size: usize,
    pub editor_auto_indent: bool,
    pub editor_word_wrap: bool,
    // Confirmation settings
    pub confirm_delete: bool,
    pub confirm_overwrite: bool,
    pub confirm_exit: bool,
    // Theme color overrides ("slot:fg:bg" triplets)
    pub theme_overrides: Vec<String>,
    // Quick-access bookmarks (paths as strings, empty = unset)
    pub bookmarks: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Config {
            left_path: home.clone(),
            right_path: home,
            left_sort: SortMode::Name,
            right_sort: SortMode::Name,
            left_mode: PanelMode::Full,
            right_mode: PanelMode::Full,
            show_hidden: false,
            editor_tab_size: 4,
            editor_auto_indent: true,
            editor_word_wrap: false,
            confirm_delete: true,
            confirm_overwrite: true,
            confirm_exit: false,
            theme_overrides: Vec::new(),
            bookmarks: Vec::new(),
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> PathBuf {
        if let Some(config) = dirs::config_dir() {
            config.join("rdn")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".config").join("rdn")
        } else {
            PathBuf::from(".rdn")
        }
    }

    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<String, String> {
        let dir = Self::config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Cannot create config dir: {}", e))?;
        }

        let content = format!(
            r#"# RDN - Rust Dos Navigator configuration
# Auto-generated. Edit carefully.

[panels]
left_path = "{}"
right_path = "{}"
left_sort = "{}"
right_sort = "{}"
left_mode = "{}"
right_mode = "{}"
show_hidden = {}

[editor]
tab_size = {}
auto_indent = {}
word_wrap = {}
"#,
            escape_path(&self.left_path),
            escape_path(&self.right_path),
            sort_to_str(&self.left_sort),
            sort_to_str(&self.right_sort),
            mode_to_str(&self.left_mode),
            mode_to_str(&self.right_mode),
            self.show_hidden,
            self.editor_tab_size,
            self.editor_auto_indent,
            self.editor_word_wrap,
        );

        // Append confirmation section
        let content = format!("{content}\n[confirmations]\ndelete = {}\noverwrite = {}\nexit = {}\n",
            self.confirm_delete,
            self.confirm_overwrite,
            self.confirm_exit,
        );

        // Append theme overrides
        let theme_str = self.theme_overrides.join(",");
        let content = format!("{content}\n[theme]\noverrides = \"{}\"\n", theme_str);

        // Append bookmarks
        let bm_str = self.bookmarks.join("|");
        let content = format!("{content}\n[bookmarks]\npaths = \"{}\"\n", bm_str);

        let path = Self::config_path();
        std::fs::write(&path, content)
            .map_err(|e| format!("Cannot save config: {}", e))?;
        Ok(format!("Config saved: {}", path.display()))
    }

    /// Load configuration from disk
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.starts_with('[') || line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "left_path" => config.left_path = PathBuf::from(value),
                    "right_path" => config.right_path = PathBuf::from(value),
                    "left_sort" => config.left_sort = str_to_sort(value),
                    "right_sort" => config.right_sort = str_to_sort(value),
                    "left_mode" => config.left_mode = str_to_mode(value),
                    "right_mode" => config.right_mode = str_to_mode(value),
                    "show_hidden" => config.show_hidden = value == "true",
                    "tab_size" => {
                        if let Ok(n) = value.parse() {
                            config.editor_tab_size = n;
                        }
                    }
                    "auto_indent" => config.editor_auto_indent = value == "true",
                    "word_wrap" => config.editor_word_wrap = value == "true",
                    "delete" => config.confirm_delete = value == "true",
                    "overwrite" => config.confirm_overwrite = value == "true",
                    "exit" => config.confirm_exit = value == "true",
                    "overrides" => {
                        if !value.is_empty() {
                            config.theme_overrides =
                                value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        }
                    }
                    "paths" => {
                        config.bookmarks =
                            value.split('|').map(|s| s.to_string()).collect();
                    }
                    _ => {}
                }
            }
        }

        config
    }
}

fn escape_path(path: &PathBuf) -> String {
    path.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"")
}

fn sort_to_str(sort: &SortMode) -> &str {
    match sort {
        SortMode::Name => "name",
        SortMode::Extension => "extension",
        SortMode::Size => "size",
        SortMode::Date => "date",
        SortMode::Unsorted => "unsorted",
    }
}

fn str_to_sort(s: &str) -> SortMode {
    match s {
        "name" => SortMode::Name,
        "extension" => SortMode::Extension,
        "size" => SortMode::Size,
        "date" => SortMode::Date,
        "unsorted" => SortMode::Unsorted,
        _ => SortMode::Name,
    }
}

fn mode_to_str(mode: &PanelMode) -> &str {
    match mode {
        PanelMode::Brief => "brief",
        PanelMode::Full => "full",
    }
}

fn str_to_mode(s: &str) -> PanelMode {
    match s {
        "brief" => PanelMode::Brief,
        "full" | _ => PanelMode::Full,
    }
}
