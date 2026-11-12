//! Palette, a little pixel canvas for drawing sprites in code, fonts and an
//! additive-blend material for lights and lasers.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

pub const fn hex(c: u32) -> Color {
    Color::new(
        ((c >> 16) & 0xff) as f32 / 255.0,
        ((c >> 8) & 0xff) as f32 / 255.0,
        (c & 0xff) as f32 / 255.0,
        1.0,
    )
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// Colours from the Melody Quack cover, plus night-club neons.
pub mod pal {
    use super::hex;
    use macroquad::color::Color;
    pub const INK: Color = hex(0x241a47);
    pub const INK2: Color = hex(0x342866);
    pub const DUSK: Color = hex(0x3b2a6b);
    pub const DUSK2: Color = hex(0x5a3f8f);
    pub const CREAM: Color = hex(0xfff7e0);
    pub const YELLOW: Color = hex(0xffd66e);
    pub const CORAL: Color = hex(0xff8062);
    pub const PINK: Color = hex(0xffa8cc);
    pub const BLUE: Color = hex(0x74acff);
    pub const GREEN: Color = hex(0x8cdea8);
    pub const SKY: Color = hex(0xbae6ff);
    pub const PURPLE: Color = hex(0x9b6bff);
    pub const DJ: Color = hex(0xb05cff);
    pub const MAGENTA: Color = hex(0xff3df2);
    pub const CYAN: Color = hex(0x3df2ff);
    pub const ACID: Color = hex(0xc6ff3d);
    pub const CONCRETE: Color = hex(0x8e8a99);
    pub const CONCRETE_D: Color = hex(0x625e70);
    pub const CONCRETE_DD: Color = hex(0x403c4f);
    pub const GRASS: Color = hex(0x4a7a5c);
    pub const GRASS_D: Color = hex(0x3b6650);
    pub const GRASS_L: Color = hex(0x5e9468);
    pub const PATH: Color = hex(0xc9a982);
    pub const PATH_D: Color = hex(0xa88a6a);
    pub const WOOD: Color = hex(0x8a5a3c);
    pub const WOOD_D: Color = hex(0x5e3a2a);
    pub const BEIGE: Color = hex(0xe8dcc0);
    pub const BEIGE_D: Color = hex(0xbfb294);
    pub const W95_TEAL: Color = hex(0x0f8f8a);
    pub const W95_GREY: Color = hex(0xc6c3bd);
    pub const W95_DARK: Color = hex(0x7d7a74);
    pub const W95_NAVY: Color = hex(0x1b2a8a);
    pub const BLACK: Color = hex(0x0d0a1a);
    pub const WHITE: Color = hex(0xffffff);
}

// ---------------------------------------------------------------------------
// Canvas
// ---------------------------------------------------------------------------

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    px: Vec<[u8; 4]>,
}

fn to_rgba(c: Color) -> [u8; 4] {
    [
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    ]
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Canvas {
        Canvas { w, h, px: vec![[0, 0, 0, 0]; (w * h) as usize] }
    }

    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = to_rgba(c);
        }
    }

    pub fn get_alpha(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize][3]
        } else {
            0
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    pub fn hline(&mut self, x: i32, y: i32, w: i32, c: Color) {
        self.rect(x, y, w, 1, c);
    }

    pub fn vline(&mut self, x: i32, y: i32, h: i32, c: Color) {
        self.rect(x, y, 1, h, c);
    }

    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        self.hline(x, y, w, c);
        self.hline(x, y + h - 1, w, c);
        self.vline(x, y, h, c);
        self.vline(x + w - 1, y, h, c);
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Color) {
        let (x0, x1) = ((cx - rx).floor() as i32, (cx + rx).ceil() as i32);
        let (y0, y1) = ((cy - ry).floor() as i32, (cy + ry).ceil() as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x, y, c);
                }
            }
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, c: Color) {
        self.ellipse(cx, cy, r, r, c);
    }

    pub fn tri(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), col: Color) {
        let minx = a.0.min(b.0).min(c.0).floor() as i32;
        let maxx = a.0.max(b.0).max(c.0).ceil() as i32;
        let miny = a.1.min(b.1).min(c.1).floor() as i32;
        let maxy = a.1.max(b.1).max(c.1).ceil() as i32;
        let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0);
        for y in miny..=maxy {
            for x in minx..=maxx {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let e0 = edge(a, b, px, py);
                let e1 = edge(b, c, px, py);
                let e2 = edge(c, a, px, py);
                if (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0) {
                    self.set(x, y, col);
                }
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Ordered 2x2 dither between two colours, `t` = share of `b`.
    pub fn dither(&mut self, x: i32, y: i32, w: i32, h: i32, a: Color, b: Color, t: f32) {
        const M: [[f32; 4]; 4] = [
            [0.0, 8.0, 2.0, 10.0],
            [12.0, 4.0, 14.0, 6.0],
            [3.0, 11.0, 1.0, 9.0],
            [15.0, 7.0, 13.0, 5.0],
        ];
        for yy in y..y + h {
            for xx in x..x + w {
                let th = (M[(yy & 3) as usize][(xx & 3) as usize] + 0.5) / 16.0;
                self.set(xx, yy, if t > th { b } else { a });
            }
        }
    }

    /// Vertical gradient, dithered.
    pub fn vgrad(&mut self, x: i32, y: i32, w: i32, h: i32, top: Color, bottom: Color) {
        for yy in 0..h {
            let t = yy as f32 / (h - 1).max(1) as f32;
            self.dither(x, y + yy, w, 1, top, bottom, t);
        }
    }

    /// Clear every pixel where `f` is false.
    pub fn keep(&mut self, f: impl Fn(i32, i32) -> bool) {
        for y in 0..self.h {
            for x in 0..self.w {
                if !f(x, y) {
                    self.px[(y * self.w + x) as usize] = [0, 0, 0, 0];
                }
            }
        }
    }

    /// Paint `c` on every transparent pixel touching an opaque one.
    pub fn outline(&mut self, c: Color) {
        let mut marks = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.get_alpha(x, y) == 0
                    && (self.get_alpha(x - 1, y) > 0
                        || self.get_alpha(x + 1, y) > 0
                        || self.get_alpha(x, y - 1) > 0
                        || self.get_alpha(x, y + 1) > 0)
                {
                    marks.push((x, y));
                }
            }
        }
        for (x, y) in marks {
            self.set(x, y, c);
        }
    }

    /// Speckle random pixels with `c` (deterministic per position).
    pub fn speckle(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color, density: f32, seed: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                let mut v = (xx as u32).wrapping_mul(374761393) ^ (yy as u32).wrapping_mul(668265263) ^ seed;
                v = (v ^ (v >> 13)).wrapping_mul(1274126177);
                if (v & 0xffff) as f32 / 65535.0 < density {
                    self.set(xx, yy, c);
                }
            }
        }
    }

    pub fn texture(&self) -> Texture2D {
        let mut bytes = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            bytes.extend_from_slice(p);
        }
        let t = Texture2D::from_rgba8(self.w as u16, self.h as u16, &bytes);
        t.set_filter(FilterMode::Nearest);
        t
    }
}

