//! Drum and bass synthesis and the beat styles.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn noise(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed as f32 / u32::MAX as f32) * 2.0 - 1.0
}

pub(crate) fn render_kick() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    (0..(0.45 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let phase = 2.0 * PI * (48.0 * t + 70.0 * (1.0 - (-t * 25.0).exp()) / 25.0);
            (phase.sin() * (-t * 6.5).exp() * 1.4).tanh() * 0.9
        })
        .collect()
}

pub(crate) fn render_clap() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut seed = 0x1357_9bdf;
    let mut lp = 0.0;
    let mut prev = 0.0;
    (0..(0.4 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let mut env = (-t * 10.0).exp() * 0.5;
            for b in [0.0, 0.012, 0.024] {
                if t >= b {
                    env += (-(t - b) * 160.0).exp();
                }
            }
            lp += (noise(&mut seed) - lp) * 0.4;
            let band = lp - prev * 0.6;
            prev = lp;
            band * env * 0.9
        })
        .collect()
}

pub(crate) fn render_hat() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut seed = 0x2468_ace1;
    let mut prev = 0.0;
    (0..(0.12 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let n = noise(&mut seed);
            let hp = n - prev;
            prev = n;
            hp * smoothstep(t / 0.003) * (-t * 35.0).exp() * 0.5
        })
        .collect()
}

pub(crate) struct BeatStyle {
    pub(crate) name: &'static str,
    pub(crate) kick: &'static str,
    pub(crate) clap: &'static str,
    pub(crate) hat: &'static str,
    pub(crate) open: &'static str,
}

pub(crate) const BEAT_STYLES: [BeatStyle; 5] = [
    BeatStyle { name: "techno", kick: "9...9...9...9...", clap: "....5.......5...", hat: "4343434343434343", open: "..8...8...8...8." },
    BeatStyle { name: "house", kick: "9...9...9...9...", clap: "....9.......9...", hat: "3.3.3.3.3.3.3.3.", open: "..7...7...7...7." },
    BeatStyle { name: "breaks", kick: "9.....9...9.....", clap: "....9.......9..6", hat: "6.6.6.6.6.6.6.6.", open: "..............7." },
    BeatStyle { name: "classic", kick: KICK_PATTERN, clap: CLAP_PATTERN, hat: HAT_PATTERN, open: "................" },
    BeatStyle { name: "off", kick: "................", clap: "................", hat: "................", open: "................" },
];

pub(crate) const DJ_PADS: [[&str; 6]; 2] = [["kick", "clap", "hi hat", "open", "low tom", "rim"], ["bass", "chord", "vox", "shaker", "bell", "snare"]];

pub(crate) fn render_open_hat() -> Vec<f32> {
    noise_burst(0.5, 6.5, 0.95, 0x51ed, 0.2)
}

pub(crate) fn render_tom() -> Vec<f32> {
    add_into(ring_tone(96.0, &[(1.0, 1.0, 7.0), (1.6, 0.35, 13.0), (2.3, 0.12, 20.0)], 0.55, 0.002, 0.75), noise_burst(0.02, 120.0, 0.5, 0x7011, 0.08))
}

pub(crate) fn render_rim() -> Vec<f32> {
    add_into(noise_burst(0.03, 120.0, 0.9, 0x77aa, 0.32), ring_tone(1650.0, &[(1.0, 0.6, 70.0), (2.7, 0.2, 90.0)], 0.06, 0.001, 0.4))
}

pub(crate) fn render_bass() -> Vec<f32> {
    ring_tone(55.0, &[(1.0, 1.0, 3.5), (2.0, 0.45, 5.5), (3.0, 0.18, 8.0)], 0.55, 0.004, 0.85)
}

pub(crate) fn velocity(pattern: &str, step: usize) -> Option<f32> {
    (pattern.as_bytes()[step % 16] as char).to_digit(10).map(|d| d as f32 / 9.0)
}
