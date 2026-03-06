use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// ── DOS VGA 16-colour palette (exact RGB values) ────────────────────
//
// Decoded from CColor palette in Dos-Navigator DNAPP.PAS + CDoubleWindow
// in DBLWND.PAS.  Panel entries (indices 85-89, 165-181) all use DOS
// colour 8 (dark gray) as background.
//
const DOS_BLACK: Color      = Color::Rgb(0, 0, 0);
const DOS_BLUE: Color       = Color::Rgb(0, 0, 170);
const DOS_GREEN: Color      = Color::Rgb(0, 170, 0);
const DOS_CYAN: Color       = Color::Rgb(0, 170, 170);
const DOS_DARK_GRAY: Color  = Color::Rgb(85, 85, 85);
const DOS_LIGHT_GRAY: Color = Color::Rgb(170, 170, 170);
const DOS_LIGHT_BLUE: Color = Color::Rgb(85, 85, 255);
const DOS_LIGHT_GREEN: Color= Color::Rgb(85, 255, 85);
const DOS_LIGHT_CYAN: Color = Color::Rgb(85, 255, 255);
const DOS_LIGHT_RED: Color  = Color::Rgb(255, 85, 85);
const DOS_LIGHT_MAGENTA: Color = Color::Rgb(255, 85, 255);
const DOS_YELLOW: Color     = Color::Rgb(255, 255, 85);
const DOS_WHITE: Color      = Color::Rgb(255, 255, 255);

// ── DOS 16-colour palette as indexed array ───────────────────────────
pub const DOS_PALETTE: [Color; 16] = [
    Color::Rgb(0, 0, 0),       // 0 Black
    Color::Rgb(0, 0, 170),     // 1 Blue
    Color::Rgb(0, 170, 0),     // 2 Green
    Color::Rgb(0, 170, 170),   // 3 Cyan
    Color::Rgb(170, 0, 0),     // 4 Red
    Color::Rgb(170, 0, 170),   // 5 Magenta
    Color::Rgb(170, 85, 0),    // 6 Brown
    Color::Rgb(170, 170, 170), // 7 Light Gray
    Color::Rgb(85, 85, 85),    // 8 Dark Gray
    Color::Rgb(85, 85, 255),   // 9 Light Blue
    Color::Rgb(85, 255, 85),   // 10 Light Green
    Color::Rgb(85, 255, 255),  // 11 Light Cyan
    Color::Rgb(255, 85, 85),   // 12 Light Red
    Color::Rgb(255, 85, 255),  // 13 Light Magenta
    Color::Rgb(255, 255, 85),  // 14 Yellow
    Color::Rgb(255, 255, 255), // 15 White
];

pub const PALETTE_NAMES: [&str; 16] = [
    "Black", "Blue", "Green", "Cyan", "Red", "Magenta", "Brown", "Lt Gray",
    "Dk Gray", "Lt Blue", "Lt Green", "Lt Cyan", "Lt Red", "Lt Magenta", "Yellow", "White",
];

// ── Color slot indices ───────────────────────────────────────────────
pub const SLOT_PANEL_BG:        u8 = 0;
pub const SLOT_FILE_NORMAL:     u8 = 1;
pub const SLOT_FILE_DIR:        u8 = 2;
pub const SLOT_CURSOR_ACTIVE:   u8 = 3;
pub const SLOT_FILE_SELECTED:   u8 = 4;
pub const SLOT_MENU_BAR:        u8 = 5;
pub const SLOT_DIALOG_BG:       u8 = 6;
pub const SLOT_STATUS_BAR:      u8 = 7;
pub const SLOT_FN_KEY:          u8 = 8;
pub const SLOT_VIEWER_TEXT:     u8 = 9;
pub const SLOT_INFO_LINE:       u8 = 10;
pub const SLOT_PANEL_BORDER:    u8 = 11;
pub const SLOT_CMD_LINE:        u8 = 12;
pub const NUM_SLOTS:            u8 = 13;

pub const SLOT_NAMES: [&str; NUM_SLOTS as usize] = [
    "Panel background",
    "File (normal)",
    "Directory entry",
    "Cursor (active)",
    "Selected file",
    "Menu bar",
    "Dialog background",
    "Status bar",
    "Fn-key bar",
    "Viewer text",
    "Info line",
    "Panel border",
    "Command line",
];

