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

pub(crate) fn ui_measure(text: &str, size: f32) -> f32 {
    let (kx, ky) = ui_scale();
    measure_text(text, None, (size * ky).round() as u16, 1.0).width / kx
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
    let (kx, ky) = ui_scale();
    let font_size = (size * ky).round() as u16;
    let shadow = (ky * 0.4).max(1.0).round();
    let (sx, sy) = ((x * kx).round(), (y * ky).round());
    draw_text_ex(
        text,
        sx + shadow,
        sy + shadow,
        TextParams {
            font_size,
            color: Color::new(0.05, 0.04, 0.1, color.a * 0.75),
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        sx,
        sy,
        TextParams {
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
