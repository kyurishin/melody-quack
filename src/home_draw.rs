//! Drawing the home, its furniture and the shop preview.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn draw_piece(b: &mut Batch, piece: &home::Piece, time: f32, alpha: f32, record_on: bool, pulse: f32, lift: f32) {
    use home::Item;
    let k = |c: Color| Color::new(c.r, c.g, c.b, alpha);
    let c = piece.center();
    let o = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + lift, home::ORIGIN.y + c.y);
    let yaw = piece.yaw();
    let wood = k(Color::from_rgba(150, 102, 64, 255));
    let wood_dark = k(Color::from_rgba(112, 74, 46, 255));
    let bx = |b: &mut Batch, at: Vec3, size: Vec3, col: Color| b.box_at(o, yaw, at, size, col);
    match piece.item {
        Item::Sampler => {
            bx(b, vec3(0.0, 0.35, 0.0), vec3(0.7, 0.7, 0.6), wood_dark);
            bx(b, vec3(0.0, 0.74, 0.0), vec3(0.66, 0.08, 0.5), k(Color::from_rgba(44, 42, 56, 255)));
            let step = vis().seq;
            b.unlit = true;
            for r in 0..4 {
                for c in 0..4 {
                    let lit = c == step % 4;
                    let col = note_color(r);
                    let g = if lit { 1.0 } else { 0.35 };
                    bx(b, vec3(-0.22 + c as f32 * 0.147, 0.79, -0.18 + r as f32 * 0.12), vec3(0.11, 0.02, 0.09), k(Color::new(col.r * g, col.g * g, col.b * g, 1.0)));
                }
            }
            b.unlit = false;
        }
        Item::MirrorBall => {
            let on = piece.state == 0;
            let spin = if on { time * 1.2 } else { 0.0 };
            bx(b, vec3(0.0, 1.95, 0.0), vec3(0.02, 0.5, 0.02), k(Color::from_rgba(150, 150, 160, 255)));
            let center = o + vec3(0.0, 1.55, 0.0);
            for (n, (sz, dy)) in [(0.2, -0.13), (0.3, -0.06), (0.32, 0.02), (0.28, 0.09), (0.18, 0.15)].iter().enumerate() {
                b.box_at(center, yaw + spin + n as f32 * 0.4, vec3(0.0, *dy, 0.0), vec3(*sz, 0.075, *sz), k(Color::from_rgba(200, 204, 214, 255)));
            }
            b.unlit = true;
            for f in 0..10 {
                let a = spin * 1.5 + f as f32 * 0.63;
                let tw = ((time * 6.0 + f as f32 * 1.7).sin() * 0.5 + 0.5) * if on { 1.0 } else { 0.3 };
                let at = vec3(a.cos() * 0.16, -0.1 + (f % 4) as f32 * 0.07, a.sin() * 0.16);
                b.box_at(center, 0.0, at, vec3(0.035, 0.035, 0.035), k(Color::new(0.8 + 0.2 * tw, 0.85 + 0.15 * tw, 1.0, 1.0)));
            }
            if on {
                for d in 0..18 {
                    let a = time * 0.7 + d as f32 * 0.349;
                    let r = 0.8 + (d % 3) as f32 * 0.7;
                    let c = hsv(d as f32 / 18.0 + time * 0.05, 0.45, 1.0);
                    b.box_at(o, 0.0, vec3(a.cos() * r, 0.02, a.sin() * r), vec3(0.09, 0.01, 0.09), k(Color::new(c.r, c.g, c.b, 0.9)));
                }
            }
            b.unlit = false;
        }
        Item::Arcade => {
            let body = k(Color::from_rgba(40, 34, 70, 255));
            let trim = k(Color::from_rgba(255, 90, 160, 255));
            bx(b, vec3(0.0, 0.85, -0.12), vec3(0.8, 1.7, 0.6), body);
            bx(b, vec3(0.0, 1.62, 0.2), vec3(0.82, 0.2, 0.08), trim);
            bx(b, vec3(0.0, 0.55, 0.19), vec3(0.6, 0.04, 0.04), trim);
            let pads = vis().pads;
            b.unlit = true;
            bx(b, vec3(0.0, 1.62, 0.245), vec3(0.6, 0.12, 0.01), k(hsv(time * 0.2, 0.5, 1.0)));
            for y in 0..5 {
                for x in 0..6 {
                    let lit = ((time * 3.0 + y as f32 * 0.7 + x as f32 * 1.3).sin() > 0.6) as u8 as f32;
                    let c = note_color(x % 4);
                    bx(b, vec3(-0.25 + x as f32 * 0.1, 1.0 + y as f32 * 0.09, 0.185), vec3(0.08, 0.07, 0.01), k(Color::new(c.r * (0.25 + 0.75 * lit), c.g * (0.25 + 0.75 * lit), c.b * (0.25 + 0.75 * lit), 1.0)));
                }
            }
            b.unlit = false;
            bx(b, vec3(0.0, 0.04, 0.95), vec3(0.96, 0.08, 0.96), k(Color::from_rgba(170, 172, 184, 255)));
            for (lane, (px, pz)) in [(-0.3, 0.95), (0.0, 1.25), (0.0, 0.65), (0.3, 0.95)].iter().enumerate() {
                let c = note_color(lane);
                let glow = pads[lane];
                b.unlit = glow > 0.05;
                bx(b, vec3(*px, 0.085, *pz), vec3(0.24, 0.02, 0.24), k(Color::new(c.r * (0.45 + 0.55 * glow), c.g * (0.45 + 0.55 * glow), c.b * (0.45 + 0.55 * glow), 1.0)));
                b.unlit = false;
            }
            bx(b, vec3(0.0, 0.7, 0.5), vec3(0.9, 0.04, 0.04), k(Color::from_rgba(190, 190, 200, 255)));
            for side in [-0.45f32, 0.45] {
                bx(b, vec3(side, 0.36, 0.5), vec3(0.04, 0.68, 0.04), k(Color::from_rgba(190, 190, 200, 255)));
            }
        }
        Item::Tv => {
            let frame = k(Color::from_rgba(24, 24, 30, 255));
            bx(b, vec3(0.0, 0.98, -0.46), vec3(1.5, 0.82, 0.06), frame);
            let (w, h) = (24usize, 14usize);
            let v = vis();
            b.unlit = true;
            for y in 0..h {
                for x in 0..w {
                    let col = if piece.state == 0 {
                        if x == w / 2 && y == h / 2 { Some(Color::new(0.3, 0.3, 0.35, 1.0)) } else { None }
                    } else {
                        tv_pixel(piece.state, x, y, w, h, time, &v)
                    };
                    let col = col.unwrap_or(Color::new(0.04, 0.04, 0.06, 1.0));
                    bx(b, vec3(-0.69 + (x as f32 + 0.5) * (1.38 / w as f32), 1.34 - (y as f32 + 0.5) * (0.72 / h as f32), -0.425), vec3(1.38 / w as f32 * 0.92, 0.72 / h as f32 * 0.92, 0.01), k(col));
                }
            }
            b.unlit = false;
        }
        Item::Rug => {
            bx(b, vec3(0.0, 0.01, 0.0), vec3(1.8, 0.02, 1.8), k(Color::from_rgba(196, 92, 98, 255)));
            bx(b, vec3(0.0, 0.022, 0.0), vec3(1.4, 0.02, 1.4), k(Color::from_rgba(240, 200, 120, 255)));
            bx(b, vec3(0.0, 0.034, 0.0), vec3(0.9, 0.02, 0.9), k(Color::from_rgba(104, 150, 196, 255)));
            bx(b, vec3(0.0, 0.046, 0.0), vec3(0.4, 0.02, 0.4), k(Color::from_rgba(250, 240, 220, 255)));
        }
        Item::Bed => {
            bx(b, vec3(0.0, 0.13, 0.0), vec3(1.86, 0.26, 1.86), wood);
            bx(b, vec3(0.0, 0.33, 0.08), vec3(1.72, 0.12, 1.62), k(Color::from_rgba(245, 242, 235, 255)));
            bx(b, vec3(0.0, 0.42, 0.24), vec3(1.68, 0.07, 1.18), k(Color::from_rgba(130, 170, 220, 255)));
            bx(b, vec3(-0.46, 0.44, -0.62), vec3(0.54, 0.1, 0.34), k(Color::from_rgba(255, 252, 245, 255)));
            bx(b, vec3(0.46, 0.44, -0.62), vec3(0.54, 0.1, 0.34), k(Color::from_rgba(255, 252, 245, 255)));
            bx(b, vec3(0.0, 0.45, -0.92), vec3(1.9, 0.66, 0.1), wood_dark);
            bx(b, vec3(0.0, 0.18, 0.92), vec3(1.82, 0.18, 0.08), wood_dark);
        }
        Item::Lamp => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.3, 0.06, 0.3), wood_dark);
            bx(b, vec3(0.0, 0.5, 0.0), vec3(0.05, 0.9, 0.05), k(Color::from_rgba(200, 180, 140, 255)));
            let glow = 0.9 + 0.1 * (time * 3.0).sin();
            let shade = if piece.state == 0 { Color::new(1.0, 0.86 * glow, 0.55 * glow, 1.0) } else { Color::from_rgba(150, 136, 112, 255) };
            b.unlit = piece.state == 0;
            bx(b, vec3(0.0, 1.0, 0.0), vec3(0.42, 0.26, 0.42), k(shade));
            b.unlit = false;
        }
        Item::Plant => {
            bx(b, vec3(0.0, 0.14, 0.0), vec3(0.34, 0.28, 0.34), k(Color::from_rgba(196, 110, 74, 255)));
            let withered = piece.state == home::WITHERED;
            let grow = if withered { 0.85 } else { 1.0 + piece.state.min(3) as f32 * 0.22 };
            for (lx, ly, lz) in [(0.0, 0.42, 0.0), (0.12, 0.55, 0.05), (-0.1, 0.6, -0.06), (0.04, 0.72, -0.08), (-0.06, 0.5, 0.12)] {
                let sway = if withered { 0.0 } else { (time * 1.4 + lx * 9.0).sin() * 0.02 };
                let droop = if withered { (ly - 0.28) * 0.45 } else { 0.0 };
                let spread = if withered { 1.6 } else { 1.0 };
                let leaf = if withered { Color::from_rgba(150, 120, 70, 255) } else { Color::from_rgba(90, 160, 84, 255) };
                bx(b, vec3((lx + sway) * grow * spread, 0.28 + (ly - 0.28) * grow - droop, lz * grow * spread), vec3(0.2, 0.16, 0.2) * grow.sqrt(), k(leaf));
            }
            if piece.state == 3 {
                for (fx, fz, col) in [(0.08, 0.06, Color::from_rgba(255, 140, 170, 255)), (-0.12, -0.02, Color::from_rgba(255, 230, 120, 255)), (0.02, -0.12, Color::from_rgba(255, 255, 255, 255))] {
                    bx(b, vec3(fx * grow, 0.28 + 0.5 * grow, fz * grow), vec3(0.09, 0.09, 0.09), k(col));
                }
            }
        }
        Item::Bookshelf => {
            bx(b, vec3(0.0, 0.62, -0.2), vec3(0.9, 1.24, 0.5), wood);
            let books = [Color::from_rgba(200, 80, 80, 255), Color::from_rgba(80, 130, 200, 255), Color::from_rgba(230, 190, 90, 255), Color::from_rgba(110, 170, 110, 255), Color::from_rgba(180, 120, 200, 255)];
            for shelf in 0..3 {
                for n in 0..5 {
                    let h = 0.22 + ((n * 7 + shelf * 3) % 4) as f32 * 0.025;
                    bx(b, vec3(-0.32 + n as f32 * 0.16, 0.12 + shelf as f32 * 0.38 + h / 2.0, 0.06), vec3(0.12, h, 0.04), k(books[(n + shelf) % 5]));
                }
            }
        }
        Item::Sofa => {
            let fabric = k(Color::from_rgba(110, 150, 120, 255));
            let fabric_light = k(Color::from_rgba(140, 180, 146, 255));
            bx(b, vec3(0.0, 0.14, 0.05), vec3(1.86, 0.28, 0.8), fabric);
            bx(b, vec3(-0.45, 0.33, 0.1), vec3(0.86, 0.1, 0.62), fabric_light);
            bx(b, vec3(0.45, 0.33, 0.1), vec3(0.86, 0.1, 0.62), fabric_light);
            bx(b, vec3(0.0, 0.5, -0.32), vec3(1.86, 0.5, 0.16), fabric);
            bx(b, vec3(-0.9, 0.36, 0.05), vec3(0.14, 0.3, 0.8), fabric);
            bx(b, vec3(0.9, 0.36, 0.05), vec3(0.14, 0.3, 0.8), fabric);
        }
        Item::Table => {
            bx(b, vec3(0.0, 0.48, 0.0), vec3(1.78, 0.06, 0.82), wood);
            for (lx, lz) in [(-0.76, -0.3), (0.76, -0.3), (-0.76, 0.3), (0.76, 0.3), (-0.1, -0.3), (0.1, 0.3)] {
                bx(b, vec3(lx, 0.22, lz), vec3(0.06, 0.44, 0.06), wood_dark);
            }
            for lx in [-0.56, -0.18, 0.18, 0.56] {
                bx(b, vec3(lx, 0.52, 0.0), vec3(0.12, 0.03, 0.12), k(Color::from_rgba(245, 245, 240, 255)));
            }
            if piece.state == 0 {
                bx(b, vec3(-0.52, 0.535, 0.0), vec3(0.14, 0.08, 0.14), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, vec3(0.52, 0.535, 0.0), vec3(0.14, 0.08, 0.14), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, vec3(0.0, 0.51, 0.0), vec3(0.28, 0.03, 0.18), k(Color::from_rgba(240, 160, 170, 255)));
            } else {
                let food = [Color::from_rgba(170, 110, 60, 255), Color::from_rgba(240, 150, 80, 255), Color::from_rgba(200, 90, 70, 255), Color::from_rgba(250, 200, 70, 255), Color::from_rgba(150, 100, 70, 255), Color::from_rgba(180, 130, 90, 255), Color::from_rgba(210, 140, 100, 255), Color::from_rgba(240, 210, 120, 255)][(piece.state as usize - 1) % 8];
                bx(b, vec3(0.0, 0.505, 0.0), vec3(0.48, 0.02, 0.34), k(Color::from_rgba(245, 245, 240, 255)));
                bx(b, vec3(0.0, 0.54, 0.0), vec3(0.28, 0.06, 0.2), k(food));
                let steam = (time * 1.5).fract();
                bx(b, vec3((time * 3.0).sin() * 0.03, 0.6 + steam * 0.25, 0.0), vec3(0.05, 0.04, 0.05), k(Color::new(1.0, 1.0, 1.0, 0.6 * (1.0 - steam))));
            }
        }
        Item::RecordPlayer => {
            bx(b, vec3(0.0, 0.25, 0.0), vec3(0.8, 0.5, 0.6), wood_dark);
            bx(b, vec3(0.0, 0.52, 0.0), vec3(0.76, 0.04, 0.56), wood);
            let spin = if record_on { time * 3.5 } else { 0.0 };
            b.box_at(o + vec3(0.0, 0.0, 0.0), yaw + spin, vec3(0.0, 0.56, 0.0), vec3(0.42, 0.02, 0.42), k(Color::from_rgba(24, 22, 28, 255)));
            b.box_at(o, yaw + spin, vec3(0.0, 0.575, 0.0), vec3(0.12, 0.012, 0.12), k(Color::from_rgba(220, 70, 80, 255)));
            b.box_at(o, yaw + spin, vec3(0.13, 0.575, 0.0), vec3(0.04, 0.014, 0.04), k(Color::from_rgba(250, 230, 200, 255)));
            bx(b, vec3(0.28, 0.6, 0.12), vec3(0.04, 0.04, 0.3), k(Color::from_rgba(200, 200, 210, 255)));
        }
        Item::Piano => {
            let lacquer = k(Color::from_rgba(40, 34, 44, 255));
            bx(b, vec3(0.0, 0.55, -0.2), vec3(1.8, 1.1, 0.5), lacquer);
            bx(b, vec3(0.0, 0.42, 0.12), vec3(1.7, 0.08, 0.3), lacquer);
            bx(b, vec3(0.0, 0.47, 0.12), vec3(1.6, 0.03, 0.26), k(Color::from_rgba(250, 248, 240, 255)));
            for n in 0..11 {
                if n % 7 == 2 || n % 7 == 6 {
                    continue;
                }
                bx(b, vec3(-0.72 + n as f32 * 0.145, 0.495, 0.06), vec3(0.07, 0.03, 0.14), lacquer);
            }
            bx(b, vec3(0.0, 1.12, -0.2), vec3(1.84, 0.05, 0.54), k(Color::from_rgba(60, 52, 66, 255)));
            bx(b, vec3(0.5, 1.2, -0.25), vec3(0.08, 0.12, 0.08), k(Color::from_rgba(255, 220, 120, 255)));
        }
        Item::Drums => {
            bx(b, vec3(0.0, 0.22, -0.12), vec3(0.5, 0.44, 0.36), k(Color::from_rgba(200, 60, 70, 255)));
            bx(b, vec3(0.0, 0.22, 0.07), vec3(0.42, 0.36, 0.02), k(Color::from_rgba(245, 240, 230, 255)));
            bx(b, vec3(-0.3, 0.42, 0.2), vec3(0.26, 0.12, 0.26), k(Color::from_rgba(230, 230, 236, 255)));
            bx(b, vec3(-0.3, 0.2, 0.2), vec3(0.03, 0.36, 0.03), k(Color::from_rgba(170, 170, 180, 255)));
            let wobble = (time * 6.0).sin() * 0.02;
            bx(b, vec3(0.32, 0.8 + wobble, 0.05), vec3(0.36, 0.015, 0.36), k(Color::from_rgba(230, 196, 90, 255)));
            bx(b, vec3(0.32, 0.4, 0.05), vec3(0.03, 0.8, 0.03), k(Color::from_rgba(170, 170, 180, 255)));
        }
        Item::WindChime => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.26, 0.06, 0.26), wood_dark);
            bx(b, vec3(0.0, 0.7, 0.0), vec3(0.04, 1.36, 0.04), wood_dark);
            bx(b, vec3(0.12, 1.36, 0.0), vec3(0.28, 0.03, 0.04), wood_dark);
            for n in 0..4 {
                let sway = (time * 2.2 + n as f32 * 1.3).sin() * 0.03;
                let len = 0.22 + n as f32 * 0.05;
                bx(b, vec3(0.04 + n as f32 * 0.07 + sway, 1.3 - len / 2.0, 0.0), vec3(0.03, len, 0.03), k(Color::from_rgba(190, 210, 230, 255)));
            }
        }
        Item::Speaker => {
            bx(b, vec3(0.0, 0.5, 0.0), vec3(0.5, 1.0, 0.45), k(Color::from_rgba(44, 40, 52, 255)));
            bx(b, vec3(0.0, 0.34, 0.23), vec3(0.32, 0.32, 0.02), k(Color::from_rgba(80, 76, 92, 255)));
            bx(b, vec3(0.0, 0.34, 0.245), vec3(0.12, 0.12, 0.02), k(Color::from_rgba(26, 24, 30, 255)));
            bx(b, vec3(0.0, 0.74, 0.23), vec3(0.16, 0.16, 0.02), k(Color::from_rgba(80, 76, 92, 255)));
            let led = if piece.state == 0 { 0.6 + 0.4 * (time * 4.0).sin().abs() } else { 0.0 };
            bx(b, vec3(0.16, 0.92, 0.23), vec3(0.04, 0.04, 0.02), k(if piece.state == 0 { Color::new(0.4, 1.0 * led, 0.6, 1.0) } else { Color::from_rgba(60, 40, 40, 255) }));
        }
        Item::DjBooth => {
            let concrete = k(Color::from_rgba(120, 120, 124, 255));
            let plinth = k(Color::from_rgba(34, 34, 38, 255));
            bx(b, vec3(0.0, 0.4, 0.1), vec3(1.9, 0.8, 0.52), concrete);
            bx(b, vec3(0.0, 0.81, 0.06), vec3(1.84, 0.02, 0.46), k(Color::from_rgba(48, 48, 54, 255)));
            let spin = if record_on { time * 3.4 } else { 0.0 };
            for (n, side) in [-0.58_f32, 0.58].iter().enumerate() {
                let c = vec3(*side, 0.0, 0.06);
                bx(b, c + vec3(0.0, 0.85, 0.0), vec3(0.6, 0.06, 0.44), plinth);
                for kk in 0..8 {
                    let turn = kk as f32 * std::f32::consts::FRAC_PI_8 / 2.0;
                    b.box_at(o + rotate_y(c, yaw), yaw + turn, vec3(0.0, 0.89, 0.0), vec3(0.34, 0.025, 0.34), k(Color::from_rgba(70, 70, 76, 255)));
                    b.box_at(o + rotate_y(c, yaw), yaw + turn + spin + n as f32, vec3(0.0, 0.905, 0.0), vec3(0.31, 0.012, 0.31), k(Color::from_rgba(16, 16, 20, 255)));
                    b.box_at(o + rotate_y(c, yaw), yaw + turn + spin, vec3(0.0, 0.913, 0.0), vec3(0.1, 0.008, 0.1), k([Color::from_rgba(220, 60, 70, 255), Color::from_rgba(240, 200, 80, 255)][n]));
                }
                b.box_at(o + rotate_y(c, yaw), yaw + spin + n as f32 * 2.0, vec3(0.03, 0.918, 0.0), vec3(0.025, 0.006, 0.025), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, c + vec3(0.21, 0.92, -0.12), vec3(0.03, 0.03, 0.03), k(Color::from_rgba(190, 190, 200, 255)));
                bx(b, c + vec3(0.16, 0.925, -0.02), vec3(0.02, 0.012, 0.22), k(Color::from_rgba(190, 190, 200, 255)));
                bx(b, c + vec3(-0.22, 0.89, 0.16), vec3(0.06, 0.02, 0.03), k(Color::from_rgba(60, 200, 120, 255)));
            }
            bx(b, vec3(0.0, 0.87, 0.06), vec3(0.42, 0.06, 0.42), k(Color::from_rgba(28, 28, 32, 255)));
            for row in 0..3 {
                for col in 0..3 {
                    bx(b, vec3(-0.11 + col as f32 * 0.11, 0.91, -0.1 + row as f32 * 0.07), vec3(0.04, 0.025, 0.04), k(Color::from_rgba(150, 150, 160, 255)));
                }
            }
            for col in 0..2 {
                let fade = (time * (1.1 + col as f32 * 0.7)).sin() * 0.04;
                bx(b, vec3(-0.06 + col as f32 * 0.12, 0.905, 0.14 + fade), vec3(0.03, 0.02, 0.05), k(Color::from_rgba(230, 230, 236, 255)));
            }
            bx(b, vec3(0.0, 0.905, 0.21), vec3(0.16, 0.015, 0.02), k(Color::from_rgba(230, 230, 236, 255)));
            for n in 0..6 {
                let level = (pulse.max(0.0) * 6.0) as i32;
                let lit = if n < level { 1.0 } else { 0.18 };
                let col = if n >= 4 { Color::new(1.0 * lit, 0.2 * lit, 0.2 * lit, 1.0) } else { Color::new(0.3 * lit, 1.0 * lit, 0.45 * lit, 1.0) };
                bx(b, vec3(0.17, 0.9, -0.12 + n as f32 * 0.035), vec3(0.025, 0.012, 0.025), k(col));
            }
            bx(b, vec3(0.0, 0.78, 0.365), vec3(1.7, 0.025, 0.012), k(Color::new(0.9, 0.1, 0.12, 1.0)));
        }
        Item::Subwoofer => {
            let cab = k(Color::from_rgba(26, 26, 30, 255));
            let cone = k(Color::from_rgba(52, 52, 58, 255));
            let pump = if piece.state == 0 { 0.025 * pulse.max(0.0) } else { 0.0 };
            for level in 0..2 {
                let y = 0.32 + level as f32 * 0.64;
                bx(b, vec3(0.0, y, 0.0), vec3(0.8, 0.62, 0.7), cab);
                bx(b, vec3(0.0, y, 0.36 + pump), vec3(0.5, 0.5, 0.02), cone);
                bx(b, vec3(0.0, y, 0.375 + pump * 1.5), vec3(0.18, 0.18, 0.02), k(Color::from_rgba(18, 18, 20, 255)));
            }
        }
        Item::RedLight => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.24, 0.06, 0.24), k(Color::from_rgba(40, 40, 44, 255)));
            bx(b, vec3(0.0, 0.75, 0.0), vec3(0.04, 1.44, 0.04), k(Color::from_rgba(40, 40, 44, 255)));
            let glow = if piece.state == 0 { 0.8 + 0.2 * pulse.max(0.0) } else { 0.3 };
            b.unlit = piece.state == 0;
            bx(b, vec3(0.0, 1.46, 0.0), vec3(0.8, 0.06, 0.06), k(Color::new(glow, 0.12 * glow, 0.14 * glow, 1.0)));
            b.unlit = false;
        }
        Item::ConcreteBench => {
            let concrete = k(Color::from_rgba(132, 132, 136, 255));
            bx(b, vec3(0.0, 0.2, 0.0), vec3(1.9, 0.4, 0.6), concrete);
            bx(b, vec3(0.0, 0.41, 0.0), vec3(1.9, 0.02, 0.6), k(Color::from_rgba(146, 146, 150, 255)));
            for n in 0..3 {
                bx(b, vec3(-0.6 + n as f32 * 0.6, 0.2, 0.301), vec3(0.03, 0.03, 0.01), k(Color::from_rgba(96, 96, 100, 255)));
            }
        }
        Item::SteelPillar => {
            let steel = k(Color::from_rgba(92, 94, 102, 255));
            bx(b, vec3(0.0, 0.85, 0.0), vec3(0.34, 1.7, 0.34), steel);
            bx(b, vec3(0.0, 0.04, 0.0), vec3(0.5, 0.08, 0.5), k(Color::from_rgba(70, 72, 78, 255)));
            bx(b, vec3(0.0, 1.66, 0.0), vec3(0.5, 0.08, 0.5), k(Color::from_rgba(70, 72, 78, 255)));
            for n in 0..4 {
                bx(b, vec3(0.0, 0.3 + n as f32 * 0.38, 0.175), vec3(0.04, 0.04, 0.01), k(Color::from_rgba(130, 132, 140, 255)));
            }
        }
    }
}

