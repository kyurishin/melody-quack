//! Gathering sticks from trees.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) enum Gathered {
    Stick,
    Stone,
    Acorns(u32),
    Resting,
    Nothing,
}

pub(crate) fn gather_at(world: &World, stick_trees: &mut Vec<(i32, i32)>, tree_rest: &mut std::collections::HashMap<(i32, i32), f32>, tile: (i32, i32), time: f32, hands_free: bool) -> Gathered {
    let kind = match world.get(tile.0, tile.1) {
        Some(t) if t.fire <= 0.0 => t.kind,
        _ => return Gathered::Nothing,
    };
    match kind {
        Kind::Tree => {
            if hands_free && stick_trees.contains(&tile) {
                stick_trees.retain(|t| *t != tile);
                return Gathered::Stick;
            }
            let ready = tree_rest.get(&tile).map_or(true, |&at| time - at > TREE_REST);
            if ready {
                tree_rest.insert(tile, time);
                Gathered::Acorns(1 + (hash(tile.0, tile.1, time as u32) * 3.0) as u32)
            } else {
                Gathered::Resting
            }
        }
        Kind::Stone if hands_free => Gathered::Stone,
        _ => Gathered::Nothing,
    }
}

pub(crate) fn stick_spot(world: &World, tile: (i32, i32)) -> Option<Vec2> {
    for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
        if world.get(tile.0 + dx, tile.1 + dz).map(|t| !t.solid()).unwrap_or(false) {
            return Some(vec2(tile.0 as f32 + 0.5 + dx as f32 * 0.9, tile.1 as f32 + 0.5 + dz as f32 * 0.9));
        }
    }
    None
}

pub(crate) fn grow_stick(world: &mut World, stick_trees: &mut Vec<(i32, i32)>) {
    let mut options = Vec::new();
    for z in 1..MAP as i32 - 1 {
        for x in 1..MAP as i32 - 1 {
            let ok = world.get(x, z).map(|t| t.kind == Kind::Tree && t.fire <= 0.0).unwrap_or(false);
            if ok && in_forest(x, z) && !stick_trees.contains(&(x, z)) && stick_spot(world, (x, z)).is_some() {
                options.push((x, z));
            }
        }
    }
    if !options.is_empty() {
        let pick = (world.rand() * options.len() as f32) as usize % options.len();
        stick_trees.push(options[pick]);
    }
}

pub(crate) fn nearest_stick_tree(world: &World, stick_trees: &[(i32, i32)], from: Vec2) -> Option<((i32, i32), Vec2)> {
    let mut best: Option<((i32, i32), Vec2)> = None;
    let mut best_d = f32::MAX;
    for &tile in stick_trees {
        for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
            let spot = vec2((tile.0 + dx) as f32 + 0.5, (tile.1 + dz) as f32 + 0.5);
            if walkable(world, spot) {
                let d = spot.distance(from);
                if d < best_d {
                    best_d = d;
                    best = Some((tile, spot));
                }
                break;
            }
        }
    }
    best
}

pub(crate) const TREE_REST: f32 = 40.0;
