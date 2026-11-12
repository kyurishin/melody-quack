//! All sprites are drawn in code at 2x the usual pixel-art density
//! (32 px tiles, 32x40 characters), in the cover's palette, then outlined in
//! its dark ink.

use crate::gfx::{bake_bang, bake_text, hex, mix, pal, Canvas};
use macroquad::prelude::*;

pub const TILE: f32 = 32.0;
pub const CHAR_W: f32 = 32.0;
pub const CHAR_H: f32 = 40.0;

pub struct Critter {
    pub name: &'static str,
    pub fur: Color,
    pub hat: Color,
}

pub const CRITTERS: [Critter; 8] = [
    Critter { name: "cat", fur: hex(0xf6e6c4), hat: hex(0x7a4fd6) },
    Critter { name: "labrador", fur: hex(0xe2b56b), hat: hex(0x3f9e5e) },
    Critter { name: "duck", fur: hex(0xffd66e), hat: hex(0x4f7fe0) },
    Critter { name: "frog", fur: hex(0x8cdea8), hat: hex(0xe85d9a) },
    Critter { name: "bunny", fur: hex(0xf4f0ff), hat: hex(0xff8062) },
    Critter { name: "fox", fur: hex(0xff9150), hat: hex(0x2a9d9a) },
    Critter { name: "bear", fur: hex(0xa8724f), hat: hex(0xe8b830) },
    Critter { name: "axolotl", fur: hex(0xffb3c8), hat: hex(0x5a3fb0) },
];

/// Frames: 0 idle, 1-2 walk, 3 arms up, 4 lean left, 5 lean right, 6 seated (back).
pub const FRAMES: usize = 7;

pub struct Sprites {
    pub critters: Vec<Vec<Texture2D>>,
    pub grass: [Texture2D; 3],
    pub path: Texture2D,
    pub plaza: Texture2D,
    pub water: [Texture2D; 2],
    pub carpet: Texture2D,
    pub chapel_floor: Texture2D,
    pub tree: [Texture2D; 3],
    pub pine: Texture2D,
    pub cottage: Texture2D,
    pub pcbang: Texture2D,
    pub chapel: Texture2D,
    pub hoarding: Texture2D,
    pub lamp: Texture2D,
    pub log: Texture2D,
    pub bush: Texture2D,
    pub flowers: Texture2D,
    pub desk: Texture2D,
    pub chair_back: Texture2D,
    pub counter: Texture2D,
    pub vending: Texture2D,
    pub booth: Texture2D,
    pub speaker: Texture2D,
    pub pillar: Texture2D,
    pub bouncer: Texture2D,
    pub owner: Texture2D,
    pub icon: Texture2D,
}

impl Sprites {
    pub fn build() -> Sprites {
        let critters = (0..CRITTERS.len())
            .map(|k| (0..FRAMES).map(|f| critter(k, f).texture()).collect())
            .collect();
        Sprites {
            critters,
            grass: [grass(1).texture(), grass(2).texture(), grass(3).texture()],
            path: path().texture(),
            plaza: plaza().texture(),
            water: [water(0).texture(), water(1).texture()],
            carpet: carpet().texture(),
            chapel_floor: chapel_floor().texture(),
            tree: [
                tree(hex(0xe8743c), hex(0xb84a2c), 1).texture(),
                tree(hex(0xf2b33d), hex(0xc47f24), 2).texture(),
                tree(hex(0xd9503f), hex(0x9c2f34), 3).texture(),
            ],
            pine: pine().texture(),
            cottage: cottage().texture(),
            pcbang: pcbang().texture(),
            chapel: chapel().texture(),
            hoarding: hoarding().texture(),
            lamp: lamp().texture(),
            log: log_seat().texture(),
            bush: bush().texture(),
            flowers: flowers().texture(),
            desk: desk().texture(),
            chair_back: chair_back().texture(),
            counter: counter().texture(),
            vending: vending().texture(),
            booth: dj_booth().texture(),
            speaker: speaker().texture(),
            pillar: pillar().texture(),
            bouncer: bouncer().texture(),
            owner: owner().texture(),
            icon: app_icon().texture(),
        }
    }
}

// ---------------------------------------------------------------------------
// Characters
// ---------------------------------------------------------------------------

fn shade(c: Color, k: f32) -> Color {
    mix(c, pal::INK, k)
}

fn light(c: Color, k: f32) -> Color {
    mix(c, pal::WHITE, k)
}