pub(crate) fn draw_home(b: &mut Batch, home: &home::Home, time: f32, ghost: Option<(home::Piece, bool)>, pulse: f32, beat_on: bool) {
    let ox = home::ORIGIN.x;
    let oz = home::ORIGIN.y;
    let w = home.w() as f32;
    let d = home.d() as f32;
    let loft = home.loft();
    let lh = home::LOFT_H;
    let door_x = home.door_x() as f32;
    let (_, plank_a, plank_b) = home::FLOORS[(home.floor as usize).min(home::FLOORS.len() - 1)];
    let checker = home.floor as usize == 2;
    let concrete_floor = home.floor == home::CONCRETE_FLOOR;
    for z in 0..home.d() {
        for x in 0..home.w() {
            let alt = if checker { (x + z) % 2 == 0 } else if concrete_floor { hash(x, z, 77) > 0.5 } else { z % 2 == 0 };
            let up = if home.level(x, z) == home::Level::Loft { lh } else { 0.0 };
            b.top(vec3(ox + x as f32 + 0.5, home::FLOOR_Y + up, oz + z as f32 + 0.5), if alt { plank_a } else { plank_b });
        }
    }
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y - 0.15, oz + d + 0.05), vec3(w + 0.2, 0.3, 0.1), tinted(plank_b, -0.25));
    let wall = home::wall_color(home);
    let trim = tinted(wall, -0.22);
    let wall_h = if loft { 1.7 + lh + 0.4 } else { 1.7 };
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + wall_h / 2.0, oz - 0.06), vec3(w + 0.24, wall_h, 0.12), wall);
    b.cube(vec3(ox - 0.06, home::FLOOR_Y + wall_h / 2.0, oz + d / 2.0), vec3(0.12, wall_h, d), wall);
    b.cube(vec3(ox + w + 0.06, home::FLOOR_Y + wall_h / 2.0, oz + d / 2.0), vec3(0.12, wall_h, d), wall);
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + 0.12, oz + 0.01), vec3(w, 0.24, 0.04), trim);
    b.cube(vec3(ox + 0.03, home::FLOOR_Y + 0.12, oz + d / 2.0), vec3(0.04, 0.24, d), trim);
    b.cube(vec3(ox + w - 0.03, home::FLOOR_Y + 0.12, oz + d / 2.0), vec3(0.04, 0.24, d), trim);
    for (x0, x1) in [(0.0_f32, door_x), (door_x + 1.0, w)] {
        b.cube(vec3(ox + (x0 + x1) / 2.0, home::FLOOR_Y + 0.1, oz + d - 0.04), vec3(x1 - x0, 0.2, 0.08), trim);
    }
    if home.wall == home::CONCRETE_WALL {
        let seam = tinted(wall, -0.12);
        let hole = tinted(wall, -0.35);
        let rows = ((wall_h - 0.3) / 0.18) as i32;
        for k in 1..rows {
            let y = home::FLOOR_Y + 0.24 + k as f32 * 0.18;
            b.cube(vec3(ox + w / 2.0, y, oz + 0.005), vec3(w, 0.012, 0.02), seam);
            b.cube(vec3(ox + 0.005, y, oz + d / 2.0), vec3(0.02, 0.012, d), seam);
            b.cube(vec3(ox + w - 0.005, y, oz + d / 2.0), vec3(0.02, 0.012, d), seam);
        }
        for row in 0..((wall_h - 0.5) / 0.45) as i32 + 1 {
            let y = home::FLOOR_Y + 0.5 + row as f32 * 0.45;
            for col in 0..home.w() {
                b.cube(vec3(ox + 0.5 + col as f32, y, oz + 0.01), vec3(0.05, 0.05, 0.02), hole);
            }
            for col in 0..home.d() {
                b.cube(vec3(ox + 0.012, y, oz + 0.5 + col as f32), vec3(0.02, 0.05, 0.05), hole);
                b.cube(vec3(ox + w - 0.012, y, oz + 0.5 + col as f32), vec3(0.02, 0.05, 0.05), hole);
            }
        }
    }
    if home.floor == home::CONCRETE_FLOOR {
        let seam = tinted(plank_b, -0.2);
        let mut x = 2.0_f32;
        while x < w {
            b.cube(vec3(ox + x, home::FLOOR_Y + 0.003, oz + d / 2.0), vec3(0.02, 0.004, d), seam);
            x += 2.0;
        }
        let mut z = 2.0_f32;
        while z < d {
            b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + 0.003, oz + z), vec3(w, 0.004, 0.02), seam);
            z += 2.0;
        }
    }
    b.cube(vec3(ox + door_x + 0.5, home::FLOOR_Y + 0.01, oz + d + 0.15), vec3(0.8, 0.02, 0.3), Color::from_rgba(204, 160, 110, 255));
    let deco_up = if loft { lh } else { 0.0 };
    let window_x = ox + 4.6;
    let brutal = home.wall == home::CONCRETE_WALL;
    if loft {
        let x0 = home::LOFT_X0 as f32;
        let ld = home.loft_d() as f32;
        let face = tinted(plank_b, -0.18);
        b.cube(vec3(ox + (x0 + w) / 2.0, home::FLOOR_Y + lh / 2.0 - 0.005, oz + ld / 2.0), vec3(w - x0, lh - 0.01, ld), face);
        b.cube(vec3(ox + (x0 + w) / 2.0, home::FLOOR_Y + lh - 0.04, oz + ld + 0.005), vec3(w - x0, 0.08, 0.04), tinted(face, -0.2));
        let metal = Color::from_rgba(40, 40, 46, 255);
        let glass = Color::new(0.75, 0.88, 1.0, 0.28);
        let sx = home.stair_x() as f32;
        let rail_y = home::FLOOR_Y + lh;
        b.cube(vec3(ox + (x0 + sx) / 2.0, rail_y + 0.5, oz + ld - 0.03), vec3(sx - x0, 0.04, 0.04), metal);
        b.cube(vec3(ox + (x0 + sx) / 2.0, rail_y + 0.25, oz + ld - 0.03), vec3(sx - x0, 0.46, 0.02), glass);
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.5, oz + ld / 2.0), vec3(0.04, 0.04, ld), metal);
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.25, oz + ld / 2.0), vec3(0.02, 0.46, ld), glass);
        let mut px = x0;
        while px <= sx + 0.01 {
            b.cube(vec3(ox + px, rail_y + 0.26, oz + ld - 0.03), vec3(0.05, 0.52, 0.05), metal);
            px += 1.0;
        }
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.26, oz + 0.05), vec3(0.05, 0.52, 0.05), metal);
        let steps = 6;
        let len = home::STAIR_LEN as f32;
        let tread = tinted(plank_a, 0.04);
        let riser = tinted(plank_b, -0.15);
        for i in 0..steps {
            let h = lh * (i + 1) as f32 / steps as f32;
            let zc = ld + len - (i as f32 + 0.5) * len / steps as f32;
            b.cube(vec3(ox + sx + 0.5, home::FLOOR_Y + h / 2.0, oz + zc), vec3(0.9, h, len / steps as f32), riser);
            b.cube(vec3(ox + sx + 0.5, home::FLOOR_Y + h + 0.01, oz + zc), vec3(0.9, 0.02, len / steps as f32 - 0.02), tread);
        }
        for i in 0..=steps {
            let h = lh * i as f32 / steps as f32;
            let zc = ld + len - i as f32 * len / steps as f32;
            b.cube(vec3(ox + sx + 0.06, home::FLOOR_Y + h + 0.5, oz + zc), vec3(0.04, 0.04, len / steps as f32 + 0.02), metal);
        }
        let sk = sky();
        let win_bottom = home::FLOOR_Y + lh + 0.22;
        let win_h = 1.78;
        let sky_y = win_bottom + win_h / 2.0;
        let sky_w = w - x0 - 0.4;
        let sky_cx = ox + x0 + 0.2 + sky_w / 2.0;
        b.cube(vec3(sky_cx, sky_y, oz + 0.015), vec3(sky_w + 0.12, win_h + 0.1, 0.04), metal);
        let night = sk.night;
        let glow = sk.glow;
        let day_top = Color::from_rgba(96, 156, 220, 255);
        let day_low = Color::from_rgba(186, 216, 236, 255);
        let night_top = Color::from_rgba(14, 16, 40, 255);
        let night_low = Color::from_rgba(40, 36, 84, 255);
        let dusk_low = Color::from_rgba(255, 140, 96, 255);
        let grey = [0.0, 0.35, 0.55, 0.3][sk.weather];
        b.unlit = true;
        let bands = 10;
        for k in 0..bands {
            let t = k as f32 / (bands - 1) as f32;
            let base = mix(mix(day_top, day_low, t), mix(night_top, night_low, t), night);
            let warm = mix(base, mix(Color::from_rgba(250, 170, 140, 255), dusk_low, t), glow * (0.3 + 0.7 * t));
            let col = mix(warm, mix(Color::from_rgba(150, 156, 168, 255), Color::from_rgba(40, 42, 56, 255), night), grey);
            let bh = win_h / bands as f32;
            b.cube(vec3(sky_cx, win_bottom + win_h - (k as f32 + 0.5) * bh, oz + 0.03), vec3(sky_w, bh + 0.002, 0.02), col);
        }
        let day_part = ((sk.hour - 6.0) / 12.0).rem_euclid(1.0);
        if night < 0.6 && sk.weather != 2 {
            let sx = sky_cx - sky_w / 2.0 + sky_w * (0.1 + 0.8 * day_part);
            let sy = win_bottom + 0.75 + (day_part * PI).sin() * 0.8;
            let sun = mix(Color::from_rgba(255, 244, 190, 255), Color::from_rgba(255, 150, 90, 255), glow);
            b.cube(vec3(sx, sy, oz + 0.04), vec3(0.2, 0.2, 0.01), Color::new(sun.r, sun.g, sun.b, if sk.weather == 1 { 0.5 } else { 1.0 }));
        } else if night >= 0.6 {
            let mx = sky_cx - sky_w / 2.0 + sky_w * (0.2 + 0.6 * ((sk.hour + 6.0) / 24.0).rem_euclid(1.0));
            b.cube(vec3(mx, win_bottom + 1.45, oz + 0.04), vec3(0.15, 0.15, 0.01), Color::from_rgba(240, 240, 220, 255));
            if sk.weather == 0 {
                for k in 0..((sky_w * 4.0) as i32) {
                    let tw = 0.5 + 0.5 * (time * (1.0 + hash(k, 1, 9) * 2.0) + k as f32).sin();
                    b.cube(vec3(sky_cx - sky_w / 2.0 + hash(k, 2, 9) * sky_w, win_bottom + 1.0 + hash(k, 3, 9) * 0.7, oz + 0.04), vec3(0.03, 0.03, 0.01), Color::new(1.0, 1.0, 0.9, tw * night));
                }
            }
        }
        let clouds = [1, 4, 6, 4][sk.weather];
        for k in 0..clouds {
            let cx = sky_cx - sky_w / 2.0 + ((hash(k, 4, 12) * sky_w + time * 0.05 * (1.0 + hash(k, 5, 12))) % sky_w);
            let cy = win_bottom + 1.2 + hash(k, 6, 12) * 0.45;
            let cc = mix(mix(Color::from_rgba(250, 250, 255, 255), Color::from_rgba(120, 124, 140, 255), grey), Color::from_rgba(50, 50, 72, 255), night * 0.8);
            let cc = mix(cc, Color::from_rgba(255, 180, 160, 255), glow * 0.5);
            b.cube(vec3(cx, cy, oz + 0.042), vec3(0.6, 0.12, 0.01), cc);
            b.cube(vec3(cx + 0.08, cy + 0.08, oz + 0.042), vec3(0.32, 0.1, 0.01), cc);
        }
        let water_h = 0.22;
        let water = mix(mix(Color::from_rgba(70, 120, 170, 255), Color::from_rgba(16, 22, 46, 255), night), Color::from_rgba(220, 120, 90, 255), glow * 0.4);
        b.cube(vec3(sky_cx, win_bottom + water_h / 2.0, oz + 0.05), vec3(sky_w, water_h, 0.02), water);
        let facade_day = Color::from_rgba(120, 128, 150, 255);
        let facade_night = Color::from_rgba(20, 22, 44, 255);
        let n = (sky_w * 3.2) as i32;
        for k in 0..n {
            let bx = sky_cx - sky_w / 2.0 + 0.14 + k as f32 * (sky_w - 0.28) / (n - 1).max(1) as f32;
            let tall = 0.3 + hash(k, 3, 41) * 0.75;
            let base_y = win_bottom + water_h;
            let tint = 0.85 + hash(k, 8, 41) * 0.3;
            let facade = mix(Color::new(facade_day.r * tint, facade_day.g * tint, facade_day.b * tint, 1.0), facade_night, night);
            let facade = mix(facade, Color::from_rgba(200, 120, 110, 255), glow * 0.35);
            b.cube(vec3(bx, base_y + tall / 2.0, oz + 0.055), vec3(0.27, tall, 0.02), facade);
            let rows = (tall / 0.12) as i32;
            for r in 0..rows {
                for c in 0..2 {
                    let lit_chance = 0.15 + night * 0.6;
                    if hash(k * 7 + c, r, 61) < lit_chance {
                        let flick = 0.8 + 0.2 * (time * 0.3 + (k * 13 + r) as f32).sin();
                        let lamp = Color::new(1.0, 0.86 * flick, 0.5 * flick, 0.35 + 0.65 * night);
                        b.cube(vec3(bx - 0.06 + c as f32 * 0.12, base_y + 0.08 + r as f32 * 0.12, oz + 0.068), vec3(0.05, 0.05, 0.01), lamp);
                    }
                }
            }
        }
        let crane = mix(Color::from_rgba(200, 80, 60, 255), Color::from_rgba(30, 28, 50, 255), night);
        for (k, fx) in [0.18f32, 0.72].iter().enumerate() {
            let cx = sky_cx - sky_w / 2.0 + sky_w * fx;
            let base_y = win_bottom + water_h;
            b.cube(vec3(cx, base_y + 0.5, oz + 0.06), vec3(0.04, 1.0, 0.01), crane);
            b.cube(vec3(cx + if k == 0 { 0.25 } else { -0.25 }, base_y + 0.98, oz + 0.06), vec3(0.7, 0.035, 0.01), crane);
            if night > 0.5 && (time * 1.5 + k as f32).sin() > 0.0 {
                b.cube(vec3(cx, base_y + 1.03, oz + 0.065), vec3(0.04, 0.04, 0.01), Color::from_rgba(255, 60, 60, 255));
            }
        }
        match sk.weather {
            2 => {
                for k in 0..((sky_w * 14.0) as i32) {
                    let x = sky_cx - sky_w / 2.0 + hash(k, 1, 71) * sky_w;
                    let y = win_bottom + (hash(k, 2, 71) * win_h - time * 2.2).rem_euclid(win_h);
                    b.cube(vec3(x, y, oz + 0.075), vec3(0.012, 0.1, 0.01), Color::new(0.75, 0.82, 0.95, 0.7));
                }
            }
            3 => {
                for k in 0..((sky_w * 10.0) as i32) {
                    let x = sky_cx - sky_w / 2.0 + hash(k, 1, 73) * sky_w + (time + k as f32).sin() * 0.04;
                    let y = win_bottom + (hash(k, 2, 73) * win_h - time * 0.25).rem_euclid(win_h);
                    b.cube(vec3(x, y, oz + 0.075), vec3(0.03, 0.03, 0.01), Color::new(1.0, 1.0, 1.0, 0.95));
                }
            }
            _ => {}
        }
        b.unlit = false;
        for k in 1..((sky_w) as i32 + 1) {
            b.cube(vec3(sky_cx - sky_w / 2.0 + k as f32, sky_y, oz + 0.08), vec3(0.04, win_h, 0.02), metal);
        }
        b.cube(vec3(sky_cx, win_bottom + win_h * 0.62, oz + 0.08), vec3(sky_w, 0.03, 0.02), metal);
        b.cube(vec3(ox + 1.0, home::FLOOR_Y + 1.15, oz + 0.02), vec3(0.6, 0.45, 0.04), Color::from_rgba(120, 84, 50, 255));
        b.cube(vec3(ox + 1.0, home::FLOOR_Y + 1.15, oz + 0.04), vec3(0.5, 0.35, 0.04), Color::from_rgba(240, 190, 120, 255));
    }
    if brutal && !loft {
        b.cube(vec3(window_x, home::FLOOR_Y + 1.25 + deco_up, oz + 0.02), vec3(1.4, 0.18, 0.05), Color::from_rgba(40, 40, 46, 255));
        b.cube(vec3(window_x, home::FLOOR_Y + 1.25 + deco_up, oz + 0.04), vec3(1.3, 0.1, 0.05), Color::from_rgba(70, 80, 96, 255));
    }
    if !brutal && !loft {
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.01), vec3(1.0, 0.7, 0.05), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.03), vec3(0.86, 0.56, 0.05), Color::from_rgba(150, 196, 226, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.05), vec3(0.04, 0.56, 0.04), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 0.62, oz + 0.08), vec3(1.06, 0.06, 0.12), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.1, oz + 0.02), vec3(0.5, 0.4, 0.04), Color::from_rgba(120, 84, 50, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.1, oz + 0.04), vec3(0.4, 0.3, 0.04), Color::from_rgba(240, 190, 120, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.06, oz + 0.06), vec3(0.16, 0.12, 0.04), Color::from_rgba(250, 120, 140, 255));
    }
    let (cx, cz) = home::CATALOGUE;
    let cat_pos = vec3(ox + cx as f32 + 0.5, home::FLOOR_Y, oz + cz as f32 + 0.5);
    b.cube(cat_pos + vec3(0.0, 0.3, 0.0), vec3(0.5, 0.6, 0.4), Color::from_rgba(150, 102, 64, 255));
    b.box_at(cat_pos, 0.0, vec3(0.0, 0.64, 0.05), vec3(0.56, 0.06, 0.4), Color::from_rgba(196, 72, 72, 255));
    b.cube(cat_pos + vec3(-0.12, 0.68, 0.05), vec3(0.24, 0.03, 0.34), Color::from_rgba(252, 248, 236, 255));
    b.cube(cat_pos + vec3(0.12, 0.68, 0.05), vec3(0.24, 0.03, 0.34), Color::from_rgba(246, 240, 226, 255));
    let bounce = (time * 3.0).sin() * 0.03;
    b.cube(cat_pos + vec3(0.0, 0.95 + bounce, 0.05), vec3(0.08, 0.08, 0.08), ACORN_COLOR);
    b.cube(cat_pos + vec3(0.0, 1.0 + bounce, 0.05), vec3(0.1, 0.03, 0.1), Color::from_rgba(120, 84, 50, 255));
    let mut order: Vec<&home::Piece> = home.pieces.iter().collect();
    order.sort_by_key(|p| if p.item == home::Item::Rug { 0 } else { 1 });
    for piece in order {
        let spinning = if piece.item == home::Item::DjBooth { beat_on } else { home.record_on };
        draw_piece(b, piece, time, 1.0, spinning, pulse, home.piece_lift(piece));
    }
    if let Some((piece, valid)) = ghost {
        let (fw, fd) = piece.footprint();
        let col = if valid { Color::new(0.4, 1.0, 0.55, 0.35) } else { Color::new(1.0, 0.35, 0.4, 0.4) };
        let lift = if home.level(piece.x, piece.z) == home::Level::Loft { home::LOFT_H } else { 0.0 };
        for z in piece.z..piece.z + fd {
            for x in piece.x..piece.x + fw {
                b.cube(vec3(ox + x as f32 + 0.5, home::FLOOR_Y + lift + 0.012, oz + z as f32 + 0.5), vec3(0.96, 0.02, 0.96), col);
            }
        }
        draw_piece(b, &piece, time, 0.6, false, 0.0, lift);
    }
}

