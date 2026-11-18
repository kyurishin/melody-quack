#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) struct Sampler {
    pub(crate) open: bool,
    pub(crate) owner: usize,
    pub(crate) grid: [[bool; 16]; 4],
    pub(crate) cursor: (usize, usize),
    pub(crate) pads: [Option<audio::Sound>; 4],
    pub(crate) clear_armed: f64,
    pub(crate) step: usize,
}

pub(crate) const SAMPLER_ROWS: [&str; 4] = ["kick", "clap", "hat", "voice"];
