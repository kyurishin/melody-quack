use macroquad::prelude::*;

pub const ORIGIN: Vec2 = Vec2::new(100.0, 100.0);
pub const ROOM_SIZES: [(i32, i32); 5] = [(7, 5), (9, 6), (11, 7), (11, 7), (11, 9)];
pub const UPGRADES: [(&str, &str, u32); 4] = [
    ("bigger room", "9 x 6 tiles", 80),
    ("even bigger room", "11 x 7 tiles", 160),
    ("penthouse loft", "stairs up to a loft", 250),
    ("bigger penthouse", "11 x 9 tiles with a loft twice as deep", 400),
];
pub const MAX_SIZE: u8 = 4;
pub const LOFT_SIZE: u8 = 3;
pub const LOFT_H: f32 = 1.25;
pub const LOFT_X0: i32 = 2;
pub const LOFT_D: i32 = 2;
pub const STAIR_LEN: i32 = 2;
pub const CATALOGUE: (i32, i32) = (0, 0);
pub const FLOOR_Y: f32 = 0.5;

pub fn loft_depth(size: u8) -> i32 {
    if size > LOFT_SIZE { LOFT_D * 2 } else { LOFT_D }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Level {
    Floor,
    Loft,
    Stair,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Item {
    Rug,
    Bed,
    Lamp,
    Plant,
    Bookshelf,
    Sofa,
    Table,
    RecordPlayer,
    Piano,
    Drums,
    WindChime,
    Speaker,
    DjBooth,
    Subwoofer,
    RedLight,
    ConcreteBench,
    SteelPillar,
    MirrorBall,
    Arcade,
    Tv,
    Sampler,
}

pub const ITEMS: [Item; 21] = [
    Item::Rug,
    Item::Plant,
    Item::Lamp,
    Item::Table,
    Item::Bookshelf,
    Item::Sofa,
    Item::Bed,
    Item::WindChime,
    Item::Speaker,
    Item::RecordPlayer,
    Item::Drums,
    Item::Piano,
    Item::ConcreteBench,
    Item::SteelPillar,
    Item::RedLight,
    Item::Subwoofer,
    Item::DjBooth,
    Item::MirrorBall,
    Item::Arcade,
    Item::Tv,
    Item::Sampler,
];

impl Item {
    pub fn name(self) -> &'static str {
        match self {
            Item::Rug => "rug",
            Item::Bed => "double bed",
            Item::Lamp => "lamp",
            Item::Plant => "plant",
            Item::Bookshelf => "bookshelf",
            Item::Sofa => "sofa",
            Item::Table => "family table",
            Item::RecordPlayer => "record player",
            Item::Piano => "piano",
            Item::Drums => "drum kit",
            Item::WindChime => "wind chime",
            Item::Speaker => "speaker",
            Item::DjBooth => "dj booth",
            Item::Subwoofer => "subwoofer stack",
            Item::RedLight => "red light",
            Item::ConcreteBench => "concrete bench",
            Item::SteelPillar => "steel pillar",
            Item::MirrorBall => "mirror ball",
            Item::Arcade => "dance arcade",
            Item::Tv => "wall tv",
            Item::Sampler => "sampler",
        }
    }

    pub fn note(self) -> &'static str {
        match self {
            Item::Rug => "soft dance floor for bare paws",
            Item::Bed => "wide cosy bed for sweet dreams",
            Item::Lamp => "warm light",
            Item::Plant => "water it and it blooms",
            Item::Bookshelf => "little facts to read",
            Item::Sofa => "a soft seat for lounging",
            Item::Table => "serve dinner for the whole family",
            Item::RecordPlayer => "plays your friend's tunes",
            Item::Piano => "sit and play piano",
            Item::Drums => "sit and play drums",
            Item::WindChime => "rings in the breeze",
            Item::Speaker => "makes the echo bigger",
            Item::DjBooth => "techno at 132 bpm",
            Item::Subwoofer => "heavier kick",
            Item::RedLight => "dark red club light",
            Item::ConcreteBench => "cool minimalist seating",
            Item::SteelPillar => "a bright metallic clang",
            Item::MirrorBall => "a spinning chandelier of light spots",
            Item::Arcade => "dance to the music on four arrows",
            Item::Tv => "five channels of visuals that move with the music",
            Item::Sampler => "build a beat on a step grid, with your own recorded sounds",
        }
    }

    pub fn price(self) -> u32 {
        match self {
            Item::Rug => 20,
            Item::Bed => 80,
            Item::Lamp => 25,
            Item::Plant => 15,
            Item::Bookshelf => 45,
            Item::Sofa => 50,
            Item::Table => 45,
            Item::RecordPlayer => 80,
            Item::Piano => 120,
            Item::Drums => 90,
            Item::WindChime => 35,
            Item::Speaker => 70,
            Item::DjBooth => 150,
            Item::Subwoofer => 90,
            Item::RedLight => 40,
            Item::ConcreteBench => 35,
            Item::SteelPillar => 30,
            Item::MirrorBall => 300,
            Item::Arcade => 500,
            Item::Tv => 1000,
            Item::Sampler => 180,
        }
    }

    pub fn size(self) -> (i32, i32) {
        match self {
            Item::Rug => (2, 2),
            Item::Bed => (2, 2),
            Item::Sofa | Item::Piano | Item::DjBooth | Item::ConcreteBench | Item::Table | Item::Tv => (2, 1),
            _ => (1, 1),
        }
    }

    pub fn solid(self) -> bool {
        !matches!(self, Item::Rug | Item::WindChime | Item::RedLight | Item::MirrorBall | Item::Tv)
    }

    pub fn index(self) -> u8 {
        ITEMS.iter().position(|i| *i == self).unwrap_or(0) as u8
    }

    pub fn from_index(i: u8) -> Item {
        ITEMS[(i as usize).min(ITEMS.len() - 1)]
    }
}