pub fn critter(kind: usize, frame: usize) -> Canvas {
    let cr = &CRITTERS[kind % CRITTERS.len()];
    let mut c = Canvas::new(32, 40);
    let fur = cr.fur;
    let fur_d = shade(fur, 0.22);
    let hat = cr.hat;
    let hat_d = shade(hat, 0.3);
    let robe = shade(hat, 0.12);
    let bob = if frame == 1 || frame == 2 { 1 } else { 0 };
    let lean = match frame {
        4 => -1,
        5 => 1,
        _ => 0,
    };
    let back = frame == 6;
    let oy = bob; // body offset

    // Feet.
    let (lf, rf) = match frame {
        1 => (-1, 1),
        2 => (1, -1),
        _ => (0, 0),
    };
    if !back {
        c.ellipse(12.0, 37.0 + lf as f32, 3.5, 2.2, fur_d);
        c.ellipse(20.0, 37.0 + rf as f32, 3.5, 2.2, fur_d);
    }

    // Robe / body.
    let bx = lean;
    c.ellipse(16.0 + bx as f32, 31.0 + oy as f32, 8.0, 7.0, robe);
    c.rect(9 + bx, 30 + oy, 15, 6, robe);
    c.hline(10 + bx, 35 + oy, 13, hat_d);
    // Belt / sash with a star.
    c.hline(9 + bx, 30 + oy, 15, hat_d);
    c.set(16 + bx, 32 + oy, pal::YELLOW);
    c.set(15 + bx, 33 + oy, pal::YELLOW);
    c.set(17 + bx, 33 + oy, pal::YELLOW);

    // Arms.
    if frame == 3 {
        c.rect(5 + bx, 18 + oy, 3, 9, robe);
        c.rect(25 + bx, 18 + oy, 3, 9, robe);
        c.circle(6.5 + bx as f32, 17.0 + oy as f32, 2.2, fur);
        c.circle(26.5 + bx as f32, 17.0 + oy as f32, 2.2, fur);
    } else if !back {
        let swing = match frame {
            1 => 1,
            2 => -1,
            _ => 0,
        };
        c.rect(7 + bx, 28 + oy + swing, 3, 6, robe);
        c.rect(23 + bx, 28 + oy - swing, 3, 6, robe);
        c.circle(8.5 + bx as f32, 34.0 + (oy + swing) as f32, 1.8, fur);
        c.circle(24.5 + bx as f32, 34.0 + (oy - swing) as f32, 1.8, fur);
    }

    // Head.
    let hx = 16.0 + lean as f32 * 1.5;
    let hy = 20.0 + oy as f32;
    let name = cr.name;
    // Ears behind the head / hat.
    match name {
        "cat" | "fox" => {
            c.tri((hx - 9.0, hy - 2.0), (hx - 7.0, hy - 11.0), (hx - 2.0, hy - 5.0), fur);
            c.tri((hx + 9.0, hy - 2.0), (hx + 7.0, hy - 11.0), (hx + 2.0, hy - 5.0), fur);
            c.tri((hx - 7.5, hy - 4.0), (hx - 6.5, hy - 8.5), (hx - 4.0, hy - 5.0), pal::PINK);
            c.tri((hx + 7.5, hy - 4.0), (hx + 6.5, hy - 8.5), (hx + 4.0, hy - 5.0), pal::PINK);
        }
        "bunny" => {
            c.ellipse(hx - 4.0, hy - 13.0, 2.5, 7.0, fur);
            c.ellipse(hx + 4.0, hy - 13.0, 2.5, 7.0, fur);
            c.ellipse(hx - 4.0, hy - 13.0, 1.0, 5.0, pal::PINK);
            c.ellipse(hx + 4.0, hy - 13.0, 1.0, 5.0, pal::PINK);
        }
        "bear" => {
            c.circle(hx - 7.0, hy - 6.0, 3.0, fur);
            c.circle(hx + 7.0, hy - 6.0, 3.0, fur);
        }
        "axolotl" => {
            for i in 0..3 {
                let yy = hy - 3.0 + i as f32 * 3.0;
                c.ellipse(hx - 10.5, yy, 2.5, 1.2, hex(0xff6f9e));
                c.ellipse(hx + 10.5, yy, 2.5, 1.2, hex(0xff6f9e));
            }
        }
        _ => {}
    }
    let (hw, hh) = match name {
        "duck" => (7.5, 7.0),
        "frog" => (9.0, 6.5),
        _ => (8.0, 7.0),
    };
    c.ellipse(hx, hy, hw, hh, fur);
    // Soft shading on the lower-right of the face.
    if !back {
        c.ellipse(hx + 3.0, hy + 4.0, 4.0, 2.0, light(fur, 0.15));
    }
    if name == "labrador" {
        // Floppy ears.
        c.ellipse(hx - 8.0, hy + 1.0, 2.6, 5.0, fur_d);
        c.ellipse(hx + 8.0, hy + 1.0, 2.6, 5.0, fur_d);
    }
    if name == "fox" && !back {
        c.ellipse(hx, hy + 3.5, 5.5, 3.0, pal::CREAM);
    }

    if !back {
        // Eyes.
        let ey = hy + 1.0;
        let (lx, rx) = (hx - 3.5, hx + 3.0);
        if name == "frog" {
            c.circle(lx - 1.0, ey - 4.0, 3.0, fur);
            c.circle(rx + 1.0, ey - 4.0, 3.0, fur);
            c.rect(lx as i32 - 2, ey as i32 - 5, 2, 3, pal::INK);
            c.rect(rx as i32, ey as i32 - 5, 2, 3, pal::INK);
        } else {
            c.rect(lx as i32, ey as i32 - 1, 2, 3, pal::INK);
            c.rect(rx as i32, ey as i32 - 1, 2, 3, pal::INK);
            c.set(lx as i32 + 1, ey as i32 - 1, pal::WHITE);
            c.set(rx as i32 + 1, ey as i32 - 1, pal::WHITE);
        }
        // Blush.
        c.hline(lx as i32 - 2, ey as i32 + 3, 2, pal::PINK);
        c.hline(rx as i32 + 2, ey as i32 + 3, 2, pal::PINK);
        // Snout / beak.
        match name {
            "duck" => {
                c.ellipse(hx, hy + 4.5, 4.5, 2.0, hex(0xff9a3d));
                c.hline(hx as i32 - 3, hy as i32 + 4, 6, hex(0xd9701f));
            }
            "labrador" | "bear" => {
                c.ellipse(hx, hy + 4.0, 3.5, 2.5, light(fur, 0.35));
                c.rect(hx as i32 - 1, hy as i32 + 2, 3, 2, pal::INK);
            }
            "frog" => {
                c.hline(hx as i32 - 3, hy as i32 + 4, 7, shade(fur, 0.5));
            }
            _ => {
                c.set(hx as i32, hy as i32 + 3, pal::PINK);
                c.set(hx as i32 - 1, hy as i32 + 4, pal::INK);
                c.set(hx as i32 + 1, hy as i32 + 4, pal::INK);
            }
        }
    }

    // Wizard hat: brim and a cone that flops to one side.
    let brim_y = hy - 6.0;
    c.ellipse(hx, brim_y, 11.0, 2.6, hat_d);
    c.ellipse(hx, brim_y - 0.5, 10.0, 2.0, hat);
    let tip_x = hx + 6.0 + lean as f32 * 2.0;
    c.tri((hx - 7.0, brim_y - 1.0), (hx + 7.0, brim_y - 1.0), (tip_x - 2.0, brim_y - 13.0), hat);
    c.tri((tip_x - 4.0, brim_y - 10.0), (tip_x - 1.0, brim_y - 13.5), (tip_x + 3.0, brim_y - 11.0), hat);
    c.set(tip_x as i32 + 3, brim_y as i32 - 11, pal::YELLOW);
    // Hat band and a little star.
    c.hline(hx as i32 - 7, brim_y as i32 - 2, 14, hat_d);
    c.set(hx as i32 - 1, brim_y as i32 - 6, pal::YELLOW);
    c.set(hx as i32 - 2, brim_y as i32 - 5, pal::YELLOW);
    c.set(hx as i32, brim_y as i32 - 5, pal::YELLOW);
    c.set(hx as i32 - 1, brim_y as i32 - 4, pal::YELLOW);
    // Highlight on the cone.
    c.line(hx as i32 - 4, brim_y as i32 - 3, tip_x as i32 - 3, brim_y as i32 - 11, light(hat, 0.25));

    if back {
        // Seated from behind: chair hides the legs.
        c.ellipse(hx, hy + 2.0, hw - 1.0, hh - 1.0, fur_d);
    }

    c.outline(pal::INK);
    c
}