pub(crate) fn booth_pov(home: &home::Home, side: f32) -> Option<(Vec3, Vec3)> {
    let booth = home.pieces.iter().find(|p| p.item == home::Item::DjBooth)?;
    let c = booth.center();
    let base = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + home.piece_lift(booth), home::ORIGIN.y + c.y);
    let yaw = booth.yaw();
    let eye = base + rotate_y(vec3(side * 0.42, 1.55, 1.05), yaw);
    let target = base + rotate_y(vec3(side * 0.22, 0.84, 0.02), yaw);
    Some((eye, target))
}

pub(crate) fn render_home(target: &RenderTarget, left: f32, width: f32, height: f32, view: &View, home: &home::Home, ghost: Option<(home::Piece, bool)>, pov: Option<(Vec3, Vec3)>, hide: Option<usize>, beat_on: bool) {
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / target.texture.width(), -2.0 / height),
        target: vec2(target.texture.width() / 2.0, height / 2.0),
        ..Default::default()
    });
    draw_rectangle(left, 0.0, width, height, Color::from_rgba(38, 30, 52, 255));
    let center = vec3(home::ORIGIN.x + home.w() as f32 / 2.0, 0.5, home::ORIGIN.y + home.d() as f32 / 2.0);
    let scale = (home.w() as f32 / 7.0).max(home.d() as f32 / 5.0);
    let raise = if home.loft() { 0.35 } else { 0.0 };
    let (eye, look_at, half_fov) = match pov {
        Some((e, t)) => (e, t, 30.0_f32),
        None => (center + vec3(0.0, 5.6 * scale + raise, 6.2 * scale), center + vec3(0.0, 0.1 + raise, 0.2), 21.0),
    };
    let aspect = width / height;
    let eye = if pov.is_some() && aspect < 1.0 { look_at + (eye - look_at) * (0.8 / aspect).clamp(1.0, 1.6) } else { eye };
    let base = (half_fov.to_radians()).tan();
    let fovy = if aspect < 4.0 / 3.0 { 2.0 * (base * (4.0 / 3.0) / aspect).min(48f32.to_radians().tan()).atan() } else { 2.0 * base.atan() };
    set_camera(&Camera3D {
        position: eye,
        target: look_at,
        up: Vec3::Y,
        fovy,
        aspect: Some(aspect),
        render_target: Some(target.clone()),
        viewport: Some((left as i32, 0, width as i32, height as i32)),
        ..Default::default()
    });
    let flicker = 0.95 + 0.05 * (view.time * 7.0).sin();
    let throb = 0.85 + 0.15 * view.dance.max(0.0);
    let mut lamps = Vec::new();
    let mut red = false;
    for piece in home.pieces.iter().filter(|p| p.state == 0) {
        let c = piece.center();
        let lift = home.piece_lift(piece);
        match piece.item {
            home::Item::Lamp => lamps.push((vec3(home::ORIGIN.x + c.x, 1.4 + lift, home::ORIGIN.y + c.y), 1.0 * flicker, vec3(0.95, 0.58, 0.22))),
            home::Item::MirrorBall => {
                for n in 0..3 {
                    let a = view.time * 0.7 + n as f32 * 2.09;
                    let col = hsv(n as f32 / 3.0 + view.time * 0.05, 0.6, 1.0);
                    lamps.push((vec3(home::ORIGIN.x + c.x + a.cos() * 1.6, 0.6 + lift, home::ORIGIN.y + c.y + a.sin() * 1.6), 0.6, vec3(col.r, col.g, col.b)));
                }
            }
            home::Item::RedLight => {
                red = true;
                lamps.push((vec3(home::ORIGIN.x + c.x, 1.3 + lift, home::ORIGIN.y + c.y), 1.5 * throb, vec3(1.1, 0.08, 0.1)));
            }
            _ => {}
        }
    }
    let night = sky().night;
    let (strength, tint, dusk) = if red {
        (0.0, vec3(1.1, 0.08, 0.1), 0.92)
    } else if !lamps.is_empty() {
        (0.0, vec3(0.95, 0.58, 0.22), 0.22 + 0.25 * night)
    } else if home.wall == home::CONCRETE_WALL {
        (0.45, vec3(0.6, 0.62, 0.7), 0.3 + 0.4 * night)
    } else {
        (0.55, vec3(0.95, 0.58, 0.22), 0.2 + 0.5 * night)
    };
    let light = Light {
        dusk,
        fire: center + vec3(0.0, 1.8, 0.0),
        fire_strength: strength * flicker,
        tint,
        lamps,
    };
    let mut b = Batch::new(if pov.is_some() { eye } else { center + vec3(0.0, 5.6, -1.8) }, light);
    draw_home(&mut b, home, view.time, ghost, view.dance, beat_on);
    for (k, p) in view.players.iter().enumerate() {
        if hide == Some(k) || pov.map(|(e, _)| vec2(e.x, e.z).distance(p.pos) < 1.0).unwrap_or(false) {
            continue;
        }
        p.draw(&mut b, view.world, view.time, view.dance);
    }
    for &(p, life) in &view.extras.hearts {
        let c = Color::new(1.0, 0.45, 0.6, life.min(1.0));
        b.cube(p + vec3(-0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.0, -0.02, 0.0), vec3(0.06, 0.06, 0.03), c);
    }
    b.flush();
}