pub const WALLS: [(&str, Color); 5] = [
    ("cream wall", Color::new(0.93, 0.86, 0.72, 1.0)),
    ("mint wall", Color::new(0.68, 0.86, 0.76, 1.0)),
    ("rose wall", Color::new(0.93, 0.7, 0.74, 1.0)),
    ("night wall", Color::new(0.34, 0.33, 0.55, 1.0)),
    ("concrete wall", Color::new(0.56, 0.56, 0.57, 1.0)),
];

pub const CONCRETE_WALL: u8 = 4;
pub const CONCRETE_FLOOR: u8 = 3;

pub const FLOORS: [(&str, Color, Color); 4] = [
    ("oak floor", Color::new(0.74, 0.55, 0.36, 1.0), Color::new(0.68, 0.5, 0.32, 1.0)),
    ("cherry floor", Color::new(0.6, 0.34, 0.26, 1.0), Color::new(0.54, 0.3, 0.23, 1.0)),
    ("checker floor", Color::new(0.92, 0.9, 0.86, 1.0), Color::new(0.3, 0.3, 0.36, 1.0)),
    ("concrete floor", Color::new(0.4, 0.4, 0.42, 1.0), Color::new(0.37, 0.37, 0.39, 1.0)),
];

pub fn wall_bit(i: usize) -> u16 {
    1 << if i < 4 { i } else { 7 }
}

pub fn floor_bit(i: usize) -> u16 {
    1 << if i < 3 { 4 + i } else { 8 }
}

pub fn shop_rows() -> usize {
    ITEMS.len() + WALLS.len() + FLOORS.len() + 2
}

#[derive(Clone, Copy)]
pub enum Row {
    Upgrade,
    Item(Item),
    Wall(usize),
    Floor(usize),
    Set,
}

pub fn row(r: usize) -> Row {
    if r == 0 {
        return Row::Upgrade;
    }
    let r = r - 1;
    if r < ITEMS.len() {
        Row::Item(ITEMS[r])
    } else if r < ITEMS.len() + WALLS.len() {
        Row::Wall(r - ITEMS.len())
    } else if r < ITEMS.len() + WALLS.len() + FLOORS.len() {
        Row::Floor(r - ITEMS.len() - WALLS.len())
    } else {
        Row::Set
    }
}

pub const STYLE_PRICE: u32 = 10;
pub const WITHERED: u8 = 4;

#[derive(Clone, Copy)]
pub struct Piece {
    pub item: Item,
    pub x: i32,
    pub z: i32,
    pub rot: u8,
    pub state: u8,
    pub day: i32,
}

impl Piece {
    pub fn footprint(&self) -> (i32, i32) {
        let (w, d) = self.item.size();
        if self.rot % 2 == 1 {
            (d, w)
        } else {
            (w, d)
        }
    }

    pub fn covers(&self, x: i32, z: i32) -> bool {
        let (w, d) = self.footprint();
        x >= self.x && x < self.x + w && z >= self.z && z < self.z + d
    }

    pub fn center(&self) -> Vec2 {
        let (w, d) = self.footprint();
        vec2(self.x as f32 + w as f32 / 2.0, self.z as f32 + d as f32 / 2.0)
    }

