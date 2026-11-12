#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const DOCK: [(i32, i32); 2] = [(14, 18), (15, 18)];

pub(crate) const WALL_HEIGHT: f32 = 0.45;

pub(crate) fn ground_top(kind: Kind) -> f32 {
    match kind {
        Kind::Water => 0.32,
        Kind::Ice => 0.46,
        Kind::Dock => 0.48,
        Kind::Floor => 0.55,
        _ => 0.5,
    }
}

pub(crate) fn surface_top(kind: Kind) -> f32 {
    match kind {
        Kind::Dock => 0.32,
        Kind::Floor => 0.5,
        k => ground_top(k),
    }
}

pub(crate) fn hides_point(x: f32, z: f32, focus: Vec2) -> bool {
    let dx = (x - focus.x).abs();
    let dz = z - focus.y;
    dz > 0.3 && dz < 2.6 && dx < 0.9 + dz * 0.1
}

pub(crate) fn draw_lantern(b: &mut Batch, base: Vec3, time: f32) {
    let post = Color::from_rgba(96, 66, 44, 255);
    let flicker = 0.9 + 0.1 * (time * 7.0 + base.x * 3.0).sin();
    b.cube(base + vec3(0.0, 0.32, 0.0), vec3(0.07, 0.64, 0.07), post);
    b.cube(base + vec3(0.0, 0.69, 0.0), vec3(0.2, 0.04, 0.2), post);
    b.cube(base + vec3(0.0, 0.6, 0.0), vec3(0.14, 0.13, 0.14), Color::new(1.0, 0.86 * flicker, 0.52 * flicker, 1.0));
    b.cube(base + vec3(0.0, 0.53, 0.0), vec3(0.17, 0.03, 0.17), post);
}