pub(crate) fn shop_row_info(row: home::Row, house: &home::Home) -> (String, String, String) {
    match row {
        home::Row::Upgrade => match house.next_upgrade() {
            Some((name, what, price)) => (format!("+ {}", name), format!("{} acorns", price), what.to_string()),
            None => ("home fully built".to_string(), "done".to_string(), "penthouse with loft".to_string()),
        },
        home::Row::Item(item) => (item.name().to_string(), format!("{} acorns", item.price()), item.note().to_string()),
        home::Row::Wall(i) => {
            let bit = home::wall_bit(i);
            let owned = house.owned_styles & bit != 0;
            let price = if house.wall as usize == i { "in use".to_string() } else if owned { "owned".to_string() } else { format!("{} acorns", home::STYLE_PRICE) };
            (home::WALLS[i].0.to_string(), price, "repaint the walls".to_string())
        }
        home::Row::Floor(i) => {
            let bit = home::floor_bit(i);
            let owned = house.owned_styles & bit != 0;
            let price = if house.floor as usize == i { "in use".to_string() } else if owned { "owned".to_string() } else { format!("{} acorns", home::STYLE_PRICE) };
            (home::FLOORS[i].0.to_string(), price, "change the floor pattern".to_string())
        }
        home::Row::Set => ("techno club set".to_string(), "free".to_string(), "concrete, red light, techno".to_string()),
    }
}

