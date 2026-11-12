//! A small immediate-mode UI in the cover's style: chunky panels with hard ink
//! shadows, key-cap buttons with an inset bottom edge, pixel type.
//!
//! Layout is done in a virtual space 360 units tall (width follows the window)
//! and scaled to the screen, so everything stays crisp at any size.

use crate::gfx::{mix, pal, text_w, with_alpha, Fonts};
use macroquad::prelude::*;

pub const VH: f32 = 360.0;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Cream,
    Yellow,
    Coral,
    Pink,
    Blue,
    Green,
    Ink,
    Dj,
}

impl Tone {
    pub fn color(self) -> Color {
        match self {
            Tone::Cream => pal::CREAM,
            Tone::Yellow => pal::YELLOW,
            Tone::Coral => pal::CORAL,
            Tone::Pink => pal::PINK,
            Tone::Blue => pal::BLUE,
            Tone::Green => pal::GREEN,
            Tone::Ink => pal::INK2,
            Tone::Dj => pal::DJ,
        }
    }
    fn text(self) -> Color {
        match self {
            Tone::Ink | Tone::Dj => pal::CREAM,
            _ => pal::INK,
        }
    }
}

pub struct Ui {
    pub s: f32,
    pub w: f32,
    pub h: f32,
    pub mouse: Vec2,
    pub clicked: bool,
    pub down: bool,
    pub released: bool,
    pub right_clicked: bool,
    pub wheel: f32,
    pub focus: Option<String>,
    pub typed: Vec<char>,
    pub backspace: bool,
    pub enter: bool,
    pub escape: bool,
    pub tab: bool,
    pub paste: Option<String>,
    pub time: f32,
    hot: bool,
    bs_timer: f32,
    /// Set when a widget used the click this frame.
    pub click_used: bool,
    pub toast_y: f32,
    toasts: Vec<(String, f32, Tone)>,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            s: 2.0,
            w: 640.0,
            h: VH,
            mouse: Vec2::ZERO,
            clicked: false,
            down: false,
            released: false,
            right_clicked: false,
            wheel: 0.0,
            focus: None,
            typed: Vec::new(),
            backspace: false,
            enter: false,
            escape: false,
            tab: false,
            paste: None,
            time: 0.0,
            hot: false,
            bs_timer: 0.0,
            click_used: false,
            toast_y: 12.0,
            toasts: Vec::new(),
        }
    }

    pub fn begin(&mut self) {
        self.s = screen_height() / VH;
        self.w = screen_width() / self.s;
        self.h = VH;
        let (mx, my) = mouse_position();
        self.mouse = vec2(mx / self.s, my / self.s);
        self.clicked = is_mouse_button_pressed(MouseButton::Left);
        self.down = is_mouse_button_down(MouseButton::Left);
        self.released = is_mouse_button_released(MouseButton::Left);
        self.right_clicked = is_mouse_button_pressed(MouseButton::Right);
        self.wheel = mouse_wheel().1;
        self.typed.clear();
        while let Some(c) = get_char_pressed() {
            if !c.is_control() {
                self.typed.push(c);
            }
        }
        let dt = get_frame_time();
        self.time += dt;
        self.backspace = false;
        if is_key_pressed(KeyCode::Backspace) {
            self.backspace = true;
            self.bs_timer = 0.4;
        } else if is_key_down(KeyCode::Backspace) {
            self.bs_timer -= dt;
            if self.bs_timer <= 0.0 {
                self.backspace = true;
                self.bs_timer = 0.035;
            }
        }
        self.enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        self.escape = is_key_pressed(KeyCode::Escape);
        self.tab = is_key_pressed(KeyCode::Tab);
        let cmd = is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper)
            || is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl);
        self.paste = if cmd && is_key_pressed(KeyCode::V) {
            miniquad::window::clipboard_get()
        } else {
            None
        };
        if cmd {
            // Don't type the "v" of cmd+v.
            self.typed.retain(|c| !matches!(c, 'v' | 'V' | 'c' | 'C'));
        }
        self.hot = false;
        self.click_used = false;
    }

    /// True if text input is going to a field (so game keys should be ignored).
    pub fn typing(&self) -> bool {
        self.focus.is_some()
    }

    pub fn end(&mut self, fonts: &Fonts) {
        if self.clicked && !self.click_used && self.focus.is_some() {
            self.focus = None;
        }
        // Toasts, top centre.
        let dt = get_frame_time();
        let mut y = self.toast_y;
        let mut toasts = std::mem::take(&mut self.toasts);
        for (msg, t, tone) in toasts.iter_mut() {
            *t -= dt;
            let a = (*t).clamp(0.0, 0.4) / 0.4;
            let tw = text_w(&fonts.body, msg, 11.0 * self.s) / self.s + 20.0;
            let r = Rect::new((self.w - tw) / 2.0, y - (1.0 - a) * 8.0, tw, 22.0);
            let col = tone.color();
            self.panel_a(r, col, a);
            self.text_a(&fonts.body, msg, r.x + 10.0, r.y + 15.5, 11.0, with_alpha(tone.text(), a));
            y += 28.0;
        }
        toasts.retain(|(_, t, _)| *t > 0.0);
        toasts.append(&mut self.toasts);
        self.toasts = toasts;
        miniquad::window::set_mouse_cursor(if self.hot {
            miniquad::CursorIcon::Pointer
        } else {
            miniquad::CursorIcon::Default
        });
    }

    pub fn toast(&mut self, msg: impl Into<String>, tone: Tone) {
        let msg = msg.into();
        self.toasts.retain(|(m, _, _)| m != &msg);
        self.toasts.push((msg, 3.5, tone));
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub fn hover(&self, r: Rect) -> bool {
        r.contains(self.mouse)
    }

    pub fn set_hot(&mut self) {
        self.hot = true;
    }

    // ----- drawing in virtual units -----

    pub fn rect(&self, r: Rect, c: Color) {
        let s = self.s;
        draw_rectangle((r.x * s).round(), (r.y * s).round(), (r.w * s).round(), (r.h * s).round(), c);
    }

    pub fn rect_lines(&self, r: Rect, t: f32, c: Color) {
        let s = self.s;
        draw_rectangle_lines((r.x * s).round(), (r.y * s).round(), (r.w * s).round(), (r.h * s).round(), t * s, c);
    }

    pub fn text(&self, font: &Font, t: &str, x: f32, y: f32, size: f32, c: Color) {
        crate::gfx::text(font, t, x * self.s, y * self.s, size * self.s, c);
    }

    fn text_a(&self, font: &Font, t: &str, x: f32, y: f32, size: f32, c: Color) {
        self.text(font, t, x, y, size, c);
    }

    pub fn text_shadow(&self, font: &Font, t: &str, x: f32, y: f32, size: f32, c: Color, sh: f32) {
        self.text(font, t, x + sh, y + sh, size, pal::INK);
        self.text(font, t, x, y, size, c);
    }

    pub fn text_center(&self, font: &Font, t: &str, cx: f32, y: f32, size: f32, c: Color) {
        let w = self.measure(font, t, size);
        self.text(font, t, cx - w / 2.0, y, size, c);
    }

    pub fn measure(&self, font: &Font, t: &str, size: f32) -> f32 {
        text_w(font, t, size * self.s) / self.s
    }

    /// Chunky panel: fill, 2-unit ink border, hard 4-unit shadow, inset base.
    pub fn panel(&self, r: Rect, fill: Color) {
        self.panel_a(r, fill, 1.0);
    }

    fn panel_a(&self, r: Rect, fill: Color, a: f32) {
        self.rect(Rect::new(r.x + 4.0, r.y + 4.0, r.w, r.h), with_alpha(pal::INK, a));
        self.rect(r, with_alpha(pal::INK, a));
        let inner = Rect::new(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0);
        self.rect(inner, with_alpha(fill, a));
        self.rect(
            Rect::new(inner.x, inner.y + inner.h - 3.0, inner.w, 3.0),
            with_alpha(mix(fill, pal::INK, 0.18), a),
        );
    }

    pub fn dim(&self, a: f32) {
        self.rect(Rect::new(0.0, 0.0, self.w, self.h), with_alpha(pal::INK, a));
    }

    // ----- widgets -----

    pub fn button(&mut self, fonts: &Fonts, r: Rect, label: &str, tone: Tone) -> bool {
        self.button_ex(fonts, r, label, tone, true)
    }

    pub fn button_ex(&mut self, fonts: &Fonts, r: Rect, label: &str, tone: Tone, enabled: bool) -> bool {
        let hover = enabled && self.hover(r);
        if hover {
            self.set_hot();
        }
        let pressed = hover && self.down;
        let lift: f32 = if pressed { 2.0 } else if hover { -1.0 } else { 0.0 };
        let fill = if !enabled {
            mix(tone.color(), pal::CONCRETE_D, 0.6)
        } else if hover {
            mix(tone.color(), pal::WHITE, 0.15)
        } else {
            tone.color()
        };
        // Shadow stays put; the cap moves.
        self.rect(Rect::new(r.x + 3.0, r.y + 3.0, r.w, r.h), pal::INK);
        let cap = Rect::new(r.x + lift.max(0.0), r.y + lift, r.w, r.h);
        self.rect(cap, pal::INK);
        let inner = Rect::new(cap.x + 2.0, cap.y + 2.0, cap.w - 4.0, cap.h - 4.0);
        self.rect(inner, fill);
        if !pressed {
            self.rect(Rect::new(inner.x, inner.y + inner.h - 3.0, inner.w, 3.0), mix(fill, pal::INK, 0.2));
        }
        let size = (r.h * 0.48).clamp(8.0, 16.0);
        let tw = self.measure(&fonts.body, label, size);
        let tc = if enabled { tone.text() } else { with_alpha(tone.text(), 0.5) };
        self.text(&fonts.body, label, cap.x + (cap.w - tw) / 2.0, cap.y + cap.h / 2.0 + size * 0.36, size, tc);
        let clicked = hover && self.clicked;
        if clicked {
            self.click_used = true;
        }
        clicked
    }

    /// Single-line text field. Returns true when Enter is pressed in it.
    pub fn field(
        &mut self,
        fonts: &Fonts,
        id: &str,
        r: Rect,
        value: &mut String,
        max: usize,
        placeholder: &str,
    ) -> bool {
        let focused = self.focus.as_deref() == Some(id);
        let hover = self.hover(r);
        if hover {
            self.set_hot();
        }
        if hover && self.clicked {
            self.focus = Some(id.to_string());
            self.click_used = true;
        }
        let focused = focused || self.focus.as_deref() == Some(id);
        self.rect(r, pal::INK);
        let inner = Rect::new(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0);
        self.rect(inner, if focused { pal::WHITE } else { pal::CREAM });
        self.rect(Rect::new(inner.x, inner.y, inner.w, 2.0), mix(pal::CREAM, pal::INK, 0.2));
        let mut submitted = false;
        if focused {
            for c in self.typed.drain(..) {
                if value.chars().count() < max {
                    value.push(c);
                }
            }
            if let Some(p) = self.paste.take() {
                for c in p.chars().filter(|c| !c.is_control()) {
                    if value.chars().count() < max {
                        value.push(c);
                    }
                }
            }
            if self.backspace {
                value.pop();
            }
            if self.enter {
                submitted = true;
            }
        }
        let size = (r.h * 0.5).clamp(8.0, 16.0);
        let ty = r.y + r.h / 2.0 + size * 0.36;
        if value.is_empty() && !focused {
            self.text(&fonts.body, placeholder, inner.x + 5.0, ty, size, with_alpha(pal::INK, 0.4));
        } else {
            // Keep the end of long text visible.
            let mut shown = value.clone();
            while self.measure(&fonts.body, &shown, size) > inner.w - 14.0 && !shown.is_empty() {
                shown.remove(0);
            }
            self.text(&fonts.body, &shown, inner.x + 5.0, ty, size, pal::INK);
            if focused && (self.time * 2.0).fract() < 0.6 {
                let cx = inner.x + 6.0 + self.measure(&fonts.body, &shown, size);
                self.rect(Rect::new(cx, inner.y + 4.0, 2.0, inner.h - 8.0), pal::INK);
            }
        }
        submitted
    }

    /// Prompt shown above something you can interact with.
    pub fn prompt(&self, fonts: &Fonts, x: f32, y: f32, key: &str, label: &str) {
        let size = 10.0;
        let kw = self.measure(&fonts.small, key, 8.0) + 10.0;
        let lw = self.measure(&fonts.body, label, size);
        let w = kw + lw + 16.0;
        let bob = (self.time * 4.0).sin() * 1.5;
        let r = Rect::new(x - w / 2.0, y - 20.0 + bob, w, 20.0);
        self.panel(r, pal::CREAM);
        let k = Rect::new(r.x + 5.0, r.y + 4.0, kw, 12.0);
        self.rect(k, pal::INK);
        self.text(&fonts.small, key, k.x + 5.0, k.y + 9.5, 8.0, pal::YELLOW);
        self.text(&fonts.body, label, k.x + kw + 5.0, r.y + 14.0, size, pal::INK);
    }
}
