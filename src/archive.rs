use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::types::FileEntry;

/// Archive entry representing a file or directory inside an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,        // full path within archive
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: u64,
    pub modified: Option<SystemTime>,
}

/// Archive browser state
#[derive(Debug)]
pub struct ArchiveBrowser {
    pub archive_path: PathBuf,
    pub archive_type: ArchiveType,
    pub entries: Vec<ArchiveEntry>,
    pub current_dir: String,   // current directory within archive (e.g., "src/")
    pub cursor: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    Zip,
    TarGz,
    Tar,
}

impl ArchiveBrowser {
    /// Try to open an archive file
    pub fn open(path: &Path) -> Result<Self, String> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let archive_type = match ext.as_str() {
            "zip" | "jar" | "war" | "apk" | "epub" => ArchiveType::Zip,
            "gz" | "tgz" => ArchiveType::TarGz,
            "tar" => ArchiveType::Tar,
            _ => return Err(format!("Unsupported archive format: .{}", ext)),
        };

        let mut browser = ArchiveBrowser {
            archive_path: path.to_path_buf(),
            archive_type,
            entries: Vec::new(),
            current_dir: String::new(),
            cursor: 0,
            scroll_offset: 0,
            visible_height: 20,
        };

        browser.read_archive()?;
        Ok(browser)
    }

    /// Read the archive contents
    fn read_archive(&mut self) -> Result<(), String> {
        self.entries.clear();
        match self.archive_type {
            ArchiveType::Zip => self.read_zip(),
            ArchiveType::TarGz => self.read_tar_gz(),
            ArchiveType::Tar => self.read_tar(),
        }
    }

    fn read_zip(&mut self) -> Result<(), String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                let is_dir = entry.is_dir();
                let size = entry.size();
                let compressed_size = entry.compressed_size();

                let modified = entry.last_modified().map(|dt| {
                    // Convert zip DateTime to SystemTime
                    use std::time::Duration;
                    let year = dt.year() as u64;
                    let month = dt.month() as u64;
                    let day = dt.day() as u64;
                    let hour = dt.hour() as u64;
                    let minute = dt.minute() as u64;
                    // Rough approximation
                    let days_since_epoch = (year - 1970) * 365 + (month - 1) * 30 + day;
                    let secs = days_since_epoch * 86400 + hour * 3600 + minute * 60;
                    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
                });

                self.entries.push(ArchiveEntry {
                    name: name.clone(),
                    path: name,
                    is_dir,
                    size,
                    compressed_size,
                    modified,
                });
            }
        }

        Ok(())
    }

    fn read_tar_gz(&mut self) -> Result<(), String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;

        let gz = flate2::read::GzDecoder::new(file);
        self.read_tar_from(gz)
    }

    fn read_tar(&mut self) -> Result<(), String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;

        self.read_tar_from(file)
    }

    fn read_tar_from<R: std::io::Read>(&mut self, reader: R) -> Result<(), String> {
        let mut archive = tar::Archive::new(reader);
        let entries = archive.entries()
            .map_err(|e| format!("Cannot read tar: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let is_dir = entry.header().entry_type().is_dir();
            let size = entry.header().size().unwrap_or(0);
            let modified = entry.header().mtime().ok().map(|t| {
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(t as u64)
            });

            self.entries.push(ArchiveEntry {
                name: path.clone(),
                path,
                is_dir,
                size,
                compressed_size: size, // tar doesn't compress
                modified,
            });
        }

        Ok(())
    }

    /// Get entries visible at the current directory level
    pub fn visible_entries(&self) -> Vec<FileEntry> {
        let mut result = Vec::new();
        let prefix = &self.current_dir;

        // Track directories we've already added at this level
        let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Add ".." if we're inside a subdirectory
        if !self.current_dir.is_empty() {
            result.push(FileEntry {
                name: "..".to_string(),
                path: PathBuf::from(".."),
                is_dir: true,
                is_symlink: false,
                size: 0,
                modified: None,
                is_readonly: true,
                is_hidden: false,
                is_executable: false,
                selected: false,
            });
        }

        for entry in &self.entries {
            let entry_path = &entry.path;

            // Only show entries at the current directory level
            if !entry_path.starts_with(prefix) {
                continue;
            }

            let relative = &entry_path[prefix.len()..];
            if relative.is_empty() {
                continue;
            }

            // Check if this is a direct child (no more '/' except trailing)
            let parts: Vec<&str> = relative.trim_end_matches('/').split('/').collect();
            if parts.len() == 1 {
                // Direct child
                let display_name = parts[0].to_string();
                if display_name.is_empty() {
                    continue;
                }

                if entry.is_dir {
                    if seen_dirs.insert(display_name.clone()) {
                        result.push(FileEntry {
                            name: display_name,
                            path: PathBuf::from(entry_path),
                            is_dir: true,
                            is_symlink: false,
                            size: entry.size,
                            modified: entry.modified,
                            is_readonly: true,
                            is_hidden: false,
                            is_executable: false,
                            selected: false,
                        });
                    }
                } else {
                    result.push(FileEntry {
                        name: display_name,
                        path: PathBuf::from(entry_path),
                        is_dir: false,
                        is_symlink: false,
                        size: entry.size,
                        modified: entry.modified,
                        is_readonly: true,
                        is_hidden: false,
                        is_executable: false,
                        selected: false,
                    });
                }
            } else if parts.len() > 1 {
                // This is inside a subdirectory - add the subdirectory if not seen
                let dir_name = parts[0].to_string();
                if seen_dirs.insert(dir_name.clone()) {
                    result.push(FileEntry {
                        name: dir_name,
                        path: PathBuf::from(format!("{}{}/", prefix, parts[0])),
                        is_dir: true,
                        is_symlink: false,
                        size: 0,
                        modified: None,
                        is_readonly: true,
                        is_hidden: false,
                        is_executable: false,
                        selected: false,
                    });
                }
            }
        }

        // Sort: dirs first, then by name
        result[if self.current_dir.is_empty() { 0 } else { 1 }..].sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        result
    }

    /// Navigate into a subdirectory within the archive
    pub fn enter_dir(&mut self) {
        let entries = self.visible_entries();
        if let Some(entry) = entries.get(self.cursor) {
            if entry.name == ".." {
                // Go up
                if let Some(pos) = self.current_dir[..self.current_dir.len().saturating_sub(1)].rfind('/') {
                    self.current_dir = self.current_dir[..=pos].to_string();
                } else {
                    self.current_dir = String::new();
                }
                self.cursor = 0;
                self.scroll_offset = 0;
            } else if entry.is_dir {
                self.current_dir = format!("{}{}/", self.current_dir, entry.name);
                self.cursor = 0;
                self.scroll_offset = 0;
            }
        }
    }

    /// Get the archive title for display
    pub fn title(&self) -> String {
        let name = self.archive_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if self.current_dir.is_empty() {
            name
        } else {
            format!("{}:{}", name, self.current_dir)
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            if self.cursor < self.scroll_offset {
                self.scroll_offset = self.cursor;
            }
        }
    }

    pub fn cursor_down(&mut self) {
        let max = self.visible_entries().len();
        if self.cursor + 1 < max {
            self.cursor += 1;
            if self.cursor >= self.scroll_offset + self.visible_height {
                self.scroll_offset = self.cursor - self.visible_height + 1;
            }
        }
    }

    pub fn page_up(&mut self) {
        if self.cursor >= self.visible_height {
            self.cursor -= self.visible_height;
        } else {
            self.cursor = 0;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
    }

    pub fn page_down(&mut self) {
        let max = self.visible_entries().len();
        self.cursor = (self.cursor + self.visible_height).min(max.saturating_sub(1));
        if self.cursor >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.cursor - self.visible_height + 1;
        }
    }

    /// Extract a file to a temp directory and return the path
    pub fn extract_file(&self, entry_path: &str) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir().join("rdn_archive");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Cannot create temp dir: {}", e))?;

        match self.archive_type {
            ArchiveType::Zip => self.extract_zip_file(entry_path, &temp_dir),
            ArchiveType::TarGz => self.extract_tar_gz_file(entry_path, &temp_dir),
            ArchiveType::Tar => self.extract_tar_file(entry_path, &temp_dir),
        }
    }

    fn extract_zip_file(&self, entry_path: &str, temp_dir: &Path) -> Result<PathBuf, String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid ZIP: {}", e))?;

        let mut entry = archive.by_name(entry_path)
            .map_err(|e| format!("File not found in archive: {}", e))?;

        let file_name = Path::new(entry_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "extracted".to_string());

        let dest = temp_dir.join(&file_name);
        let mut out = std::fs::File::create(&dest)
            .map_err(|e| format!("Cannot create temp file: {}", e))?;

        std::io::copy(&mut entry, &mut out)
            .map_err(|e| format!("Extraction failed: {}", e))?;

        Ok(dest)
    }

    fn extract_tar_gz_file(&self, entry_path: &str, temp_dir: &Path) -> Result<PathBuf, String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;
        let gz = flate2::read::GzDecoder::new(file);
        self.extract_from_tar(gz, entry_path, temp_dir)
    }

    fn extract_tar_file(&self, entry_path: &str, temp_dir: &Path) -> Result<PathBuf, String> {
        let file = std::fs::File::open(&self.archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;
        self.extract_from_tar(file, entry_path, temp_dir)
    }

    fn extract_from_tar<R: std::io::Read>(&self, reader: R, entry_path: &str, temp_dir: &Path) -> Result<PathBuf, String> {
        let mut archive = tar::Archive::new(reader);
        let entries = archive.entries()
            .map_err(|e| format!("Cannot read tar: {}", e))?;

        for entry_result in entries {
            if let Ok(mut entry) = entry_result {
                let path = entry.path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                if path == entry_path {
                    let file_name = Path::new(entry_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "extracted".to_string());

                    let dest = temp_dir.join(&file_name);
                    let mut out = std::fs::File::create(&dest)
                        .map_err(|e| format!("Cannot create temp file: {}", e))?;

                    std::io::copy(&mut entry, &mut out)
                        .map_err(|e| format!("Extraction failed: {}", e))?;

                    return Ok(dest);
                }
            }
        }

        Err("File not found in archive".to_string())
    }

    /// Check if a path is a supported archive
    pub fn is_archive(path: &Path) -> bool {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        matches!(ext.as_str(),
            "zip" | "jar" | "war" | "apk" | "epub" |
            "gz" | "tgz" | "tar"
        )
    }
}