// ---------------------------------------------------------------------------
// Tiles
// ---------------------------------------------------------------------------

fn grass(seed: u32) -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, pal::GRASS);
    c.speckle(0, 0, 32, 32, pal::GRASS_D, 0.12, seed);
    c.speckle(0, 0, 32, 32, pal::GRASS_L, 0.05, seed * 7 + 1);
    // Tufts.
    let spots = [(5, 8), (20, 4), (12, 22), (26, 18), (3, 27)];
    for (i, (x, y)) in spots.iter().enumerate() {
        if (seed as usize + i) % 2 == 0 {
            c.set(*x, *y, pal::GRASS_L);
            c.set(x + 2, *y, pal::GRASS_L);
            c.set(x + 1, y - 1, pal::GRASS_L);
        }
    }
    if seed == 3 {
        c.set(14, 10, pal::PINK);
        c.set(24, 26, pal::YELLOW);
        c.set(8, 16, pal::CREAM);
    }
    c
}

fn path() -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, pal::PATH);
    // Cobbles.
    for (i, (x, y, w, h)) in [(1, 1, 9, 7), (12, 1, 8, 7), (22, 1, 9, 7), (1, 10, 6, 7), (9, 10, 10, 7), (21, 10, 10, 7), (1, 19, 11, 6), (14, 19, 7, 6), (23, 19, 8, 6), (1, 27, 8, 5), (11, 27, 11, 5), (24, 27, 7, 5)]
        .iter()
        .enumerate()
    {
        let col = if i % 3 == 0 { pal::PATH_D } else { mix(pal::PATH, pal::CREAM, 0.12) };
        c.rect(*x, *y, *w, *h, col);
        c.hline(*x, y + h - 1, *w, mix(pal::PATH_D, pal::INK, 0.15));
    }
    c
}

