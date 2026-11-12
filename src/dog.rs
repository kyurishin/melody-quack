//! The dog's helper brain.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) enum Task {
    Idle,
    Seek { spot: Vec2, tile: (i32, i32), carry: Carry },
    Sniff { tile: (i32, i32), timer: f32, carry: Carry },
    Return,
    Sit,
}

pub(crate) struct DogBrain {
    pub(crate) task: Task,
    pub(crate) stuck: f32,
    pub(crate) trip: f32,
    pub(crate) rest: f32,
}

impl DogBrain {
    pub(crate) fn new() -> DogBrain {
        DogBrain { task: Task::Idle, stuck: 0.0, trip: 0.0, rest: 0.0 }
    }
}

pub(crate) fn in_forest(x: i32, z: i32) -> bool {
    z < 11 || x > 13
}

pub(crate) fn find_source(world: &mut World, from: Vec2, kind: Kind) -> Option<((i32, i32), Vec2)> {
    let mut options = Vec::new();
    for z in 1..MAP as i32 - 1 {
        for x in 1..MAP as i32 - 1 {
            if world.get(x, z).map(|t| t.kind != kind || t.fire > 0.0).unwrap_or(true) {
                continue;
            }
            if kind == Kind::Tree && !in_forest(x, z) {
                continue;
            }
            for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                let spot = vec2((x + dx) as f32 + 0.5, (z + dz) as f32 + 0.5);
                let dist = spot.distance(from);
                if walkable(world, spot) && dist > 2.0 && dist < 14.0 {
                    options.push(((x, z), spot));
                    break;
                }
            }
        }
    }
    if options.is_empty() {
        return None;
    }
    let pick = (world.rand() * options.len() as f32) as usize % options.len();
    Some(options[pick])
}

pub(crate) fn steer(world: &World, from: Vec2, to: Vec2) -> Vec2 {
    let want = to - from;
    if want.length() < 0.05 {
        return Vec2::ZERO;
    }
    let dir = want.normalize();
    for angle in [0.0_f32, 0.6, -0.6, 1.2, -1.2, 1.6, -1.6] {
        let (s, c) = angle.sin_cos();
        let d = vec2(dir.x * c - dir.y * s, dir.x * s + dir.y * c);
        if walkable(world, from + d * 0.35) {
            return d;
        }
    }
    dir
}
