#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct Vis {
    pub(crate) bands: [f32; 4],
    pub(crate) pulse: f32,
    pub(crate) pads: [f32; 4],
    pub(crate) seq: usize,
}

thread_local! {
    pub(crate) static VIS: std::cell::Cell<Vis> = std::cell::Cell::new(Vis { bands: [0.0; 4], pulse: 0.0, pads: [0.0; 4], seq: 0 });
}

pub(crate) fn vis() -> Vis {
    VIS.with(|v| v.get())
}

pub(crate) const TV_CHANNELS: [&str; 5] = ["spectrum", "tunnel", "plasma", "rain", "scope"];

pub(crate) fn tv_pixel(channel: u8, x: usize, y: usize, w: usize, h: usize, time: f32, v: &Vis) -> Option<Color> {
    let fx = x as f32 / w as f32;
    let fy = y as f32 / h as f32;
    let bass = v.bands[0];
    let mids = (v.bands[1] + v.bands[2]) * 0.5;
    let highs = v.bands[3];
    let energy = (bass + mids + highs) / 3.0;
    match channel {
        1 => {
            let band = (fx * 4.0) as usize % 4;
            let local = (fx * 4.0).fract();
            if local < 0.15 || local > 0.85 {
                return None;
            }
            let level = (v.bands[band] * 0.85 + v.pulse * 0.15).min(1.0);
            let from_mid = (fy - 0.5).abs() * 2.0;
            if from_mid < level {
                Some(hsv(0.55 - band as f32 * 0.14 + time * 0.03, 0.8, 0.5 + 0.5 * (1.0 - from_mid)))
            } else {
                None
            }
        }
        2 => {
            let dx = (fx - 0.5) * 1.8;
            let dy = fy - 0.5;
            let r = dx.abs().max(dy.abs());
            let wave = (r * 9.0 - time * (2.0 + bass * 6.0)).sin();
            if wave > 0.2 - energy * 0.8 {
                Some(hsv(r * 0.8 + time * 0.1 + bass * 0.3, 0.75, 0.35 + 0.65 * (wave * 0.5 + 0.5)))
            } else {
                Some(Color::new(0.03, 0.02, 0.06, 1.0))
            }
        }
        3 => {
            let a = (fx * 7.0 + time * 1.3).sin() + (fy * 6.0 - time * 1.1).sin() + ((fx + fy) * 5.0 + time * (0.8 + bass * 3.0)).sin();
            Some(hsv(a * 0.12 + time * 0.05 + mids * 0.4, 0.7, 0.35 + 0.4 * energy + 0.25 * v.pulse))
        }
        4 => {
            let speed = 0.6 + highs * 3.0 + v.pulse;
            let col_seed = ((x as u32).wrapping_mul(2654435761) >> 16) as f32 / 65535.0;
            let head = (time * speed * (0.6 + col_seed) + col_seed * 10.0).fract();
            let d = (fy - head).rem_euclid(1.0);
            if d < 0.25 {
                let glow = 1.0 - d / 0.25;
                Some(hsv(0.33 + mids * 0.3, 0.7, glow * (0.4 + 0.6 * energy.max(0.3))))
            } else {
                None
            }
        }
        5 => {
            let dx = (fx - 0.5) * 1.8;
            let dy = fy - 0.5;
            let ang = dy.atan2(dx);
            let ring = 0.22 + 0.12 * bass + 0.05 * (ang * 6.0 + time * 3.0).sin() * (0.3 + highs);
            let dist = (dx * dx + dy * dy).sqrt();
            if (dist - ring).abs() < 0.035 + mids * 0.04 {
                Some(hsv(ang / 6.283 + time * 0.2, 0.6, 1.0))
            } else if (dist - ring * 0.55).abs() < 0.02 {
                Some(hsv(0.9 + time * 0.1, 0.6, 0.5 + v.pulse * 0.5))
            } else {
                None
            }
        }
        _ => None,
    }
}