fn plaza() -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, hex(0x9d8fa8));
    c.frame(0, 0, 16, 16, hex(0x7f7290));
    c.frame(16, 16, 16, 16, hex(0x7f7290));
    c.frame(16, 0, 16, 16, hex(0x8a7d9a));
    c.frame(0, 16, 16, 16, hex(0x8a7d9a));
    c.speckle(0, 0, 32, 32, hex(0xb3a6bd), 0.04, 9);
    c
}

fn water(frame: i32) -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, hex(0x3d5fa8));
    c.speckle(0, 0, 32, 32, hex(0x34509a), 0.2, 4);
    for (x, y) in [(4, 6), (18, 12), (8, 22), (24, 26)] {
        let xx = x + frame * 2;
        c.hline(xx, y, 5, pal::SKY);
        c.hline(xx + 1, y + 1, 3, hex(0x74acff));
    }
    c
}

fn carpet() -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, hex(0x2a2550));
    for y in (0..32).step_by(8) {
        for x in (0..32).step_by(8) {
            let col = if (x / 8 + y / 8) % 2 == 0 { hex(0x3a3070) } else { hex(0x2f2a5e) };
            c.rect(x + 2, y + 2, 4, 4, col);
        }
    }
    c.set(5, 5, pal::CYAN);
    c.set(21, 13, pal::MAGENTA);
    c.set(13, 29, pal::ACID);
    c
}

fn chapel_floor() -> Canvas {
    let mut c = Canvas::new(32, 32);
    c.rect(0, 0, 32, 32, hex(0x221f2e));
    c.speckle(0, 0, 32, 32, hex(0x2a2638), 0.18, 21);
    c.hline(0, 31, 32, hex(0x1d1a28));
    c.vline(31, 0, 32, hex(0x1d1a28));
    c
}

// ---------------------------------------------------------------------------
// Props & buildings
// ---------------------------------------------------------------------------

