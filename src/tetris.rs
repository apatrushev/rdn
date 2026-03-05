use rand::Rng;
use std::time::{Duration, Instant};

/// Tetris board dimensions
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

/// Tetromino types (7 classic pieces)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Piece {
    /// Get the shape as a 4x4 grid of booleans for a given rotation
    pub fn shape(&self, rotation: u8) -> [[bool; 4]; 4] {
        let base = match self {
            Piece::I => [
                [false, false, false, false],
                [true, true, true, true],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::O => [
                [false, true, true, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::T => [
                [false, true, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::S => [
                [false, true, true, false],
                [true, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::Z => [
                [true, true, false, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::J => [
                [true, false, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Piece::L => [
                [false, false, true, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
        };

        let mut result = base;
        for _ in 0..(rotation % 4) {
            result = rotate_cw(result);
        }
        result
    }

    /// Color index for each piece type (1-7)
    pub fn color_index(&self) -> u8 {
        match self {
            Piece::I => 1,
            Piece::O => 2,
            Piece::T => 3,
            Piece::S => 4,
            Piece::Z => 5,
            Piece::J => 6,
            Piece::L => 7,
        }
    }

    fn random() -> Piece {
        let mut rng = rand::rng();
        match rng.random_range(0..7) {
            0 => Piece::I,
            1 => Piece::O,
            2 => Piece::T,
            3 => Piece::S,
            4 => Piece::Z,
            5 => Piece::J,
            _ => Piece::L,
        }
    }
}

/// Rotate a 4x4 grid clockwise
fn rotate_cw(grid: [[bool; 4]; 4]) -> [[bool; 4]; 4] {
    let mut result = [[false; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            result[c][3 - r] = grid[r][c];
        }
    }
    result
}

/// Active falling piece state
#[derive(Debug, Clone)]
pub struct FallingPiece {
    pub piece: Piece,
    pub rotation: u8,
    pub x: i32, // column (can be negative during wall kicks)
    pub y: i32, // row (0 = top)
}

impl FallingPiece {
    pub fn new(piece: Piece) -> Self {
        FallingPiece {
            piece,
            rotation: 0,
            x: (BOARD_WIDTH as i32 - 4) / 2,
            y: 0,
        }
    }

    pub fn shape(&self) -> [[bool; 4]; 4] {
        self.piece.shape(self.rotation)
    }

    /// Get all occupied cell positions (row, col) on the board
    pub fn cells(&self) -> Vec<(i32, i32)> {
        let shape = self.shape();
        let mut cells = Vec::new();
        for r in 0..4 {
            for c in 0..4 {
                if shape[r][c] {
                    cells.push((self.y + r as i32, self.x + c as i32));
                }
            }
        }
        cells
    }
}

/// Tetris game state
pub struct Tetris {
    /// Board cells: 0 = empty, 1-7 = piece color
    pub board: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current: FallingPiece,
    pub next: Piece,
    pub score: u32,
    pub lines_cleared: u32,
    pub level: u32,
    pub game_over: bool,
    pub paused: bool,
    pub last_tick: Instant,
    pub drop_interval: Duration,
}

impl Tetris {
    pub fn new() -> Self {
        let current_piece = Piece::random();
        let next_piece = Piece::random();
        Tetris {
            board: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            current: FallingPiece::new(current_piece),
            next: next_piece,
            score: 0,
            lines_cleared: 0,
            level: 1,
            game_over: false,
            paused: false,
            last_tick: Instant::now(),
            drop_interval: Duration::from_millis(800),
        }
    }

    /// Check if a piece fits at the given position
    fn fits(&self, piece: &Piece, rotation: u8, x: i32, y: i32) -> bool {
        let shape = piece.shape(rotation);
        for r in 0..4 {
            for c in 0..4 {
                if shape[r][c] {
                    let br = y + r as i32;
                    let bc = x + c as i32;
                    if bc < 0 || bc >= BOARD_WIDTH as i32 || br >= BOARD_HEIGHT as i32 {
                        return false;
                    }
                    if br >= 0 && self.board[br as usize][bc as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Lock the current piece into the board
    fn lock_piece(&mut self) {
        let color = self.current.piece.color_index();
        for (r, c) in self.current.cells() {
            if r >= 0 && r < BOARD_HEIGHT as i32 && c >= 0 && c < BOARD_WIDTH as i32 {
                self.board[r as usize][c as usize] = color;
            }
        }
    }

    /// Clear completed lines and update score
    fn clear_lines(&mut self) {
        let mut lines = 0u32;
        let mut row = BOARD_HEIGHT as i32 - 1;
        while row >= 0 {
            let full = self.board[row as usize].iter().all(|&c| c != 0);
            if full {
                // Shift everything above down
                for r in (1..=row as usize).rev() {
                    self.board[r] = self.board[r - 1];
                }
                self.board[0] = [0; BOARD_WIDTH];
                lines += 1;
                // Don't decrement row — check this row again
            } else {
                row -= 1;
            }
        }

        if lines > 0 {
            self.lines_cleared += lines;
            // Scoring: 100, 300, 500, 800 for 1-4 lines
            self.score += match lines {
                1 => 100 * self.level,
                2 => 300 * self.level,
                3 => 500 * self.level,
                4 => 800 * self.level,
                _ => 0,
            };

            // Level up every 10 lines
            let new_level = (self.lines_cleared / 10) + 1;
            if new_level > self.level {
                self.level = new_level;
                // Speed up: decrease interval by ~15% per level
                let ms = (800.0 * 0.85f64.powi((self.level - 1) as i32)) as u64;
                self.drop_interval = Duration::from_millis(ms.max(50));
            }
        }
    }

    /// Spawn next piece
    fn spawn_next(&mut self) {
        self.current = FallingPiece::new(self.next);
        self.next = Piece::random();

        // Check game over
        if !self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x,
            self.current.y,
        ) {
            self.game_over = true;
        }
    }

    /// Called on each tick (gravity)
    pub fn tick(&mut self) {
        if self.game_over || self.paused {
            return;
        }

        if self.last_tick.elapsed() >= self.drop_interval {
            self.last_tick = Instant::now();
            self.move_down();
        }
    }

    /// Move piece down; if can't, lock and spawn
    pub fn move_down(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        if self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x,
            self.current.y + 1,
        ) {
            self.current.y += 1;
            true
        } else {
            self.lock_piece();
            self.clear_lines();
            self.spawn_next();
            false
        }
    }

    /// Hard drop — instantly drop piece to bottom
    pub fn hard_drop(&mut self) {
        if self.game_over {
            return;
        }
        let mut rows = 0;
        while self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x,
            self.current.y + 1,
        ) {
            self.current.y += 1;
            rows += 1;
        }
        self.score += rows * 2; // bonus for hard drop
        self.lock_piece();
        self.clear_lines();
        self.spawn_next();
    }

    /// Move piece left
    pub fn move_left(&mut self) {
        if self.game_over {
            return;
        }
        if self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x - 1,
            self.current.y,
        ) {
            self.current.x -= 1;
        }
    }

    /// Move piece right
    pub fn move_right(&mut self) {
        if self.game_over {
            return;
        }
        if self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x + 1,
            self.current.y,
        ) {
            self.current.x += 1;
        }
    }

    /// Rotate piece clockwise
    pub fn rotate(&mut self) {
        if self.game_over {
            return;
        }
        let new_rot = (self.current.rotation + 1) % 4;
        // Try normal rotation
        if self.fits(
            &self.current.piece,
            new_rot,
            self.current.x,
            self.current.y,
        ) {
            self.current.rotation = new_rot;
            return;
        }
        // Wall kick: try shifting left/right
        for &dx in &[-1, 1, -2, 2] {
            if self.fits(
                &self.current.piece,
                new_rot,
                self.current.x + dx,
                self.current.y,
            ) {
                self.current.x += dx;
                self.current.rotation = new_rot;
                return;
            }
        }
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if !self.paused {
            self.last_tick = Instant::now();
        }
    }

    /// Restart the game
    pub fn restart(&mut self) {
        *self = Tetris::new();
    }

    /// Get the ghost piece Y position (where piece would land)
    pub fn ghost_y(&self) -> i32 {
        let mut gy = self.current.y;
        while self.fits(
            &self.current.piece,
            self.current.rotation,
            self.current.x,
            gy + 1,
        ) {
            gy += 1;
        }
        gy
    }
}
