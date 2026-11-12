//! The campfire, seats and cooking.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const CAMPFIRE: (i32, i32) = (8, 17);

pub(crate) const DOOR_OUT: Vec2 = Vec2::new(4.5, 15.55);

pub(crate) const CAT_SEAT: Vec2 = Vec2::new(7.3, 17.5);

pub(crate) const DOG_SEAT: Vec2 = Vec2::new(9.7, 17.5);

pub(crate) const STICK_DROP: Vec2 = Vec2::new(8.5, 16.3);

pub(crate) fn fire_center() -> Vec3 {
    vec3(CAMPFIRE.0 as f32 + 0.5, 0.5, CAMPFIRE.1 as f32 + 0.5)
}

pub(crate) struct Camp {
    pub(crate) fuel: f32,
    pub(crate) lit: bool,
    pub(crate) pile: u32,
}

pub(crate) fn draw_camp(b: &mut Batch, camp: &Camp, time: f32) {
    let c = fire_center();
    let stone = Color::from_rgba(140, 144, 158, 255);
    let log = Color::from_rgba(120, 82, 52, 255);
    let log_end = Color::from_rgba(200, 160, 110, 255);
    for k in 0..8 {
        let a = k as f32 / 8.0 * 2.0 * PI;
        b.cube(c + vec3(a.cos() * 0.36, 0.06, a.sin() * 0.36), vec3(0.16, 0.12, 0.16), tinted(stone, (k % 3) as f32 * 0.05));
    }
    b.cube(c + vec3(0.0, 0.07, 0.0), vec3(0.5, 0.08, 0.1), log);
    b.cube(c + vec3(0.0, 0.07, 0.0), vec3(0.1, 0.08, 0.5), log);
    for seat in [CAT_SEAT, DOG_SEAT] {
        b.cube(vec3(seat.x, 0.58, seat.y), vec3(0.24, 0.16, 0.7), log);
        b.cube(vec3(seat.x, 0.58, seat.y + 0.36), vec3(0.2, 0.13, 0.02), log_end);
    }
    for k in 0..camp.pile.min(5) {
        let kf = k as f32;
        b.cube(vec3(c.x - 0.75 + kf * 0.03, 0.54 + kf * 0.05, c.z - 0.5), vec3(0.4, 0.05, 0.06), tinted(log, kf * 0.04));
    }
    if camp.lit {
        let scale = 0.7 + (camp.fuel / 120.0).min(1.0) * 0.5;
        for k in 0..4 {
            let flick = hash(k, 3, (time * 12.0) as u32);
            let h = (0.22 + flick * 0.3) * scale;
            let a = k as f32 / 4.0 * 2.0 * PI + time;
            let off = vec3(a.cos() * 0.08, 0.0, a.sin() * 0.08);
            let col = match k {
                0 => Color::from_rgba(240, 90, 40, 255),
                1 => Color::from_rgba(255, 140, 50, 255),
                2 => Color::from_rgba(255, 190, 70, 255),
                _ => Color::from_rgba(255, 235, 140, 255),
            };
            let w = 0.16 - k as f32 * 0.02;
            b.cube(c + off + vec3(0.0, 0.1 + h / 2.0, 0.0), vec3(w, h, w), col);
        }
    }
}

pub(crate) const DISHES: [&str; 8] = ["campfire tea", "grilled fish", "fish stew", "golden koi soup", "grilled mushrooms", "mushroom stew", "fish & mushroom hotpot", "golden truffle soup"];

pub(crate) const COOK_GOAL: u32 = 24;

pub(crate) const STICK_FUEL: f32 = 45.0;

pub(crate) const FUEL_WANT: f32 = 140.0;

pub(crate) const FIRE_SPOT: Vec2 = Vec2::new(8.5, 17.5);

pub(crate) const SEAT_TILES: [(i32, i32); 2] = [(7, 17), (9, 17)];

pub(crate) const PERC_NAMES: [&str; 4] = ["kick", "lid", "chop", "sizzle"];

pub(crate) fn near_fire(p: &Player) -> bool {
    p.pos.distance(FIRE_SPOT) < 2.2
}
