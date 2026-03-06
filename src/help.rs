/// Built-in help system (inspired by DN's HELPKERN.PAS / HELPFILE.PAS)
/// Provides a scrollable help viewer with multiple topics and cross-references.

#[derive(Debug)]
pub struct HelpViewer {
    pub topics: Vec<HelpTopic>,
    pub current_topic: usize,
    pub scroll: usize,
    pub visible_height: usize,
    pub history: Vec<usize>, // navigation history for "Back"
}

#[derive(Debug, Clone)]
pub struct HelpTopic {
    pub title: String,
    pub content: Vec<String>,   // lines of text
    pub links: Vec<HelpLink>,   // clickable cross-references
}

#[derive(Debug, Clone)]
pub struct HelpLink {
    pub label: String,
    pub target_topic: usize,
    pub line: usize,   // which line the link appears on
}

impl HelpViewer {
    pub fn new() -> Self {
        let topics = build_help_topics();
        HelpViewer {
            topics,
            current_topic: 0,
            scroll: 0,
            visible_height: 20,
            history: Vec::new(),
        }
    }

    pub fn current(&self) -> &HelpTopic {
        &self.topics[self.current_topic]
    }

    pub fn visible_lines(&self) -> &[String] {
        let content = &self.topics[self.current_topic].content;
        let start = self.scroll.min(content.len().saturating_sub(1));
        let end = (start + self.visible_height).min(content.len());
        &content[start..end]
    }

    pub fn scroll_down(&mut self) {
        let max = self.topics[self.current_topic].content.len()
            .saturating_sub(self.visible_height);
        if self.scroll < max {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn page_down(&mut self) {
        let max = self.topics[self.current_topic].content.len()
            .saturating_sub(self.visible_height);
        self.scroll = (self.scroll + self.visible_height).min(max);
    }

    pub fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(self.visible_height);
    }

    pub fn goto_topic(&mut self, index: usize) {
        if index < self.topics.len() {
            self.history.push(self.current_topic);
            self.current_topic = index;
            self.scroll = 0;
        }
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.current_topic = prev;
            self.scroll = 0;
        }
    }

    pub fn next_topic(&mut self) {
        if self.current_topic + 1 < self.topics.len() {
            self.history.push(self.current_topic);
            self.current_topic += 1;
            self.scroll = 0;
        }
    }

    pub fn prev_topic(&mut self) {
        if self.current_topic > 0 {
            self.history.push(self.current_topic);
            self.current_topic -= 1;
            self.scroll = 0;
        }
    }
}