fn tree(leaf: Color, leaf_d: Color, seed: u32) -> Canvas {
    let mut c = Canvas::new(64, 80);
    c.rect(28, 50, 8, 26, pal::WOOD_D);
    c.vline(30, 52, 22, pal::WOOD);
    c.ellipse(32.0, 76.0, 10.0, 3.0, pal::WOOD_D);
    c.circle(32.0, 30.0, 24.0, leaf_d);
    c.circle(22.0, 38.0, 14.0, leaf_d);
    c.circle(43.0, 38.0, 14.0, leaf_d);
    c.circle(30.0, 26.0, 21.0, leaf);
    c.circle(20.0, 34.0, 11.0, leaf);
    c.circle(42.0, 33.0, 11.0, leaf);
    c.circle(26.0, 18.0, 9.0, light(leaf, 0.2));
    c.speckle(8, 6, 50, 50, leaf_d, 0.06, seed);
    c.speckle(8, 6, 40, 30, light(leaf, 0.35), 0.03, seed + 5);
    // Keep speckles inside the canopy only.
    c.keep(|x, y| {
        let d = |cx: i32, cy: i32, r: i32| (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r;
        d(32, 30, 24) || d(22, 38, 14) || d(43, 38, 14) || (x >= 22 && x < 42 && y >= 50)
    });
    c.outline(pal::INK);
    c
}

fn pine() -> Canvas {
    let mut c = Canvas::new(48, 80);
    c.rect(21, 62, 6, 14, pal::WOOD_D);
    for (i, w) in [8.0, 13.0, 18.0, 22.0].iter().enumerate() {
        let y = 6.0 + i as f32 * 14.0;
        c.tri((24.0, y), (24.0 - w, y + 20.0), (24.0 + w, y + 20.0), hex(0x2c5a4a));
        c.tri((24.0, y + 2.0), (24.0 - w * 0.6, y + 16.0), (24.0 + w * 0.3, y + 16.0), hex(0x3c7360));
    }
    c.outline(pal::INK);
    c
}

fn cottage() -> Canvas {
    let mut c = Canvas::new(144, 128);
    // Walls: timber frame.
    c.rect(14, 56, 116, 64, pal::CREAM);
    c.speckle(14, 56, 116, 64, pal::BEIGE, 0.15, 3);
    for x in [14, 50, 92, 126] {
        c.rect(x, 56, 4, 64, pal::WOOD_D);
    }
    c.rect(14, 84, 116, 4, pal::WOOD_D);
    // Roof.
    c.tri((4.0, 60.0), (72.0, 8.0), (140.0, 60.0), hex(0xc4572e));
    c.tri((14.0, 58.0), (72.0, 16.0), (130.0, 58.0), hex(0xe0703c));
    for y in (20..58).step_by(6) {
        c.hline(72 - (y - 10), y, (y - 10) * 2, hex(0xc4572e));
    }
    // Chimney.
    c.rect(102, 18, 12, 26, hex(0x8a4a3a));
    c.rect(100, 16, 16, 4, hex(0x6b3a2e));
    // Windows (lit).
    for x in [24, 100] {
        c.rect(x, 64, 20, 16, pal::YELLOW);
        c.rect(x + 2, 66, 16, 12, hex(0xffe9a8));
        c.vline(x + 10, 64, 16, pal::WOOD_D);
        c.hline(x, 72, 20, pal::WOOD_D);
    }
    // Door.
    c.rect(62, 88, 20, 32, pal::WOOD);
    c.rect(64, 90, 16, 30, hex(0x9c6a48));
    c.set(76, 104, pal::YELLOW);
    c.ellipse(72.0, 90.0, 10.0, 4.0, pal::WOOD);
    c.outline(pal::INK);
    c
}

fn pcbang() -> Canvas {
    let mut c = Canvas::new(208, 176);
    // Building body: dark tiles, Seoul side-street vibe.
    c.rect(4, 28, 200, 144, hex(0x4a4566));
    for y in (28..172).step_by(8) {
        c.hline(4, y, 200, hex(0x403b5a));
    }
    // Roof parapet + AC units.
    c.rect(0, 22, 208, 8, hex(0x2f2b45));
    c.rect(20, 8, 26, 14, hex(0xb8b6c4));
    c.rect(22, 10, 22, 10, hex(0x8f8da0));
    c.circle(33.0, 15.0, 4.0, hex(0x5f5d70));
    c.rect(160, 12, 22, 10, hex(0xb8b6c4));
    // Big sign board.
    c.rect(16, 34, 176, 44, pal::BLACK);
    c.frame(16, 34, 176, 44, pal::MAGENTA);
    c.frame(18, 36, 172, 40, hex(0x7a1f76));
    bake_text(&mut c, 30, 42, "PC", pal::CYAN, 4);
    bake_bang(&mut c, 84, 41, pal::MAGENTA, 3);
    bake_text(&mut c, 126, 44, "24H", pal::ACID, 3);
    bake_text(&mut c, 126, 64, "...", pal::ACID, 1);
    // Upper windows with blinds glowing blue from monitors.
    for x in [16, 64, 112, 160] {
        c.rect(x, 86, 36, 24, hex(0x1c2650));
        for y in (88..110).step_by(3) {
            c.hline(x + 2, y, 32, hex(0x3b5bb8));
        }
        c.frame(x, 86, 36, 24, hex(0x26223a));
    }
    // Storefront glass.
    c.rect(12, 120, 184, 52, hex(0x1a1835));
    c.rect(14, 122, 70, 48, hex(0x253070));
    c.rect(124, 122, 70, 48, hex(0x253070));
    // Silhouettes of CRTs in the window.
    for x in [20, 48, 130, 158] {
        c.rect(x, 140, 20, 16, pal::BEIGE_D);
        c.rect(x + 3, 143, 14, 10, pal::CYAN);
        c.rect(x + 6, 156, 8, 4, pal::BEIGE_D);
    }
    // Door.
    c.rect(88, 124, 32, 48, hex(0x30345a));
    c.rect(90, 126, 13, 44, hex(0x6f8fd8));
    c.rect(105, 126, 13, 44, hex(0x6f8fd8));
    c.vline(103, 126, 44, hex(0x30345a));
    c.rect(98, 146, 2, 8, pal::W95_GREY);
    c.rect(108, 146, 2, 8, pal::W95_GREY);
    // Little "OPEN" light box.
    c.rect(130, 112, 34, 7, pal::BLACK);
    bake_text(&mut c, 132, 113, "OPEN", pal::CORAL, 1);
    c.outline(pal::INK);
    c
}

fn chapel() -> Canvas {
    // Brutalist concrete mass with gothic bones.
    let (w, h) = (320, 304);
    let mut c = Canvas::new(w, h);
    let conc = pal::CONCRETE;
    let conc_d = pal::CONCRETE_D;
    let conc_dd = pal::CONCRETE_DD;
    // Tower.
    c.rect(124, 8, 72, 120, conc_d);
    c.rect(130, 8, 60, 120, conc);
    // Tower slot window with a glowing strip.
    c.rect(152, 24, 16, 64, pal::BLACK);
    c.rect(156, 28, 8, 56, pal::MAGENTA);
    c.rect(158, 30, 4, 52, hex(0xffb0f8));
    // Tower crown: blocky crenellations.
    for x in (124..196).step_by(12) {
        c.rect(x, 0, 8, 10, conc_d);
    }
    // Main mass.
    c.rect(8, 96, 304, 204, conc_d);
    c.rect(16, 96, 288, 204, conc);
    // Board-formed concrete: faint horizontal lines and tie holes.
    for y in (100..300).step_by(10) {
        c.hline(16, y, 288, mix(conc, conc_d, 0.35));
    }
    for y in (104..300).step_by(20) {
        for x in (24..300).step_by(24) {
            c.set(x, y, conc_dd);
        }
    }
    c.speckle(16, 96, 288, 204, mix(conc, pal::WHITE, 0.12), 0.03, 77);
    // Heavy cornice.
    c.rect(0, 88, 320, 12, conc_dd);
    c.rect(0, 88, 320, 4, conc_d);
    // Buttress fins.
    for x in [8, 64, 248, 304] {
        c.rect(x, 100, 10, 200, conc_d);
        c.vline(x, 100, 200, conc_dd);
    }
    // Lancet windows (pointed arches) with stained-glass glow.
    let lancet = |c: &mut Canvas, x: i32, y: i32, ww: i32, hh: i32, glow: Color, glow2: Color| {
        let cx = x as f32 + ww as f32 / 2.0;
        c.rect(x - 3, y + ww / 2, ww + 6, hh - ww / 2 + 3, conc_dd);
        c.tri((x as f32 - 3.0, (y + ww / 2) as f32), (cx, y as f32 - 8.0), ((x + ww + 3) as f32, (y + ww / 2) as f32), conc_dd);
        c.rect(x, y + ww / 2, ww, hh - ww / 2, glow);
        c.tri((x as f32, (y + ww / 2) as f32), (cx, y as f32 - 4.0), ((x + ww) as f32, (y + ww / 2) as f32), glow);
        // Leading.
        c.vline(x + ww / 2, y, hh, pal::INK);
        for yy in (y + ww / 2 + 6..y + hh).step_by(10) {
            c.hline(x, yy, ww, pal::INK);
        }
        c.dither(x + 1, y + hh / 2, ww / 2 - 1, hh / 2, glow, glow2, 0.5);
    };
    lancet(&mut c, 32, 136, 20, 96, pal::MAGENTA, pal::PURPLE);
    lancet(&mut c, 268, 136, 20, 96, pal::CYAN, pal::BLUE);
    lancet(&mut c, 92, 128, 24, 72, pal::PURPLE, pal::MAGENTA);
    lancet(&mut c, 204, 128, 24, 72, pal::BLUE, pal::CYAN);
    // Rose window: a ring of speaker cones.
    c.circle(160.0, 150.0, 28.0, conc_dd);
    c.circle(160.0, 150.0, 24.0, pal::BLACK);
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let (x, y) = (160.0 + a.cos() * 15.0, 150.0 + a.sin() * 15.0);
        c.circle(x, y, 5.0, hex(0x3a2f6a));
        c.circle(x, y, 2.0, pal::MAGENTA);
    }
    c.circle(160.0, 150.0, 7.0, hex(0x3a2f6a));
    c.circle(160.0, 150.0, 3.0, pal::CYAN);
    // Portal: deep pointed arch with steel doors.
    c.rect(120, 210, 80, 90, conc_dd);
    c.tri((120.0, 212.0), (160.0, 176.0), (200.0, 212.0), conc_dd);
    c.rect(128, 216, 64, 84, hex(0x1a1726));
    c.tri((128.0, 218.0), (160.0, 188.0), (192.0, 218.0), hex(0x1a1726));
    c.rect(134, 228, 25, 72, hex(0x45405c));
    c.rect(161, 228, 25, 72, hex(0x45405c));
    for y in (232..300).step_by(8) {
        c.hline(134, y, 25, hex(0x3a3550));
        c.hline(161, y, 25, hex(0x3a3550));
    }
    c.rect(154, 260, 3, 10, pal::W95_GREY);
    c.rect(163, 260, 3, 10, pal::W95_GREY);
    // A single red bulb over the door.
    c.circle(160.0, 204.0, 3.0, hex(0xff3d5a));
    // Stencilled name on the cornice band.
    bake_text(&mut c, 84, 108, "ST. BASSILICA", pal::CONCRETE_DD, 2);
    c.outline(pal::INK);
    c
}

