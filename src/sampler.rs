//! The step sampler and recording clean-up.

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
    pub(crate) recording: Option<(usize, f32)>,
    pub(crate) step: usize,
}

pub(crate) const SAMPLER_ROWS: [&str; 4] = ["kick", "clap", "hat", "voice"];

pub(crate) fn tidy_recording(raw: &[f32], rate: f32) -> Option<Vec<f32>> {
    let start = raw.iter().position(|s| s.abs() > 0.03)?;
    let ratio = rate / SAMPLE_RATE as f32;
    let len = ((raw.len() - start) as f32 / ratio).min(SAMPLE_RATE as f32 * 1.2) as usize;
    let mut out: Vec<f32> = (0..len)
        .map(|i| {
            let p = start as f32 + i as f32 * ratio;
            let k = p as usize;
            let f = p - k as f32;
            let a = raw.get(k).copied().unwrap_or(0.0);
            let b = raw.get(k + 1).copied().unwrap_or(a);
            a * (1.0 - f) + b * f
        })
        .collect();
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak < 0.03 {
        return None;
    }
    let fade = (SAMPLE_RATE as f32 * 0.04) as usize;
    let n = out.len();
    for (i, s) in out.iter_mut().enumerate() {
        *s *= 0.8 / peak;
        if i + fade > n {
            *s *= (n - i) as f32 / fade as f32;
        }
    }
    Some(out)
}
