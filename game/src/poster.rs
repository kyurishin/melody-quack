//! 90s rave flyers, drawn in pixel art. Every flyer is generated from its
//! rave's seed, so everyone in town sees the same one, and the secret code is
//! slapped on top at random spots and angles.

use crate::gfx::{hex, mix, pal, text_rot, text_w, with_alpha, Fonts};
use macroquad::prelude::*;
use mq_shared::{RaveInfo, Rng, Sound};
use std::f32::consts::{PI, TAU};

pub const FLYER_W: f32 = 144.0;
pub const FLYER_H: f32 = 200.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Flyer {
    pub code: String,
    pub title: String,
    pub host: String,
    pub host_is_dj: bool,
    pub seed: u64,
    pub sound: Sound,
    pub bpm: f32,
    pub live: bool,
    pub open: bool,
    /// An old flyer for a night that's over. Its code won't work.
    pub expired: bool,
}

impl Flyer {
    pub fn from_rave(r: &RaveInfo, server_ms: f64) -> Flyer {
        Flyer {
            code: r.code.clone(),
            title: r.title.clone(),
            host: r.host_name.clone(),
            host_is_dj: r.host_is_dj,
            seed: r.seed,
            sound: r.sound,
            bpm: r.bpm,
            live: r.live,
            open: server_ms >= r.doors_ms as f64,
            expired: false,
        }
    }

    /// Old, torn flyers for nights long gone, so the walls are never bare.
    pub fn decoys(seed: u64, n: usize) -> Vec<Flyer> {
        let mut r = Rng::new(seed);
        let titles = [
            "ACID MASS", "BASS VESPERS", "FEATHER FEVER", "POND LIFE", "MIDNIGHT MOULT", "DUCKSTEP",
            "QUACK HOUSE", "THE LONG WADDLE", "WARP CHAPEL", "NEST OF BASS",
        ];
        (0..n)
            .map(|_| Flyer {
                code: mq_shared::generate_code(&mut r),
                title: r.pick(&titles).to_string(),
                host: lineup_name(&mut r),
                host_is_dj: true,
                seed: r.next_u64(),
                sound: *r.pick(&Sound::ALL),
                bpm: r.range(128.0, 148.0).round(),
                live: false,
                open: false,
                expired: true,
            })
            .collect()
    }
}

fn lineup_name(r: &mut Rng) -> String {
    let a = [
        "DJ", "MC", "DJ", "THE", "LIL", "DJ",
    ];
    let b = [
        "MALLARD", "QUILL", "DOWNY", "PUDDLE", "DRAKE", "HERON", "WADDLE", "FEATHERS", "COO", "PLUME", "BILLY",
        "GOSLING", "TEAL", "EIDER",
    ];
    format!("{} {}", r.pick(&a), r.pick(&b))
}

struct Scheme {
    a: Color,
    b: Color,
    c: Color,
    ink: Color,
}

fn schemes() -> [Scheme; 6] {
    [
        Scheme { a: hex(0xfff23d), b: hex(0x111111), c: hex(0xff2fd0), ink: hex(0x111111) },
        Scheme { a: hex(0x2ff3ff), b: hex(0x5a1fd1), c: hex(0xfff23d), ink: hex(0x10082a) },
        Scheme { a: hex(0xff8a1f), b: hex(0x1f3dff), c: hex(0xffffff), ink: hex(0x0a0a2a) },
        Scheme { a: hex(0xb6ff3d), b: hex(0xff3d9a), c: hex(0x111111), ink: hex(0x111111) },
        Scheme { a: hex(0xff2a2a), b: hex(0xfff23d), c: hex(0x1f6bff), ink: hex(0x111111) },
        Scheme { a: hex(0xd9c8ff), b: hex(0x241a47), c: hex(0xff3df2), ink: hex(0x0d0a1a) },
    ]
}

