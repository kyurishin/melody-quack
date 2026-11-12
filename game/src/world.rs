//! Maps: the town, the PC방 interior and the chapel arena.

use crate::sprites::TILE;
use macroquad::prelude::*;
use mq_shared::Rng;

#[derive(Clone, Copy, PartialEq)]
pub enum Ground {
    Grass(u8),
    Path,
    Plaza,
    Water,
    Carpet,
    Concrete,
    Void,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PropKind {
    Tree(u8),
    Pine,
    Cottage,
    PcBang,
    Chapel,
    Hoarding,
    Lamp,
    Log,
    Bush,
    Flowers,
    Desk,
    Counter,
    Vending,
    Owner,
    Bouncer,
    Booth,
    Speaker,
    Pillar,
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
    pub pos: Vec2, // top-left
    pub sort_y: f32,
    pub flip: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpotKind {
    EnterPcBang,
    ExitPcBang,
    Seat(usize),
    Owner,
    ChapelDoor,
    ExitChapel,
    Decks,
    Poster(usize),
    Campfire,
    Cottage,
}

#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub kind: SpotKind,
    pub pos: Vec2,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PosterSlot {
    pub pos: Vec2, // top-left of the poster in world px
    pub scale: f32,
}

pub struct Map {
    pub w: usize,
    pub h: usize,
    pub ground: Vec<Ground>,
    pub solids: Vec<Rect>,
    pub props: Vec<Prop>,
    pub spots: Vec<Spot>,
    pub posters: Vec<PosterSlot>,
    pub lights: Vec<(Vec2, f32, Color)>,
    pub spawn: Vec2,
    pub campfire: Option<Vec2>,
    pub seats: Vec<Vec2>,
}

impl Map {
    fn new(w: usize, h: usize, fill: Ground) -> Map {
        Map {
            w,
            h,
            ground: vec![fill; w * h],
            solids: Vec::new(),
            props: Vec::new(),
            spots: Vec::new(),
            posters: Vec::new(),
            lights: Vec::new(),
            spawn: Vec2::ZERO,
            campfire: None,
            seats: Vec::new(),
        }
    }

    pub fn px_size(&self) -> Vec2 {
        vec2(self.w as f32 * TILE, self.h as f32 * TILE)
    }

    pub fn ground_at(&self, x: usize, y: usize) -> Ground {
        self.ground[y * self.w + x]
    }

    fn set(&mut self, x: usize, y: usize, g: Ground) {
        if x < self.w && y < self.h {
            self.ground[y * self.w + x] = g;
        }
    }

    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, g: Ground) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, g);
            }
        }
    }

    fn prop(&mut self, kind: PropKind, x: f32, y: f32, _w: f32, h: f32, solid: Option<Rect>) {
        self.props.push(Prop { kind, pos: vec2(x, y), sort_y: y + h, flip: false });
        if let Some(r) = solid {
            self.solids.push(Rect::new(x + r.x, y + r.y, r.w, r.h));
        }
    }

    /// Is a feet-box at `p` free to stand on?
    pub fn free(&self, p: Vec2) -> bool {
        let r = Rect::new(p.x - 7.0, p.y - 5.0, 14.0, 6.0);
        let size = self.px_size();
        if r.x < 0.0 || r.y < 0.0 || r.right() > size.x || r.bottom() > size.y {
            return false;
        }
        for (cx, cy) in [(r.x, r.y), (r.right(), r.y), (r.x, r.bottom()), (r.right(), r.bottom())] {
            let tx = (cx / TILE) as usize;
            let ty = (cy / TILE) as usize;
            if tx >= self.w || ty >= self.h {
                return false;
            }
            if matches!(self.ground_at(tx, ty), Ground::Water | Ground::Void) {
                return false;
            }
        }
        !self.solids.iter().any(|s| s.overlaps(&r))
    }

    fn tree_ok(&self, x: f32, y: f32) -> bool {
        let base = Rect::new(x + 14.0, y + 50.0, 36.0, 30.0);
        let tx = ((x + 32.0) / TILE) as usize;
        let ty = ((y + 74.0) / TILE) as usize;
        if tx >= self.w || ty >= self.h {
            return false;
        }
        if !matches!(self.ground_at(tx, ty), Ground::Grass(_)) {
            return false;
        }
        // Nothing on or next to paths, and well clear of doors and signs.
        for dy in [50.0, 66.0, 80.0] {
            for dx in [8.0, 32.0, 56.0] {
                let gx = ((x + dx) / TILE) as usize;
                let gy = ((y + dy) / TILE) as usize;
                if gx < self.w && gy < self.h && !matches!(self.ground_at(gx, gy), Ground::Grass(_)) {
                    return false;
                }
            }
        }
        let canopy = Rect::new(x, y, 64.0, 80.0);
        let pad = Rect::new(base.x - 16.0, base.y - 16.0, base.w + 32.0, base.h + 32.0);
        !self.solids.iter().any(|s| s.overlaps(&pad))
            && !self.spots.iter().any(|s| pad.contains(s.pos) || canopy.contains(s.pos) || s.pos.distance(vec2(x + 32.0, y + 70.0)) < 70.0)
    }
}