    pub fn yaw(&self) -> f32 {
        self.rot as f32 * std::f32::consts::FRAC_PI_2
    }
}

pub struct Home {
    pub pieces: Vec<Piece>,
    pub wall: u8,
    pub floor: u8,
    pub owned_styles: u16,
    pub record_on: bool,
    pub size: u8,
}

impl Home {
    pub fn new() -> Home {
        Home { pieces: Vec::new(), wall: 0, floor: 0, owned_styles: 1 | (1 << 4), record_on: true, size: 0 }
    }

    pub fn w(&self) -> i32 {
        ROOM_SIZES[(self.size as usize).min(ROOM_SIZES.len() - 1)].0
    }

    pub fn d(&self) -> i32 {
        ROOM_SIZES[(self.size as usize).min(ROOM_SIZES.len() - 1)].1
    }

    pub fn loft(&self) -> bool {
        self.size >= LOFT_SIZE
    }

    pub fn loft_d(&self) -> i32 {
        loft_depth(self.size)
    }

    pub fn door(&self) -> Vec2 {
        vec2(self.door_x() as f32 + 0.5, self.d() as f32 - 0.45)
    }

    pub fn door_x(&self) -> i32 {
        self.w() / 2
    }

    pub fn stair_x(&self) -> i32 {
        self.w() - 1
    }

    pub fn level(&self, x: i32, z: i32) -> Level {
        if !self.loft() {
            return Level::Floor;
        }
        let ld = self.loft_d();
        if z < ld && x >= LOFT_X0 {
            Level::Loft
        } else if x == self.stair_x() && z >= ld && z < ld + STAIR_LEN {
            Level::Stair
        } else {
            Level::Floor
        }
    }

    pub fn height_at(&self, p: Vec2) -> f32 {
        let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
        match self.level(x, z) {
            Level::Floor => 0.0,
            Level::Loft => LOFT_H,
            Level::Stair => LOFT_H * ((self.loft_d() + STAIR_LEN) as f32 - p.y).clamp(0.0, STAIR_LEN as f32) / STAIR_LEN as f32,
        }
    }

    pub fn piece_lift(&self, piece: &Piece) -> f32 {
        let c = piece.center();
        if self.level(c.x.floor() as i32, c.y.floor() as i32) == Level::Loft || self.level((c.x - 0.01).floor() as i32, (c.y - 0.01).floor() as i32) == Level::Loft {
            LOFT_H
        } else {
            0.0
        }
    }

    pub fn next_upgrade(&self) -> Option<(&'static str, &'static str, u32)> {
        UPGRADES.get(self.size as usize).copied()
    }

    pub fn in_the_way(&self) -> Vec<usize> {
        let target = self.size + 1;
        if target < LOFT_SIZE || target > MAX_SIZE {
            return Vec::new();
        }
        let w = ROOM_SIZES[target as usize].0;
        let stair_x = w - 1;
        let ld = loft_depth(target);
        let mut out = Vec::new();
        for (i, p) in self.pieces.iter().enumerate() {
            let (fw, fd) = p.footprint();
            let mut loft = false;
            let mut floor = false;
            let mut stairs = false;
            for z in p.z..p.z + fd {
                for x in p.x..p.x + fw {
                    if x == stair_x && z >= ld && z < ld + STAIR_LEN {
                        stairs = true;
                    }
                    if z < ld && x >= LOFT_X0 {
                        loft = true;
                    } else {
                        floor = true;
                    }
                }
            }
            if stairs || (loft && floor) || (p.item.solid() && p.covers(stair_x, ld - 1)) {
                out.push(i);
            }
        }
        out
    }

    pub fn piece_at(&self, x: i32, z: i32) -> Option<usize> {
        self.pieces.iter().rposition(|p| p.covers(x, z) && p.item.solid()).or_else(|| self.pieces.iter().rposition(|p| p.covers(x, z)))
    }

    pub fn blocked_tile(&self, x: i32, z: i32) -> bool {
        if x < 0 || z < 0 || x >= self.w() || z >= self.d() {
            return true;
        }
        if (x, z) == CATALOGUE {
            return true;
        }
        self.pieces.iter().any(|p| p.item.solid() && p.covers(x, z))
    }

