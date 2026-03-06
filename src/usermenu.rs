/// User menu system (inspired by DN's USERMENU.PAS)
/// Reads menu entries from ~/.config/rdn/menu.txt
/// Format: label=command
/// Lines starting with # are comments, empty lines are separators.
/// Supports parameter substitution: %f = current file, %d = current directory,
/// %n = filename without extension, %e = extension, %p = other panel path

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct UserMenuItem {
    pub label: String,
    pub command: String,
    pub is_separator: bool,
}

#[derive(Debug)]
pub struct UserMenuData {
    pub items: Vec<UserMenuItem>,
    pub cursor: usize,
}

impl UserMenuData {
    pub fn load() -> Self {
        let items = load_menu_file();
        UserMenuData {
            items,
            cursor: 0,
        }
    }

    pub fn selectable_count(&self) -> usize {
        self.items.iter().filter(|i| !i.is_separator).count()
    }

    pub fn current_item(&self) -> Option<&UserMenuItem> {
        let mut idx = 0;
        for item in &self.items {
            if !item.is_separator {
                if idx == self.cursor {
                    return Some(item);
                }
                idx += 1;
            }
        }
        None
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        let max = self.selectable_count().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    /// Substitute parameters in command string
    pub fn substitute_command(
        command: &str,
        current_file: Option<&str>,
        current_dir: &Path,
        other_panel_dir: Option<&Path>,
    ) -> String {
        let mut result = command.to_string();

        if let Some(file) = current_file {
            let path = Path::new(file);
            let name_no_ext = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let ext = path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            result = result.replace("%f", file);
            result = result.replace("%n", name_no_ext);
            result = result.replace("%e", ext);
        } else {
            result = result.replace("%f", "");
            result = result.replace("%n", "");
            result = result.replace("%e", "");
        }

        result = result.replace("%d", &current_dir.to_string_lossy());

        if let Some(other) = other_panel_dir {
            result = result.replace("%p", &other.to_string_lossy());
        } else {
            result = result.replace("%p", "");
        }

        result
    }
}

fn menu_file_path() -> PathBuf {
    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("rdn");
    config_dir.join("menu.txt")
}

fn load_menu_file() -> Vec<UserMenuItem> {
    let path = menu_file_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            // Create default menu file
            let default = default_menu();
            let config_dir = path.parent().unwrap();
            let _ = std::fs::create_dir_all(config_dir);
            let _ = std::fs::write(&path, &default);
            default
        }
    };

    let mut items = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            items.push(UserMenuItem {
                label: String::new(),
                command: String::new(),
                is_separator: true,
            });
        } else if trimmed.starts_with('#') {
            continue;
        } else if let Some((label, command)) = trimmed.split_once('=') {
            items.push(UserMenuItem {
                label: label.trim().to_string(),
                command: command.trim().to_string(),
                is_separator: false,
            });
        }
    }

    if items.is_empty() {
        items.push(UserMenuItem {
            label: "No items configured".to_string(),
            command: String::new(),
            is_separator: false,
        });
    }

    items
}

fn default_menu() -> String {
    r#"# RDN User Menu
# Format: label=command
# Variables: %f=file, %d=dir, %n=name, %e=ext, %p=other panel
# Empty lines are separators, lines starting with # are comments.

Git Status=git status
Git Log=git log --oneline -20
Git Diff=git diff

Disk Usage=du -sh *
Find Large Files=find . -size +10M -exec ls -lh {} \;

List All Files=ls -laR
Count Files=find . -type f | wc -l

Open Terminal=echo "Current dir: %d" && $SHELL
"#.to_string()
}
