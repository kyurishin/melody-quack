#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn screen_aspect() -> f32 {
    screen_width() / screen_height().max(1.0)
}

pub(crate) fn low_w() -> f32 {
    let aspect = screen_aspect();
    if aspect >= 1.0 { (LOW_H as f32 * aspect).round().clamp(240.0, 720.0) } else { LOW_H as f32 }
}

pub(crate) fn low_h() -> f32 {
    let aspect = screen_aspect();
    if aspect >= 1.0 { LOW_H as f32 } else { (LOW_H as f32 / aspect).round().clamp(240.0, 560.0) }
}

pub(crate) const LOW_H: u32 = 270;

pub(crate) const VIEW_W: u32 = 360;

pub(crate) const VIEW_H: u32 = 270;

pub(crate) fn help_box() -> (f32, f32, f32, f32) {
    let w = 300f32.min(low_w() - 12.0);
    let h = 210f32.min(low_h() - 24.0);
    (((low_w() - w) / 2.0).round(), ((low_h() - h) / 2.0).round().max(20.0), w, h)
}

pub(crate) fn ui_scale() -> (f32, f32) {
    (screen_width() / low_w(), screen_height() / low_h())
}

pub(crate) fn ui_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let (kx, ky) = ui_scale();
    draw_rectangle((x * kx).round(), (y * ky).round(), (w * kx).round().max(1.0), (h * ky).round().max(1.0), color);
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Face {
    Body,
    Label,
    Display,
    Data,
    Accent,
    Icon,
}

impl Face {
    fn index(self) -> usize {
        self as usize
    }

    fn grid(self) -> f32 {
        if self == Face::Display { 7.0 } else { 8.0 }
    }

    fn scale(self) -> f32 {
        match self {
            Face::Display => 1.25,
            Face::Data => 0.85,
            Face::Accent => 0.62,
            _ => 1.0,
        }
    }
}

thread_local! {
    static UI_FONTS: std::cell::RefCell<Vec<Option<Font>>> = const { std::cell::RefCell::new(Vec::new()) };
    static PIXEL_TEXT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FACE: std::cell::Cell<Face> = const { std::cell::Cell::new(Face::Body) };
}

pub(crate) fn load_ui_font() {
    let files: [&[u8]; 6] = [
        include_bytes!("../assets/fonts/04B_03.ttf"),
        include_bytes!("../assets/fonts/04B_09.ttf"),
        include_bytes!("../assets/fonts/04B_19.ttf"),
        include_bytes!("../assets/fonts/04B_11.ttf"),
        include_bytes!("../assets/fonts/04B_20.ttf"),
        include_bytes!("../assets/fonts/04B_21.ttf"),
    ];
    let fonts: Vec<Option<Font>> = files
        .iter()
        .map(|bytes| {
            load_ttf_font_from_bytes(bytes).ok().map(|mut f| {
                f.set_filter(FilterMode::Nearest);
                f
            })
        })
        .collect();
    UI_FONTS.with(|f| *f.borrow_mut() = fonts);
}

pub(crate) fn pixel_text(on: bool) {
    PIXEL_TEXT.with(|p| p.set(on));
}

pub(crate) fn with_face<T>(face: Face, draw: impl FnOnce() -> T) -> T {
    let before = FACE.with(|f| f.replace(face));
    let out = draw();
    FACE.with(|f| f.set(before));
    out
}

pub(crate) fn ui_text_as(face: Face, text: &str, x: f32, y: f32, size: f32, color: Color) {
    with_face(face, || ui_text(text, x, y, size, color));
}

pub(crate) fn ui_text_bold(face: Face, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let (kx, ky) = ui_scale();
    let o = (ky / 1.5).max(1.0).round();
    let edge = Color::new(0.1, 0.06, 0.18, color.a);
    with_face(face, || {
        for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (o, o), (-o, o)] {
            ui_text_raw(text, x + dx / kx, y + dy / ky, size, edge);
        }
        ui_text_raw(text, x, y, size, color);
    });
}

pub(crate) fn ui_text_fit(text: &str, x: f32, y: f32, size: f32, max_w: f32, color: Color) {
    let mut s = size;
    while s > 5.0 && ui_measure(text, s) > max_w {
        s -= 1.0;
    }
    ui_text(text, x, y, s, color);
}

pub(crate) fn ui_measure_as(face: Face, text: &str, size: f32) -> f32 {
    with_face(face, || ui_measure(text, size))
}

fn current() -> (Option<Font>, Face) {
    let face = FACE.with(|f| f.get());
    if PIXEL_TEXT.with(|p| p.get()) {
        return (None, face);
    }
    (UI_FONTS.with(|f| f.borrow().get(face.index()).cloned().flatten()), face)
}