/// Default (fg_idx, bg_idx) from DOS palette for each slot
pub const SLOT_DEFAULTS: [(u8, u8); NUM_SLOTS as usize] = [
    (7, 8),  // SLOT_PANEL_BG:      light-gray on dark-gray
    (7, 8),  // SLOT_FILE_NORMAL:   light-gray on dark-gray
    (15, 8), // SLOT_FILE_DIR:      white on dark-gray
    (0, 3),  // SLOT_CURSOR_ACTIVE: black on cyan
    (14, 8), // SLOT_FILE_SELECTED: yellow on dark-gray
    (0, 3),  // SLOT_MENU_BAR:      black on cyan
    (0, 7),  // SLOT_DIALOG_BG:     black on light-gray (DN $70)
    (0, 3),  // SLOT_STATUS_BAR:    black on cyan
    (0, 3),  // SLOT_FN_KEY:        black on cyan
    (7, 8),  // SLOT_VIEWER_TEXT:   light-gray on dark-gray
    (11, 8), // SLOT_INFO_LINE:     light-cyan on dark-gray
    (15, 8), // SLOT_PANEL_BORDER:  white on dark-gray
    (7, 0),  // SLOT_CMD_LINE:      light-gray on black
];

// ── Runtime color overrides ──────────────────────────────────────────
fn overrides() -> std::sync::MutexGuard<'static, HashMap<u8, (u8, u8)>> {
    static OVERRIDES: OnceLock<Mutex<HashMap<u8, (u8, u8)>>> = OnceLock::new();
    OVERRIDES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

impl Theme {
    fn slot(id: u8, default_fg: Color, default_bg: Color) -> Style {
        let map = overrides();
        if let Some(&(fi, bi)) = map.get(&id) {
            Style::default()
                .fg(DOS_PALETTE[fi as usize % 16])
                .bg(DOS_PALETTE[bi as usize % 16])
        } else {
            Style::default().fg(default_fg).bg(default_bg)
        }
    }

    /// Get the current (fg_idx, bg_idx) for a slot
    pub fn slot_indices(id: u8) -> (u8, u8) {
        overrides()
            .get(&id)
            .copied()
            .unwrap_or(SLOT_DEFAULTS[id as usize % NUM_SLOTS as usize])
    }

    /// Set override for a slot (persists for the session)
    pub fn set_slot(id: u8, fg: u8, bg: u8) {
        let mut map = overrides();
        let (def_fg, def_bg) = SLOT_DEFAULTS[id as usize % NUM_SLOTS as usize];
        if fg == def_fg && bg == def_bg {
            map.remove(&id);
        } else {
            map.insert(id, (fg, bg));
        }
    }

    /// Export all non-default overrides as `slot:fg:bg` strings
    pub fn export_overrides() -> Vec<String> {
        overrides()
            .iter()
            .map(|(&slot, &(fg, bg))| format!("{}:{}:{}", slot, fg, bg))
            .collect()
    }

    /// Import overrides from `slot:fg:bg` strings
    pub fn import_overrides(entries: &[String]) {
        let mut map = overrides();
        map.clear();
        for s in entries {
            let parts: Vec<&str> = s.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let (Ok(slot), Ok(fg), Ok(bg)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                ) {
                    if slot < NUM_SLOTS && fg < 16 && bg < 16 {
                        map.insert(slot, (fg, bg));
                    }
                }
            }
        }
    }
}

/// Classic Dos Navigator colour theme.
///
/// Decoded by tracing the Turbo-Vision palette chain:
///   TFilePanel → CPanel → CDoubleWindow → CColor (app palette)
///
/// File panels use **dark gray** (DOS colour 8) as background,
/// with a **cyan** cursor bar.  Menu & F-key bars sit on cyan.
pub struct Theme;

impl Theme {
    // ── Panel borders & titles ──────────────────────────────────────
    // CColor[81]=$8F (active frame)  → white on dark gray
    // CColor[80]=$87 (passive frame) → light gray on dark gray
    // CColor[82]=$8B (frame icons)   → light cyan on dark gray

    pub fn panel_border_active() -> Style {
        Self::slot(SLOT_PANEL_BORDER, DOS_WHITE, DOS_DARK_GRAY)
    }

    pub fn panel_border_inactive() -> Style {
        let s = Self::slot(SLOT_PANEL_BORDER, DOS_WHITE, DOS_DARK_GRAY);
        // passive border: same bg, dim fg
        s.fg(DOS_LIGHT_GRAY)
    }

