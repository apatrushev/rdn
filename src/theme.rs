use ratatui::style::{Color, Modifier, Style};

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
        Style::default().fg(DOS_WHITE).bg(DOS_DARK_GRAY)
    }

    pub fn panel_border_inactive() -> Style {
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_DARK_GRAY)
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
        // $87: light gray on dark gray
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_DARK_GRAY)
    }

    pub fn file_dir() -> Style {
        // Directories: white on dark gray ($8F) – highlight type 1
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_DARK_GRAY)
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
        // CColor[87]=$8E: yellow on dark gray
        Style::default()
            .fg(DOS_YELLOW)
            .bg(DOS_DARK_GRAY)
            .add_modifier(Modifier::BOLD)
    }

    // ── Cursor bar ──────────────────────────────────────────────────
    // CColor[88]=$30: black on cyan  (normal cursor)
    // CColor[89]=$3E: yellow on cyan (selected cursor)

    pub fn cursor_active() -> Style {
        Style::default()
            .fg(DOS_BLACK)
            .bg(DOS_CYAN)
    }

    pub fn cursor_inactive() -> Style {
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_DARK_GRAY)
    }

    // ── Panel background (fills gaps) ───────────────────────────────

    pub fn panel_bg() -> Style {
        Style::default().bg(DOS_DARK_GRAY)
    }

    // ── Info line (bottom of panel) ─────────────────────────────────
    // CColor[86]=$8F separator / info → white on dark gray

    pub fn info_line() -> Style {
        Style::default().fg(DOS_LIGHT_CYAN).bg(DOS_DARK_GRAY)
    }

    // ── Status bar ──────────────────────────────────────────────────

    pub fn status_bar() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_CYAN)
    }

    // ── Function key bar (F1-F10 at the bottom) ─────────────────────

    pub fn fn_key_number() -> Style {
        Style::default().fg(DOS_WHITE).bg(DOS_BLACK)
    }

    pub fn fn_key_label() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_CYAN)
    }

    // ── Menu bar (top line) ─────────────────────────────────────────

    pub fn menu_bar() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_CYAN)
    }

    pub fn menu_bar_highlight() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_BLACK)
            .add_modifier(Modifier::BOLD)
    }

    // ── Dialogs ─────────────────────────────────────────────────────

    pub fn dialog_border() -> Style {
        Style::default().fg(DOS_WHITE).bg(DOS_DARK_GRAY)
    }

    pub fn dialog_text() -> Style {
        Style::default().fg(DOS_WHITE).bg(DOS_DARK_GRAY)
    }

    pub fn dialog_input() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_LIGHT_CYAN)
    }

    pub fn dialog_button() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_GREEN)
            .add_modifier(Modifier::BOLD)
    }

    pub fn dialog_button_focused() -> Style {
        Style::default()
            .fg(DOS_WHITE)
            .bg(DOS_LIGHT_RED)
            .add_modifier(Modifier::BOLD)
    }

    // ── Command line ────────────────────────────────────────────────

    pub fn command_line() -> Style {
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_BLACK)
    }

    // ── Quick search overlay ────────────────────────────────────────

    pub fn quick_search() -> Style {
        Style::default().fg(DOS_BLACK).bg(DOS_YELLOW)
    }

    // ── Viewer ──────────────────────────────────────────────────────
    // CColor[92]=$87 viewer normal → light gray on dark gray
    // CColor[93]=$70 viewer highlight → black on light gray

    pub fn viewer_text() -> Style {
        Style::default().fg(DOS_LIGHT_GRAY).bg(DOS_DARK_GRAY)
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
        if entry.is_executable {
            return Self::file_executable();
        }
        Self::file_normal()
    }
}