fn font_px(size: f32, ky: f32, font: &Option<Font>, face: Face) -> u16 {
    if font.is_some() {
        let px = size * ky * face.scale();
        ((px / face.grid()).round().max(1.0) * face.grid()) as u16
    } else {
        (size * ky).round().max(1.0) as u16
    }
}

pub(crate) fn ui_measure(text: &str, size: f32) -> f32 {
    let (kx, ky) = ui_scale();
    let (font, face) = current();
    measure_text(text, font.as_ref(), font_px(size, ky, &font, face), 1.0).width / kx
}

pub(crate) fn ui_wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let joined = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if ui_measure(&joined, size) <= width {
            line = joined;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for ch in word.chars() {
            line.push(ch);
            if ui_measure(&line, size) > width && line.chars().count() > 1 {
                let last = line.pop().unwrap();
                lines.push(std::mem::take(&mut line));
                line.push(last);
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub(crate) fn ui_ticker(text: &str, size: f32, width: f32, time: f32) -> String {
    if ui_measure(text, size) <= width {
        return text.to_string();
    }
    let looped: Vec<char> = format!("{text}     ").chars().collect();
    let start = (time * 5.0) as usize % looped.len();
    let mut out = String::new();
    for k in 0..looped.len() * 2 {
        let ch = looped[(start + k) % looped.len()];
        out.push(ch);
        if ui_measure(&out, size) > width {
            out.pop();
            break;
        }
    }
    out
}

pub(crate) fn ui_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    text_at(text, x, y, size, color, true);
}

pub(crate) fn ui_text_raw(text: &str, x: f32, y: f32, size: f32, color: Color) {
    text_at(text, x, y, size, color, false);
}

fn text_at(text: &str, x: f32, y: f32, size: f32, color: Color, with_shadow: bool) {
    let (kx, ky) = ui_scale();
    let (font, face) = current();
    let mut font_size = font_px(size, ky, &font, face);
    let sx0 = (x * kx).round();
    if font.is_some() {
        let step = face.grid() as u16;
        while font_size > step && sx0 + measure_text(text, font.as_ref(), font_size, 1.0).width > screen_width() - 2.0 {
            font_size -= step;
        }
    }
    let shadow = (ky * 0.4).max(1.0).round();
    let lift = if font.is_some() { (font_size as f32 / 8.0).round() } else { 0.0 };
    let (sx, sy) = ((x * kx).round(), (y * ky).round() - lift);
    if with_shadow {
        draw_text_ex(
        text,
        sx + shadow,
        sy + shadow,
        TextParams {
            font: font.as_ref(),
            font_size,
            color: Color::new(0.05, 0.04, 0.1, color.a * 0.75),
            ..Default::default()
        },
        );
    }
    draw_text_ex(
        text,
        sx,
        sy,
        TextParams {
            font: font.as_ref(),
            font_size,
            color,
            ..Default::default()
        },
    );
}

pub(crate) fn draw_pixel_grid(pixels: &[(usize, usize, Color)], x: f32, y: f32, px: f32, tint: impl Fn(Color) -> Color) {
    let (kx, ky) = ui_scale();
    let cell = (px * kx.min(ky)).round().max(1.0);
    let (ox, oy) = ((x * kx).round(), (y * ky).round());
    for (ax, ay, c) in pixels {
        draw_rectangle(ox + *ax as f32 * cell, oy + *ay as f32 * cell, cell, cell, tint(*c));
    }
}

pub(crate) fn draw_fish_art(art: &[(usize, usize, Color)], x: f32, y: f32, px: f32, known: bool) {
    draw_pixel_grid(art, x, y, px, |c| if known { c } else { Color::new(0.27, 0.3, 0.44, 1.0) });
}

pub(crate) fn draw_emoji_ui(id: usize, x: f32, y: f32, px: f32, alpha: f32) {
    draw_pixel_grid(&emoji::pixels(id), x, y, px, |c| Color::new(c.r, c.g, c.b, alpha));
}

pub(crate) fn emoji_strip(who: usize, two_player: bool) -> (f32, f32, f32) {
    let cell = 13.0;
    let w = cell * emoji::COUNT as f32;
    let half = low_w() / 2.0;
    let x = if two_player { who as f32 * half + (half - w) / 2.0 } else { (low_w() - w) / 2.0 };
    let y = if low_h() > low_w() { 96.0 } else { 58.0 };
    (x, y, cell)
}

pub(crate) fn wrap_px(text: &str, size: u16, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
        if !line.is_empty() && measure_text(&candidate, None, size, 1.0).width > width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub(crate) fn ui_star(x: f32, y: f32, size: f32, color: Color) {
    let a = size / 5.0;
    ui_rect(x + a * 2.0, y, a, size, color);
    ui_rect(x, y + a * 2.0, size, a, color);
    ui_rect(x + a, y + a, a * 3.0, a * 3.0, color);
}

pub(crate) fn file_stem(name: &str) -> String {
    let name = name.trim();
    let name = name.strip_suffix(".wav").or_else(|| name.strip_suffix(".WAV")).unwrap_or(name);
    let clean: String = name.chars().filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '(' | ')')).take(48).collect();
    let clean = clean.trim().trim_start_matches('.').trim().to_string();
    if clean.is_empty() { "jam".to_string() } else { clean }
}

fn round_points(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Vec2> {
    let (kx, ky) = ui_scale();
    let (px, py, pw, ph) = (x * kx, y * ky, w * kx, h * ky);
    let rr = (r * kx.min(ky)).min(pw / 2.0).min(ph / 2.0).max(0.0);
    let corners = [(px + pw - rr, py + rr, -0.5), (px + pw - rr, py + ph - rr, 0.0), (px + rr, py + ph - rr, 0.5), (px + rr, py + rr, 1.0)];
    let mut pts = Vec::with_capacity(32);
    for (cx, cy, start) in corners {
        for k in 0..=7 {
            let a = (start + k as f32 / 7.0 * 0.5) * std::f32::consts::PI;
            pts.push(vec2(cx + rr * a.cos(), cy + rr * a.sin()));
        }
    }
    pts
}

pub(crate) fn ui_round(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let pts = round_points(x, y, w, h, r);
    let (kx, ky) = ui_scale();
    let c = vec2((x + w / 2.0) * kx, (y + h / 2.0) * ky);
    for i in 0..pts.len() {
        draw_triangle(c, pts[i], pts[(i + 1) % pts.len()], color);
    }
}

pub(crate) fn ui_round_frame(x: f32, y: f32, w: f32, h: f32, r: f32, fill: Color, border: Color) {
    ui_round(x, y, w, h, r, fill);
    let pts = round_points(x, y, w, h, r);
    let thick = (ui_scale().1 * 0.5).max(1.0);
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        draw_line(a.x, a.y, b.x, b.y, thick, border);
    }
}

pub(crate) fn ui_circle(x: f32, y: f32, r: f32, color: Color) {
    let (kx, ky) = ui_scale();
    draw_circle(x * kx, y * ky, r * kx.min(ky), color);
}

pub(crate) fn ui_tri(a: Vec2, b: Vec2, c: Vec2, color: Color) {
    let (kx, ky) = ui_scale();
    let k = vec2(kx, ky);
    draw_triangle(a * k, b * k, c * k, color);
}

pub(crate) fn ui_line(a: Vec2, b: Vec2, thick: f32, color: Color) {
    let (kx, ky) = ui_scale();
    draw_line(a.x * kx, a.y * ky, b.x * kx, b.y * ky, (thick * ky).max(1.0), color);
}

pub(crate) mod skin {
    use macroquad::prelude::Color;
    pub(crate) const PANEL: Color = Color::new(0.161, 0.114, 0.251, 0.94);
    pub(crate) const PANEL_SOLID: Color = Color::new(0.161, 0.114, 0.251, 1.0);
    pub(crate) const EDGE: Color = Color::new(0.404, 0.259, 0.498, 1.0);
    pub(crate) const EDGE_STRONG: Color = Color::new(0.541, 0.353, 0.643, 1.0);
    pub(crate) const TAB: Color = Color::new(0.2, 0.137, 0.302, 1.0);
    pub(crate) const PINK: Color = Color::new(0.957, 0.471, 0.718, 1.0);
    pub(crate) const PINK_EDGE: Color = Color::new(0.976, 0.682, 0.827, 1.0);
    pub(crate) const CREAM: Color = Color::new(1.0, 0.941, 0.78, 1.0);
    pub(crate) const MUTED: Color = Color::new(0.733, 0.667, 0.804, 1.0);
    pub(crate) const GOLD: Color = Color::new(1.0, 0.827, 0.365, 1.0);
    pub(crate) const MINT: Color = Color::new(0.502, 0.878, 0.698, 1.0);
    pub(crate) const BLUE: Color = Color::new(0.478, 0.722, 1.0, 1.0);
    pub(crate) const CYAN: Color = Color::new(0.455, 0.847, 0.863, 1.0);
    pub(crate) const INK: Color = Color::new(0.0902, 0.0706, 0.149, 1.0);
}