    pub fn walkable(&self, local: Vec2, radius: f32) -> bool {
        let (w, d) = (self.w() as f32, self.d() as f32);
        let door = self.door();
        let mut low = f32::MAX;
        let mut high = f32::MIN;
        for (ox, oz) in [(-radius, -radius), (radius, -radius), (-radius, radius), (radius, radius)] {
            let p = local + vec2(ox, oz);
            if p.x < 0.05 || p.y < 0.05 || p.x > w - 0.05 || p.y > d + 0.4 {
                return false;
            }
            if p.y >= d {
                if (p.x - door.x).abs() > 0.45 {
                    return false;
                }
                continue;
            }
            let h = self.height_at(p);
            low = low.min(h);
            high = high.max(h);
            if high - low > 0.35 {
                return false;
            }
            if self.blocked_tile(p.x.floor() as i32, p.y.floor() as i32) {
                return false;
            }
        }
        true
    }

    pub fn can_place(&self, piece: &Piece, standing: &[Vec2]) -> bool {
        let (w, d) = piece.footprint();
        for z in piece.z..piece.z + d {
            for x in piece.x..piece.x + w {
                if x < 0 || z < 0 || x >= self.w() || z >= self.d() || (x, z) == CATALOGUE {
                    return false;
                }
                if x == self.door_x() && z == self.d() - 1 && piece.item.solid() {
                    return false;
                }
                if self.level(x, z) == Level::Stair || self.level(x, z) != self.level(piece.x, piece.z) {
                    return false;
                }
                if piece.item.solid() && self.loft() && z == self.loft_d() - 1 && x == self.stair_x() {
                    return false;
                }
                let clash = self.pieces.iter().any(|p| p.covers(x, z) && (p.item.solid() == piece.item.solid() || (piece.item.solid() && p.item == Item::WindChime)));
                if clash {
                    return false;
                }
                if piece.item.solid() && standing.iter().any(|s| s.x.floor() as i32 == x && s.y.floor() as i32 == z) {
                    return false;
                }
            }
        }
        true
    }

    pub fn has(&self, item: Item) -> bool {
        self.pieces.iter().any(|p| p.item == item)
    }

    pub fn switched_on(&self, item: Item) -> bool {
        self.pieces.iter().any(|p| p.item == item && p.state == 0)
    }



    pub fn techno_club(&mut self, standing: &[Vec2]) -> usize {
        self.wall = CONCRETE_WALL;
        self.floor = CONCRETE_FLOOR;
        self.owned_styles |= wall_bit(CONCRETE_WALL as usize) | floor_bit(CONCRETE_FLOOR as usize);
        let wanted = [
            (Item::DjBooth, 2, 0, 0),
            (Item::Subwoofer, 1, 0, 0),
            (Item::Subwoofer, 4, 0, 0),
            (Item::RedLight, 5, 0, 0),
            (Item::RedLight, 6, 2, 0),
            (Item::SteelPillar, 1, 2, 0),
            (Item::SteelPillar, 5, 2, 0),
            (Item::ConcreteBench, 0, 3, 1),
            (Item::ConcreteBench, 5, 4, 0),
        ];
        let mut added = 0;
        for (item, x, z, rot) in wanted {
            let piece = Piece { item, x, z, rot, state: 0, day: 0 };
            if self.level(x, z) != Level::Floor {
                continue;
            }
            if self.can_place(&piece, standing) {
                self.pieces.push(piece);
                added += 1;
            }
        }
        added
    }

    pub fn free_spot(&self, seed: f32) -> Vec2 {
        let mut spots = Vec::new();
        for z in 0..self.d() {
            for x in 0..self.w() {
                if !self.blocked_tile(x, z) && self.level(x, z) == Level::Floor {
                    spots.push(vec2(x as f32 + 0.5, z as f32 + 0.5));
                }
            }
        }
        if spots.is_empty() {
            return self.door();
        }
        spots[((seed.abs() * 997.0) as usize) % spots.len()]
    }
}

pub fn world(local: Vec2) -> Vec2 {
    ORIGIN + local
}

pub fn local(world: Vec2) -> Vec2 {
    world - ORIGIN
}

pub fn wall_color(home: &Home) -> Color {
    WALLS[(home.wall as usize).min(WALLS.len() - 1)].1
}

impl Home {
    pub fn free_spot_near(&self, local: Vec2) -> Vec2 {
        let mut best = self.door() - vec2(0.0, 0.4);
        let mut best_d = f32::MAX;
        for z in 0..self.d() {
            for x in 0..self.w() {
                if self.blocked_tile(x, z) || self.level(x, z) == Level::Stair {
                    continue;
                }
                let spot = vec2(x as f32 + 0.5, z as f32 + 0.5);
                let d = spot.distance(local);
                if d < best_d {
                    best_d = d;
                    best = spot;
                }
            }
        }
        best
    }
}