fn hoarding() -> Canvas {
    let mut c = Canvas::new(128, 64);
    c.rect(0, 4, 128, 56, pal::CONCRETE_D);
    c.rect(2, 6, 124, 50, pal::CONCRETE);
    for y in (10..56).step_by(8) {
        c.hline(2, y, 124, mix(pal::CONCRETE, pal::CONCRETE_D, 0.4));
    }
    c.rect(0, 56, 128, 8, pal::CONCRETE_DD);
    // Old paste scraps.
    c.rect(6, 10, 10, 6, pal::BEIGE_D);
    c.rect(110, 40, 12, 8, hex(0xd9a3c4));
    c.outline(pal::INK);
    c
}

fn lamp() -> Canvas {
    let mut c = Canvas::new(16, 64);
    c.rect(7, 12, 3, 48, hex(0x2f2b45));
    c.rect(4, 58, 9, 5, hex(0x2f2b45));
    c.rect(3, 4, 11, 9, hex(0x2f2b45));
    c.rect(5, 6, 7, 6, pal::YELLOW);
    c.rect(6, 7, 5, 4, hex(0xfff3c4));
    c.tri((2.0, 4.0), (8.5, 0.0), (15.0, 4.0), hex(0x2f2b45));
    c.outline(pal::INK);
    c
}