/// 5x7 pixel font for lettering baked into sprites (signs).
pub fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '.' => [0, 0, 0, 0, 0, 0x0c, 0x0c],
        _ => [0; 7],
    }
}

pub fn bake_text(c: &mut Canvas, x: i32, y: i32, text: &str, col: Color, scale: i32) {
    let mut cx = x;
    for ch in text.chars() {
        if ch == ' ' {
            cx += 4 * scale;
            continue;
        }
        let g = glyph(ch);
        for (row, bits) in g.iter().enumerate() {
            for b in 0..5 {
                if bits & (0x10 >> b) != 0 {
                    c.rect(cx + b * scale, y + row as i32 * scale, scale, scale, col);
                }
            }
        }
        cx += 6 * scale;
    }
}

/// Hand-drawn 방 (bang), 11x11, for the PC방 sign.
pub fn bake_bang(c: &mut Canvas, x: i32, y: i32, col: Color, scale: i32) {
    const ROWS: [&str; 11] = [
        "X...X....X.",
        "X...X....X.",
        "XXXXX....XX",
        "X...X....X.",
        "XXXXX....X.",
        "...........",
        "...XXXXX...",
        "..X.....X..",
        "..X.....X..",
        "..X.....X..",
        "...XXXXX...",
    ];
    for (r, row) in ROWS.iter().enumerate() {
        for (i, ch) in row.chars().enumerate() {
            if ch == 'X' {
                c.rect(x + i as i32 * scale, y + r as i32 * scale, scale, scale, col);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

pub struct Fonts {
    pub title: Font,
    pub body: Font,
    pub mono: Font,
    pub small: Font,
}

impl Fonts {
    pub fn load() -> Fonts {
        let load = |bytes: &[u8]| {
            let mut f = load_ttf_font_from_bytes(bytes).expect("bundled font");
            f.set_filter(FilterMode::Nearest);
            f
        };
        Fonts {
            title: load(include_bytes!("../assets/fonts/PixelifySans.ttf")),
            body: load(include_bytes!("../assets/fonts/PixelifySans.ttf")),
            mono: load(include_bytes!("../assets/fonts/VT323.ttf")),
            small: load(include_bytes!("../assets/fonts/Silkscreen.ttf")),
        }
    }
}

pub fn text(font: &Font, s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(
        s,
        x.round(),
        y.round(),
        TextParams {
            font: Some(font),
            font_size: size.max(1.0).round() as u16,
            color,
            ..Default::default()
        },
    );
}

pub fn text_rot(font: &Font, s: &str, x: f32, y: f32, size: f32, color: Color, rotation: f32) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: size.max(1.0).round() as u16,
            color,
            rotation,
            ..Default::default()
        },
    );
}

pub fn text_w(font: &Font, s: &str, size: f32) -> f32 {
    measure_text(s, Some(font), size.max(1.0).round() as u16, 1.0).width
}

/// Text with the cover's hard drop shadow.
pub fn text_shadow(font: &Font, s: &str, x: f32, y: f32, size: f32, color: Color, shadow: f32) {
    text(font, s, x + shadow, y + shadow, size, pal::INK);
    text(font, s, x, y, size, color);
}

pub fn text_center(font: &Font, s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let w = text_w(font, s, size);
    text(font, s, cx - w / 2.0, y, size, color);
}

// ---------------------------------------------------------------------------
// Additive blending
// ---------------------------------------------------------------------------

const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const FRAG: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;

pub fn additive_material() -> Material {
    load_material(
        ShaderSource::Glsl { vertex: VERT, fragment: FRAG },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("additive material")
}

/// Soft round light, white, for tinting.
pub fn glow_texture() -> Texture2D {
    let n = 64;
    let mut bytes = Vec::with_capacity(n * n * 4);
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let d = (dx * dx + dy * dy).sqrt();
            let a = (1.0 - d).max(0.0).powf(1.8);
            bytes.extend_from_slice(&[255, 255, 255, (a * 255.0) as u8]);
        }
    }
    let t = Texture2D::from_rgba8(n as u16, n as u16, &bytes);
    t.set_filter(FilterMode::Linear);
    t
}

pub fn draw_glow(tex: &Texture2D, x: f32, y: f32, r: f32, c: Color) {
    draw_texture_ex(
        tex,
        x - r,
        y - r,
        c,
        DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() },
    );
}