pub(crate) fn draw_world(b: &mut Batch, world: &World, time: f32, cat: Vec2, see_through: bool) {
    let sk = sky();
    let (mut grass, leaves, mut leaves_light) = match sk.season {
        0 => (rgba(118, 186, 96), rgba(72, 146, 84), rgba(240, 176, 200)),
        2 => (rgba(150, 168, 88), rgba(190, 104, 50), rgba(230, 162, 64)),
        3 => (rgba(140, 158, 134), rgba(52, 104, 76), rgba(78, 126, 92)),
        _ => (Color::from_rgba(112, 178, 92, 255), Color::from_rgba(58, 128, 78, 255), Color::from_rgba(92, 160, 86, 255)),
    };
    if sk.weather == 3 {
        grass = rgba(230, 236, 242);
        leaves_light = rgba(240, 244, 250);
    }
    let wet_grass = Color::from_rgba(70, 138, 112, 255);
    let trunk = Color::from_rgba(122, 86, 60, 255);
    let wood = Color::from_rgba(178, 126, 80, 255);
    let stone = Color::from_rgba(160, 162, 176, 255);
    let plank = Color::from_rgba(204, 160, 110, 255);
    let path = Color::from_rgba(196, 162, 116, 255);

    for z in 0..MAP as i32 {
        for x in 0..MAP as i32 {
            let t = world.get(x, z).unwrap();
            let fx = x as f32 + 0.5;
            let fz = z as f32 + 0.5;
            let is_wall = matches!(t.kind, Kind::WoodWall | Kind::StoneWall);
            let in_the_way = (t.kind == Kind::Tree || (is_wall && t.stack >= 2)) && hides_point(fx, fz, cat);
            if see_through && !in_the_way {
                continue;
            }
            let top = surface_top(t.kind);
            let alpha = if see_through { 0.35 } else { 1.0 };
            let ground = match t.kind {
                Kind::Water | Kind::Dock => {
                    let shimmer = (time * 2.0 + x as f32 * 0.7 + z as f32 * 1.1).sin() * 0.05;
                    tinted(Color::from_rgba(78, 146, 204, 255), shimmer)
                }
                Kind::Ice => Color::from_rgba(206, 232, 246, 255),
                Kind::Ash => tinted(Color::from_rgba(96, 90, 88, 255), t.tint),
                Kind::Path | Kind::Campfire => tinted(path, t.tint * 0.5),
                _ => tinted(mix(grass, wet_grass, t.wet * 0.7), t.tint),
            };
            if !see_through {
                let lower = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dz)| world.get(x + dx, z + dz).map(|n| surface_top(n.kind) < top - 0.01).unwrap_or(true));
                if lower {
                    b.cube(vec3(fx, (top - 0.6) / 2.0, fz), vec3(1.0, top + 0.6, 1.0), ground);
                } else {
                    b.top(vec3(fx, top, fz), ground);
                }
            }

            match t.kind {
                Kind::Flowers if !see_through => {
                    for k in 0..3 {
                        let ox = hash(x, z, 10 + k) * 0.7 - 0.35;
                        let oz = hash(x, z, 20 + k) * 0.7 - 0.35;
                        let petal = match k {
                            0 => Color::from_rgba(255, 170, 200, 255),
                            1 => Color::from_rgba(255, 236, 140, 255),
                            _ => Color::from_rgba(250, 250, 255, 255),
                        };
                        b.cube(vec3(fx + ox, top + 0.06, fz + oz), vec3(0.1, 0.1, 0.1), petal);
                    }
                }
                Kind::Sprout if !see_through => {
                    b.cube(vec3(fx, top + 0.1, fz), vec3(0.08, 0.2, 0.08), leaves_light);
                    b.cube(vec3(fx, top + 0.24, fz), vec3(0.22, 0.12, 0.22), leaves_light);
                }
                Kind::Tree if in_the_way == see_through => {
                    let sway = (time * 1.3 + x as f32 + z as f32 * 0.5).sin() * 0.03;
                    let fade = |c: Color| Color::new(c.r, c.g, c.b, alpha);
                    b.cube(vec3(fx, top + 0.35, fz), vec3(0.22, 0.7, 0.22), fade(trunk));
                    b.cube(vec3(fx + sway, top + 0.95, fz), vec3(0.9, 0.55, 0.9), fade(tinted(leaves, t.tint)));
                    b.cube(vec3(fx + sway * 1.5, top + 1.38, fz), vec3(0.6, 0.38, 0.6), fade(tinted(leaves_light, t.tint)));
                }
                Kind::Stone if !see_through => {
                    b.cube(vec3(fx, top + 0.25, fz), vec3(0.75, 0.5, 0.7), Color::from_rgba(150, 152, 164, 255));
                    b.cube(vec3(fx + 0.15, top + 0.55, fz - 0.05), vec3(0.4, 0.2, 0.4), Color::from_rgba(170, 172, 182, 255));
                }
                Kind::WoodWall | Kind::StoneWall if in_the_way == see_through => {
                    for level in 0..t.stack {
                        let base = if t.kind == Kind::WoodWall {
                            tinted(wood, if level % 2 == 0 { 0.0 } else { -0.08 })
                        } else {
                            tinted(stone, hash(x, z, level as u32) * 0.1 - 0.05)
                        };
                        let y = 0.5 + WALL_HEIGHT * (level as f32 + 0.5);
                        b.cube(vec3(fx, y, fz), vec3(1.0, WALL_HEIGHT, 1.0), Color::new(base.r, base.g, base.b, alpha));
                    }
                }
                Kind::Floor if !see_through => {
                    let p = if (x + z) % 2 == 0 { plank } else { tinted(plank, -0.06) };
                    b.cube(vec3(fx, 0.525, fz), vec3(0.98, 0.05, 0.98), p);
                }
                Kind::Dock if !see_through => {
                    for k in 0..4 {
                        let px = x as f32 + 0.125 + k as f32 * 0.25;
                        let c = if (k + x) % 2 == 0 { plank } else { tinted(plank, -0.07) };
                        b.cube(vec3(px, 0.455, fz), vec3(0.22, 0.05, 0.7), c);
                    }
                    for (ox, oz) in [(0.1, -0.33), (0.1, 0.33), (0.9, -0.33), (0.9, 0.33)] {
                        b.cube(vec3(x as f32 + ox, 0.38, z as f32 + 0.5 + oz), vec3(0.08, 0.26, 0.08), tinted(plank, -0.3));
                    }
                }
                _ => {}
            }

            if t.fire > 0.0 && !see_through {
                let flame_base = if t.kind == Kind::Tree { top + 0.6 } else { top };
                for k in 0..3 {
                    let flick = hash(x * 3 + k, z, (time * 12.0) as u32);
                    let h = 0.25 + flick * 0.45;
                    let ox = (k as f32 - 1.0) * 0.22;
                    let col = match k {
                        0 => Color::from_rgba(240, 90, 40, 255),
                        1 => Color::from_rgba(255, 170, 60, 255),
                        _ => Color::from_rgba(255, 225, 110, 255),
                    };
                    b.cube(vec3(fx + ox, flame_base + h / 2.0, fz + ox * 0.5), vec3(0.2, h, 0.2), col);
                }
            }
        }
    }
}

