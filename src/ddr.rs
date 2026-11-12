//! The dance arcade.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) struct Arrow {
    pub(crate) lane: usize,
    pub(crate) at: f32,
    pub(crate) slot: i64,
    pub(crate) hit: [bool; 2],
    pub(crate) missed: [bool; 2],
}

pub(crate) struct Ddr {
    pub(crate) on: [bool; 2],
    pub(crate) arrows: Vec<Arrow>,
    pub(crate) score: [u32; 2],
    pub(crate) combo: [u32; 2],
    pub(crate) judge: [Option<(&'static str, Color, f32)>; 2],
    pub(crate) flash: [[f32; 4]; 2],
    pub(crate) last_mic: f32,
}

impl Ddr {
    pub(crate) fn add(&mut self, lane: usize, at: f32, slot_len: f32) {
        let slot = (at / slot_len).round() as i64;
        if self.arrows.iter().any(|a| a.slot == slot && a.lane == lane) || self.arrows.iter().filter(|a| a.slot == slot).count() >= 2 {
            return;
        }
        self.arrows.push(Arrow { lane, at, slot, hit: [false; 2], missed: [false; 2] });
    }
}

pub(crate) const DDR_LANE_OF_BAND: [usize; 4] = [0, 1, 3, 2];

pub(crate) fn ddr_arrow_pixels(lane: usize) -> Vec<(usize, usize, Color)> {
    let rows = ["....x....", "...xxx...", "..xxxxx..", ".xxxxxxx.", "xxxxxxxxx", "...xxx...", "...xxx...", "...xxx...", "...xxx..."];
    let mut out = Vec::new();
    let c = note_color(lane);
    let edge = Color::new(c.r * 0.55, c.g * 0.55, c.b * 0.55, 1.0);
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch != 'x' {
                continue;
            }
            let filled = |xx: i32, yy: i32| xx >= 0 && yy >= 0 && (yy as usize) < 9 && (xx as usize) < 9 && rows[yy as usize].as_bytes()[xx as usize] == b'x';
            let border = !(filled(x as i32 - 1, y as i32) && filled(x as i32 + 1, y as i32) && filled(x as i32, y as i32 - 1) && filled(x as i32, y as i32 + 1));
            let (rx, ry) = match lane {
                0 => (y, 8 - x),
                1 => (8 - x, 8 - y),
                3 => (8 - y, x),
                _ => (x, y),
            };
            out.push((rx, ry, if border { edge } else { c }));
        }
    }
    out
}