fn log_seat() -> Canvas {
    let mut c = Canvas::new(48, 20);
    c.ellipse(24.0, 11.0, 22.0, 7.0, pal::WOOD);
    c.ellipse(24.0, 9.0, 21.0, 5.0, hex(0xa06a48));
    c.ellipse(44.0, 11.0, 3.5, 6.5, hex(0xd9a774));
    c.circle(44.0, 11.0, 2.0, pal::WOOD);
    c.outline(pal::INK);
    c
}

fn bush() -> Canvas {
    let mut c = Canvas::new(40, 28);
    c.circle(12.0, 16.0, 10.0, pal::GRASS_D);
    c.circle(26.0, 14.0, 12.0, pal::GRASS_D);
    c.circle(20.0, 12.0, 10.0, hex(0x6a9a5a));
    c.set(18, 8, pal::CORAL);
    c.set(28, 12, pal::CORAL);
    c.set(10, 14, pal::CORAL);
    c.outline(pal::INK);
    c
}

fn flowers() -> Canvas {
    let mut c = Canvas::new(32, 16);
    for (i, (x, y)) in [(4, 6), (12, 10), (20, 4), (27, 9), (8, 12)].iter().enumerate() {
        let col = [pal::PINK, pal::YELLOW, pal::CREAM, pal::CORAL, pal::SKY][i];
        c.vline(*x, y + 1, 4, pal::GRASS_D);
        c.set(*x, *y, col);
        c.set(x - 1, *y, col);
        c.set(x + 1, *y, col);
        c.set(*x, y - 1, col);
    }
    c
}

/// One PC방 seat: a fat beige CRT and tower on a desk, seen from the front.
fn desk() -> Canvas {
    let mut c = Canvas::new(48, 56);
    // Desk.
    c.rect(0, 34, 48, 10, hex(0x3b3552));
    c.rect(0, 32, 48, 4, hex(0x5a5378));
    c.rect(2, 44, 4, 12, hex(0x2a2540));
    c.rect(42, 44, 4, 12, hex(0x2a2540));
    // CRT: deep beige box.
    c.rect(6, 2, 30, 26, pal::BEIGE_D);
    c.rect(8, 0, 26, 26, pal::BEIGE);
    c.rect(11, 3, 20, 16, hex(0x1b1f3a));
    c.rect(12, 4, 18, 14, pal::W95_TEAL);
    c.rect(12, 16, 18, 2, pal::W95_GREY);
    c.rect(13, 16, 3, 2, hex(0x2f8a3a));
    c.set(13, 5, pal::WHITE);
    c.set(13, 8, pal::WHITE);
    c.rect(26, 21, 4, 2, pal::BEIGE_D);
    c.set(30, 22, pal::ACID);
    c.rect(14, 26, 14, 3, pal::BEIGE_D);
    c.rect(10, 29, 22, 3, pal::BEIGE);
    // Tower.
    c.rect(37, 6, 10, 26, pal::BEIGE);
    c.rect(37, 6, 2, 26, pal::BEIGE_D);
    c.rect(39, 9, 6, 2, pal::BEIGE_D);
    c.rect(39, 13, 6, 2, pal::BEIGE_D);
    c.set(44, 27, pal::ACID);
    // Keyboard + mouse.
    c.rect(10, 33, 24, 4, pal::BEIGE);
    c.hline(11, 34, 22, pal::BEIGE_D);
    c.ellipse(39.0, 35.0, 2.5, 2.0, pal::BEIGE);
    c.outline(pal::INK);
    c
}

fn chair_back() -> Canvas {
    let mut c = Canvas::new(28, 28);
    c.rect(4, 0, 20, 18, hex(0x1d1a2e));
    c.rect(6, 2, 16, 14, hex(0xb02a4a));
    c.rect(6, 2, 16, 3, hex(0xd94a6a));
    c.rect(12, 18, 4, 6, hex(0x1d1a2e));
    c.rect(4, 24, 20, 3, hex(0x1d1a2e));
    c.outline(pal::INK);
    c
}

fn counter() -> Canvas {
    let mut c = Canvas::new(128, 48);
    c.rect(0, 16, 128, 32, hex(0x3b3552));
    c.rect(0, 12, 128, 6, hex(0x6b6390));
    c.rect(4, 22, 120, 2, pal::MAGENTA);
    // Snacks.
    for (i, col) in [pal::CORAL, pal::YELLOW, pal::GREEN, pal::PINK, pal::BLUE].iter().enumerate() {
        let x = 70 + i as i32 * 10;
        c.rect(x, 2, 8, 10, *col);
        c.hline(x, 4, 8, pal::WHITE);
    }
    // Cup ramyeon stack.
    for i in 0..3 {
        c.rect(10 + i * 12, 4, 10, 8, pal::CREAM);
        c.hline(10 + i * 12, 6, 10, pal::CORAL);
    }
    // Till.
    c.rect(44, 0, 16, 12, pal::W95_GREY);
    c.rect(46, 2, 12, 4, hex(0x1b3a1b));
    c.outline(pal::INK);
    c
}