pub fn town() -> Map {
    let (w, h) = (56usize, 40usize);
    let mut m = Map::new(w, h, Ground::Grass(0));
    let mut r = Rng::new(7);
    for i in 0..w * h {
        m.ground[i] = Ground::Grass(match r.below(10) {
            0..=5 => 0,
            6..=8 => 1,
            _ => 2,
        });
    }
    // Paths.
    m.fill(3, 22, 50, 2, Ground::Path);
    m.fill(27, 8, 2, 30, Ground::Path);
    m.fill(7, 8, 2, 15, Ground::Path); // from the PC방 door
    m.fill(36, 9, 2, 14, Ground::Path); // to the chapel door
    m.fill(10, 24, 2, 4, Ground::Path); // to the cottage
    // Plaza.
    m.fill(22, 18, 12, 10, Ground::Plaza);
    // Pond.
    m.fill(42, 29, 8, 5, Ground::Water);
    m.fill(43, 28, 6, 1, Ground::Water);
    m.fill(44, 34, 4, 1, Ground::Water);

    // PC방 (208x176), door at its bottom centre.
    let (px, py) = (5.0 * TILE - 8.0, 3.0 * TILE - 8.0);
    m.prop(PropKind::PcBang, px, py, 208.0, 176.0, Some(Rect::new(0.0, 60.0, 208.0, 112.0)));
    let pc_door = vec2(px + 104.0, py + 182.0);
    m.spots.push(Spot { kind: SpotKind::EnterPcBang, pos: pc_door, radius: 26.0 });
    m.lights.push((vec2(px + 104.0, py + 56.0), 110.0, crate::gfx::pal::MAGENTA));
    m.lights.push((vec2(px + 104.0, py + 150.0), 70.0, crate::gfx::pal::BLUE));

    // Chapel (320x304), door at its bottom centre.
    let (cx, cy) = (32.0 * TILE, 0.0 * TILE + 8.0);
    m.prop(PropKind::Chapel, cx, cy, 320.0, 304.0, Some(Rect::new(0.0, 96.0, 320.0, 204.0)));
    let chapel_door = vec2(cx + 160.0, cy + 308.0);
    m.spots.push(Spot { kind: SpotKind::ChapelDoor, pos: chapel_door, radius: 30.0 });
    m.prop(PropKind::Bouncer, cx + 196.0, cy + 262.0, 40.0, 56.0, Some(Rect::new(6.0, 44.0, 28.0, 10.0)));
    m.lights.push((vec2(cx + 160.0, cy + 205.0), 90.0, Color::new(1.0, 0.25, 0.35, 1.0)));
    m.lights.push((vec2(cx + 42.0, cy + 190.0), 80.0, crate::gfx::pal::MAGENTA));
    m.lights.push((vec2(cx + 278.0, cy + 190.0), 80.0, crate::gfx::pal::CYAN));
    m.lights.push((vec2(cx + 160.0, cy + 60.0), 70.0, crate::gfx::pal::MAGENTA));
    // Flyers pasted straight onto the chapel wall either side of the door.
    m.posters.push(PosterSlot { pos: vec2(cx + 84.0, cy + 238.0), scale: 0.2 });
    m.posters.push(PosterSlot { pos: vec2(cx + 208.0, cy + 238.0), scale: 0.2 });

    // Cottage.
    let (hx, hy) = (5.0 * TILE, 27.0 * TILE);
    m.prop(PropKind::Cottage, hx, hy, 144.0, 128.0, Some(Rect::new(10.0, 56.0, 124.0, 66.0)));
    m.spots.push(Spot { kind: SpotKind::Cottage, pos: vec2(hx + 72.0, hy + 130.0), radius: 22.0 });
    m.lights.push((vec2(hx + 34.0, hy + 72.0), 50.0, crate::gfx::pal::YELLOW));
    m.lights.push((vec2(hx + 110.0, hy + 72.0), 50.0, crate::gfx::pal::YELLOW));

    // Campfire in the plaza, with logs.
    let fire = vec2(28.0 * TILE, 22.5 * TILE + 8.0);
    m.campfire = Some(fire);
    m.solids.push(Rect::new(fire.x - 12.0, fire.y - 8.0, 24.0, 12.0));
    m.spots.push(Spot { kind: SpotKind::Campfire, pos: fire + vec2(0.0, 24.0), radius: 30.0 });
    m.lights.push((fire, 150.0, crate::gfx::pal::CORAL));
    m.prop(PropKind::Log, fire.x - 70.0, fire.y - 4.0, 48.0, 20.0, Some(Rect::new(2.0, 6.0, 44.0, 10.0)));
    m.prop(PropKind::Log, fire.x + 22.0, fire.y - 4.0, 48.0, 20.0, Some(Rect::new(2.0, 6.0, 44.0, 10.0)));
    m.props.last_mut().unwrap().flip = true;

    // Poster hoardings: two flyers each.
    let hoardings = [
        vec2(13.0 * TILE, 18.0 * TILE),
        vec2(34.0 * TILE + 8.0, 18.5 * TILE),
        vec2(17.0 * TILE, 28.0 * TILE),
        vec2(32.0 * TILE, 28.5 * TILE),
        vec2(44.0 * TILE, 13.0 * TILE),
    ];
    for hp in hoardings {
        m.prop(PropKind::Hoarding, hp.x, hp.y, 128.0, 64.0, Some(Rect::new(0.0, 46.0, 128.0, 16.0)));
        for i in 0..2 {
            let slot = m.posters.len();
            let pos = vec2(hp.x + 20.0 + i as f32 * 52.0, hp.y + 8.0);
            m.posters.push(PosterSlot { pos, scale: 0.26 });
            m.spots.push(Spot { kind: SpotKind::Poster(slot), pos: pos + vec2(19.0, 62.0), radius: 24.0 });
        }
    }
    // Chapel-wall poster spots.
    for slot in 0..2 {
        let p = m.posters[slot].pos;
        m.spots.push(Spot { kind: SpotKind::Poster(slot), pos: p + vec2(14.0, 66.0), radius: 22.0 });
    }

    // Lamps.
    for (x, y) in [(21.0, 17.0), (34.0, 17.0), (21.0, 27.5), (34.5, 27.5), (8.0, 21.0), (46.0, 21.0), (26.0, 10.0), (38.0, 11.0), (12.0, 13.0)] {
        let p = vec2(x * TILE, y * TILE);
        m.prop(PropKind::Lamp, p.x, p.y - 30.0, 16.0, 64.0, Some(Rect::new(4.0, 56.0, 8.0, 6.0)));
        m.lights.push((p + vec2(8.0, -22.0), 80.0, crate::gfx::pal::YELLOW));
    }

    // Bushes and flowers.
    for (x, y) in [(15.0, 25.0), (24.0, 30.0), (36.0, 25.0), (19.0, 11.0), (48.0, 25.0), (30.0, 34.0)] {
        m.prop(PropKind::Bush, x * TILE, y * TILE, 40.0, 28.0, Some(Rect::new(4.0, 14.0, 32.0, 12.0)));
    }
    for (x, y) in [(13.0, 25.0), (20.0, 32.0), (37.0, 31.0), (51.0, 27.0), (24.0, 14.0), (31.0, 15.0)] {
        m.props.push(Prop { kind: PropKind::Flowers, pos: vec2(x * TILE, y * TILE), sort_y: y * TILE, flip: false });
    }

    // Trees: a border and some scattered.
    let mut tr = Rng::new(42);
    let place_tree = |m: &mut Map, x: f32, y: f32, tr: &mut Rng| {
        if m.tree_ok(x, y) {
            let kind = if tr.chance(0.2) { PropKind::Pine } else { PropKind::Tree(tr.below(3) as u8) };
            m.prop(kind, x, y, 64.0, 80.0, Some(Rect::new(22.0, 66.0, 20.0, 12.0)));
        }
    };
    let mut x = -16.0;
    while x < w as f32 * TILE {
        place_tree(&mut m, x + tr.range(-6.0, 6.0), -40.0 + tr.range(0.0, 10.0), &mut tr);
        place_tree(&mut m, x + tr.range(-6.0, 6.0), h as f32 * TILE - 84.0, &mut tr);
        x += 44.0;
    }
    let mut y = 20.0;
    while y < h as f32 * TILE - 80.0 {
        place_tree(&mut m, -20.0, y, &mut tr);
        place_tree(&mut m, w as f32 * TILE - 46.0, y, &mut tr);
        y += 48.0;
    }
    for _ in 0..70 {
        let x = tr.range(60.0, w as f32 * TILE - 120.0);
        let y = tr.range(60.0, h as f32 * TILE - 140.0);
        place_tree(&mut m, x, y, &mut tr);
    }

    m.spawn = vec2(28.0 * TILE, 25.5 * TILE);
    m
}