pub(crate) const COTTAGE: (i32, i32, i32, i32) = (3, 13, 6, 15);

pub(crate) fn draw_cottage(b: &mut Batch, dusk: f32, see_through: bool, focus: Vec2) {
    let (x0, z0, x1, z1) = COTTAGE;
    let in_the_way = focus.y < z0 as f32 && focus.y > z0 as f32 - 2.6 && focus.x > x0 as f32 - 0.8 && focus.x < x1 as f32 + 0.8;
    if in_the_way != see_through {
        return;
    }
    let a = if see_through { 0.35 } else { 1.0 };
    let log = Color::from_rgba(170, 118, 74, 255);
    let log_dark = Color::from_rgba(130, 88, 54, 255);
    let roof = Color::from_rgba(180, 78, 62, 255);
    let roof_dark = Color::from_rgba(140, 56, 46, 255);
    let fade = |c: Color| Color::new(c.r, c.g, c.b, a);
    for tz in z0..z1 {
        for tx in x0..x1 {
            for level in 0..4 {
                let c = if level % 2 == 0 { log } else { log_dark };
                b.cube(vec3(tx as f32 + 0.5, 0.5 + 0.125 + level as f32 * 0.25, tz as f32 + 0.5), vec3(1.0, 0.25, 1.0), fade(c));
            }
        }
    }
    for (cx, cz) in [(x0, z0), (x1, z0), (x0, z1), (x1, z1)] {
        b.cube(vec3(cx as f32, 1.02, cz as f32), vec3(0.16, 1.04, 0.16), fade(log_dark));
    }
    let depth = (z1 - z0) as f32;
    let mid_z = (z0 + z1) as f32 / 2.0;
    let width = (x1 - x0) as f32 + 0.5;
    let mid_x = (x0 + x1) as f32 / 2.0;
    for layer in 0..5 {
        let half = depth / 2.0 + 0.25 - layer as f32 * 0.27;
        let y = 1.6 + layer as f32 * 0.2;
        let c = if layer % 2 == 0 { roof } else { roof_dark };
        b.cube(vec3(mid_x, y, mid_z), vec3(width, 0.2, half * 2.0), fade(c));
    }
    b.cube(vec3(x1 as f32 - 0.6, 2.1, z0 as f32 + 0.6), vec3(0.34, 1.0, 0.34), fade(Color::from_rgba(150, 152, 166, 255)));
    if see_through {
        return;
    }
    let front = z1 as f32 + 0.02;
    let door_x = x0 as f32 + 1.5;
    b.cube(vec3(door_x, 0.85, front), vec3(0.42, 0.7, 0.04), Color::from_rgba(110, 70, 42, 255));
    b.cube(vec3(door_x + 0.13, 0.85, front + 0.025), vec3(0.05, 0.05, 0.02), Color::from_rgba(255, 210, 90, 255));
    let glass = mix(Color::from_rgba(150, 190, 220, 255), Color::from_rgba(255, 214, 130, 255), (dusk * 2.0).min(1.0));
    for wx in [x0 as f32 + 0.5, x1 as f32 - 0.5] {
        b.cube(vec3(wx, 1.0, front), vec3(0.42, 0.34, 0.04), Color::from_rgba(150, 100, 60, 255));
        b.cube(vec3(wx, 1.0, front + 0.01), vec3(0.32, 0.25, 0.04), glass);
        b.cube(vec3(wx, 0.8, front + 0.06), vec3(0.44, 0.07, 0.12), Color::from_rgba(150, 100, 60, 255));
        for (k, col) in [(-1.0, Color::from_rgba(255, 140, 170, 255)), (0.0, Color::from_rgba(255, 230, 110, 255)), (1.0, Color::from_rgba(250, 250, 250, 255))] {
            b.cube(vec3(wx + k * 0.13, 0.86, front + 0.06), vec3(0.08, 0.06, 0.08), col);
        }
    }
    b.cube(vec3(door_x, 0.53, front + 0.15), vec3(0.6, 0.06, 0.3), Color::from_rgba(204, 160, 110, 255));
}