fn vending() -> Canvas {
    let mut c = Canvas::new(36, 64);
    c.rect(0, 0, 36, 64, hex(0xd9433d));
    c.rect(4, 4, 22, 44, hex(0x1d1a2e));
    for row in 0..4 {
        for col in 0..3 {
            let colr = [pal::CYAN, pal::YELLOW, pal::GREEN, pal::PINK][(row + col) % 4];
            c.rect(6 + col as i32 * 7, 7 + row as i32 * 10, 5, 8, colr);
        }
    }
    c.rect(28, 10, 5, 12, pal::W95_GREY);
    c.rect(4, 52, 22, 8, hex(0x1d1a2e));
    c.outline(pal::INK);
    c
}

fn dj_booth() -> Canvas {
    let mut c = Canvas::new(160, 64);
    // Concrete altar.
    c.rect(0, 24, 160, 40, pal::CONCRETE_D);
    c.rect(4, 24, 152, 36, pal::CONCRETE);
    for y in (28..60).step_by(8) {
        c.hline(4, y, 152, mix(pal::CONCRETE, pal::CONCRETE_D, 0.4));
    }
    c.rect(0, 18, 160, 8, pal::CONCRETE_DD);
    // Decks.
    for x in [12, 108] {
        c.rect(x, 6, 40, 14, hex(0x1d1a2e));
        c.circle(x as f32 + 16.0, 13.0, 6.0, hex(0x3a3550));
        c.circle(x as f32 + 16.0, 13.0, 2.0, pal::MAGENTA);
        c.rect(x + 30, 8, 3, 10, pal::W95_GREY);
    }
    // Mixer + laptop.
    c.rect(58, 4, 44, 16, hex(0x1d1a2e));
    for i in 0..4 {
        c.rect(62 + i * 10, 7, 2, 10, pal::W95_DARK);
        c.rect(61 + i * 10, 10 + (i % 3) * 2, 4, 2, pal::CYAN);
    }
    c.rect(70, -2 + 2, 20, 4, pal::W95_GREY);
    // Glowing strip.
    c.rect(4, 40, 152, 3, pal::MAGENTA);
    c.outline(pal::INK);
    c
}

fn speaker() -> Canvas {
    let mut c = Canvas::new(40, 72);
    for (i, y) in [0, 36].iter().enumerate() {
        c.rect(0, *y, 40, 36, hex(0x1d1a2e));
        c.rect(2, y + 2, 36, 32, hex(0x25223a));
        let r = if i == 0 { 9.0 } else { 13.0 };
        c.circle(20.0, *y as f32 + 18.0, r, hex(0x0f0d18));
        c.circle(20.0, *y as f32 + 18.0, r - 3.0, hex(0x3a3550));
        c.circle(20.0, *y as f32 + 18.0, 3.0, hex(0x0f0d18));
    }
    c.outline(pal::INK);
    c
}

fn pillar() -> Canvas {
    let mut c = Canvas::new(32, 160);
    c.rect(4, 0, 24, 160, pal::CONCRETE_D);
    c.rect(6, 0, 18, 160, pal::CONCRETE);
    for y in (0..160).step_by(10) {
        c.hline(6, y, 18, mix(pal::CONCRETE, pal::CONCRETE_D, 0.35));
    }
    c.rect(0, 148, 32, 12, pal::CONCRETE_DD);
    c.rect(0, 0, 32, 8, pal::CONCRETE_DD);
    c.outline(pal::INK);
    c
}

fn bouncer() -> Canvas {
    // A very large, very calm goose in black.
    let mut c = Canvas::new(40, 56);
    c.ellipse(20.0, 40.0, 14.0, 13.0, hex(0x1d1a2e));
    c.rect(6, 38, 28, 16, hex(0x1d1a2e));
    c.rect(14, 10, 12, 24, pal::WHITE);
    c.ellipse(20.0, 10.0, 8.0, 7.0, pal::WHITE);
    c.ellipse(28.0, 12.0, 5.0, 2.5, hex(0xff9a3d));
    c.rect(14, 7, 13, 3, pal::BLACK); // sunglasses
    c.set(25, 8, pal::CYAN);
    c.rect(18, 34, 4, 6, pal::WHITE); // shirt collar
    c.set(31, 30, pal::ACID); // earpiece light
    c.outline(pal::INK);
    c
}

fn owner() -> Canvas {
    let mut c = critter(6, 0);
    // Apron.
    c.rect(11, 30, 11, 6, pal::CREAM);
    c
}

fn app_icon() -> Canvas {
    let mut c = critter(2, 3);
    c.outline(pal::INK);
    c
}