pub fn pcbang() -> Map {
    let (w, h) = (20usize, 14usize);
    let mut m = Map::new(w, h, Ground::Carpet);
    m.fill(0, 0, w, 2, Ground::Void); // back wall (drawn separately)
    m.solids.push(Rect::new(0.0, 0.0, w as f32 * TILE, 2.0 * TILE + 8.0));
    // Seats: 3 rows of 5 desks.
    for row in 0..3 {
        for col in 0..5 {
            let x = 2.0 * TILE + col as f32 * 56.0 + if col >= 3 { 40.0 } else { 0.0 };
            let y = 2.0 * TILE + 16.0 + row as f32 * 104.0;
            m.prop(PropKind::Desk, x, y, 48.0, 56.0, Some(Rect::new(0.0, 30.0, 48.0, 22.0)));
            let seat = vec2(x + 22.0, y + 74.0);
            let i = m.seats.len();
            m.seats.push(seat);
            m.spots.push(Spot { kind: SpotKind::Seat(i), pos: seat, radius: 16.0 });
            m.lights.push((vec2(x + 21.0, y + 12.0), 34.0, crate::gfx::pal::CYAN));
        }
    }
    // Counter + owner near the door.
    let (kx, ky) = (13.0 * TILE + 8.0, 10.0 * TILE);
    m.prop(PropKind::Owner, kx + 48.0, ky - 30.0, 32.0, 40.0, None);
    m.prop(PropKind::Counter, kx, ky, 128.0, 48.0, Some(Rect::new(0.0, 14.0, 128.0, 32.0)));
    m.spots.push(Spot { kind: SpotKind::Owner, pos: vec2(kx + 64.0, ky + 60.0), radius: 22.0 });
    m.prop(PropKind::Vending, 18.0 * TILE, 2.0 * TILE, 36.0, 64.0, Some(Rect::new(0.0, 40.0, 36.0, 24.0)));
    // Exit.
    let door = vec2(w as f32 * TILE / 2.0, h as f32 * TILE - 14.0);
    m.spots.push(Spot { kind: SpotKind::ExitPcBang, pos: door, radius: 26.0 });
    m.spawn = door + vec2(0.0, -24.0);
    m
}

