use std::path::PathBuf;

use crate::file_ops;
use crate::panel::Panel;
use crate::tetris::Tetris;
use crate::types::*;
use crate::viewer::Viewer;

/// Main application state
pub struct App {
    pub left_panel: Panel,
    pub right_panel: Panel,
    pub active: ActivePanel,
    pub mode: AppMode,
    pub command_line: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub viewer: Option<Viewer>,
    pub tetris: Option<Tetris>,
    pub show_menu: bool,
    pub menu_index: usize,
}

impl App {
    pub fn new() -> Self {
        let home = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        App {
            left_panel: Panel::new(home.clone()),
            right_panel: Panel::new(home),
            active: ActivePanel::Left,
            mode: AppMode::Normal,
            command_line: String::new(),
            status_message: None,
            should_quit: false,
            viewer: None,
            tetris: None,
            show_menu: false,
            menu_index: 0,
        }
    }

    /// Get mutable reference to the active panel
    pub fn active_panel_mut(&mut self) -> &mut Panel {
        match self.active {
            ActivePanel::Left => &mut self.left_panel,
            ActivePanel::Right => &mut self.right_panel,
        }
    }

    /// Get reference to the active panel
    pub fn active_panel(&self) -> &Panel {
        match self.active {
            ActivePanel::Left => &self.left_panel,
            ActivePanel::Right => &self.right_panel,
        }
    }

    /// Get reference to the inactive panel
    pub fn inactive_panel(&self) -> &Panel {
        match self.active {
            ActivePanel::Left => &self.right_panel,
            ActivePanel::Right => &self.left_panel,
        }
    }

    /// Get mutable reference to the inactive panel
    pub fn inactive_panel_mut(&mut self) -> &mut Panel {
        match self.active {
            ActivePanel::Left => &mut self.right_panel,
            ActivePanel::Right => &mut self.left_panel,
        }
    }

    /// Switch active panel
    pub fn switch_panel(&mut self) {
        self.active = self.active.other();
    }

    /// Enter directory or open file
    pub fn enter(&mut self) {
        let panel = self.active_panel();
        if let Some(entry) = panel.current_entry() {
            if entry.is_dir {
                self.active_panel_mut().enter_dir();
            } else {
                // Open file in viewer
                let path = entry.path.clone();
                self.open_viewer(&path);
            }
        }
    }

    /// Open file viewer
    pub fn open_viewer(&mut self, path: &PathBuf) {
        match Viewer::open(path) {
            Ok(viewer) => {
                self.viewer = Some(viewer);
                self.mode = AppMode::Viewer(path.clone());
            }
            Err(e) => {
                self.status_message = Some(format!("Cannot open file: {}", e));
            }
        }
    }

    /// Close viewer
    pub fn close_viewer(&mut self) {
        self.viewer = None;
        self.mode = AppMode::Normal;
    }

    /// Start copy operation
    pub fn start_copy(&mut self) {
        let files = self.active_panel().get_selected_or_current();
        if files.is_empty() {
            self.status_message = Some("No files selected".to_string());
            return;
        }

        let dest = self.inactive_panel().path.to_string_lossy().to_string();
        let count = files.len();
        let title = if count == 1 {
            format!("Copy \"{}\"", files[0].name)
        } else {
            format!("Copy {} files", count)
        };

        self.mode = AppMode::Dialog(DialogKind::Input {
            title,
            prompt: "to:".to_string(),
            value: dest,
            op: FileOp::Copy,
        });
    }

    /// Start move operation
    pub fn start_move(&mut self) {
        let files = self.active_panel().get_selected_or_current();
        if files.is_empty() {
            self.status_message = Some("No files selected".to_string());
            return;
        }

        let dest = self.inactive_panel().path.to_string_lossy().to_string();
        let count = files.len();
        let title = if count == 1 {
            format!("Move \"{}\"", files[0].name)
        } else {
            format!("Move {} files", count)
        };

        self.mode = AppMode::Dialog(DialogKind::Input {
            title,
            prompt: "to:".to_string(),
            value: dest,
            op: FileOp::Move,
        });
    }

    /// Start delete operation
    pub fn start_delete(&mut self) {
        let files = self.active_panel().get_selected_or_current();
        if files.is_empty() {
            self.status_message = Some("No files selected".to_string());
            return;
        }

        let count = files.len();
        let message = if count == 1 {
            format!("Delete \"{}\"?", files[0].name)
        } else {
            format!("Delete {} files?", count)
        };

        self.mode = AppMode::Dialog(DialogKind::Confirm {
            title: "Delete".to_string(),
            message,
            op: FileOp::Delete,
        });
    }