pub fn render(f: &Flyer, fonts: &Fonts) -> Texture2D {
    let rt = render_target(FLYER_W as u32, FLYER_H as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, FLYER_W, FLYER_H));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);
    clear_background(BLANK);
    draw(f, fonts);
    // Render now, then copy into a plain texture: dropping a render target
    // deletes its texture too.
    unsafe { get_internal_gl() }.flush();
    set_default_camera();
    let mut img = rt.texture.get_texture_data();
    // GL render targets read back bottom-up.
    let row = img.width as usize * 4;
    let h = img.height as usize;
    for y in 0..h / 2 {
        let (top, bottom) = img.bytes.split_at_mut((h - 1 - y) * row);
        top[y * row..(y + 1) * row].swap_with_slice(&mut bottom[..row]);
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn draw(f: &Flyer, fonts: &Fonts) {
    let mut r = Rng::new(f.seed ^ 0xF1A7E5);
    let sch = &schemes()[r.below(6)];
    let (w, h) = (FLYER_W, FLYER_H);

    // Paper.
    draw_rectangle(0.0, 0.0, w, h, sch.b);

    // Background.
    match r.below(5) {
        0 => {
            // Sunburst.
            let (cx, cy) = (w / 2.0, h * r.range(0.35, 0.55));
            let n = 2 * (8 + r.below(6));
            for i in 0..n {
                if i % 2 == 0 {
                    let a0 = i as f32 / n as f32 * TAU;
                    let a1 = (i + 1) as f32 / n as f32 * TAU;
                    let rr = 400.0;
                    draw_triangle(
                        vec2(cx, cy),
                        vec2(cx + a0.cos() * rr, cy + a0.sin() * rr),
                        vec2(cx + a1.cos() * rr, cy + a1.sin() * rr),
                        sch.a,
                    );
                }
            }
        }
        1 => {
            // Perspective checker floor under a banded sky.
            let horizon = h * 0.55;
            for i in 0..8 {
                let y = i as f32 * horizon / 8.0;
                draw_rectangle(0.0, y, w, horizon / 8.0 + 1.0, mix(sch.b, sch.c, i as f32 / 10.0));
            }
            draw_rectangle(0.0, horizon, w, h - horizon, sch.ink);
            let rows = 9;
            for row in 0..rows {
                let t0 = row as f32 / rows as f32;
                let t1 = (row + 1) as f32 / rows as f32;
                let y0 = horizon + (h - horizon) * t0 * t0;
                let y1 = horizon + (h - horizon) * t1 * t1;
                for col in -8..8 {
                    if (row as i32 + col).rem_euclid(2) == 0 {
                        let x = |t: f32, c: f32| w / 2.0 + c * 18.0 * (0.15 + t * 1.6);
                        draw_triangle(vec2(x(t0, col as f32), y0), vec2(x(t0, col as f32 + 1.0), y0), vec2(x(t1, col as f32 + 1.0), y1), sch.a);
                        draw_triangle(vec2(x(t0, col as f32), y0), vec2(x(t1, col as f32), y1), vec2(x(t1, col as f32 + 1.0), y1), sch.a);
                    }
                }
            }
        }
        2 => {
            // Concentric rings.
            let (cx, cy) = (w * r.range(0.3, 0.7), h * r.range(0.3, 0.6));
            let mut rr = 260.0;
            let mut i = 0;
            while rr > 4.0 {
                draw_circle(cx, cy, rr, if i % 2 == 0 { sch.a } else { sch.b });
                rr -= 10.0;
                i += 1;
            }
        }
        3 => {
            // Wavy stripes.
            for y in 0..(h as i32) {
                let phase = (y as f32 * 0.09).sin() * 10.0;
                let mut x = -20.0 + phase;
                let mut k = 0;
                while x < w + 20.0 {
                    if k % 2 == 0 {
                        draw_rectangle(x, y as f32, 8.0, 1.0, sch.a);
                    }
                    x += 8.0;
                    k += 1;
                }
            }
        }
        _ => {
            // Night sky, stars and a spiral.
            draw_rectangle(0.0, 0.0, w, h, sch.ink);
            for _ in 0..80 {
                let c = if r.chance(0.3) { sch.a } else { sch.c };
                draw_rectangle(r.range(0.0, w).floor(), r.range(0.0, h).floor(), 1.0, 1.0, c);
            }
            let (cx, cy) = (w / 2.0, h * 0.45);
            for i in 0..220 {
                let t = i as f32 * 0.12;
                let rr = t * 6.5;
                draw_rectangle((cx + t.cos() * rr).floor(), (cy + t.sin() * rr).floor(), 2.0, 2.0, sch.a);
            }
        }
    }

    // Centerpiece.
    let (cx, cy) = (w * r.range(0.4, 0.6), h * r.range(0.38, 0.5));
    let rad = r.range(26.0, 36.0);
    match r.below(5) {
        0 => smiley(cx, cy, rad, sch, &mut r),
        1 => globe(cx, cy, rad, sch),
        2 => eye_pyramid(cx, cy, rad, sch),
        3 => duck(cx, cy, rad, sch),
        _ => bolt(cx, cy, rad, sch),
    }

    // Title, in a band so it always reads.
    let title = f.title.as_str();
    let band_y = if r.chance(0.5) { 10.0 } else { h * 0.62 };
    let words: Vec<&str> = title.split_whitespace().collect();
    let lines: Vec<String> = if title.len() > 9 && words.len() > 1 {
        let mid = words.len().div_ceil(2);
        vec![words[..mid].join(" "), words[mid..].join(" ")]
    } else {
        vec![title.to_string()]
    };
    let mut y = band_y;
    for line in &lines {
        let mut size = 26.0;
        while text_w(&fonts.title, line, size) > w - 12.0 && size > 10.0 {
            size -= 1.0;
        }
        let tw = text_w(&fonts.title, line, size);
        let x = (w - tw) / 2.0;
        let base = y + size * 0.8;
        // Chunky outline + drop shadow, like a photocopied flyer.
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (3.0, 3.0), (2.0, 3.0), (3.0, 2.0)] {
            text_rot(&fonts.title, line, x + dx, base + dy, size, sch.ink, 0.0);
        }
        text_rot(&fonts.title, line, x, base, size, sch.c, 0.0);
        y += size * 0.95;
    }

    // Line-up and details.
    let mut info = Vec::new();
    let mut lr = Rng::new(f.seed);
    if f.host_is_dj {
        info.push(format!("DJ {}", f.host.to_uppercase().trim_start_matches("DJ ")));
    } else {
        info.push(format!("HOSTED BY {}", f.host.to_uppercase()));
    }
    info.push(format!("{} B2B {}", lineup_name(&mut lr), lineup_name(&mut lr)));
    info.push(format!("{} / {} BPM", f.sound.label(), f.bpm.round()));
    info.push(if f.expired {
        "ONE NIGHT ONLY".into()
    } else if f.live {
        "LIVE SET NOW".into()
    } else if f.open {
        "DOORS OPEN".into()
    } else {
        "DOORS SOON".into()
    });
    info.push("ST. BASSILICA".into());
    let block_h = info.len() as f32 * 9.0 + 6.0;
    let by = if band_y < 20.0 { h - block_h - 4.0 } else { 6.0 };
    draw_rectangle(4.0, by - 1.0, w - 8.0, block_h, with_alpha(sch.ink, 0.85));
    for (i, line) in info.iter().enumerate() {
        let size = 8.0;
        let tw = text_w(&fonts.small, line, size);
        let col = if i == 0 { sch.a } else { pal::WHITE };
        text_rot(&fonts.small, line, ((w - tw) / 2.0).floor(), by + 8.0 + i as f32 * 9.0, size, col, 0.0);
    }

    // The codes, slapped on top.
    let code = f.code.as_str();
    // 1. A white sticker.
    {
        let rot = r.range(-0.5, 0.5);
        let (sx, sy) = (r.range(14.0, w - 70.0), r.range(h * 0.2, h * 0.75));
        let tw = text_w(&fonts.mono, code, 16.0) + 8.0;
        draw_rectangle_ex(
            sx + 2.0,
            sy + 2.0,
            tw,
            16.0,
            DrawRectangleParams { rotation: rot, color: with_alpha(pal::BLACK, 0.5), ..Default::default() },
        );
        draw_rectangle_ex(sx, sy, tw, 16.0, DrawRectangleParams { rotation: rot, color: pal::WHITE, ..Default::default() });
        let (s, c) = rot.sin_cos();
        let (ox, oy) = (4.0, 13.0);
        text_rot(&fonts.mono, code, sx + ox * c - oy * s, sy + ox * s + oy * c, 16.0, pal::BLACK, rot);
    }
    // 2. A marker scrawl.
    {
        let rot = r.range(-1.1, 1.1);
        let (sx, sy) = (r.range(8.0, w - 60.0), r.range(30.0, h - 30.0));
        let col = if r.chance(0.5) { pal::ACID } else { pal::MAGENTA };
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            text_rot(&fonts.mono, code, sx + dx, sy + dy, 14.0, pal::BLACK, rot);
        }
        text_rot(&fonts.mono, code, sx, sy, 14.0, col, rot);
    }
    // 3. A tiny stamp down the edge.
    {
        let x = if r.chance(0.5) { 3.0 } else { w - 4.0 };
        text_rot(&fonts.small, code, x + 6.0, h * 0.25, 8.0, sch.c, PI / 2.0);
    }

    // Tape.
    for (x, y, rot) in [(-4.0, -2.0, -0.6), (w - 18.0, -4.0, 0.6)] {
        draw_rectangle_ex(
            x,
            y,
            24.0,
            9.0,
            DrawRectangleParams { rotation: rot, color: Color::new(0.95, 0.9, 0.75, 0.75), ..Default::default() },
        );
    }

    // Expired flyers: faded, ripped, crossed out.
    if f.expired {
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.85, 0.8, 0.7, 0.35));
        let mut tr = Rng::new(f.seed ^ 77);
        let y0 = tr.range(h * 0.55, h * 0.85);
        let mut pts = Vec::new();
        let mut x = 0.0;
        while x <= w {
            pts.push(vec2(x, y0 + tr.range(-6.0, 6.0)));
            x += 6.0;
        }
        for p in pts.windows(2) {
            draw_triangle(p[0], p[1], vec2(p[1].x, h + 2.0), BLANK);
            draw_triangle(p[0], vec2(p[0].x, h + 2.0), vec2(p[1].x, h + 2.0), BLANK);
            draw_triangle(p[0], p[1], vec2(p[1].x, h + 2.0), pal::CONCRETE);
            draw_triangle(p[0], vec2(p[0].x, h + 2.0), vec2(p[1].x, h + 2.0), pal::CONCRETE);
        }
        draw_line(10.0, 20.0, w - 10.0, y0 - 20.0, 3.0, with_alpha(hex(0xd92a2a), 0.85));
    }

    // Border.
    draw_rectangle_lines(0.0, 0.0, w, h, 2.0, pal::INK);
}

