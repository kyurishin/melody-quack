#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const DEFAULT_BPM: f32 = 121.0;

pub(crate) const LOOP_STEPS: usize = 32;

pub(crate) const KICK_PATTERN: &str = "9...9...9...9...";

pub(crate) const CLAP_PATTERN: &str = "....8.......8...";

pub(crate) const HAT_PATTERN: &str = "2.7.2.7.2.7.2.75";

pub(crate) const NOTES: [u8; 6] = [74, 76, 78, 81, 83, 84];

pub(crate) const NOTE_NAMES: [&str; 6] = ["D", "E", "F#", "A", "B", "C"];

pub(crate) const NOTE_KEYS: [KeyCode; 6] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
];

pub(crate) fn note_color(i: usize) -> Color {
    match i {
        0 => Color::from_rgba(255, 128, 98, 255),
        1 => Color::from_rgba(255, 168, 204, 255),
        2 => Color::from_rgba(116, 172, 255, 255),
        3 => Color::from_rgba(140, 222, 168, 255),
        4 => Color::from_rgba(186, 230, 255, 255),
        _ => Color::from_rgba(255, 214, 110, 255),
    }
}

pub(crate) const LIVE_WINDOW: f32 = 0.03;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Quantise {
    Off,
    Soft,
    Tight,
}

impl Quantise {
    pub(crate) fn next(self) -> Quantise {
        match self {
            Quantise::Off => Quantise::Soft,
            Quantise::Soft => Quantise::Tight,
            Quantise::Tight => Quantise::Off,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Quantise::Off => "off",
            Quantise::Soft => "soft",
            Quantise::Tight => "tight",
        }
    }
}

pub(crate) fn note_owner(id: usize) -> usize {
    match id {
        6..=11 | 22..=27 | 28..=31 => 1,
        _ => 0,
    }
}

pub(crate) fn note_slot(id: usize) -> usize {
    match id {
        0..=11 => id % 6,
        12..=15 => id - 12,
        28..=31 => id - 28,
        _ => (id - 16) % 6,
    }
}

pub(crate) fn wrap_steps(p: f32) -> f32 {
    p.rem_euclid(LOOP_STEPS as f32)
}

pub(crate) struct Looper {
    pub(crate) notes: Vec<(f32, usize)>,
    pub(crate) skip: Vec<(f32, usize)>,
    pub(crate) pending: Vec<(f32, usize)>,
    pub(crate) recording: bool,
    pub(crate) beat: bool,
    pub(crate) bpm: f32,
    pub(crate) position: f32,
    pub(crate) last_step: Option<usize>,
    pub(crate) quantise: Quantise,
}

impl Looper {
    pub(crate) fn new() -> Looper {
        Looper {
            notes: Vec::new(),
            skip: Vec::new(),
            pending: Vec::new(),
            recording: false,
            beat: false,
            bpm: DEFAULT_BPM,
            position: 0.0,
            last_step: None,
            quantise: Quantise::Soft,
        }
    }

    pub(crate) fn step_seconds(&self) -> f32 {
        60.0 / self.bpm / 4.0
    }

    pub(crate) fn current_step(&self) -> usize {
        self.position as usize % LOOP_STEPS
    }

    pub(crate) fn sync_to_beat(&mut self, since_beat: f32) {
        let target = (since_beat * self.bpm / 60.0 * 4.0) % 4.0;
        let mut diff = target - self.position % 4.0;
        if diff > 2.0 {
            diff -= 4.0;
        }
        if diff < -2.0 {
            diff += 4.0;
        }
        self.position = (self.position + diff * 0.5).max(0.0);
    }

    pub(crate) fn press(&mut self, note: usize) -> bool {
        let late_by = self.position.fract() * self.step_seconds();
        let wait = self.quantise == Quantise::Tight && late_by > LIVE_WINDOW;
        let grid = if wait { self.position.floor() + 1.0 } else { self.position.floor() };
        if wait {
            self.pending.push((grid, note));
        }
        if self.recording {
            let raw = self.position;
            let at = match self.quantise {
                Quantise::Off => raw,
                Quantise::Soft => raw + (raw.round() - raw) * 0.5,
                Quantise::Tight => grid,
            };
            self.store(at, note);
        }
        !wait
    }

    pub(crate) fn store(&mut self, at_abs: f32, note: usize) {
        let at = wrap_steps(at_abs);
        let close = |p: f32| {
            let d = (p - at).rem_euclid(LOOP_STEPS as f32);
            d.min(LOOP_STEPS as f32 - d) < 0.25
        };
        if self.notes.iter().any(|&(p, n)| n == note && close(p)) {
            return;
        }
        self.notes.push((at, note));
        if at_abs > self.position {
            self.skip.push((at, note));
        }
    }

    pub(crate) fn tick(&mut self, dt: f32) -> Vec<usize> {
        let before = self.position;
        self.position += dt * self.bpm / 60.0 * 4.0;
        if self.position >= LOOP_STEPS as f32 * 64.0 {
            self.position -= LOOP_STEPS as f32 * 64.0;
            for p in self.pending.iter_mut() {
                p.0 -= LOOP_STEPS as f32 * 64.0;
            }
        }
        let after = self.position;

        let mut due = Vec::new();
        self.pending.retain(|&(at, note)| {
            if after >= at {
                due.push(note);
                false
            } else {
                true
            }
        });

        let span = after - before;
        if span > 0.0 && span < 4.0 {
            let from = wrap_steps(before);
            for k in 0..self.notes.len() {
                let (at, note) = self.notes[k];
                let d = (at - from).rem_euclid(LOOP_STEPS as f32);
                if d > 0.0 && d <= span {
                    if let Some(i) = self.skip.iter().position(|e| *e == (at, note)) {
                        self.skip.remove(i);
                    } else {
                        due.push(note);
                    }
                }
            }
        }
        due
    }

    pub(crate) fn beat_pulse(&self) -> f32 {
        (1.0 - (self.position / 4.0).fract()).powi(3)
    }
}

pub(crate) fn next_beat(p: f32) -> f32 {
    (p / 4.0).floor() * 4.0 + 4.0
}