    /// Start mkdir operation
    pub fn start_mkdir(&mut self) {
        self.mode = AppMode::Dialog(DialogKind::Input {
            title: "Make Directory".to_string(),
            prompt: "Name:".to_string(),
            value: String::new(),
            op: FileOp::MkDir,
        });
    }

    /// Execute file operation
    pub fn execute_op(&mut self, op: FileOp, value: Option<String>) {
        match op {
            FileOp::Copy => {
                if let Some(dest) = value {
                    let dest_path = PathBuf::from(&dest);
                    let files: Vec<PathBuf> = self
                        .active_panel()
                        .get_selected_or_current()
                        .iter()
                        .map(|f| f.path.clone())
                        .collect();

                    let mut errors = Vec::new();
                    for src in &files {
                        if let Err(e) = file_ops::copy_entry(src, &dest_path) {
                            errors.push(e);
                        }
                    }

                    if errors.is_empty() {
                        self.status_message =
                            Some(format!("Copied {} file(s)", files.len()));
                    } else {
                        self.status_message = Some(errors.join("; "));
                    }

                    self.active_panel_mut().clear_selection();
                    self.refresh_panels();
                }
            }
            FileOp::Move => {
                if let Some(dest) = value {
                    let dest_path = PathBuf::from(&dest);
                    let files: Vec<PathBuf> = self
                        .active_panel()
                        .get_selected_or_current()
                        .iter()
                        .map(|f| f.path.clone())
                        .collect();

                    let mut errors = Vec::new();
                    for src in &files {
                        if let Err(e) = file_ops::move_entry(src, &dest_path) {
                            errors.push(e);
                        }
                    }

                    if errors.is_empty() {
                        self.status_message =
                            Some(format!("Moved {} file(s)", files.len()));
                    } else {
                        self.status_message = Some(errors.join("; "));
                    }

                    self.active_panel_mut().clear_selection();
                    self.refresh_panels();
                }
            }
            FileOp::Delete => {
                let files: Vec<PathBuf> = self
                    .active_panel()
                    .get_selected_or_current()
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();

                let mut errors = Vec::new();
                for path in &files {
                    if let Err(e) = file_ops::delete_entry(path, true) {
                        errors.push(e);
                    }
                }

                if errors.is_empty() {
                    self.status_message =
                        Some(format!("Deleted {} file(s)", files.len()));
                } else {
                    self.status_message = Some(errors.join("; "));
                }

                self.active_panel_mut().clear_selection();
                self.refresh_panels();
            }
            FileOp::MkDir => {
                if let Some(name) = value {
                    let parent = self.active_panel().path.clone();
                    match file_ops::make_dir(&parent, &name) {
                        Ok(msg) => self.status_message = Some(msg),
                        Err(e) => self.status_message = Some(e),
                    }
                    self.refresh_panels();
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    /// Refresh both panels
    pub fn refresh_panels(&mut self) {
        self.left_panel.load_directory();
        self.right_panel.load_directory();
    }

    /// Toggle sort mode on active panel
    pub fn toggle_sort(&mut self) {
        let panel = self.active_panel_mut();
        panel.sort_mode = panel.sort_mode.next();
        panel.sort_entries();
    }

    /// Make both panels show the same directory
    pub fn sync_panels(&mut self) {
        let path = self.active_panel().path.clone();
        let inactive = self.inactive_panel_mut();
        inactive.path = path;
        inactive.cursor = 0;
        inactive.scroll_offset = 0;
        inactive.load_directory();
    }

    /// Swap panels
    pub fn swap_panels(&mut self) {
        std::mem::swap(&mut self.left_panel, &mut self.right_panel);
    }

    /// Toggle hidden files visibility
    pub fn toggle_hidden(&mut self) {
        let panel = self.active_panel_mut();
        panel.show_hidden = !panel.show_hidden;
        panel.load_directory();
    }

    /// Start quick search
    pub fn start_quick_search(&mut self) {
        self.mode = AppMode::QuickSearch(String::new());
    }

    /// Open file with system default application
    pub fn open_file_external(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if !entry.is_dir {
                let _ = open::that(&entry.path);
            }
        }
    }

    /// Show file info dialog
    pub fn show_file_info(&mut self) {
        self.mode = AppMode::Dialog(DialogKind::FileInfo);
    }

    /// Start Tetris game
    pub fn start_tetris(&mut self) {
        self.tetris = Some(Tetris::new());
        self.mode = AppMode::Tetris;
    }

    /// Close Tetris game
    pub fn close_tetris(&mut self) {
        self.tetris = None;
        self.mode = AppMode::Normal;
    }
}
