use std::fs;
use std::path::{Path, PathBuf};

/// Directory tree browser (inspired by DN's TREE.PAS)
#[derive(Debug)]
pub struct DirTree {
    pub root: PathBuf,
    pub nodes: Vec<TreeNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub expanded: bool,
    pub has_children: bool,
}

impl DirTree {
    pub fn new(root: &Path) -> Self {
        let mut tree = DirTree {
            root: root.to_path_buf(),
            nodes: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            visible_height: 20,
        };
        tree.build_root();
        tree
    }

    fn build_root(&mut self) {
        self.nodes.clear();
        let has_children = Self::dir_has_subdirs(&self.root);
        self.nodes.push(TreeNode {
            name: self.root.to_string_lossy().to_string(),
            path: self.root.clone(),
            depth: 0,
            expanded: true,
            has_children,
        });
        self.expand_node(0);
    }

    pub fn expand_node(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }
        self.nodes[index].expanded = true;
        let path = self.nodes[index].path.clone();
        let depth = self.nodes[index].depth + 1;

        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        let child_path = entry.path();
                        let has_children = Self::dir_has_subdirs(&child_path);
                        children.push(TreeNode {
                            name,
                            path: child_path,
                            depth,
                            expanded: false,
                            has_children,
                        });
                    }
                }
            }
        }
        children.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // Insert children after the current node, before the next sibling
        let insert_pos = self.find_insert_pos(index);
        for (i, child) in children.into_iter().enumerate() {
            self.nodes.insert(insert_pos + i, child);
        }
    }

    pub fn collapse_node(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }
        self.nodes[index].expanded = false;
        let depth = self.nodes[index].depth;

        // Remove all children (nodes with depth > current depth until we hit same/lower depth)
        let mut remove_count = 0;
        for i in (index + 1)..self.nodes.len() {
            if self.nodes[i].depth > depth {
                remove_count += 1;
            } else {
                break;
            }
        }
        if remove_count > 0 {
            self.nodes.drain((index + 1)..(index + 1 + remove_count));
        }
    }

    pub fn toggle_expand(&mut self) {
        if self.cursor >= self.nodes.len() {
            return;
        }
        let expanded = self.nodes[self.cursor].expanded;
        let has_children = self.nodes[self.cursor].has_children;
        if has_children {
            if expanded {
                self.collapse_node(self.cursor);
            } else {
                self.expand_node(self.cursor);
            }
        }
    }

    pub fn enter(&mut self) {
        // Expand/enter the selected directory
        if self.cursor < self.nodes.len() && !self.nodes[self.cursor].expanded {
            self.expand_node(self.cursor);
        }
    }

    fn find_insert_pos(&self, parent_index: usize) -> usize {
        let depth = self.nodes[parent_index].depth;
        for i in (parent_index + 1)..self.nodes.len() {
            if self.nodes[i].depth <= depth {
                return i;
            }
        }
        self.nodes.len()
    }

    fn dir_has_subdirs(path: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.starts_with('.') {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // Navigation
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.ensure_visible();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.nodes.len() {
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
        self.cursor = (self.cursor + page).min(self.nodes.len().saturating_sub(1));
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

    pub fn selected_path(&self) -> Option<&Path> {
        self.nodes.get(self.cursor).map(|n| n.path.as_path())
    }
}
