#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const SPELL_NAMES: [&str; 6] = ["ember", "bloom", "rain", "gust", "frost", "sun"];

pub(crate) struct Particle {
    pub(crate) pos: Vec3,
    pub(crate) vel: Vec3,
    pub(crate) life: f32,
    pub(crate) max_life: f32,
    pub(crate) color: Color,
    pub(crate) size: f32,
    pub(crate) gravity: f32,
}

pub(crate) fn spawn(particles: &mut Vec<Particle>, pos: Vec3, vel: Vec3, life: f32, color: Color, size: f32, gravity: f32) {
    particles.push(Particle {
        pos,
        vel,
        life,
        max_life: life,
        color,
        size,
        gravity,
    });
}

pub(crate) fn puff(particles: &mut Vec<Particle>, x: i32, z: i32, color: Color, count: usize) {
    for k in 0..count {
        let a = hash(x, z, k as u32 + (get_time() * 60.0) as u32) * 2.0 * PI;
        spawn(
            particles,
            vec3(x as f32 + 0.5, 0.7, z as f32 + 0.5),
            vec3(a.cos() * 0.4, 0.9 + hash(z, x, k as u32) * 0.6, a.sin() * 0.4),
            0.9,
            color,
            0.06,
            -0.2,
        );
    }
}

pub(crate) fn touches_cat(x: i32, z: i32, cat: Vec2) -> bool {
    let nearest_x = cat.x.clamp(x as f32, x as f32 + 1.0);
    let nearest_z = cat.y.clamp(z as f32, z as f32 + 1.0);
    vec2(nearest_x, nearest_z).distance(cat) < CAT_RADIUS + 0.05
}

pub(crate) fn cast(spell: usize, world: &mut World, cat: Vec2, facing: Vec2, particles: &mut Vec<Particle>) {
    let cx = cat.x.floor() as i32;
    let cz = cat.y.floor() as i32;
    let ahead = cat + facing * 1.0;
    let tx = ahead.x.floor() as i32;
    let tz = ahead.y.floor() as i32;

    match spell {
        0 => {
            if let Some(t) = world.get_mut(tx, tz) {
                if t.flammable() {
                    t.fire = t.burn_time();
                } else if t.kind == Kind::Ice {
                    t.kind = Kind::Water;
                }
            }
            puff(particles, tx, tz, Color::from_rgba(255, 150, 60, 255), 10);
        }
        1 => {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (x, z) = (tx + dx, tz + dz);
                    let center = dx == 0 && dz == 0;
                    let blocked = touches_cat(x, z, cat);
                    if let Some(t) = world.get_mut(x, z) {
                        if t.fire > 0.0 {
                            continue;
                        }
                        t.kind = match t.kind {
                            Kind::Ash => Kind::Grass,
                            Kind::Grass => Kind::Flowers,
                            Kind::Flowers if center && !crate::venues::reserved(x, z) => Kind::Sprout,
                            Kind::Sprout if center && !blocked && !crate::venues::reserved(x, z) => Kind::Tree,
                            k => k,
                        };
                    }
                    puff(particles, x, z, Color::from_rgba(255, 170, 210, 255), 2);
                }
            }
        }
        2 => {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if let Some(t) = world.get_mut(cx + dx, cz + dz) {
                        t.wet = 1.0;
                        if t.fire > 0.0 {
                            t.fire = 0.0;
                        }
                    }
                }
            }
            for k in 0..70 {
                let ox = hash(k, 1, (get_time() * 10.0) as u32) * 5.0 - 2.5;
                let oz = hash(k, 2, (get_time() * 10.0) as u32) * 5.0 - 2.5;
                let h = 2.5 + hash(k, 3, 7) * 2.5;
                spawn(
                    particles,
                    vec3(cat.x + ox, h, cat.y + oz),
                    vec3(0.0, -7.0, 0.0),
                    h / 7.0,
                    Color::from_rgba(170, 210, 255, 255),
                    0.03,
                    0.0,
                );
            }
        }
        3 => {
            world.wind = facing;
            world.wind_time = 5.0;
            for k in 0..28 {
                let side = vec2(-facing.y, facing.x) * (hash(k, 4, (get_time() * 10.0) as u32) * 4.0 - 2.0);
                let start = cat + side - facing * 1.5;
                spawn(
                    particles,
                    vec3(start.x, 0.6 + hash(k, 5, 3) * 0.8, start.y),
                    vec3(facing.x * 5.0, 0.3, facing.y * 5.0),
                    1.2,
                    Color::from_rgba(160, 225, 150, 255),
                    0.045,
                    0.0,
                );
            }
        }
        4 => {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    if let Some(t) = world.get_mut(tx + dx, tz + dz) {
                        if t.kind == Kind::Water {
                            t.kind = Kind::Ice;
                        }
                        t.fire = 0.0;
                    }
                    puff(particles, tx + dx, tz + dz, Color::from_rgba(230, 245, 255, 255), 2);
                }
            }
        }
        _ => {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if let Some(t) = world.get_mut(cx + dx, cz + dz) {
                        if t.kind != Kind::Water {
                            t.wet = 0.0;
                        }
                        if t.kind == Kind::Ice {
                            t.kind = Kind::Water;
                        }
                    }
                }
            }
            for k in 0..24 {
                let a = k as f32 / 24.0 * 2.0 * PI;
                spawn(
                    particles,
                    vec3(cat.x + a.cos() * 0.5, 0.8, cat.y + a.sin() * 0.5),
                    vec3(a.cos() * 2.0, 1.2, a.sin() * 2.0),
                    0.8,
                    Color::from_rgba(255, 225, 120, 255),
                    0.05,
                    0.0,
                );
            }
        }
    }
}
