#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use std::cell::RefCell;
use steps::venues::{button, hit};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Visit,
    Friend,
    Message,
    Quest,
    Update,
    Rave,
    News,
}

#[derive(Clone)]
pub(crate) struct Note {
    pub(crate) kind: Kind,
    pub(crate) ch: Option<usize>,
    pub(crate) text: String,
    pub(crate) when: f64,
    pub(crate) id: i64,
}

#[derive(Default)]
pub(crate) struct Notes {
    pub(crate) list: Vec<Note>,
    pub(crate) unread: usize,
    pub(crate) scroll: usize,
    pub(crate) rave_told: f64,
}

const KEEP: usize = 40;

thread_local! {
    static QUEUE: RefCell<Vec<Note>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn push(kind: Kind, ch: Option<usize>, text: String) {
    QUEUE.with(|q| q.borrow_mut().push(Note { kind, ch, text, when: miniquad::date::now(), id: 0 }));
}

pub(crate) fn news(id: i64, text: String) {
    QUEUE.with(|q| q.borrow_mut().push(Note { kind: Kind::News, ch: None, text, when: id as f64 / 1000.0, id }));
}

pub(crate) fn news_gone(g: &mut Game, id: i64) {
    g.venue.notes.list.retain(|n| !(n.kind == Kind::News && n.id == id));
}

pub(crate) fn tick(g: &mut Game) {
    let fresh: Vec<Note> = QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()));
    for n in fresh {
        let notes = &mut g.venue.notes;
        if n.kind == Kind::News && notes.list.iter().any(|m| m.kind == Kind::News && m.id == n.id) {
            continue;
        }
        notes.list.push(n);
        notes.unread += 1;
        notes.list.sort_by(|a, b| a.when.partial_cmp(&b.when).unwrap_or(std::cmp::Ordering::Equal));
        if notes.list.len() > KEEP {
            notes.list.remove(0);
        }
    }
    if let Some(t) = &g.venue.tonight {
        let left = t.starts - get_time();
        if left > 0.0 && left <= 600.0 && (g.venue.notes.rave_told - t.starts).abs() > 30.0 {
            g.venue.notes.rave_told = t.starts;
            push(Kind::Rave, None, format!("the secret rave starts in {} minutes at st. bassilica", (left / 60.0).ceil() as i32));
        }
    }
}

fn ago(when: f64) -> String {
    let s = (miniquad::date::now() - when).max(0.0);
    if s < 60.0 {
        "now".to_string()
    } else if s < 3600.0 {
        format!("{}m", (s / 60.0) as i64)
    } else if s < 86400.0 {
        format!("{}h", (s / 3600.0) as i64)
    } else {
        format!("{}d", (s / 86400.0) as i64)
    }
}

pub(crate) fn bell(x: f32, y: f32, col: Color) {
    ui_round(x + 2.0, y + 1.5, 7.0, 7.0, 3.2, col);
    ui_rect(x + 1.0, y + 6.0, 9.0, 2.0, col);
    ui_rect(x + 4.5, y, 2.0, 1.6, col);
    ui_circle(x + 5.5, y + 9.4, 1.3, col);
}

fn icon(kind: Kind, x: f32, y: f32, s: f32) {
    let (bg, fg) = match kind {
        Kind::Quest => (Color::from_rgba(255, 214, 110, 255), ui::skin::INK),
        Kind::Update => (ui::skin::MINT, ui::skin::INK),
        Kind::Rave => (Color::from_rgba(196, 130, 255, 255), ui::skin::INK),
        Kind::News => (ui::skin::PINK, ui::skin::INK),
        _ => (ui::skin::BLUE, ui::skin::INK),
    };
    ui_round(x, y, s, s, 4.0, bg);
    let (cx, cy) = (x + s / 2.0, y + s / 2.0);
    match kind {
        Kind::Quest => {
            ui_rect(cx - 3.5, cy - 0.5, 2.0, 2.0, fg);
            ui_rect(cx - 1.5, cy + 1.5, 2.0, 2.0, fg);
            ui_rect(cx + 0.5, cy - 0.5, 2.0, 2.0, fg);
            ui_rect(cx + 2.5, cy - 2.5, 2.0, 2.0, fg);
        }
        Kind::Update => {
            ui_rect(cx - 1.0, cy - 4.0, 2.0, 6.0, fg);
            ui_rect(cx - 3.0, cy, 6.0, 2.0, fg);
            ui_rect(cx - 2.0, cy + 2.0, 4.0, 1.0, fg);
            ui_rect(cx - 4.0, cy + 4.0, 8.0, 1.2, fg);
        }
        Kind::Rave => {
            ui_rect(cx - 0.6, cy - 4.0, 1.2, 8.0, fg);
            ui_rect(cx - 3.0, cy - 2.0, 6.0, 1.2, fg);
            ui_rect(cx - 2.0, cy + 1.0, 4.0, 3.0, fg);
        }
        Kind::News => bell(cx - 5.5, cy - 5.0, fg),
        _ => {
            ui_round(cx - 4.0, cy - 3.0, 8.0, 6.0, 2.0, fg);
            ui_rect(cx - 2.5, cy + 3.0, 2.0, 2.0, fg);
        }
    }
}