fn smiley(cx: f32, cy: f32, r: f32, s: &Scheme, rng: &mut Rng) {
    draw_circle(cx + 3.0, cy + 3.0, r, s.ink);
    draw_circle(cx, cy, r, hex(0xfff23d));
    draw_circle_lines(cx, cy, r, 2.0, s.ink);
    let wonky = rng.range(-3.0, 3.0);
    draw_ellipse(cx - r * 0.33, cy - r * 0.25 + wonky * 0.3, r * 0.1, r * 0.22, 0.0, s.ink);
    draw_ellipse(cx + r * 0.33, cy - r * 0.25, r * 0.1, r * 0.22, 0.0, s.ink);
    for i in 0..24 {
        let a = PI * 0.15 + PI * 0.7 * i as f32 / 23.0;
        draw_circle(cx + a.cos() * r * 0.55, cy + a.sin() * r * 0.55, r * 0.07, s.ink);
    }
}

fn globe(cx: f32, cy: f32, r: f32, s: &Scheme) {
    draw_circle(cx, cy, r, s.ink);
    for i in 1..4 {
        let k = i as f32 / 4.0;
        draw_ellipse_lines(cx, cy, r * k, r, 0.0, 1.0, s.c);
        draw_line(cx - r * (1.0 - (k - 0.5).powi(2) * 2.0).sqrt().min(1.0), cy - r + 2.0 * r * k, cx + r * (1.0 - (k - 0.5).powi(2) * 2.0).sqrt().min(1.0), cy - r + 2.0 * r * k, 1.0, s.c);
    }
    draw_line(cx - r, cy, cx + r, cy, 1.0, s.c);
    draw_circle_lines(cx, cy, r, 2.0, s.c);
}

