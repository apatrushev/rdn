/// Screen saver (inspired by DN's IDLERS.PAS — Starfield / StarSky)
/// Activates after inactivity, shows animated starfield.

use rand::Rng;

#[derive(Debug, Clone)]
pub struct Star {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub char: char,
}

#[derive(Debug)]
pub struct ScreenSaver {
    pub stars: Vec<Star>,
    pub width: u16,
    pub height: u16,
    pub active: bool,
    pub idle_ticks: u32,
    pub idle_threshold: u32, // ticks before activating (roughly 5 min at 10fps)
}

impl ScreenSaver {
    pub fn new() -> Self {
        ScreenSaver {
            stars: Vec::new(),
            width: 80,
            height: 25,
            active: false,
            idle_ticks: 0,
            idle_threshold: 3000, // ~5 minutes at ~10 checks/sec
        }
    }

    pub fn reset_idle(&mut self) {
        self.idle_ticks = 0;
        self.active = false;
        self.stars.clear();
    }

    pub fn tick(&mut self) {
        if self.active {
            self.update_stars();
        } else {
            self.idle_ticks += 1;
            if self.idle_ticks >= self.idle_threshold {
                self.activate();
            }
        }
    }

    fn activate(&mut self) {
        self.active = true;
        self.stars.clear();
        let mut rng = rand::rng();
        for _ in 0..80 {
            self.stars.push(Star {
                x: rng.random_range(0.0..self.width as f64),
                y: rng.random_range(0.0..self.height as f64),
                speed: rng.random_range(0.1..1.0),
                char: match rng.random_range(0..3) {
                    0 => '.',
                    1 => '*',
                    _ => '·',
                },
            });
        }
    }

    fn update_stars(&mut self) {
        let mut rng = rand::rng();
        for star in &mut self.stars {
            star.x += star.speed;
            if star.x >= self.width as f64 {
                star.x = 0.0;
                star.y = rng.random_range(0.0..self.height as f64);
                star.speed = rng.random_range(0.1..1.0);
            }
        }
    }

    /// Get stars as (x, y, char) tuples for rendering
    pub fn render_stars(&self) -> Vec<(u16, u16, char, u8)> {
        self.stars.iter().map(|s| {
            let brightness = if s.speed > 0.7 {
                2 // bright
            } else if s.speed > 0.3 {
                1 // medium
            } else {
                0 // dim
            };
            (s.x as u16, s.y as u16, s.char, brightness)
        }).collect()
    }
}