fn build_help_topics() -> Vec<HelpTopic> {
    vec![
        // Topic 0: Index / Table of Contents
        HelpTopic {
            title: "RDN Help — Table of Contents".to_string(),
            content: vec![
                "═══════════════════════════════════════════════════".to_string(),
                "          RDN — Rust Dos Navigator                ".to_string(),
                "     A modern file manager inspired by DN 1.51    ".to_string(),
                "═══════════════════════════════════════════════════".to_string(),
                "".to_string(),
                " Use ← → to switch topics, ↑ ↓ PgUp PgDn to scroll.".to_string(),
                " Press Backspace to go back, Esc to close help.".to_string(),
                "".to_string(),
                " Topics:".to_string(),
                "  1. General Keys .......................... →".to_string(),
                "  2. Panel Navigation ...................... →".to_string(),
                "  3. File Operations ....................... →".to_string(),
                "  4. Built-in Viewer ....................... →".to_string(),
                "  5. Built-in Editor ....................... →".to_string(),
                "  6. Menu System ........................... →".to_string(),
                "  7. Archive Browser ....................... →".to_string(),
                "  8. Utilities ............................. →".to_string(),
                "  9. Command Line .......................... →".to_string(),
                " 10. Configuration ......................... →".to_string(),
                " 11. Quick Reference ....................... →".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 1: General Keys
        HelpTopic {
            title: "General Keys".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │              GENERAL KEYS                      │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " F1          — This help screen".to_string(),
                " F2          — User menu".to_string(),
                " F3          — View file".to_string(),
                " F4          — Edit file".to_string(),
                " F5          — Copy file(s)".to_string(),
                " F6          — Move/Rename file(s)".to_string(),
                " F7          — Make directory".to_string(),
                " F8          — Delete file(s)".to_string(),
                " F9          — Open dropdown menu".to_string(),
                " F10         — Quit (with confirmation)".to_string(),
                " F11         — Panel filter (left)".to_string(),
                " F12         — Panel filter (right)".to_string(),
                "".to_string(),
                " Tab         — Switch active panel".to_string(),
                " Enter       — Enter dir / Open file".to_string(),
                " Alt+X       — Quit immediately".to_string(),
                " Ctrl+O      — Show user screen".to_string(),
                " Ctrl+L      — Refresh display".to_string(),
                " Ctrl+R      — Re-read panels".to_string(),
                "".to_string(),
                " Alt+F1      — Change drive (left panel)".to_string(),
                " Alt+F2      — Change drive (right panel)".to_string(),
                " Shift+F4    — Edit new file".to_string(),
                " Shift+F6    — Quick rename".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 2: Panel Navigation
        HelpTopic {
            title: "Panel Navigation".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            PANEL NAVIGATION                    │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " ↑ / ↓       — Move cursor up/down".to_string(),
                " PgUp / PgDn — Page up/down".to_string(),
                " Home        — Jump to first file".to_string(),
                " End         — Jump to last file".to_string(),
                " Enter       — Enter directory / Open file".to_string(),
                " Backspace   — Go to parent directory".to_string(),
                "".to_string(),
                " Alt+F7      — Find file".to_string(),
                " Alt+F10     — Directory tree".to_string(),
                " Alt+F12     — Directory history".to_string(),
                "".to_string(),
                " Ctrl+\\      — Go to root directory (/)".to_string(),
                " Ctrl+F5     — Count directory sizes".to_string(),
                " Ctrl+B      — Directory branch (flat listing)".to_string(),
                " Ctrl+P      — Quick view toggle".to_string(),
                " Ctrl+H      — Toggle hidden files".to_string(),
                "".to_string(),
                " Alt+1..9    — Quick directory bookmarks".to_string(),
                " Ctrl+1..9   — Set directory bookmark".to_string(),
                "".to_string(),
                " Selection:".to_string(),
                " Insert      — Toggle selection on current file".to_string(),
                " Gray +      — Select files by pattern".to_string(),
                " Gray -      — Unselect files by pattern".to_string(),
                " Gray *      — Invert selection".to_string(),
                "".to_string(),
                " Quick Search:".to_string(),
                " Alt+letter  — Quick search for file name".to_string(),
                "".to_string(),
                " Panel Mode:".to_string(),
                " Ctrl+1      — Brief panel mode".to_string(),
                " Ctrl+2      — Full panel mode".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 3: File Operations
        HelpTopic {
            title: "File Operations".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            FILE OPERATIONS                     │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " F5          — Copy selected files to other panel".to_string(),
                " F6          — Move/Rename selected files".to_string(),
                " F7          — Create new directory".to_string(),
                " F8          — Delete selected files".to_string(),
                " Shift+F6    — Quick rename (in-place)".to_string(),
                " Ctrl+T      — Touch file (set current time)".to_string(),
                " Ctrl+F      — Make file list".to_string(),
                "".to_string(),
                " File Attributes:".to_string(),
                "  Available via menu: Files → Attributes".to_string(),
                "  Toggle permission bits with Space".to_string(),
                "  Navigate bits with ↑  ↓".to_string(),
                "  Apply with Enter, cancel with Esc".to_string(),
                "".to_string(),
                " Split File:".to_string(),
                "  Ctrl+F3    — Split file into chunks".to_string(),
                "  Ctrl+F4    — Combine file chunks".to_string(),
                "".to_string(),
                " Compare Directories:".to_string(),
                "  Alt+C      — Compare left and right panels".to_string(),
                "  Files unique to one side are selected".to_string(),
                "  Files with different sizes are selected".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 4: Built-in Viewer
        HelpTopic {
            title: "Built-in Viewer".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │             BUILT-IN VIEWER                    │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " F3          — Open viewer for current file".to_string(),
                " Esc / F10   — Close viewer".to_string(),
                "".to_string(),
                " Navigation:".to_string(),
                " ↑ / ↓       — Scroll up/down one line".to_string(),
                " PgUp / PgDn — Scroll one page".to_string(),
                " Home        — Jump to beginning of file".to_string(),
                " End         — Jump to end of file".to_string(),
                "".to_string(),
                " Modes:".to_string(),
                " F4          — Toggle hex/text mode".to_string(),
                " F2          — Toggle word wrap".to_string(),
                "".to_string(),
                " Search:".to_string(),
                " F7 / Ctrl+F — Search for text".to_string(),
                " Ctrl+N      — Find next match".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 5: Built-in Editor
        HelpTopic {
            title: "Built-in Editor".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │             BUILT-IN EDITOR                    │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " F4          — Open editor for current file".to_string(),
                " Shift+F4    — Open editor with new empty file".to_string(),
                " Esc / F10   — Close editor (prompt save if modified)".to_string(),
                " F2          — Save file".to_string(),
                " Shift+F2    — Save as...".to_string(),
                "".to_string(),
                " Navigation:".to_string(),
                " ↑ ↓ ← →     — Cursor movement".to_string(),
                " Home / End  — Start/end of line".to_string(),
                " PgUp / PgDn — Page up/down".to_string(),
                " Ctrl+Home   — Start of file".to_string(),
                " Ctrl+End    — End of file".to_string(),
                " Ctrl+G      — Go to line number".to_string(),
                "".to_string(),
                " Editing:".to_string(),
                " Insert      — Toggle insert/overwrite mode".to_string(),
                " Delete      — Delete character under cursor".to_string(),
                " Backspace   — Delete character before cursor".to_string(),
                " Enter       — Insert new line (auto-indent)".to_string(),
                " Tab         — Insert tab/spaces".to_string(),
                "".to_string(),
                " Selection & Clipboard:".to_string(),
                " Shift+arrows — Select text".to_string(),
                " Ctrl+C      — Copy selection".to_string(),
                " Ctrl+X      — Cut selection".to_string(),
                " Ctrl+V      — Paste from clipboard".to_string(),
                " Ctrl+A      — Select all".to_string(),
                "".to_string(),
                " Search & Replace:".to_string(),
                " Ctrl+F / F7 — Search".to_string(),
                " Ctrl+H      — Search and replace".to_string(),
                " Ctrl+N / F3 — Find next".to_string(),
                "".to_string(),
                " Undo/Redo:".to_string(),
                " Ctrl+Z      — Undo".to_string(),
                " Ctrl+Y      — Redo".to_string(),
                "".to_string(),
                " Macros:".to_string(),
                " Ctrl+M      — Start/stop macro recording".to_string(),
                " Ctrl+P      — Play recorded macro".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 6: Menu System
        HelpTopic {
            title: "Menu System".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │              MENU SYSTEM                       │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " F9          — Open/activate dropdown menu".to_string(),
                " ← / →       — Switch between menus".to_string(),
                " ↑ / ↓       — Navigate menu items".to_string(),
                " Enter       — Execute selected item".to_string(),
                " Esc / F10   — Close menu".to_string(),
                " First letter — Jump to menu item by hotkey".to_string(),
                "".to_string(),
                " Menu structure:".to_string(),
                "  Left    — Left panel mode and sorting".to_string(),
                "  Files   — File operations".to_string(),
                "  Commands — Directory/navigation commands".to_string(),
                "  Options — Settings, utilities, help".to_string(),
                "  Right   — Right panel mode and sorting".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 7: Archive Browser
        HelpTopic {
            title: "Archive Browser".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            ARCHIVE BROWSER                     │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " Supported formats: ZIP, TAR, TAR.GZ".to_string(),
                "".to_string(),
                " Enter an archive file (press Enter on it) to".to_string(),
                " browse its contents.".to_string(),
                "".to_string(),
                " Navigation:".to_string(),
                " ↑ / ↓       — Move cursor".to_string(),
                " PgUp / PgDn — Page up/down".to_string(),
                " Enter       — Enter sub-directory / View file".to_string(),
                " Backspace   — Go up one directory level".to_string(),
                " Esc         — Close archive browser".to_string(),
                " F3          — View selected file".to_string(),
                " F5          — Extract to disk".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 8: Utilities
        HelpTopic {
            title: "Utilities".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │              UTILITIES                         │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " Calculator (Ctrl+K):".to_string(),
                "  Type expression and press Enter to evaluate.".to_string(),
                "  Supports: + - * / ^ ( ) sqrt sin cos".to_string(),
                "  M+ to store result, MR to recall.".to_string(),
                "".to_string(),
                " ASCII Table (Alt+F9):".to_string(),
                "  Browse all 256 ASCII characters.".to_string(),
                "  Navigate with arrow keys.".to_string(),
                "".to_string(),
                " Disk Info (Alt+I):".to_string(),
                "  Shows disk usage for current directory.".to_string(),
                "  Total, used, and available space.".to_string(),
                "".to_string(),
                " Tetris (Ctrl+G):".to_string(),
                "  Classic Tetris game built-in.".to_string(),
                "  Use arrow keys to move, Up to rotate.".to_string(),
                "  Space to drop immediately.".to_string(),
                "".to_string(),
                " System Info (Ctrl+I):".to_string(),
                "  Shows system information: OS, CPU, memory.".to_string(),
                "".to_string(),
                " Environment Variables (Ctrl+E):".to_string(),
                "  Browse all environment variables.".to_string(),
                "  Navigate with arrow keys.".to_string(),
                "".to_string(),
                " Screen Saver:".to_string(),
                "  Activates after inactivity. Press any key.".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 9: Command Line
        HelpTopic {
            title: "Command Line".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            COMMAND LINE                        │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " The command line is at the bottom of the screen.".to_string(),
                " Start typing to enter command line mode.".to_string(),
                "".to_string(),
                " Enter       — Execute the command".to_string(),
                " Esc         — Cancel and return to panels".to_string(),
                "".to_string(),
                " Commands are executed in the current panel's".to_string(),
                " directory using the system shell (sh -c).".to_string(),
                "".to_string(),
                " After execution, the output is displayed and".to_string(),
                " you're prompted to press any key to return.".to_string(),
                "".to_string(),
                " Ctrl+O      — Toggle user screen (show last".to_string(),
                "                command output without leaving)".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 10: Configuration
        HelpTopic {
            title: "Configuration".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            CONFIGURATION                       │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " Desktop Save/Load:".to_string(),
                "  Options → Save desktop   — saves panel state".to_string(),
                "  Options → Load desktop   — restores panel state".to_string(),
                "  Config file: ~/.config/rdn/config.toml".to_string(),
                "".to_string(),
                " Saved settings:".to_string(),
                "  - Left and right panel paths".to_string(),
                "  - Sort modes for each panel".to_string(),
                "  - Panel display modes (Brief/Full)".to_string(),
                "  - Show hidden files".to_string(),
                "  - Editor settings (tab size, etc.)".to_string(),
                "".to_string(),
                " User Menu:".to_string(),
                "  Create ~/.config/rdn/menu.txt with entries:".to_string(),
                "  Format: label=command".to_string(),
                "  Example:".to_string(),
                "    Git Status=git status".to_string(),
                "    Git Log=git log --oneline -20".to_string(),
                "    Disk Usage=du -sh *".to_string(),
                "".to_string(),
                " File Associations (~/.config/rdn/assoc.txt):".to_string(),
                "  Format: .ext=command %f".to_string(),
                "  %f = current file, %d = current directory".to_string(),
                "  %n = filename without extension".to_string(),
                "  Example:".to_string(),
                "    .pdf=open %f".to_string(),
                "    .jpg=open %f".to_string(),
                "    .py=python3 %f".to_string(),
                "".to_string(),
                " Directory Bookmarks:".to_string(),
                "  Ctrl+1..9  — Set bookmark to current dir".to_string(),
                "  Alt+1..9   — Jump to bookmarked directory".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },

        // Topic 11: Quick Reference
        HelpTopic {
            title: "Quick Reference".to_string(),
            content: vec![
                " ┌────────────────────────────────────────────────┐".to_string(),
                " │            QUICK REFERENCE                     │".to_string(),
                " └────────────────────────────────────────────────┘".to_string(),
                "".to_string(),
                " ─── Function Keys ────────────────────────────── ".to_string(),
                " F1  Help        F2  UserMenu   F3  View".to_string(),
                " F4  Edit        F5  Copy       F6  Move".to_string(),
                " F7  MkDir       F8  Delete     F9  Menu".to_string(),
                " F10 Quit".to_string(),
                "".to_string(),
                " ─── Shift+Function Keys ──────────────────────── ".to_string(),
                " Shift+F4 Edit new    Shift+F6 Quick rename".to_string(),
                "".to_string(),
                " ─── Alt+Function Keys ────────────────────────── ".to_string(),
                " Alt+F1  Change drive left".to_string(),
                " Alt+F2  Change drive right".to_string(),
                " Alt+F7  Find file       Alt+F9  ASCII table".to_string(),
                " Alt+F10 Directory tree  Alt+F12 Dir history".to_string(),
                "".to_string(),
                " ─── Ctrl Keys ────────────────────────────────── ".to_string(),
                " Ctrl+B  Branch     Ctrl+C  Copy (editor)".to_string(),
                " Ctrl+E  Env vars   Ctrl+F  File list / Search".to_string(),
                " Ctrl+G  Tetris     Ctrl+H  Hidden files".to_string(),
                " Ctrl+I  Sys info   Ctrl+K  Calculator".to_string(),
                " Ctrl+L  Refresh    Ctrl+O  User screen".to_string(),
                " Ctrl+P  Quick view Ctrl+R  Re-read panels".to_string(),
                " Ctrl+S  Sync panels Ctrl+T Touch file".to_string(),
                " Ctrl+U  Swap panels".to_string(),
                "".to_string(),
                " ─── Alt Keys ─────────────────────────────────── ".to_string(),
                " Alt+C   Compare dirs   Alt+I  Disk info".to_string(),
                " Alt+X   Quit           Alt+1..9 Bookmarks".to_string(),
                "".to_string(),
                " ─── Selection ────────────────────────────────── ".to_string(),
                " Insert  Toggle select".to_string(),
                " +       Select pattern   -  Unselect pattern".to_string(),
                " *       Invert selection".to_string(),
                "".to_string(),
                " ─── Split/Combine ────────────────────────────── ".to_string(),
                " Ctrl+F3 Split file   Ctrl+F4 Combine parts".to_string(),
                "".to_string(),
            ],
            links: vec![],
        },
    ]
}