pub fn chapel() -> Map {
    let (w, h) = (30usize, 19usize);
    let mut m = Map::new(w, h, Ground::Concrete);
    m.fill(0, 0, w, 6, Ground::Void);
    m.solids.push(Rect::new(0.0, 0.0, w as f32 * TILE, 6.0 * TILE + 8.0));
    let mid = w as f32 * TILE / 2.0;
    // Altar / DJ booth.
    m.prop(PropKind::Booth, mid - 80.0, 6.0 * TILE - 24.0, 160.0, 64.0, Some(Rect::new(0.0, 24.0, 160.0, 36.0)));
    m.spots.push(Spot { kind: SpotKind::Decks, pos: vec2(mid, 6.0 * TILE + 52.0), radius: 30.0 });
    // Speaker stacks.
    for x in [mid - 150.0, mid + 110.0, 2.0 * TILE, w as f32 * TILE - 2.0 * TILE - 40.0] {
        m.prop(PropKind::Speaker, x, 6.0 * TILE - 30.0, 40.0, 72.0, Some(Rect::new(0.0, 50.0, 40.0, 22.0)));
    }
    // Pillars down both sides.
    for i in 0..3 {
        let y = 7.0 * TILE + i as f32 * 128.0;
        for x in [3.0 * TILE, w as f32 * TILE - 4.0 * TILE] {
            m.prop(PropKind::Pillar, x, y - 120.0, 32.0, 160.0, Some(Rect::new(0.0, 146.0, 32.0, 14.0)));
        }
    }
    let door = vec2(mid, h as f32 * TILE - 14.0);
    m.spots.push(Spot { kind: SpotKind::ExitChapel, pos: door, radius: 28.0 });
    m.spawn = door + vec2(0.0, -30.0);
    m
}