    pub fn panel_title_active() -> Style {
        // Title in the active frame: light cyan on dark gray ($8B)
        Style::default()
            .fg(DOS_LIGHT_CYAN)
            .bg(DOS_DARK_GRAY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn panel_title_inactive() -> Style {
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_DARK_GRAY)
    }

    // ── Column headers inside a panel ───────────────────────────────
    // CColor[90]=$3F  (active panel top)  → white on cyan
    // CColor[91]=$8F  (passive panel top) → white on dark gray
    // In practice DN shows yellow "Name" headers; we use yellow on dark gray.

    pub fn column_header() -> Style {
        Style::default().fg(DOS_YELLOW).bg(DOS_DARK_GRAY)
    }

    // ── File list entries ───────────────────────────────────────────
    // CColor[85]=$87  normal text   → light gray fg on dark gray bg
    // CColor[87]=$8E  selected/mark → yellow on dark gray
    // Highlight types (CColor[172-181]):
    //   $8F=white, $8B=light cyan, $8A=light green,
    //   $83=cyan, $82=green, $8D=light magenta, ...

    pub fn file_normal() -> Style {
        Self::slot(SLOT_FILE_NORMAL, DOS_LIGHT_GRAY, DOS_DARK_GRAY)
    }

    pub fn file_dir() -> Style {
        Self::slot(SLOT_FILE_DIR, DOS_WHITE, DOS_DARK_GRAY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn file_executable() -> Style {
        // Executables: light green on dark gray ($8A)
        Style::default().fg(DOS_LIGHT_GREEN).bg(DOS_DARK_GRAY)
    }

    pub fn file_symlink() -> Style {
        // Symlinks: light magenta on dark gray ($8D)
        Style::default().fg(DOS_LIGHT_MAGENTA).bg(DOS_DARK_GRAY)
    }

    pub fn file_hidden() -> Style {
        // Hidden files: cyan on dark gray ($83)
        Style::default().fg(DOS_CYAN).bg(DOS_DARK_GRAY)
    }

    pub fn file_selected() -> Style {
        Self::slot(SLOT_FILE_SELECTED, DOS_YELLOW, DOS_DARK_GRAY)
            .add_modifier(Modifier::BOLD)
    }

    // ── Cursor bar ──────────────────────────────────────────────────
    // CColor[88]=$30: black on cyan  (normal cursor)
    // CColor[89]=$3E: yellow on cyan (selected cursor)

    pub fn cursor_active() -> Style {
        Self::slot(SLOT_CURSOR_ACTIVE, DOS_BLACK, DOS_CYAN)
    }

    pub fn cursor_inactive() -> Style {
        // inactive cursor: same bg as PANEL_BORDER slot, fg from SLOT_FILE_NORMAL
        let bg = Self::slot(SLOT_PANEL_BG, DOS_DARK_GRAY, DOS_DARK_GRAY);
        bg.fg(DOS_LIGHT_GRAY)
    }

    // ── Panel background (fills gaps) ───────────────────────────────

    pub fn panel_bg() -> Style {
        Self::slot(SLOT_PANEL_BG, DOS_LIGHT_GRAY, DOS_DARK_GRAY)
    }

    // ── Info line (bottom of panel) ─────────────────────────────────
    // CColor[86]=$8F separator / info → white on dark gray

    pub fn info_line() -> Style {
        Self::slot(SLOT_INFO_LINE, DOS_LIGHT_CYAN, DOS_DARK_GRAY)
    }

    // ── Status bar ──────────────────────────────────────────────────

    pub fn status_bar() -> Style {
        Self::slot(SLOT_STATUS_BAR, DOS_BLACK, DOS_CYAN)
    }

    // ── Function key bar (F1-F10 at the bottom) ─────────────────────

    pub fn fn_key_number() -> Style {
        Style::default().fg(DOS_WHITE).bg(DOS_BLACK)
    }

    pub fn fn_key_label() -> Style {
        Self::slot(SLOT_FN_KEY, DOS_BLACK, DOS_CYAN)
    }

    // ── Menu bar (top line) ─────────────────────────────────────────

    pub fn menu_bar() -> Style {
        Self::slot(SLOT_MENU_BAR, DOS_BLACK, DOS_CYAN)
    }

    pub fn menu_bar_highlight() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_BLACK)
            .add_modifier(Modifier::BOLD)
    }

    // ── Dialogs ─────────────────────────────────────────────────────

    /// Dialog frame: White on Light Gray (DN CColor[33] = $7F)
    pub fn dialog_border() -> Style {
        let s = Self::slot(SLOT_DIALOG_BG, DOS_BLACK, DOS_LIGHT_GRAY);
        // Border uses brighter fg than text on same bg
        s.fg(DOS_WHITE)
    }

    /// Dialog text: Black on Light Gray (DN CColor[37] = $70)
    pub fn dialog_text() -> Style {
        Self::slot(SLOT_DIALOG_BG, DOS_BLACK, DOS_LIGHT_GRAY)
    }

    /// Dialog input field: Black on Cyan (DN CColor[49] = $30)
    pub fn dialog_input() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_LIGHT_CYAN)
    }

    /// Unfocused button: Black on Cyan
    pub fn dialog_button() -> Style {
        Style::default()
            .fg(DOS_BLACK)
            .bg(DOS_CYAN)
            .add_modifier(Modifier::BOLD)
    }

    /// Focused/active button: White on Green
    pub fn dialog_button_focused() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_GREEN)
            .add_modifier(Modifier::BOLD)
    }

    // ── Command line ────────────────────────────────────────────────

    pub fn command_line() -> Style {
        Self::slot(SLOT_CMD_LINE, DOS_LIGHT_GRAY, DOS_BLACK)
    }

    // ── Cursor bar (used in list dialogs) ───────────────────────────

    pub fn cursor_bar() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_CYAN)
    }

    // ── Input text (editable field) ─────────────────────────────────

    pub fn input_text() -> Style {
        Style::default().fg(DOS_YELLOW).bg(DOS_DARK_GRAY)
    }

    // ── Quick search overlay ────────────────────────────────────────

    pub fn quick_search() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_YELLOW)
    }

    // ── Viewer ──────────────────────────────────────────────────────
    // CColor[92]=$87 viewer normal → light gray on dark gray
    // CColor[93]=$70 viewer highlight → black on light gray

    pub fn viewer_text() -> Style {
        Self::slot(SLOT_VIEWER_TEXT, DOS_LIGHT_GRAY, DOS_DARK_GRAY)
    }

    pub fn viewer_header() -> Style {
        Style::default().fg(DOS_YELLOW).bg(DOS_DARK_GRAY)
    }

    pub fn viewer_hex_offset() -> Style {
        Style::default().fg(DOS_YELLOW).bg(DOS_DARK_GRAY)
    }

    pub fn viewer_hex_bytes() -> Style {
        Style::default().fg(DOS_LIGHT_CYAN).bg(DOS_DARK_GRAY)
    }

    pub fn viewer_hex_ascii() -> Style {
        Style::default().fg(DOS_WHITE).bg(DOS_DARK_GRAY)
    }

    // ── Error ───────────────────────────────────────────────────────

    pub fn error() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_LIGHT_RED)
            .add_modifier(Modifier::BOLD)
    }

    // ── Helper: pick the right style for a file entry ───────────────

    pub fn file_style(entry: &crate::types::FileEntry, is_cursor: bool, is_active: bool) -> Style {
        if is_cursor {
            return if is_active {
                Self::cursor_active()
            } else {
                Self::cursor_inactive()
            };
        }
        if entry.selected {
            return Self::file_selected();
        }
        if entry.is_dir {
            return Self::file_dir();
        }
        if entry.is_symlink {
            return Self::file_symlink();
        }
        if entry.is_hidden {
            return Self::file_hidden();
        }
        if entry.is_executable {
            return Self::file_executable();
        }

        // Highlight groups: color files by extension
        let ext = entry.extension().to_ascii_lowercase();
        if let Some(color) = Self::highlight_group_color(&ext) {
            return Style::default().fg(color).bg(DOS_DARK_GRAY);
        }

        Self::file_normal()
    }

    /// Get color for a file extension based on highlight groups (like DN's HighlightGroups)
    pub fn highlight_group_color(ext: &str) -> Option<Color> {
        match ext {
            // Archives (light red)
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "arj"
            | "tgz" | "zst" | "lz" | "lzma" | "cab" | "iso" | "dmg" => {
                Some(DOS_LIGHT_RED)
            }
            // Source code (light cyan)
            "rs" | "c" | "cpp" | "h" | "hpp" | "py" | "js" | "ts" | "java"
            | "go" | "rb" | "swift" | "kt" | "scala" | "cs" | "pas" | "asm"
            | "inc" | "lua" | "zig" | "nim" | "v" | "d" => {
                Some(DOS_LIGHT_CYAN)
            }
            // Markup / config (light green)
            "html" | "htm" | "xml" | "json" | "yaml" | "yml" | "toml"
            | "ini" | "cfg" | "conf" | "css" | "scss" | "less" => {
                Some(DOS_LIGHT_GREEN)
            }
            // Documents (light magenta)
            "txt" | "md" | "rst" | "doc" | "docx" | "pdf" | "rtf"
            | "odt" | "tex" | "log" => {
                Some(DOS_LIGHT_MAGENTA)
            }
            // Images (yellow)
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico"
            | "webp" | "tiff" | "tif" | "psd" | "ai" => {
                Some(DOS_YELLOW)
            }
            // Media (light blue)
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "wma"
            | "mp4" | "avi" | "mkv" | "mov" | "wmv" | "webm" | "flv" => {
                Some(DOS_LIGHT_BLUE)
            }
            // Scripts / batch (green)
            "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1"
            | "pl" | "awk" | "sed" => {
                Some(DOS_GREEN)
            }
            // Object / binary (cyan)
            "o" | "obj" | "a" | "lib" | "so" | "dylib" | "dll" | "exe"
            | "class" | "pyc" | "pyo" | "wasm" => {
                Some(DOS_CYAN)
            }
            _ => None,
        }
    }
}
