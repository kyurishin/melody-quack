#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const MAP: usize = 24;

pub(crate) const TICK: f32 = 0.2;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Grass,
    Flowers,
    Sprout,
    Tree,
    Water,
    Ice,
    Ash,
    Stone,
    House,
    Campfire,
    Path,
    WoodWall,
    StoneWall,
    Floor,
    Dock,
}

#[derive(Clone, Copy)]
pub(crate) struct Tile {
    pub(crate) kind: Kind,
    pub(crate) fire: f32,
    pub(crate) wet: f32,
    pub(crate) tint: f32,
    pub(crate) stack: u8,
}

impl Tile {
    pub(crate) fn flammable(&self) -> bool {
        matches!(self.kind, Kind::Grass | Kind::Flowers | Kind::Sprout | Kind::Tree)
            && self.wet < 0.3
            && self.fire <= 0.0
    }

    pub(crate) fn solid(&self) -> bool {
        matches!(self.kind, Kind::Tree | Kind::Water | Kind::Stone | Kind::House | Kind::Campfire | Kind::WoodWall | Kind::StoneWall)
    }

    pub(crate) fn burn_time(&self) -> f32 {
        match self.kind {
            Kind::Tree => 4.0,
            Kind::Sprout => 1.2,
            _ => 1.6,
        }
    }
}

pub(crate) struct World {
    pub(crate) tiles: Vec<Tile>,
    pub(crate) wind: Vec2,
    pub(crate) wind_time: f32,
    pub(crate) seed: u32,
}

impl World {
    pub(crate) fn new() -> World {
        let path: [(i32, i32); 16] = [
            (4, 15), (4, 16), (5, 16), (6, 16), (7, 16), (8, 16), (9, 16), (9, 15),
            (9, 14), (9, 13), (10, 12), (10, 11), (11, 10), (12, 9), (12, 8), (13, 7),
        ];
        let mut tiles = Vec::with_capacity(MAP * MAP);
        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let edge = x == 0 || z == 0 || x == MAP as i32 - 1 || z == MAP as i32 - 1;
                let lake = vec2(x as f32 - 16.5, z as f32 - 18.5).length() < 3.1;
                let (cx0, cz0, cx1, cz1) = COTTAGE;
                let house = x >= cx0 && x < cx1 && z >= cz0 && z < cz1;
                let home = !in_forest(x, z) && !lake;
                let clear_home = (x - 8).abs() <= 2 && (z - 17).abs() <= 1 || (x >= 2 && x <= 7 && z >= 15 && z <= 16);
                let kind = if edge {
                    Kind::Tree
                } else if house {
                    Kind::House
                } else if (x, z) == CAMPFIRE {
                    Kind::Campfire
                } else if path.contains(&(x, z)) {
                    Kind::Path
                } else if DOCK.contains(&(x, z)) {
                    Kind::Dock
                } else if lake {
                    Kind::Water
                } else if clear_home {
                    Kind::Grass
                } else if home {
                    if hash(x, z, 4) > 0.72 {
                        Kind::Flowers
                    } else if hash(x, z, 3) > 0.96 {
                        Kind::Stone
                    } else if hash(x, z, 2) > 0.95 && (x < 3 || z > 19) {
                        Kind::Tree
                    } else {
                        Kind::Grass
                    }
                } else if hash(x, z, 1) > 0.56 {
                    Kind::Tree
                } else if hash(x, z, 3) > 0.93 {
                    Kind::Stone
                } else if hash(x, z, 4) > 0.8 {
                    Kind::Flowers
                } else {
                    Kind::Grass
                };
                tiles.push(Tile {
                    kind,
                    fire: 0.0,
                    wet: 0.0,
                    tint: hash(x, z, 5) * 0.14 - 0.07,
                    stack: 0,
                });
            }
        }
        World {
            tiles,
            wind: Vec2::ZERO,
            wind_time: 0.0,
            seed: 0x9e37_79b9,
        }
    }

    pub(crate) fn index(x: i32, z: i32) -> Option<usize> {
        if x < 0 || z < 0 || x >= MAP as i32 || z >= MAP as i32 {
            None
        } else {
            Some(z as usize * MAP + x as usize)
        }
    }

    pub(crate) fn get(&self, x: i32, z: i32) -> Option<&Tile> {
        World::index(x, z).map(|i| &self.tiles[i])
    }

    pub(crate) fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut Tile> {
        World::index(x, z).map(move |i| &mut self.tiles[i])
    }

    pub(crate) fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed & 0xffff) as f32 / 65535.0
    }

    pub(crate) fn tick(&mut self, particles: &mut Vec<Particle>) {
        let old = self.tiles.clone();
        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let i = World::index(x, z).unwrap();
                let t = old[i];

                if t.fire > 0.0 {
                    if t.wet > 0.3 {
                        self.tiles[i].fire = 0.0;
                        puff(particles, x, z, WHITE, 6);
                    } else {
                        self.tiles[i].fire -= TICK;
                        if self.tiles[i].fire <= 0.0 {
                            self.tiles[i].fire = 0.0;
                            self.tiles[i].kind = Kind::Ash;
                        }
                        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            let Some(ni) = World::index(x + dx, z + dz) else { continue };
                            let n = old[ni];
                            if n.kind == Kind::Ice {
                                self.tiles[ni].kind = Kind::Water;
                                puff(particles, x + dx, z + dz, WHITE, 4);
                            }
                            if n.flammable() && self.tiles[ni].fire <= 0.0 {
                                let mut chance = if n.kind == Kind::Tree { 0.07 } else { 0.12 };
                                if self.wind_time > 0.0 {
                                    let along = vec2(dx as f32, dz as f32).dot(self.wind);
                                    chance *= (1.0 + along * 2.0).max(0.15);
                                }
                                if self.rand() < chance {
                                    self.tiles[ni].fire = n.burn_time();
                                }
                            }
                        }
                    }
                }

                if t.kind != Kind::Water {
                    self.tiles[i].wet = (self.tiles[i].wet - 0.012).max(0.0);
                }

                if t.kind == Kind::Ash && t.fire <= 0.0 && self.rand() < 0.004 {
                    self.tiles[i].kind = Kind::Grass;
                }
            }
        }
    }
}

pub(crate) fn walkable(world: &World, p: Vec2) -> bool {

    let r = CAT_RADIUS;
    for (ox, oz) in [(-r, -r), (r, -r), (-r, r), (r, r)] {
        let x = (p.x + ox).floor() as i32;
        let z = (p.y + oz).floor() as i32;
        match world.get(x, z) {
            Some(t) if t.kind == Kind::Tree => {
                let trunk = vec2(x as f32 + 0.5, z as f32 + 0.5);
                if (p.x - trunk.x).abs() < TRUNK_HALF + r && (p.y - trunk.y).abs() < TRUNK_HALF + r {
                    return false;
                }
            }
            Some(t) if !t.solid() => {}
            _ => return false,
        }
    }
    true
}

pub(crate) const TRUNK_HALF: f32 = 0.12;

pub(crate) fn rescue(world: &World, cat: Vec2) -> Vec2 {
    let cx = cat.x.floor() as i32;
    let cz = cat.y.floor() as i32;
    let mut best = cat;
    let mut best_dist = f32::MAX;
    for dz in -3..=3 {
        for dx in -3..=3 {
            let spot = vec2((cx + dx) as f32 + 0.5, (cz + dz) as f32 + 0.5);
            let d = spot.distance(cat);
            if d < best_dist && walkable(world, spot) {
                best = spot;
                best_dist = d;
            }
        }
    }
    best
}
