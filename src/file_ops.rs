use std::fs;
use std::io;
use std::path::Path;

/// Result type for file operations
pub type FileOpResult = Result<String, String>;

/// Copy a file or directory to destination
pub fn copy_entry(src: &Path, dest_dir: &Path) -> FileOpResult {
    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid source path".to_string())?;
    let dest = dest_dir.join(file_name);

    if src.is_dir() {
        copy_dir_recursive(src, &dest)
            .map(|_| format!("Copied directory: {}", file_name.to_string_lossy()))
            .map_err(|e| format!("Copy failed: {}", e))
    } else {
        // Avoid copying to self
        if src == dest {
            return Err("Source and destination are the same".to_string());
        }
        fs::copy(src, &dest)
            .map(|_| format!("Copied: {}", file_name.to_string_lossy()))
            .map_err(|e| format!("Copy failed: {}", e))
    }
}

/// Move a file or directory to destination
pub fn move_entry(src: &Path, dest_dir: &Path) -> FileOpResult {
    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid source path".to_string())?;
    let dest = dest_dir.join(file_name);

    if src == dest {
        return Err("Source and destination are the same".to_string());
    }

    // Try rename first (fast for same filesystem)
    match fs::rename(src, &dest) {
        Ok(_) => Ok(format!("Moved: {}", file_name.to_string_lossy())),
        Err(_) => {
            // Fallback: copy then delete
            if src.is_dir() {
                copy_dir_recursive(src, &dest)
                    .and_then(|_| fs::remove_dir_all(src))
                    .map(|_| format!("Moved directory: {}", file_name.to_string_lossy()))
                    .map_err(|e| format!("Move failed: {}", e))
            } else {
                fs::copy(src, &dest)
                    .and_then(|_| fs::remove_file(src))
                    .map(|_| format!("Moved: {}", file_name.to_string_lossy()))
                    .map_err(|e| format!("Move failed: {}", e))
            }
        }
    }
}

/// Delete a file or directory (to trash if available, otherwise permanent)
pub fn delete_entry(path: &Path, use_trash: bool) -> FileOpResult {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    if use_trash {
        match trash::delete(path) {
            Ok(_) => return Ok(format!("Moved to trash: {}", name)),
            Err(e) => {
                // Fallback: if trash fails (e.g. no permission for Apple Events on macOS),
                // try moving to ~/.Trash manually
                #[cfg(target_os = "macos")]
                {
                    if let Some(home) = std::env::var_os("HOME") {
                        let trash_dir = Path::new(&home).join(".Trash");
                        let dest = trash_dir.join(&name);
                        if trash_dir.is_dir() {
                            let move_result = if path.is_dir() {
                                // Use fs::rename, fall back to copy+delete
                                fs::rename(path, &dest).or_else(|_| {
                                    copy_dir_recursive(path, &dest)?;
                                    fs::remove_dir_all(path)
                                })
                            } else {
                                fs::rename(path, &dest)
                            };
                            if move_result.is_ok() {
                                return Ok(format!("Moved to trash: {}", name));
                            }
                        }
                    }
                }
                // If all trash methods failed, fall back to permanent deletion
                eprintln!("Trash unavailable ({}), deleting permanently", e);
            }
        }
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
            .map(|_| format!("Deleted directory: {}", name))
            .map_err(|e| format!("Delete failed: {}", e))
    } else {
        fs::remove_file(path)
            .map(|_| format!("Deleted: {}", name))
            .map_err(|e| format!("Delete failed: {}", e))
    }
}

/// Create a new directory
pub fn make_dir(parent: &Path, name: &str) -> FileOpResult {
    let path = parent.join(name);
    fs::create_dir_all(&path)
        .map(|_| format!("Created directory: {}", name))
        .map_err(|e| format!("MkDir failed: {}", e))
}

/// Rename a file or directory
pub fn rename_entry(old: &Path, new_name: &str) -> FileOpResult {
    let parent = old.parent().ok_or_else(|| "Invalid path".to_string())?;
    let new_path = parent.join(new_name);
    fs::rename(old, &new_path)
        .map(|_| format!("Renamed to: {}", new_name))
        .map_err(|e| format!("Rename failed: {}", e))
}

/// Recursive directory copy
fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Get disk usage info for a path (placeholder)
pub fn disk_info(_path: &Path) -> Option<(u64, u64)> {
    None
}