thread_local! {
    pub(crate) static SHROOMS: std::cell::RefCell<Vec<(Vec2, usize)>> = std::cell::RefCell::new(Vec::new());
}

pub(crate) fn draw_shrooms(b: &mut Batch, time: f32) {
    SHROOMS.with(|list| {
        for (at, species) in list.borrow().iter() {
            let look = mushroom::look(*species);
            let base = vec3(at.x, 0.5, at.y);
            let bob = 0.01 * (time * 2.0 + at.x).sin();
            match look.shape {
                1 => b.cube(base + vec3(0.0, 0.07 + bob, 0.0), vec3(0.16, 0.13, 0.16), look.cap),
                2 => {
                    b.cube(base + vec3(0.0, 0.06, 0.0), vec3(0.05, 0.12, 0.05), look.stem);
                    b.cube(base + vec3(0.0, 0.17 + bob, 0.0), vec3(0.09, 0.14, 0.09), look.cap);
                }
                3 => {
                    b.cube(base + vec3(0.0, 0.05 + bob, 0.0), vec3(0.2, 0.04, 0.14), look.cap);
                    b.cube(base + vec3(0.03, 0.1 + bob, 0.0), vec3(0.15, 0.04, 0.11), look.cap);
                }
                4 => {
                    for k in 0..3 {
                        let ox = (k as f32 - 1.0) * 0.05;
                        b.cube(base + vec3(ox, 0.08 + bob, (k as f32 * 1.7).sin() * 0.03), vec3(0.035, 0.16 - k as f32 * 0.02, 0.035), look.cap);
                    }
                }
                _ => {
                    b.cube(base + vec3(0.0, 0.05, 0.0), vec3(0.05, 0.1, 0.05), look.stem);
                    b.cube(base + vec3(0.0, 0.12 + bob, 0.0), vec3(0.18, 0.06, 0.18), look.cap);
                    if let Some(spot) = look.spot {
                        b.cube(base + vec3(0.04, 0.155 + bob, -0.03), vec3(0.04, 0.012, 0.04), spot);
                        b.cube(base + vec3(-0.05, 0.155 + bob, 0.04), vec3(0.03, 0.012, 0.03), spot);
                    }
                }
            }
            if ((time * 0.8 + at.x * 3.1 + at.y).sin()) > 0.97 {
                b.unlit = true;
                b.cube(base + vec3(0.06, 0.3, 0.0), vec3(0.03, 0.03, 0.03), Color::from_rgba(255, 250, 200, 255));
                b.unlit = false;
            }
        }
    });
}

thread_local! {
    pub(crate) static BOBBER_AT: std::cell::Cell<[(f32, f32); 2]> = std::cell::Cell::new([(0.0, 0.0); 2]);
}