fn eye_pyramid(cx: f32, cy: f32, r: f32, s: &Scheme) {
    let a = vec2(cx, cy - r * 1.1);
    let b = vec2(cx - r * 1.1, cy + r * 0.8);
    let c = vec2(cx + r * 1.1, cy + r * 0.8);
    draw_triangle(a + vec2(3.0, 3.0), b + vec2(3.0, 3.0), c + vec2(3.0, 3.0), s.ink);
    draw_triangle(a, b, c, s.c);
    draw_triangle_lines(a, b, c, 2.0, s.ink);
    draw_ellipse(cx, cy + r * 0.1, r * 0.45, r * 0.25, 0.0, pal::WHITE);
    draw_circle(cx, cy + r * 0.1, r * 0.18, s.ink);
    for i in 0..7 {
        let ang = -PI / 2.0 + (i as f32 - 3.0) * 0.25;
        draw_line(cx, cy - r * 1.1, cx + ang.cos() * r * 2.0, cy - r * 1.1 + ang.sin() * r * 2.0, 1.0, s.c);
    }
}

fn duck(cx: f32, cy: f32, r: f32, s: &Scheme) {
    draw_circle(cx + 3.0, cy + 3.0, r, s.ink);
    draw_circle(cx, cy, r, hex(0xffd66e));
    draw_ellipse(cx + r * 0.75, cy + r * 0.2, r * 0.55, r * 0.22, 0.0, hex(0xff8a1f));
    draw_rectangle(cx - r * 0.35, cy - r * 0.35, r * 0.6, r * 0.22, s.ink); // shades
    draw_rectangle(cx + r * 0.3, cy - r * 0.35, r * 0.4, r * 0.22, s.ink);
    draw_line(cx - r * 0.35, cy - r * 0.25, cx - r, cy - r * 0.35, 2.0, s.ink);
    draw_circle_lines(cx, cy, r, 2.0, s.ink);
}

fn bolt(cx: f32, cy: f32, r: f32, s: &Scheme) {
    let pts = [
        vec2(cx + r * 0.3, cy - r * 1.2),
        vec2(cx - r * 0.6, cy + r * 0.1),
        vec2(cx - r * 0.05, cy + r * 0.1),
        vec2(cx - r * 0.35, cy + r * 1.2),
        vec2(cx + r * 0.65, cy - r * 0.2),
        vec2(cx + r * 0.1, cy - r * 0.2),
    ];
    for off in [vec2(4.0, 4.0), vec2(0.0, 0.0)] {
        let col = if off.x > 0.0 { s.ink } else { s.c };
        draw_triangle(pts[0] + off, pts[1] + off, pts[5] + off, col);
        draw_triangle(pts[1] + off, pts[2] + off, pts[5] + off, col);
        draw_triangle(pts[2] + off, pts[3] + off, pts[4] + off, col);
        draw_triangle(pts[2] + off, pts[4] + off, pts[5] + off, col);
    }
}