pub(crate) fn open(g: &mut Game) {
    g.venue.overlay = venues::Overlay::Notes;
    g.venue.opened_at = get_time();
    g.venue.notes.unread = 0;
    g.venue.notes.scroll = 0;
}

pub(crate) fn panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let w = 230f32.min(low_w() - 12.0);
    let row_h = 22.0;
    let n = g.venue.notes.list.len();
    let rows = (((low_h() - 90.0) / row_h).floor() as usize).clamp(3, 8);
    let shown = n.clamp(1, rows);
    let h = 34.0 + shown as f32 * row_h + 8.0;
    let x = 6.0;
    let y = 46.0;
    ui_round(x + 2.0, y + 3.0, w, h, 9.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 9.0, ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG);
    bell(x + 9.0, y + 6.0, ui::skin::PINK_EDGE);
    ui_text_as(ui::Face::Display, "notifications", x + 25.0, y + 17.0, 11.0, ui::skin::GOLD);
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && !fresh;
    let clear = n > 0 && button(fr, "clear", x + w - 56.0, y + 6.0, 31.0, 12.0, false) && !fresh;
    ui_rect(x + 6.0, y + 24.0, w - 12.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
    if n == 0 {
        ui_text("nothing new. knocks, messages, quests and news show up here.", x + 10.0, y + 40.0, 7.0, ui::skin::MUTED);
    }
    let max_scroll = n.saturating_sub(rows);
    if !fresh && hit(fr, x, y, w, h) {
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            g.venue.notes.scroll = (g.venue.notes.scroll + 1).min(max_scroll);
        }
        if wheel < 0.0 {
            g.venue.notes.scroll = g.venue.notes.scroll.saturating_sub(1);
        }
    }
    let scroll = g.venue.notes.scroll.min(max_scroll);
    let list: Vec<Note> = g.venue.notes.list.iter().rev().skip(scroll).take(rows).cloned().collect();
    for (i, note) in list.iter().enumerate() {
        let ry = y + 30.0 + i as f32 * row_h;
        ui_round(x + 6.0, ry, w - 12.0, row_h - 3.0, 5.0, Color::new(1.0, 1.0, 1.0, 0.05));
        match note.ch {
            Some(ch) => steps::chat::avatar(ch, x + 9.0, ry + 1.5, 16.0, Color::from_rgba(250, 196, 120, 255)),
            None => icon(note.kind, x + 9.0, ry + 1.5, 16.0),
        }
        let when = ago(note.when);
        let tw = ui_measure(&when, 7.0);
        let lines = ui::with_face(ui::Face::Notice, || ui_wrap(&note.text, 7.0, w - 46.0 - tw));
        let two = lines.len() > 1;
        for (k, l) in lines.iter().take(2).enumerate() {
            let shown = if k == 1 && lines.len() > 2 { format!("{l}...") } else { l.clone() };
            ui_text_as(ui::Face::Notice, &shown, x + 30.0, ry + if two { 8.5 } else { 12.5 } + k as f32 * 8.0, 7.0, ui::skin::CREAM);
        }
        ui_text(&when, x + w - 10.0 - tw, ry + 12.0, 7.0, ui::skin::MUTED);
    }
    if max_scroll > 0 {
        ui_text(&format!("{} older - scroll", max_scroll - scroll), x + 10.0, y + h - 2.0, 6.0, ui::skin::MUTED);
    }
    if fresh {
        return;
    }
    if clear {
        g.venue.notes.list.clear();
        fr.click = false;
    }
    if close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h)) {
        g.venue.overlay = venues::Overlay::None;
        fr.click = false;
    }
}
