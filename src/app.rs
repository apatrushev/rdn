use std::path::PathBuf;
use std::collections::HashMap;

use crate::archive::ArchiveBrowser;
use crate::calculator::Calculator;
use crate::config::Config;
use crate::dbf::DbfData;
use crate::dirtree::DirTree;
use crate::editor::Editor;
use crate::file_ops;
use crate::filefind::FileFinder;
use crate::help::HelpViewer;
use crate::panel::Panel;
use crate::screensaver::ScreenSaver;
use crate::splitfile;
use crate::tetris::Tetris;
use crate::theme::{Theme, NUM_SLOTS, SLOT_DEFAULTS};
use crate::types::*;
use crate::usermenu::UserMenuData;
use crate::viewer::Viewer;

/// Compute total size of a directory recursively
fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += dir_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

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
    pub editor: Option<Editor>,
    pub tetris: Option<Tetris>,
    pub finder: Option<FileFinder>,
    pub dir_tree: Option<DirTree>,
    pub calculator: Option<Calculator>,
    pub show_menu: bool,
    pub menu_index: usize,
    pub dir_history: Vec<PathBuf>,
    pub dir_history_cursor: usize,
    pub file_history: Vec<PathBuf>,
    pub file_history_cursor: usize,
    pub select_pattern_buf: String,
    pub ascii_cursor: u8,
    pub viewer_search_buf: String,
    pub filter_buf: String,
    pub drive_list: Vec<PathBuf>,
    pub drive_cursor: usize,
    pub quick_view: bool,
    pub menu_item_cursor: usize,
    pub pending_command: Option<String>,
    pub show_user_screen: bool,
    pub last_command_output: String,
    pub archive_browser: Option<ArchiveBrowser>,
    // New features
    pub help_viewer: Option<HelpViewer>,
    pub bookmarks: [Option<PathBuf>; 9],   // Alt+1..9 quick directory bookmarks
    pub user_menu_data: Option<UserMenuData>,
    pub screensaver: ScreenSaver,
    pub env_vars: Vec<(String, String)>,    // environment variables cache
    pub env_cursor: usize,
    pub env_scroll: usize,
    pub split_size_buf: String,             // for split file dialog
    pub file_associations: Vec<(String, String)>, // (.ext, command)
    pub clock_visible: bool,
    pub confirm_delete: bool,
    pub confirm_overwrite: bool,
    pub confirm_exit: bool,
    // Viewer stack (multiple viewer windows like DN)
    pub viewer_stack: Vec<Viewer>,
    // Theme editor state
    pub theme_editor_cursor: u8,            // which slot row is selected
    pub theme_editor_fg: [u8; 13],          // current fg index (0-15) per slot
    pub theme_editor_bg: [u8; 13],          // current bg index (0-15) per slot
    // File descriptions (DESCRIPT.ION)
    pub file_descriptions: HashMap<String, String>,
    pub desc_dir: PathBuf,                  // which dir descriptions were loaded from
    pub desc_panel_visible: bool,
    // DBF/CSV viewer
    pub dbf_data: Option<DbfData>,
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
            editor: None,
            tetris: None,
            finder: None,
            dir_tree: None,
            calculator: None,
            show_menu: false,
            menu_index: 0,
            dir_history: Vec::new(),
            dir_history_cursor: 0,
            file_history: Vec::new(),
            file_history_cursor: 0,
            select_pattern_buf: String::new(),
            ascii_cursor: 0,
            viewer_search_buf: String::new(),
            filter_buf: String::new(),
            drive_list: Vec::new(),
            drive_cursor: 0,
            quick_view: false,
            menu_item_cursor: 0,
            pending_command: None,
            show_user_screen: false,
            last_command_output: String::new(),
            archive_browser: None,
            help_viewer: None,
            bookmarks: [None, None, None, None, None, None, None, None, None],
            user_menu_data: None,
            screensaver: ScreenSaver::new(),
            env_vars: Vec::new(),
            env_cursor: 0,
            env_scroll: 0,
            split_size_buf: "1048576".to_string(),
            file_associations: Self::load_associations(),
            clock_visible: true,
            confirm_delete: true,
            confirm_overwrite: true,
            confirm_exit: false,
            viewer_stack: Vec::new(),
            theme_editor_cursor: 0,
            theme_editor_fg: {
                let mut arr = [0u8; 13];
                for i in 0..13 { arr[i] = SLOT_DEFAULTS[i].0; }
                arr
            },
            theme_editor_bg: {
                let mut arr = [0u8; 13];
                for i in 0..13 { arr[i] = SLOT_DEFAULTS[i].1; }
                arr
            },
            file_descriptions: HashMap::new(),
            desc_dir: PathBuf::new(),
            desc_panel_visible: false,
            dbf_data: None,
        }
    }

    fn first_overwrite_conflict_name(&self, dest: &str) -> Option<String> {
        let dest_path = PathBuf::from(dest);
        self.active_panel()
            .get_selected_or_current()
            .iter()
            .find_map(|entry| {
                let target = dest_path.join(&entry.name);
                if target.exists() {
                    Some(entry.name.clone())
                } else {
                    None
                }
            })
    }

    pub fn request_quit(&mut self) {
        if self.confirm_exit {
            self.mode = AppMode::Dialog(DialogKind::Confirm {
                title: "Quit".to_string(),
                message: "Exit RDN?".to_string(),
                op: FileOp::Quit,
                value: None,
            });
        } else {
            self.should_quit = true;
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

    /// Open file viewer (pushes any existing viewer onto the stack)
    pub fn open_viewer(&mut self, path: &PathBuf) {
        match Viewer::open(path) {
            Ok(viewer) => {
                self.add_to_file_history(path.clone());
                if let Some(old) = self.viewer.take() {
                    self.viewer_stack.push(old);
                }
                self.viewer = Some(viewer);
                self.mode = AppMode::Viewer(path.clone());
            }
            Err(e) => {
                self.status_message = Some(format!("Cannot open file: {}", e));
            }
        }
    }

    /// Open a second viewer window over the current one (Alt+F3)
    pub fn open_viewer_new_window(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if !entry.is_dir {
                let path = entry.path.clone();
                self.open_viewer(&path);
            }
        }
    }

    /// Close viewer (restores previous viewer from stack if any)
    pub fn close_viewer(&mut self) {
        self.viewer = self.viewer_stack.pop();
        if let Some(ref v) = self.viewer {
            let p = v.path.clone();
            self.mode = AppMode::Viewer(p);
        } else {
            self.mode = AppMode::Normal;
        }
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

        if !self.confirm_delete {
            self.execute_op(FileOp::Delete, None);
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
            value: None,
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
                    if self.confirm_overwrite {
                        if let Some(name) = self.first_overwrite_conflict_name(&dest) {
                            self.mode = AppMode::Dialog(DialogKind::Confirm {
                                title: "Overwrite".to_string(),
                                message: format!("Overwrite \"{}\"?", name),
                                op: FileOp::Copy,
                                value: Some(dest),
                            });
                            return;
                        }
                    }

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
                    if self.confirm_overwrite {
                        if let Some(name) = self.first_overwrite_conflict_name(&dest) {
                            self.mode = AppMode::Dialog(DialogKind::Confirm {
                                title: "Overwrite".to_string(),
                                message: format!("Overwrite \"{}\"?", name),
                                op: FileOp::Move,
                                value: Some(dest),
                            });
                            return;
                        }
                    }

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
            FileOp::Rename => {
                if let Some(new_name) = value {
                    if let Some(entry) = self.active_panel().current_entry() {
                        let old_path = entry.path.clone();
                        let new_path = old_path.parent()
                            .unwrap_or(std::path::Path::new("."))
                            .join(&new_name);
                        match std::fs::rename(&old_path, &new_path) {
                            Ok(_) => self.status_message = Some(format!("Renamed to {}", new_name)),
                            Err(e) => self.status_message = Some(format!("Rename error: {}", e)),
                        }
                        self.refresh_panels();
                    }
                }
            }
            FileOp::Quit => {
                self.should_quit = true;
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

    // ── Editor ──────────────────────────────────────────────────

    /// Open built-in editor (F4)
    pub fn open_editor(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if !entry.is_dir {
                let path = entry.path.clone();
                match Editor::open(&path) {
                    Ok(editor) => {
                        self.add_to_file_history(path);
                        self.editor = Some(editor);
                        self.mode = AppMode::Editor;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Cannot open file: {}", e));
                    }
                }
            }
        }
    }

    /// Open editor with new empty file
    pub fn new_editor(&mut self) {
        self.editor = Some(Editor::new());
        self.mode = AppMode::Editor;
    }

    /// Close editor
    pub fn close_editor(&mut self) {
        self.editor = None;
        self.mode = AppMode::Normal;
        self.refresh_panels();
    }

    // ── Archive Browser ─────────────────────────────────────────

    pub fn open_archive(&mut self, path: &PathBuf) {
        match ArchiveBrowser::open(path) {
            Ok(browser) => {
                self.archive_browser = Some(browser);
                self.mode = AppMode::ArchiveView;
            }
            Err(e) => {
                self.status_message = Some(format!("Cannot open archive: {}", e));
            }
        }
    }

    pub fn close_archive(&mut self) {
        self.archive_browser = None;
        self.mode = AppMode::Normal;
    }

    pub fn archive_view_file(&mut self) {
        let extracted = {
            let browser = match &self.archive_browser {
                Some(b) => b,
                None => return,
            };
            let entries = browser.visible_entries();
            let entry = match entries.get(browser.cursor) {
                Some(e) => e,
                None => return,
            };
            if entry.is_dir {
                return;
            }
            let entry_path = entry.path.to_string_lossy().to_string();
            browser.extract_file(&entry_path)
        };

        match extracted {
            Ok(path) => {
                self.open_viewer(&path);
            }
            Err(e) => {
                self.status_message = Some(format!("Extract error: {}", e));
            }
        }
    }

    // ── File Find ───────────────────────────────────────────────

    pub fn start_file_find(&mut self) {
        let root = self.active_panel().path.clone();
        self.finder = Some(FileFinder::new(&root));
        self.mode = AppMode::FileFind;
    }

    pub fn close_file_find(&mut self) {
        // If a result is selected, navigate to it
        let target = self.finder.as_ref().and_then(|f| {
            f.current_result().map(|r| (
                r.path.parent().map(|p| p.to_path_buf()),
                r.name.clone(),
            ))
        });
        if let Some((Some(dir), name)) = target {
            self.active_panel_mut().path = dir;
            self.active_panel_mut().load_directory();
            let pos = self.active_panel().entries.iter()
                .position(|e| e.name == name);
            if let Some(idx) = pos {
                self.active_panel_mut().cursor = idx;
                self.active_panel_mut().ensure_visible();
            }
        }
        self.finder = None;
        self.mode = AppMode::Normal;
    }

    // ── Directory Tree ──────────────────────────────────────────

    pub fn start_dir_tree(&mut self) {
        let root = PathBuf::from("/");
        let tree = DirTree::new(&root);
        self.dir_tree = Some(tree);
        self.mode = AppMode::DirTree;
    }

    pub fn close_dir_tree(&mut self, navigate: bool) {
        let target = if navigate {
            self.dir_tree.as_ref()
                .and_then(|t| t.selected_path().map(|p| p.to_path_buf()))
        } else {
            None
        };
        if let Some(path) = target {
            self.active_panel_mut().path = path.clone();
            self.active_panel_mut().cursor = 0;
            self.active_panel_mut().scroll_offset = 0;
            self.active_panel_mut().load_directory();
            self.push_dir_history(path);
        }
        self.dir_tree = None;
        self.mode = AppMode::Normal;
    }

    // ── Calculator ──────────────────────────────────────────────

    pub fn start_calculator(&mut self) {
        self.calculator = Some(Calculator::new());
        self.mode = AppMode::Calculator;
    }

    pub fn close_calculator(&mut self) {
        self.calculator = None;
        self.mode = AppMode::Normal;
    }

    // ── Directory History ───────────────────────────────────────

    fn push_dir_history(&mut self, path: PathBuf) {
        // Avoid duplicates at the end
        if self.dir_history.last() != Some(&path) {
            self.dir_history.push(path);
            if self.dir_history.len() > 50 {
                self.dir_history.remove(0);
            }
        }
    }

    /// Enter directory and track history
    pub fn enter(&mut self) {
        let panel = self.active_panel();
        if let Some(entry) = panel.current_entry() {
            if entry.is_dir {
                let old_path = self.active_panel().path.clone();
                self.push_dir_history(old_path);
                self.active_panel_mut().enter_dir();
            } else if ArchiveBrowser::is_archive(&entry.path) {
                // Open archive browser
                let path = entry.path.clone();
                self.open_archive(&path);
            } else {
                // Open file in viewer
                let path = entry.path.clone();
                self.open_viewer(&path);
            }
        }
    }

    // ── Compare Directories ─────────────────────────────────────

    pub fn compare_directories(&mut self) {
        let left_entries: std::collections::HashSet<String> = self.left_panel.entries.iter()
            .filter(|e| e.name != "..")
            .map(|e| e.name.clone())
            .collect();
        let right_entries: std::collections::HashSet<String> = self.right_panel.entries.iter()
            .filter(|e| e.name != "..")
            .map(|e| e.name.clone())
            .collect();

        // Select files that exist only in one panel or differ in size/date
        let mut left_unique = 0usize;
        let mut right_unique = 0usize;
        let mut differ = 0usize;

        for entry in &mut self.left_panel.entries {
            if entry.name == ".." {
                continue;
            }
            if !right_entries.contains(&entry.name) {
                entry.selected = true;
                left_unique += 1;
            } else {
                // Check if size differs
                let right = self.right_panel.entries.iter().find(|e| e.name == entry.name);
                if let Some(r) = right {
                    if entry.size != r.size {
                        entry.selected = true;
                        differ += 1;
                    }
                }
            }
        }

        for entry in &mut self.right_panel.entries {
            if entry.name == ".." {
                continue;
            }
            if !left_entries.contains(&entry.name) {
                entry.selected = true;
                right_unique += 1;
            } else {
                let left = self.left_panel.entries.iter().find(|e| e.name == entry.name);
                if let Some(l) = left {
                    if entry.size != l.size {
                        entry.selected = true;
                    }
                }
            }
        }

        let msg = format!(
            "Left only: {}, Right only: {}, Different: {}",
            left_unique, right_unique, differ
        );
        self.mode = AppMode::Dialog(DialogKind::CompareResult(msg));
    }

    // ── Select/Unselect by pattern ──────────────────────────────

    pub fn start_select_pattern(&mut self, selecting: bool) {
        self.select_pattern_buf = "*.*".to_string();
        self.mode = AppMode::SelectPattern { selecting };
    }

    pub fn apply_select_pattern(&mut self, selecting: bool) {
        let pattern = self.select_pattern_buf.clone();
        if selecting {
            self.active_panel_mut().select_by_pattern(&pattern);
        } else {
            self.active_panel_mut().deselect_by_pattern(&pattern);
        }
        self.mode = AppMode::Normal;
    }

    // ── Count Directory Sizes ───────────────────────────────────

    pub fn count_dir_sizes(&mut self) {
        let panel = self.active_panel_mut();
        for entry in &mut panel.entries {
            if entry.is_dir && entry.name != ".." {
                entry.size = dir_size(&entry.path);
            }
        }
        self.status_message = Some("Directory sizes calculated".to_string());
    }

    // ── Touch file ──────────────────────────────────────────────

    pub fn touch_file(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.name != ".." {
                let path = entry.path.clone();
                let now = filetime::FileTime::now();
                if let Err(e) = filetime::set_file_mtime(&path, now) {
                    self.status_message = Some(format!("Touch failed: {}", e));
                } else {
                    self.status_message = Some(format!("Touched: {}", entry.name));
                    self.refresh_panels();
                }
            }
        }
    }

    // ── Make file list ──────────────────────────────────────────

    pub fn make_file_list(&mut self) {
        let selected = self.active_panel().get_selected_or_current();
        if selected.is_empty() {
            self.status_message = Some("No files selected".to_string());
            return;
        }
        let list_path = self.active_panel().path.join("files.lst");
        let content: String = selected.iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        match std::fs::write(&list_path, content) {
            Ok(_) => {
                self.status_message = Some(format!("File list saved: {}", list_path.display()));
                self.refresh_panels();
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }

    // ── In-place rename ─────────────────────────────────────────

    pub fn start_quick_rename(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.name != ".." {
                self.mode = AppMode::Dialog(DialogKind::Input {
                    title: "Rename".to_string(),
                    prompt: "New name:".to_string(),
                    value: entry.name.clone(),
                    op: FileOp::Rename,
                });
            }
        }
    }

    // ── ASCII Table ──────────────────────────────────────────────

    pub fn start_ascii_table(&mut self) {
        self.ascii_cursor = 0;
        self.mode = AppMode::AsciiTable;
    }

    // ── Disk Info ────────────────────────────────────────────────

    pub fn start_disk_info(&mut self) {
        self.mode = AppMode::DiskInfo;
    }

    // ── File Attributes ─────────────────────────────────────────

    pub fn start_attributes_edit(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.name == ".." {
                return;
            }
            let path = entry.path.clone();
            let meta = std::fs::metadata(&path);
            let (mode, readonly) = match &meta {
                Ok(m) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        (m.permissions().mode() & 0o7777, m.permissions().readonly())
                    }
                    #[cfg(not(unix))]
                    {
                        (0o644, m.permissions().readonly())
                    }
                }
                Err(_) => (0o644, false),
            };
            self.mode = AppMode::Dialog(DialogKind::AttributesEdit {
                path,
                mode,
                readonly,
                cursor: 0,
            });
        }
    }

    pub fn apply_attributes(&mut self, path: &PathBuf, mode: u32, _readonly: bool) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(mode);
            match std::fs::set_permissions(path, perms) {
                Ok(_) => self.status_message = Some(format!("Permissions set: {:o}", mode)),
                Err(e) => self.status_message = Some(format!("Error: {}", e)),
            }
        }
        #[cfg(not(unix))]
        {
            let mut perms = match std::fs::metadata(path) {
                Ok(m) => m.permissions(),
                Err(e) => {
                    self.status_message = Some(format!("Error: {}", e));
                    return;
                }
            };
            perms.set_readonly(_readonly);
            match std::fs::set_permissions(path, perms) {
                Ok(_) => self.status_message = Some("Permissions updated".to_string()),
                Err(e) => self.status_message = Some(format!("Error: {}", e)),
            }
        }
        self.refresh_panels();
        self.mode = AppMode::Normal;
    }

    // ── Shell execution ─────────────────────────────────────────

    pub fn execute_command(&mut self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        // Store the command for execution by the main loop
        // (which will leave alternate screen, run it, and return)
        self.pending_command = Some(cmd.to_string());
    }

    // ── Directory History ───────────────────────────────────────

    pub fn start_dir_history(&mut self) {
        if self.dir_history.is_empty() {
            self.status_message = Some("No directory history".to_string());
            return;
        }
        self.dir_history_cursor = self.dir_history.len().saturating_sub(1);
        self.mode = AppMode::DirHistory;
    }

    pub fn close_dir_history(&mut self, navigate: bool) {
        if navigate {
            if let Some(path) = self.dir_history.get(self.dir_history_cursor).cloned() {
                self.active_panel_mut().path = path;
                self.active_panel_mut().cursor = 0;
                self.active_panel_mut().scroll_offset = 0;
                self.active_panel_mut().load_directory();
            }
        }
        self.mode = AppMode::Normal;
    }

    pub fn add_to_file_history(&mut self, path: PathBuf) {
        self.file_history.retain(|p| p != &path);
        self.file_history.push(path);
        if self.file_history.len() > 100 {
            let remove_count = self.file_history.len() - 100;
            self.file_history.drain(0..remove_count);
        }
    }

    pub fn start_file_history(&mut self) {
        if self.file_history.is_empty() {
            self.status_message = Some("No file history".to_string());
            return;
        }
        self.file_history_cursor = self.file_history.len().saturating_sub(1);
        self.mode = AppMode::FileHistory;
    }

    pub fn close_file_history(&mut self, open_file: bool) {
        if open_file {
            if let Some(path) = self.file_history.get(self.file_history_cursor).cloned() {
                if path.exists() {
                    self.open_viewer(&path);
                    return;
                }
                self.status_message = Some("File no longer exists".to_string());
            }
        }
        self.mode = AppMode::Normal;
    }

    // ── Viewer search ───────────────────────────────────────────

    pub fn start_viewer_search(&mut self) {
        self.viewer_search_buf.clear();
        self.mode = AppMode::ViewerSearch;
    }

    pub fn apply_viewer_search(&mut self) {
        if let Some(ref mut v) = self.viewer {
            let query = self.viewer_search_buf.clone();
            if !query.is_empty() {
                v.search(&query);
                v.next_match();
            }
        }
        self.mode = if let Some(ref v) = self.viewer {
            AppMode::Viewer(v.path.clone())
        } else {
            AppMode::Normal
        };
    }

    // ── Panel filter ────────────────────────────────────────────

    pub fn start_panel_filter(&mut self) {
        let current = self.active_panel().filter.clone().unwrap_or_else(|| "*".to_string());
        self.filter_buf = current;
        self.mode = AppMode::PanelFilter;
    }

    pub fn apply_panel_filter(&mut self) {
        let f = self.filter_buf.clone();
        let panel = self.active_panel_mut();
        if f == "*" || f == "*.*" || f.is_empty() {
            panel.filter = None;
        } else {
            panel.filter = Some(f);
        }
        panel.cursor = 0;
        panel.scroll_offset = 0;
        panel.load_directory();
        self.mode = AppMode::Normal;
    }

    // ── Drive / bookmark select ─────────────────────────────────

    pub fn start_drive_select(&mut self) {
        self.drive_list = Self::collect_drives();
        self.drive_cursor = 0;
        self.mode = AppMode::DriveSelect;
    }

    pub fn close_drive_select(&mut self, navigate: bool) {
        if navigate {
            if let Some(path) = self.drive_list.get(self.drive_cursor).cloned() {
                self.active_panel_mut().path = path.clone();
                self.active_panel_mut().cursor = 0;
                self.active_panel_mut().scroll_offset = 0;
                self.active_panel_mut().load_directory();
                self.push_dir_history(path);
            }
        }
        self.mode = AppMode::Normal;
    }

    fn collect_drives() -> Vec<PathBuf> {
        let mut drives = Vec::new();

        // Home directory
        if let Some(home) = dirs::home_dir() {
            drives.push(home);
        }

        // Root
        drives.push(PathBuf::from("/"));

        // Common mount points
        #[cfg(target_os = "macos")]
        {
            if let Ok(entries) = std::fs::read_dir("/Volumes") {
                for entry in entries.flatten() {
                    drives.push(entry.path());
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // /mnt and /media mounts
            for base in &["/mnt", "/media"] {
                if let Ok(entries) = std::fs::read_dir(base) {
                    for entry in entries.flatten() {
                        drives.push(entry.path());
                    }
                }
            }
        }

        // Desktop, Downloads, Documents if they exist
        if let Some(home) = dirs::home_dir() {
            for dir in &["Desktop", "Downloads", "Documents"] {
                let p = home.join(dir);
                if p.exists() {
                    drives.push(p);
                }
            }
        }

        drives
    }

    // ── Quick view ──────────────────────────────────────────────

    pub fn toggle_quick_view(&mut self) {
        self.quick_view = !self.quick_view;
    }

    // ── Directory branch (flat recursive listing) ───────────────

    pub fn toggle_dir_branch(&mut self) {
        let panel = self.active_panel_mut();
        if panel.filter == Some("__branch__".to_string()) {
            // Turn off branch mode
            panel.filter = None;
            panel.load_directory();
            self.status_message = Some("Branch mode off".to_string());
            return;
        }

        // Collect all files recursively
        let root = panel.path.clone();
        let mut all_entries = Vec::new();
        Self::collect_branch_entries(&root, &root, &mut all_entries, panel.show_hidden);

        panel.entries.clear();
        // Add ".." entry
        if let Some(parent) = root.parent() {
            if parent != root.as_path() {
                panel.entries.push(FileEntry {
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
        panel.entries.extend(all_entries);
        panel.filter = Some("__branch__".to_string()); // marker
        panel.cursor = 0;
        panel.scroll_offset = 0;
        self.status_message = Some(format!("Branch: {} files", panel.entries.len() - 1));
    }

    fn collect_branch_entries(
        root: &PathBuf,
        dir: &PathBuf,
        entries: &mut Vec<FileEntry>,
        show_hidden: bool,
    ) {
        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for de in read_dir.flatten() {
                let path = de.path();
                let name = path.strip_prefix(root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| de.file_name().to_string_lossy().to_string());

                if !show_hidden && de.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }

                let meta = de.metadata();
                let (size, modified, is_readonly, is_dir, is_symlink) = match &meta {
                    Ok(m) => (m.len(), m.modified().ok(), m.permissions().readonly(), m.is_dir(), m.is_symlink()),
                    Err(_) => (0, None, false, false, false),
                };

                let is_executable = {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        !is_dir && meta.as_ref().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
                    }
                    #[cfg(not(unix))]
                    { false }
                };

                if !is_dir {
                    entries.push(FileEntry {
                        name,
                        path: path.clone(),
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

                if is_dir {
                    Self::collect_branch_entries(root, &path, entries, show_hidden);
                }
            }
        }
    }

    // ── Menu system ─────────────────────────────────────────────

    // ── Help System ─────────────────────────────────────────────

    pub fn start_help(&mut self) {
        self.help_viewer = Some(HelpViewer::new());
        self.mode = AppMode::Help;
    }

    pub fn close_help(&mut self) {
        self.help_viewer = None;
        self.mode = AppMode::Normal;
    }

    // ── Directory Bookmarks ─────────────────────────────────────

    pub fn set_bookmark(&mut self, idx: usize) {
        if idx < 9 {
            let path = self.active_panel().path.clone();
            self.bookmarks[idx] = Some(path.clone());
            self.status_message = Some(format!("Bookmark {} set: {}", idx + 1, path.display()));
        }
    }

    pub fn goto_bookmark(&mut self, idx: usize) {
        if idx < 9 {
            if let Some(ref path) = self.bookmarks[idx].clone() {
                if path.exists() {
                    self.push_dir_history(self.active_panel().path.clone());
                    self.active_panel_mut().path = path.clone();
                    self.active_panel_mut().cursor = 0;
                    self.active_panel_mut().scroll_offset = 0;
                    self.active_panel_mut().load_directory();
                    self.status_message = Some(format!("Bookmark {}: {}", idx + 1, path.display()));
                } else {
                    self.status_message = Some(format!("Bookmark {} path doesn't exist", idx + 1));
                }
            } else {
                self.status_message = Some(format!("Bookmark {} not set", idx + 1));
            }
        }
    }

    // ── User Menu ───────────────────────────────────────────────

    pub fn start_user_menu(&mut self) {
        self.user_menu_data = Some(UserMenuData::load());
        self.mode = AppMode::UserMenu;
    }

    pub fn execute_user_menu_item(&mut self) {
        let cmd = {
            let data = match &self.user_menu_data {
                Some(d) => d,
                None => return,
            };
            let item = match data.current_item() {
                Some(i) => i,
                None => return,
            };
            if item.command.is_empty() {
                return;
            }
            let current_file = self.active_panel().current_entry()
                .map(|e| e.name.clone());
            let current_dir = self.active_panel().path.clone();
            let other_dir = self.inactive_panel().path.clone();
            UserMenuData::substitute_command(
                &item.command,
                current_file.as_deref(),
                &current_dir,
                Some(&other_dir),
            )
        };
        self.user_menu_data = None;
        self.mode = AppMode::Normal;
        self.execute_command(&cmd);
    }

    pub fn close_user_menu(&mut self) {
        self.user_menu_data = None;
        self.mode = AppMode::Normal;
    }

    // ── Environment Variables Viewer ────────────────────────────

    pub fn start_env_viewer(&mut self) {
        let mut vars: Vec<(String, String)> = std::env::vars().collect();
        vars.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        self.env_vars = vars;
        self.env_cursor = 0;
        self.env_scroll = 0;
        self.mode = AppMode::EnvViewer;
    }

    pub fn close_env_viewer(&mut self) {
        self.env_vars.clear();
        self.mode = AppMode::Normal;
    }

    // ── System Info ─────────────────────────────────────────────

    pub fn start_system_info(&mut self) {
        self.mode = AppMode::SystemInfo;
    }

    // ── Split / Combine Files ───────────────────────────────────

    pub fn start_split_file(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.is_dir || entry.name == ".." {
                self.status_message = Some("Select a file to split".to_string());
                return;
            }
            self.split_size_buf = "1048576".to_string(); // 1MB default
            self.mode = AppMode::SplitFileDialog;
        }
    }

    pub fn execute_split_file(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            let path = entry.path.clone();
            let dest_dir = self.inactive_panel().path.clone();
            let chunk_size: u64 = self.split_size_buf.parse().unwrap_or(splitfile::DEFAULT_CHUNK_SIZE);
            match splitfile::split_file(&path, chunk_size, &dest_dir) {
                Ok(count) => {
                    self.status_message = Some(format!("Split into {} chunks", count));
                    self.refresh_panels();
                }
                Err(e) => {
                    self.status_message = Some(format!("Split error: {}", e));
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    pub fn start_combine_file(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.is_dir || entry.name == ".." {
                self.status_message = Some("Select a .001 file to combine".to_string());
                return;
            }
            if !splitfile::is_first_chunk(&entry.path) {
                self.status_message = Some("Select the first chunk (.001) to combine".to_string());
                return;
            }
            self.mode = AppMode::CombineFileDialog;
        }
    }

    pub fn execute_combine_file(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            let first_chunk = entry.path.clone();
            let stem = first_chunk.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("combined")
                .to_string();
            let dest = self.inactive_panel().path.join(&stem);
            match splitfile::combine_files(&first_chunk, &dest) {
                Ok(total) => {
                    self.status_message = Some(format!(
                        "Combined into {} ({} bytes)", stem, total
                    ));
                    self.refresh_panels();
                }
                Err(e) => {
                    self.status_message = Some(format!("Combine error: {}", e));
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    // ── File Associations ───────────────────────────────────────

    fn load_associations() -> Vec<(String, String)> {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("rdn");
        let assoc_path = config_dir.join("assoc.txt");
        if let Ok(content) = std::fs::read_to_string(&assoc_path) {
            content.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .filter_map(|l| {
                    l.split_once('=').map(|(ext, cmd)| {
                        (ext.trim().to_lowercase().to_string(), cmd.trim().to_string())
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn open_with_association(&mut self) -> bool {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.is_dir {
                return false;
            }
            let ext = entry.path.extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();
            let filename = entry.name.clone();
            let dir = self.active_panel().path.clone();

            for (assoc_ext, cmd_template) in &self.file_associations {
                if assoc_ext == &ext {
                    let cmd = cmd_template
                        .replace("%f", &filename)
                        .replace("%d", &dir.to_string_lossy())
                        .replace("%n", entry.path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(""));
                    self.execute_command(&cmd);
                    return true;
                }
            }
        }
        false
    }

    // ── Screen Saver ────────────────────────────────────────────

    pub fn activate_screensaver(&mut self) {
        self.screensaver.active = true;
        self.mode = AppMode::ScreenSaver;
    }

    pub fn deactivate_screensaver(&mut self) {
        self.screensaver.reset_idle();
        self.mode = AppMode::Normal;
    }

    pub fn menu_labels() -> Vec<&'static str> {
        vec![" Left ", " Files ", " Commands ", " Options ", " Right "]
    }

    pub fn menu_items(index: usize) -> Vec<MenuItem> {
        match index {
            0 => vec![ // Left panel
                MenuItem::item("Brief mode",       "",         MenuAction::PanelBrief),
                MenuItem::item("Full mode",        "",         MenuAction::PanelFull),
                MenuItem::separator(),
                MenuItem::item("Sort by Name",     "",         MenuAction::SortName),
                MenuItem::item("Sort by Extension","",         MenuAction::SortExt),
                MenuItem::item("Sort by Size",     "",         MenuAction::SortSize),
                MenuItem::item("Sort by Date",     "",         MenuAction::SortDate),
                MenuItem::item("Unsorted",         "",         MenuAction::SortUnsorted),
                MenuItem::separator(),
                MenuItem::item("Filter...",        "F11",      MenuAction::PanelFilter),
                MenuItem::item("Re-read",          "Ctrl+R",   MenuAction::PanelReread),
                MenuItem::item("Change drive...",  "Alt+F1",   MenuAction::ChangeDriveLeft),
            ],
            1 => vec![ // Files
                MenuItem::item("View",             "F3",       MenuAction::ViewFile),
                MenuItem::item("Edit",             "F4",       MenuAction::EditFile),
                MenuItem::item("Edit new file",    "Shift+F4", MenuAction::EditNewFile),
                MenuItem::separator(),
                MenuItem::item("Copy",             "F5",       MenuAction::Copy),
                MenuItem::item("Rename/Move",      "F6",       MenuAction::Move),
                MenuItem::item("Make directory",   "F7",       MenuAction::MakeDir),
                MenuItem::item("Delete",           "F8",       MenuAction::Delete),
                MenuItem::separator(),
                MenuItem::item("Find file...",     "Alt+F7",   MenuAction::FileFind),
                MenuItem::item("Attributes...",    "",         MenuAction::FileAttributes),
                MenuItem::item("Quick rename",     "Shift+F6", MenuAction::QuickRename),
                MenuItem::item("Touch file",       "Ctrl+T",   MenuAction::TouchFile),
                MenuItem::item("Make file list",   "Ctrl+F",   MenuAction::MakeFileList),
                MenuItem::separator(),
                MenuItem::item("Split file...",    "Ctrl+F3",  MenuAction::SplitFile),
                MenuItem::item("Combine files...", "Ctrl+F4",  MenuAction::CombineFile),
                MenuItem::separator(),
                MenuItem::item("Base64 Encode",    "",          MenuAction::Base64Encode),
                MenuItem::item("Base64 Decode",    "",          MenuAction::Base64Decode),
                MenuItem::item("UU Encode",         "",          MenuAction::UUEncode),
                MenuItem::item("UU Decode",         "",          MenuAction::UUDecode),
                MenuItem::separator(),
                MenuItem::item("Quit",             "Alt+X",    MenuAction::Quit),
            ],
            2 => vec![ // Commands
                MenuItem::item("Directory tree",   "Alt+F10",  MenuAction::DirTree),
                MenuItem::item("Find file...",     "Alt+F7",   MenuAction::FileFind),
                MenuItem::item("History of dirs",  "Alt+F12",  MenuAction::DirHistory),
                MenuItem::item("Compare dirs",     "Alt+C",    MenuAction::CompareDirs),
                MenuItem::item("Count dir sizes",  "Ctrl+F5",  MenuAction::CountDirSizes),
                MenuItem::separator(),
                MenuItem::item("Dir branch",       "Ctrl+B",   MenuAction::DirBranch),
                MenuItem::item("Swap panels",      "Ctrl+U",   MenuAction::SwapPanels),
                MenuItem::item("Sync panels",      "Ctrl+S",   MenuAction::SyncPanels),
                MenuItem::separator(),
                MenuItem::item("Select group",     "Gray +",   MenuAction::SelectGroup),
                MenuItem::item("Unselect group",   "Gray -",   MenuAction::UnselectGroup),
                MenuItem::item("Invert selection", "Gray *",   MenuAction::InvertSelection),
                MenuItem::separator(),
                MenuItem::item("User menu",        "F2",       MenuAction::UserMenu),
            ],
            3 => vec![ // Options
                MenuItem::item("Sort mode...",     "",         MenuAction::SortMenu),
                MenuItem::item("Show hidden",      "Ctrl+H",  MenuAction::ShowHidden),
                MenuItem::item("Quick view",       "Ctrl+P",  MenuAction::QuickView),
                MenuItem::separator(),
                MenuItem::item("Calculator",       "Ctrl+K",  MenuAction::Calculator),
                MenuItem::item("ASCII table",      "Alt+F9",  MenuAction::AsciiTable),
                MenuItem::item("Disk info",        "Alt+I",   MenuAction::DiskInfo),
                MenuItem::item("System info",      "Ctrl+I",  MenuAction::SystemInfo),
                MenuItem::item("Env variables",    "Ctrl+E",  MenuAction::EnvViewer),
                MenuItem::item("Tetris",           "Ctrl+G",  MenuAction::Tetris),
                MenuItem::separator(),
                MenuItem::item("Theme editor",     "Ctrl+W",  MenuAction::ThemeEditor),
                MenuItem::item("Descriptions",     "",         MenuAction::ToggleDescPanel),
                MenuItem::item("File history",     "Ctrl+Y",  MenuAction::FileHistory),
                MenuItem::item("Confirmations...", "",         MenuAction::ConfirmSettings),
                MenuItem::separator(),
                MenuItem::item("Help",             "F1",       MenuAction::Help),
                MenuItem::item("Refresh display",  "Ctrl+L",   MenuAction::RefreshDisplay),
                MenuItem::separator(),
                MenuItem::item("Save desktop",     "",         MenuAction::SaveDesktop),
                MenuItem::item("Load desktop",     "",         MenuAction::LoadDesktop),
            ],
            4 => vec![ // Right panel
                MenuItem::item("Brief mode",       "",         MenuAction::PanelBrief),
                MenuItem::item("Full mode",        "",         MenuAction::PanelFull),
                MenuItem::separator(),
                MenuItem::item("Sort by Name",     "",         MenuAction::SortName),
                MenuItem::item("Sort by Extension","",         MenuAction::SortExt),
                MenuItem::item("Sort by Size",     "",         MenuAction::SortSize),
                MenuItem::item("Sort by Date",     "",         MenuAction::SortDate),
                MenuItem::item("Unsorted",         "",         MenuAction::SortUnsorted),
                MenuItem::separator(),
                MenuItem::item("Filter...",        "F12",      MenuAction::PanelFilter),
                MenuItem::item("Re-read",          "Ctrl+R",   MenuAction::PanelReread),
                MenuItem::item("Change drive...",  "Alt+F2",   MenuAction::ChangeDriveRight),
            ],
            _ => vec![],
        }
    }

    /// Open/activate menu bar
    pub fn open_menu(&mut self) {
        self.show_menu = true;
        self.menu_item_cursor = 0;
        self.mode = AppMode::Menu;
    }

    /// Execute a menu action
    pub fn execute_menu_action(&mut self, action: MenuAction) {
        self.show_menu = false;
        self.mode = AppMode::Normal;

        match action {
            MenuAction::Separator => {}
            MenuAction::ViewFile => {
                if let Some(entry) = self.active_panel().current_entry() {
                    if !entry.is_dir {
                        let path = entry.path.clone();
                        self.open_viewer(&path);
                    }
                }
            }
            MenuAction::EditFile => self.open_editor(),
            MenuAction::EditNewFile => self.new_editor(),
            MenuAction::Copy => self.start_copy(),
            MenuAction::Move => self.start_move(),
            MenuAction::MakeDir => self.start_mkdir(),
            MenuAction::Delete => self.start_delete(),
            MenuAction::FileFind => self.start_file_find(),
            MenuAction::FileAttributes => self.start_attributes_edit(),
            MenuAction::QuickRename => self.start_quick_rename(),
            MenuAction::TouchFile => self.touch_file(),
            MenuAction::MakeFileList => self.make_file_list(),
            MenuAction::Quit => self.request_quit(),
            MenuAction::DirTree => self.start_dir_tree(),
            MenuAction::DirHistory => self.start_dir_history(),
            MenuAction::CompareDirs => self.compare_directories(),
            MenuAction::CountDirSizes => self.count_dir_sizes(),
            MenuAction::DirBranch => self.toggle_dir_branch(),
            MenuAction::SwapPanels => self.swap_panels(),
            MenuAction::SyncPanels => self.sync_panels(),
            MenuAction::UserMenu => {
                self.start_user_menu();
            }
            MenuAction::Calculator => self.start_calculator(),
            MenuAction::AsciiTable => self.start_ascii_table(),
            MenuAction::DiskInfo => self.start_disk_info(),
            MenuAction::Tetris => self.start_tetris(),
            MenuAction::ShowHidden => self.toggle_hidden(),
            MenuAction::QuickView => self.toggle_quick_view(),
            MenuAction::SortMenu => {
                self.mode = AppMode::Dialog(DialogKind::SortMenu);
            }
            MenuAction::SelectGroup => self.start_select_pattern(true),
            MenuAction::UnselectGroup => self.start_select_pattern(false),
            MenuAction::InvertSelection => {
                self.active_panel_mut().invert_selection();
            }
            MenuAction::SystemInfo => self.start_system_info(),
            MenuAction::EnvViewer => self.start_env_viewer(),
            MenuAction::SplitFile => self.start_split_file(),
            MenuAction::CombineFile => self.start_combine_file(),
            MenuAction::PanelBrief => {
                // For Left/Right menus, target the appropriate panel
                let target = if self.menu_index == 0 {
                    &mut self.left_panel
                } else {
                    &mut self.right_panel
                };
                target.panel_mode = PanelMode::Brief;
            }
            MenuAction::PanelFull => {
                let target = if self.menu_index == 0 {
                    &mut self.left_panel
                } else {
                    &mut self.right_panel
                };
                target.panel_mode = PanelMode::Full;
            }
            MenuAction::SortName => {
                let target = if self.menu_index == 0 || self.menu_index == 4 {
                    if self.menu_index == 0 { &mut self.left_panel } else { &mut self.right_panel }
                } else {
                    self.active_panel_mut()
                };
                target.sort_mode = SortMode::Name;
                target.sort_entries();
            }
            MenuAction::SortExt => {
                let target = if self.menu_index == 0 { &mut self.left_panel } else if self.menu_index == 4 { &mut self.right_panel } else { self.active_panel_mut() };
                target.sort_mode = SortMode::Extension;
                target.sort_entries();
            }
            MenuAction::SortSize => {
                let target = if self.menu_index == 0 { &mut self.left_panel } else if self.menu_index == 4 { &mut self.right_panel } else { self.active_panel_mut() };
                target.sort_mode = SortMode::Size;
                target.sort_entries();
            }
            MenuAction::SortDate => {
                let target = if self.menu_index == 0 { &mut self.left_panel } else if self.menu_index == 4 { &mut self.right_panel } else { self.active_panel_mut() };
                target.sort_mode = SortMode::Date;
                target.sort_entries();
            }
            MenuAction::SortUnsorted => {
                let target = if self.menu_index == 0 { &mut self.left_panel } else if self.menu_index == 4 { &mut self.right_panel } else { self.active_panel_mut() };
                target.sort_mode = SortMode::Unsorted;
                target.sort_entries();
            }
            MenuAction::PanelFilter => self.start_panel_filter(),
            MenuAction::PanelReread => self.refresh_panels(),
            MenuAction::ChangeDriveLeft => {
                self.active = ActivePanel::Left;
                self.start_drive_select();
            }
            MenuAction::ChangeDriveRight => {
                self.active = ActivePanel::Right;
                self.start_drive_select();
            }
            MenuAction::Help => {
                self.start_help();
            }
            MenuAction::RefreshDisplay => self.refresh_panels(),
            MenuAction::SaveDesktop => self.save_desktop(),
            MenuAction::LoadDesktop => self.load_desktop(),
            MenuAction::Base64Encode => self.base64_encode_file(),
            MenuAction::Base64Decode => self.base64_decode_file(),
            MenuAction::UUEncode => self.uuencode_file(),
            MenuAction::UUDecode => self.uudecode_file(),
            MenuAction::ThemeEditor => self.start_theme_editor(),
            MenuAction::ToggleDescPanel => self.toggle_desc_panel(),
            MenuAction::FileHistory => self.start_file_history(),
            MenuAction::ConfirmSettings => {
                self.mode = AppMode::Dialog(crate::types::DialogKind::ConfirmSettings { cursor: 0 });
            }
        }
    }

    /// Get the number of selectable (non-separator) items in current menu
    pub fn menu_selectable_count(&self) -> usize {
        let items = Self::menu_items(self.menu_index);
        items.iter().filter(|i| !i.is_separator()).count()
    }

    /// Convert menu_item_cursor (selectable index) to actual index in items vec
    pub fn menu_cursor_to_index(&self) -> usize {
        let items = Self::menu_items(self.menu_index);
        let mut selectable = 0;
        for (i, item) in items.iter().enumerate() {
            if !item.is_separator() {
                if selectable == self.menu_item_cursor {
                    return i;
                }
                selectable += 1;
            }
        }
        0
    }

    // ── Config / Desktop save/load ──────────────────────────────

    pub fn save_desktop(&mut self) {
        let config = Config {
            left_path: self.left_panel.path.clone(),
            right_path: self.right_panel.path.clone(),
            left_sort: self.left_panel.sort_mode,
            right_sort: self.right_panel.sort_mode,
            left_mode: self.left_panel.panel_mode,
            right_mode: self.right_panel.panel_mode,
            show_hidden: self.left_panel.show_hidden,
            editor_tab_size: 4,
            editor_auto_indent: true,
            editor_word_wrap: false,
            confirm_delete: self.confirm_delete,
            confirm_overwrite: self.confirm_overwrite,
            confirm_exit: self.confirm_exit,
            theme_overrides: crate::theme::Theme::export_overrides(),
            bookmarks: self
                .bookmarks
                .iter()
                .map(|b| {
                    b.as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                })
                .collect(),
        };
        match config.save() {
            Ok(msg) => self.status_message = Some(msg),
            Err(e) => self.status_message = Some(e),
        }
    }

    pub fn load_desktop(&mut self) {
        let config = Config::load();
        self.left_panel.path = config.left_path;
        self.left_panel.sort_mode = config.left_sort;
        self.left_panel.panel_mode = config.left_mode;
        self.left_panel.show_hidden = config.show_hidden;
        self.left_panel.load_directory();

        self.right_panel.path = config.right_path;
        self.right_panel.sort_mode = config.right_sort;
        self.right_panel.panel_mode = config.right_mode;
        self.right_panel.show_hidden = config.show_hidden;
        self.right_panel.load_directory();

        self.confirm_delete = config.confirm_delete;
        self.confirm_overwrite = config.confirm_overwrite;
        self.confirm_exit = config.confirm_exit;

        // Restore theme overrides
        if !config.theme_overrides.is_empty() {
            crate::theme::Theme::import_overrides(&config.theme_overrides);
        }

        // Restore bookmarks
        for (i, bm) in config.bookmarks.iter().enumerate() {
            if i < 9 && !bm.is_empty() {
                self.bookmarks[i] = Some(PathBuf::from(bm));
            }
        }

        self.status_message = Some("Desktop loaded".to_string());
    }

    /// Create app from saved config
    pub fn from_config() -> Self {
        let config = Config::load();
        let mut app = App::new();

        if config.left_path.exists() {
            app.left_panel.path = config.left_path;
            app.left_panel.load_directory();
        }
        if config.right_path.exists() {
            app.right_panel.path = config.right_path;
            app.right_panel.load_directory();
        }

        app.left_panel.sort_mode = config.left_sort;
        app.left_panel.panel_mode = config.left_mode;
        app.left_panel.show_hidden = config.show_hidden;
        app.left_panel.sort_entries();

        app.right_panel.sort_mode = config.right_sort;
        app.right_panel.panel_mode = config.right_mode;
        app.right_panel.show_hidden = config.show_hidden;
        app.right_panel.sort_entries();

        app.confirm_delete = config.confirm_delete;
        app.confirm_overwrite = config.confirm_overwrite;
        app.confirm_exit = config.confirm_exit;

        // Restore theme overrides
        if !config.theme_overrides.is_empty() {
            crate::theme::Theme::import_overrides(&config.theme_overrides);
        }

        // Restore bookmarks
        for (i, bm) in config.bookmarks.iter().enumerate() {
            if i < 9 && !bm.is_empty() {
                app.bookmarks[i] = Some(PathBuf::from(bm));
            }
        }

        app
    }

    // ── Theme Editor ────────────────────────────────────────────────

    pub fn start_theme_editor(&mut self) {
        // Load current slot indices from Theme
        for i in 0..NUM_SLOTS as usize {
            let (fg, bg) = crate::theme::Theme::slot_indices(i as u8);
            self.theme_editor_fg[i] = fg;
            self.theme_editor_bg[i] = bg;
        }
        self.theme_editor_cursor = 0;
        self.mode = AppMode::ThemeEditor;
    }

    pub fn apply_theme_editor(&mut self) {
        for i in 0..NUM_SLOTS as usize {
            crate::theme::Theme::set_slot(i as u8, self.theme_editor_fg[i], self.theme_editor_bg[i]);
        }
        self.mode = AppMode::Normal;
    }

    pub fn theme_editor_up(&mut self) {
        if self.theme_editor_cursor > 0 {
            self.theme_editor_cursor -= 1;
        }
    }

    pub fn theme_editor_down(&mut self) {
        if (self.theme_editor_cursor as usize) + 1 < NUM_SLOTS as usize {
            self.theme_editor_cursor += 1;
        }
    }

    pub fn theme_editor_fg_next(&mut self) {
        let i = self.theme_editor_cursor as usize;
        self.theme_editor_fg[i] = (self.theme_editor_fg[i] + 1) % 16;
    }

    pub fn theme_editor_fg_prev(&mut self) {
        let i = self.theme_editor_cursor as usize;
        self.theme_editor_fg[i] = (self.theme_editor_fg[i] + 15) % 16;
    }

    pub fn theme_editor_bg_next(&mut self) {
        let i = self.theme_editor_cursor as usize;
        self.theme_editor_bg[i] = (self.theme_editor_bg[i] + 1) % 16;
    }

    pub fn theme_editor_bg_prev(&mut self) {
        let i = self.theme_editor_cursor as usize;
        self.theme_editor_bg[i] = (self.theme_editor_bg[i] + 15) % 16;
    }

    // ── File Descriptions (DESCRIPT.ION) ───────────────────────────

    pub fn load_descriptions(&mut self) {
        let dir = self.active_panel().path.clone();
        if dir == self.desc_dir {
            return; // already loaded for this dir
        }
        self.file_descriptions.clear();
        self.desc_dir = dir.clone();

        // Try DESCRIPT.ION first, then FILES.BBS
        for fname in &["DESCRIPT.ION", "descript.ion", "FILES.BBS", "files.bbs"] {
            let path = dir.join(fname);
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    // Format: filename description...
                    if let Some((name, desc)) = line.splitn(2, ' ').collect::<Vec<_>>().windows(2).next().map(|w| (w[0], w[1])) {
                        self.file_descriptions.insert(name.trim_matches('"').to_string(), desc.trim().to_string());
                    }
                }
                break;
            }
        }
    }

    pub fn save_descriptions(&mut self) {
        let path = self.desc_dir.join("DESCRIPT.ION");
        let mut lines: Vec<String> = self
            .file_descriptions
            .iter()
            .map(|(k, v)| format!("{} {}", k, v))
            .collect();
        lines.sort();
        let content = lines.join("\n") + "\n";
        match std::fs::write(&path, content) {
            Ok(_) => self.status_message = Some("Descriptions saved".to_string()),
            Err(e) => self.status_message = Some(format!("Save error: {}", e)),
        }
    }

    pub fn set_file_description(&mut self, name: String, desc: String) {
        if desc.is_empty() {
            self.file_descriptions.remove(&name);
        } else {
            self.file_descriptions.insert(name, desc);
        }
        self.save_descriptions();
    }

    pub fn get_file_description(&self, name: &str) -> Option<&str> {
        self.file_descriptions.get(name).map(|s| s.as_str())
    }

    pub fn toggle_desc_panel(&mut self) {
        self.desc_panel_visible = !self.desc_panel_visible;
        if self.desc_panel_visible {
            // Force reload for current dir
            self.desc_dir = PathBuf::new();
            self.load_descriptions();
        }
    }

    pub fn start_edit_description(&mut self) {
        let name = self.active_panel().current_entry()
            .map(|e| e.name.clone())
            .unwrap_or_default();
        if name.is_empty() { return; }
        let current_desc = self.get_file_description(&name)
            .unwrap_or_default()
            .to_string();
        self.mode = AppMode::Dialog(DialogKind::Input {
            title: format!("Description for {}", name),
            prompt: "Desc:".to_string(),
            value: current_desc,
            op: FileOp::MkDir, // reuse as a generic text op – handled specially
        });
    }

    // ── DBF / CSV Viewer ────────────────────────────────────────────

    pub fn open_dbf_viewer(&mut self) {
        if let Some(entry) = self.active_panel().current_entry() {
            if entry.is_dir { return; }
            let path = entry.path.clone();
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            let result = if ext == "csv" {
                DbfData::open_csv(&path)
            } else {
                DbfData::open(&path)
            };
            match result {
                Ok(data) => {
                    self.dbf_data = Some(data);
                    self.mode = AppMode::DbfView;
                }
                Err(e) => {
                    self.status_message = Some(format!("Cannot open: {}", e));
                }
            }
        }
    }

    pub fn close_dbf_viewer(&mut self) {
        self.dbf_data = None;
        self.mode = AppMode::Normal;
    }

    // ── Base64 / UU Encode-Decode ───────────────────────────────────

    pub fn base64_encode_file(&mut self) {
        use base64::Engine;
        let entries = self.active_panel().get_selected_or_current();
        let mut count = 0usize;
        for entry in &entries {
            if entry.is_dir { continue; }
            match std::fs::read(&entry.path) {
                Ok(data) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                    let out_path = entry.path.with_extension(
                        format!("{}.b64", entry.path.extension().and_then(|e| e.to_str()).unwrap_or(""))
                    );
                    let _ = std::fs::write(&out_path, encoded);
                    count += 1;
                }
                Err(_) => {}
            }
        }
        self.status_message = Some(format!("Base64 encoded {} file(s)", count));
        self.refresh_panels();
    }

    pub fn base64_decode_file(&mut self) {
        use base64::Engine;
        let entries = self.active_panel().get_selected_or_current();
        let mut count = 0usize;
        let mut errors = 0usize;
        for entry in &entries {
            if entry.is_dir { continue; }
            let text = match std::fs::read_to_string(&entry.path) {
                Ok(t) => t,
                Err(_) => { errors += 1; continue; }
            };
            let decoded = match base64::engine::general_purpose::STANDARD
                .decode(text.trim().as_bytes())
            {
                Ok(d) => d,
                Err(_) => { errors += 1; continue; }
            };
            // Output filename: strip .b64 extension if present, else add .decoded
            let out_path = {
                let name = entry.path.to_string_lossy();
                if name.ends_with(".b64") {
                    PathBuf::from(&name[..name.len() - 4])
                } else {
                    entry.path.with_extension("decoded")
                }
            };
            let _ = std::fs::write(&out_path, &decoded);
            count += 1;
        }
        if errors > 0 {
            self.status_message = Some(format!("Decoded {}, {} error(s)", count, errors));
        } else {
            self.status_message = Some(format!("Base64 decoded {} file(s)", count));
        }
        self.refresh_panels();
    }

    pub fn uuencode_file(&mut self) {
        let entries = self.active_panel().get_selected_or_current();
        let mut count = 0usize;
        for entry in &entries {
            if entry.is_dir { continue; }
            match std::fs::read(&entry.path) {
                Ok(data) => {
                    let name = entry.name.clone();
                    let encoded = uu_encode(&data, &name);
                    let out_path = entry.path.with_extension("uu");
                    let _ = std::fs::write(&out_path, encoded);
                    count += 1;
                }
                Err(_) => {}
            }
        }
        self.status_message = Some(format!("UU encoded {} file(s)", count));
        self.refresh_panels();
    }

    pub fn uudecode_file(&mut self) {
        let entries = self.active_panel().get_selected_or_current();
        let mut count = 0usize;
        let mut errors = 0usize;
        for entry in &entries {
            if entry.is_dir { continue; }
            let text = match std::fs::read_to_string(&entry.path) {
                Ok(t) => t,
                Err(_) => { errors += 1; continue; }
            };
            match uu_decode(&text) {
                Some((filename, data)) => {
                    let dir = entry.path.parent().unwrap_or(std::path::Path::new("."));
                    let out_path = dir.join(&filename);
                    let _ = std::fs::write(&out_path, &data);
                    count += 1;
                }
                None => { errors += 1; }
            }
        }
        if errors > 0 {
            self.status_message = Some(format!("Decoded {}, {} error(s)", count, errors));
        } else {
            self.status_message = Some(format!("UU decoded {} file(s)", count));
        }
        self.refresh_panels();
    }
}

// ── UU encoding helpers ──────────────────────────────────────────────

fn uu_encode(data: &[u8], filename: &str) -> String {
    let mut out = format!("begin 644 {}\n", filename);
    for chunk in data.chunks(45) {
        let len = chunk.len() as u8;
        let mut line = String::new();
        line.push((len + 32) as char);
        for triple in chunk.chunks(3) {
            let (a, b, c) = (
                *triple.get(0).unwrap_or(&0),
                *triple.get(1).unwrap_or(&0),
                *triple.get(2).unwrap_or(&0),
            );
            let chars = [
                ((a >> 2) + 32) as char,
                (((a & 3) << 4 | b >> 4) + 32) as char,
                (((b & 0xf) << 2 | c >> 6) + 32) as char,
                ((c & 0x3f) + 32) as char,
            ];
            for ch in &chars {
                line.push(if *ch == ' ' { '`' } else { *ch });
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("`\nend\n");
    out
}

fn uu_decode(text: &str) -> Option<(String, Vec<u8>)> {
    let mut lines = text.lines();
    // Find begin line
    let begin = lines.find(|l| l.starts_with("begin "))?;
    let parts: Vec<&str> = begin.splitn(3, ' ').collect();
    let filename = parts.get(2).unwrap_or(&"decoded").to_string();

    let mut data: Vec<u8> = Vec::new();
    for line in &mut lines {
        if line == "end" || line == "`" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let bytes: Vec<u8> = line.bytes().map(|b| if b == b'`' { 32 } else { b }).collect();
        if bytes.is_empty() {
            continue;
        }
        let count = (bytes[0] - 32) as usize;
        let mut i = 1usize;
        let mut decoded = 0usize;
        while decoded < count && i + 3 < bytes.len() {
            let (a, b, c, d) = (
                bytes[i] - 32,
                bytes[i + 1] - 32,
                bytes[i + 2] - 32,
                bytes[i + 3] - 32,
            );
            if decoded < count { data.push((a << 2 | b >> 4) as u8); decoded += 1; }
            if decoded < count { data.push((b << 4 | c >> 2) as u8); decoded += 1; }
            if decoded < count { data.push((c << 6 | d) as u8); decoded += 1; }
            i += 4;
        }
    }
    Some((filename, data))
}
