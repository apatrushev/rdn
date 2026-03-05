use std::path::PathBuf;
use std::time::SystemTime;

/// Sorting mode for file panels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Extension,
    Size,
    Date,
    Unsorted,
}

impl SortMode {
    pub fn label(&self) -> &str {
        match self {
            SortMode::Name => "Name",
            SortMode::Extension => "Ext",
            SortMode::Size => "Size",
            SortMode::Date => "Date",
            SortMode::Unsorted => "Unsorted",
        }
    }

    pub fn next(&self) -> SortMode {
        match self {
            SortMode::Name => SortMode::Extension,
            SortMode::Extension => SortMode::Size,
            SortMode::Size => SortMode::Date,
            SortMode::Date => SortMode::Unsorted,
            SortMode::Unsorted => SortMode::Name,
        }
    }
}

/// Display mode for file panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMode {
    Brief,
    Full,
}

/// Represents one entry in a file panel
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub is_executable: bool,
    pub selected: bool,
}

impl FileEntry {
    pub fn extension(&self) -> &str {
        self.path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }

    pub fn formatted_size(&self) -> String {
        if self.is_dir {
            "<DIR>".to_string()
        } else if self.size < 1024 {
            format!("{}", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{}K", self.size / 1024)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1}M", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    pub fn formatted_date(&self) -> String {
        match self.modified {
            Some(time) => {
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                datetime.format("%d.%m.%y %H:%M").to_string()
            }
            None => String::new(),
        }
    }
}

/// Which panel is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Left,
    Right,
}

impl ActivePanel {
    pub fn other(&self) -> ActivePanel {
        match self {
            ActivePanel::Left => ActivePanel::Right,
            ActivePanel::Right => ActivePanel::Left,
        }
    }
}

/// File operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOp {
    Copy,
    Move,
    Delete,
    MkDir,
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    CommandLine,
    QuickSearch(String),
    Dialog(DialogKind),
    Viewer(PathBuf),
    Tetris,
    Help,
}

/// Dialog types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    Confirm {
        title: String,
        message: String,
        op: FileOp,
    },
    Input {
        title: String,
        prompt: String,
        value: String,
        op: FileOp,
    },
    Error(String),
    FileInfo,
}