pub(crate) fn render_shop_preview(target: &RenderTarget, left: f32, top: f32, width: f32, height: f32, row: home::Row, house: &home::Home, time: f32, beat_on: bool) {
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / target.texture.width(), -2.0 / target.texture.height()),
        target: vec2(target.texture.width() / 2.0, target.texture.height() / 2.0),
        ..Default::default()
    });
    clear_background(Color::from_rgba(28, 24, 42, 255));
    let (wall_col, floor_a, floor_b) = match row {
        home::Row::Wall(i) => (home::WALLS[i].1, home::FLOORS[house.floor as usize].1, home::FLOORS[house.floor as usize].2),
        home::Row::Floor(i) => (home::WALLS[house.wall as usize].1, home::FLOORS[i].1, home::FLOORS[i].2),
        home::Row::Set => (home::WALLS[home::CONCRETE_WALL as usize].1, home::FLOORS[home::CONCRETE_FLOOR as usize].1, home::FLOORS[home::CONCRETE_FLOOR as usize].2),
        _ => (home::WALLS[house.wall as usize].1, home::FLOORS[house.floor as usize].1, home::FLOORS[house.floor as usize].2),
    };
    let base_center = vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y + 0.4, home::ORIGIN.y + 1.8);
    let eye = base_center + vec3(3.0, 2.6, 3.8);
    set_camera(&Camera3D {
        position: eye,
        target: base_center + vec3(0.0, 0.15, 0.0),
        up: Vec3::Y,
        fovy: 0.62,
        aspect: Some(width / height),
        render_target: Some(target.clone()),
        viewport: Some((left as i32, top as i32, width as i32, height as i32)),
        ..Default::default()
    });
    let light = Light {
        dusk: 0.4,
        fire: base_center + vec3(0.6, 1.8, -0.8),
        fire_strength: if beat_on { 0.9 } else { 0.7 },
        tint: vec3(0.95, 0.58, 0.22),
        lamps: Vec::new(),
    };
    let mut b = Batch::new(eye, light);
    let dark_wall = Color::new(wall_col.r * 0.84, wall_col.g * 0.84, wall_col.b * 0.84, 1.0);
    b.cube(vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y - 0.04, home::ORIGIN.y + 1.8), vec3(3.9, 0.06, 3.9), floor_a);
    b.cube(vec3(home::ORIGIN.x + 1.8, 1.1, home::ORIGIN.y - 0.15), vec3(3.9, 2.3, 0.08), wall_col);
    b.cube(vec3(home::ORIGIN.x - 0.15, 1.1, home::ORIGIN.y + 1.8), vec3(0.08, 2.3, 3.9), dark_wall);
    for n in 0..4 {
        let x = home::ORIGIN.x + 0.45 + n as f32 * 0.9;
        b.cube(vec3(x, home::FLOOR_Y - 0.015, home::ORIGIN.y + 1.8), vec3(0.02, 0.01, 3.7), floor_b);
    }
    for n in 0..4 {
        let z = home::ORIGIN.y + 0.45 + n as f32 * 0.9;
        b.cube(vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y - 0.015, z), vec3(3.7, 0.01, 0.02), floor_b);
    }
    let bob = 0.08 + 0.03 * (time * 2.0).sin();
    match row {
        home::Row::Item(item) => {
            let state = match item {
                home::Item::Plant => 3,
                home::Item::Table => 1,
                home::Item::Lamp | home::Item::Speaker | home::Item::Subwoofer | home::Item::RedLight => 0,
                _ => 0,
            };
            let piece = home::Piece { item, x: 1, z: 1, rot: 0, state, day: 0 };
            let c = piece.center();
            let shadow = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + 0.01, home::ORIGIN.y + c.y);
            let (fw, fd) = piece.footprint();
            b.cube(shadow, vec3(fw as f32 * 0.9 + 0.3, 0.02, fd as f32 * 0.9 + 0.3), Color::new(0.0, 0.0, 0.0, 0.18));
            draw_piece(&mut b, &piece, time, 1.0, true, if beat_on { (time * 6.0).sin() * 0.5 + 0.5 } else { 0.0 }, bob);
        }
        home::Row::Upgrade => {
            let piece = home::Piece { item: home::Item::Bed, x: 1, z: 1, rot: 0, state: 0, day: 0 };
            draw_piece(&mut b, &piece, time, 0.9, false, 0.0, 0.0);
            b.cube(vec3(home::ORIGIN.x + 3.2, 0.8, home::ORIGIN.y + 2.8), vec3(1.1, 1.6, 1.1), Color::new(1.0, 0.9, 0.5, 0.75));
        }
        home::Row::Wall(_) | home::Row::Floor(_) => {}
        home::Row::Set => {
            for piece in [
                home::Piece { item: home::Item::DjBooth, x: 1, z: 2, rot: 0, state: 0, day: 0 },
                home::Piece { item: home::Item::RedLight, x: 0, z: 0, rot: 0, state: 0, day: 0 },
                home::Piece { item: home::Item::ConcreteBench, x: 0, z: 2, rot: 0, state: 0, day: 0 },
            ] {
                draw_piece(&mut b, &piece, time, 1.0, false, 0.7, 0.0);
            }
        }
    }
    b.flush();
}
