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
            b.unlit = true;
            b.cube(c + off + vec3(0.0, 0.1 + h / 2.0, 0.0), vec3(w, h, w), col);
            b.unlit = false;
        }
    }
}

pub(crate) const DISHES: [&str; 21] = [
    "campfire tea",
    "grilled fish",
    "fish stew",
    "golden koi soup",
    "grilled mushrooms",
    "mushroom stew",
    "fish & mushroom hotpot",
    "golden truffle soup",
    "smoked trout",
    "golden fish chowder",
    "fisherman's feast",
    "river king's banquet",
    "buttered mushrooms",
    "truffle toast",
    "forest risotto",
    "truffle risotto",
    "golden hotpot",
    "lakeside pie",
    "golden lakeside pie",
    "woodland broth",
    "moonlit broth",
];

pub(crate) const RECIPES: [&str; 21] = [
    "an empty pot",
    "1 common fish",
    "2 fish, none rare",
    "1 rare fish",
    "1 common mushroom",
    "2 mushrooms, none rare",
    "1 fish + 1 mushroom",
    "2 mushrooms, one rare",
    "1 uncommon fish",
    "2 fish, one rare",
    "3 fish",
    "3 fish, one rare",
    "1 uncommon mushroom",
    "1 rare mushroom",
    "3 mushrooms",
    "3 mushrooms, one rare",
    "1 fish + 1 mushroom, one rare",
    "2 fish + 1 mushroom",
    "2 fish + 1 mushroom, one rare",
    "1 fish + 2 mushrooms",
    "1 fish + 2 mushrooms, one rare",
];

pub(crate) const DISH_COLOURS: [(u8, u8, u8); 21] = [
    (170, 110, 60),
    (240, 150, 80),
    (200, 90, 70),
    (250, 200, 70),
    (150, 100, 70),
    (180, 130, 90),
    (210, 140, 100),
    (240, 210, 120),
    (196, 120, 84),
    (246, 214, 150),
    (226, 132, 96),
    (255, 206, 96),
    (214, 176, 112),
    (232, 190, 120),
    (236, 222, 180),
    (226, 206, 150),
    (250, 186, 90),
    (214, 150, 88),
    (244, 196, 104),
    (150, 140, 96),
    (170, 170, 210),
];

pub(crate) const POT_SLOTS: [&str; 6] = ["common fish", "uncommon fish", "rare fish", "common mushroom", "uncommon mushroom", "rare mushroom"];

pub(crate) const POT_MAX: u32 = 3;

pub(crate) fn dish_for(pot: [u32; 6]) -> usize {
    let f = pot[0] + pot[1] + pot[2];
    let m = pot[3] + pot[4] + pot[5];
    let rare_fish = pot[2] > 0;
    let rare = rare_fish || pot[5] > 0;
    match (f, m) {
        (1, 0) if rare_fish => 3,
        (1, 0) if pot[1] > 0 => 8,
        (1, 0) => 1,
        (2, 0) => if rare_fish { 9 } else { 2 },
        (3, 0) => if rare_fish { 11 } else { 10 },
        (0, 1) if pot[5] > 0 => 13,
        (0, 1) if pot[4] > 0 => 12,
        (0, 1) => 4,
        (0, 2) => if rare { 7 } else { 5 },
        (0, 3) => if rare { 15 } else { 14 },
        (1, 1) => if rare { 16 } else { 6 },
        (2, 1) => if rare { 18 } else { 17 },
        (1, 2) => if rare { 20 } else { 19 },
        _ => 0,
    }
}

pub(crate) fn auto_pot(fish: [u32; 3], mush: [u32; 3]) -> [u32; 6] {
    let mut pot = [0u32; 6];
    let low = |stock: [u32; 3]| (0..3).find(|k| stock[*k] > 0);
    let fish_n: u32 = fish.iter().sum();
    let mush_n: u32 = mush.iter().sum();
    if mush[2] > 0 {
        pot[5] = 1;
        if mush_n > 1 {
            pot[3 + low([mush[0], mush[1], 0]).unwrap_or(2)] += 1;
        }
    } else if fish[2] > 0 {
        pot[2] = 1;
    } else if fish_n >= 1 && mush_n >= 1 {
        pot[low(fish).unwrap_or(0)] += 1;
        pot[3 + low(mush).unwrap_or(0)] += 1;
    } else if mush_n >= 2 || fish_n >= 2 {
        let (stock, off) = if mush_n >= 2 { (mush, 3) } else { (fish, 0) };
        let mut left = stock;
        for _ in 0..2 {
            if let Some(k) = low(left) {
                left[k] -= 1;
                pot[off + k] += 1;
            }
        }
    } else if mush_n == 1 {
        pot[3 + low(mush).unwrap_or(0)] = 1;
    } else if fish_n == 1 {
        pot[low(fish).unwrap_or(0)] = 1;
    }
    pot
}

pub(crate) const COOK_GOAL: u32 = 24;

pub(crate) const STICK_FUEL: f32 = 45.0;

pub(crate) const FUEL_WANT: f32 = 140.0;

pub(crate) const FIRE_SPOT: Vec2 = Vec2::new(8.5, 17.5);

pub(crate) const PERC_NAMES: [&str; 4] = ["kick", "lid", "chop", "sizzle"];

pub(crate) fn near_fire(p: &Player) -> bool {
    p.pos.distance(FIRE_SPOT) < 2.2
}
