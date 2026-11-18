#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use std::cell::RefCell;
use steps::venues::{button, held_repeat, hit, paste_into, shortcut_held};

#[derive(Clone)]
pub(crate) struct Msg {
    pub(crate) who: String,
    pub(crate) ch: usize,
    pub(crate) text: String,
    pub(crate) at: String,
    pub(crate) mine: bool,
}

#[derive(Clone)]
pub(crate) struct Friend {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) ch: usize,
    pub(crate) st: u8,
}

#[derive(Default)]
pub(crate) struct Chat {
    pub(crate) room: Vec<Msg>,
    pub(crate) dms: Vec<(String, Vec<Msg>)>,
    pub(crate) friends: Vec<Friend>,
    pub(crate) tab: usize,
    pub(crate) peer: Option<String>,
    pub(crate) draft: String,
    pub(crate) adding: bool,
    pub(crate) add_draft: String,
    pub(crate) note: String,
    pub(crate) scroll: usize,
    pub(crate) sent_watch: bool,
    pub(crate) sent_me: bool,
    pub(crate) unread: Vec<String>,
}

const KEEP: usize = 200;
const ROW: f32 = 11.0;

pub(crate) fn now_label() -> String {
    let sk = sky();
    format!("{:02}:{:02}", sk.hour as u32, (sk.hour.fract() * 60.0) as u32)
}

fn trim(list: &mut Vec<Msg>) {
    if list.len() > KEEP {
        let extra = list.len() - KEEP;
        list.drain(..extra);
    }
}

pub(crate) fn room_line(g: &mut Game, who: String, ch: usize, text: String, mine: bool) {
    let c = &mut g.venue.chat;
    c.room.push(Msg { who, ch, text, at: now_label(), mine });
    trim(&mut c.room);
    c.scroll = 0;
}

fn dm_line(g: &mut Game, code: &str, msg: Msg) {
    let c = &mut g.venue.chat;
    match c.dms.iter_mut().find(|(k, _)| k == code) {
        Some((_, list)) => {
            list.push(msg);
            trim(list);
        }
        None => c.dms.push((code.to_string(), vec![msg])),
    }
    c.scroll = 0;
}

pub(crate) fn friends_text(g: &Game) -> String {
    g.venue.chat.friends.iter().map(|f| format!("{}={}", f.code, f.name.replace([',', '=', ' '], "_"))).collect::<Vec<_>>().join(",")
}

pub(crate) fn load_friends(g: &mut Game, text: &str) {
    g.venue.chat.friends = text
        .split(',')
        .filter_map(|p| {
            let (code, name) = p.split_once('=')?;
            let code = venues::tidy_code(code)?;
            Some(Friend { code, name: name.replace('_', " ").chars().take(16).collect(), ch: 0, st: 0 })
        })
        .take(50)
        .collect();
}

fn wire(name: &str) -> String {
    let n: String = name.trim().replace(' ', "_").chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').take(16).collect();
    if n.is_empty() { "friend".to_string() } else { n }
}

pub(crate) fn tick(g: &mut Game) {
    if net::state() != 2 {
        g.venue.chat.sent_watch = false;
        g.venue.chat.sent_me = false;
        for f in g.venue.chat.friends.iter_mut() {
            f.st = 0;
        }
        return;
    }
    if !g.venue.visits.sent_code {
        return;
    }
    if !g.venue.chat.sent_me {
        net::send(&format!("fme {} {}", g.players[0].character, wire(&g.venue.name)));
        g.venue.chat.sent_me = true;
    }
    if !g.venue.chat.sent_watch {
        let list: Vec<String> = g.venue.chat.friends.iter().map(|f| f.code.clone()).collect();
        net::send(&format!("fwatch {}", list.join(",")));
        g.venue.chat.sent_watch = true;
    }
}

pub(crate) fn on_message(g: &mut Game, parts: &[&str]) -> bool {
    match parts.first().copied().unwrap_or("") {
        "fstat" if parts.len() >= 5 => {
            let (code, st, ch) = (parts[1], parts[2].parse::<u8>().unwrap_or(0), parts[3].parse::<usize>().unwrap_or(0) % g.cast_list.len());
            let name = parts[4].replace('_', " ");
            if let Some(f) = g.venue.chat.friends.iter_mut().find(|f| f.code == code) {
                let came = f.st == 0 && st > 0;
                f.st = st;
                if st > 0 {
                    f.ch = ch;
                    if name != "-" {
                        f.name = name;
                    }
                }
                if came {
                    let n = f.name.clone();
                    g.toasts.push((format!("{n} is online"), 2.5));
                    steps::notify::push(steps::notify::Kind::Friend, Some(ch), format!("{n} is now online"));
                }
            }
        }
        "dmfrom" if parts.len() >= 5 => {
            let code = parts[1].to_string();
            let ch = parts[2].parse::<usize>().unwrap_or(0) % g.cast_list.len();
            let name = parts[3].replace('_', " ");
            let text = parts[4..].join(" ");
            dm_line(g, &code, Msg { who: name.clone(), ch, text, at: now_label(), mine: false });
            let reading = g.venue.overlay == venues::Overlay::Talk && g.venue.chat.tab == 2 && g.venue.chat.peer.as_deref() == Some(code.as_str());
            if !reading && !g.venue.chat.unread.contains(&code) {
                g.venue.chat.unread.push(code);
            }
            if !reading {
                steps::notify::push(steps::notify::Kind::Message, Some(ch), format!("{name} sent you a message"));
                g.toasts.push((format!("{name} sent you a message"), 3.0));
                g.audio.play_ch(3, &g.bells[3], 0.35);
            }
        }
        "dmno" if parts.len() >= 3 => {
            let name = g.venue.chat.friends.iter().find(|f| f.code == parts[1]).map(|f| f.name.clone()).unwrap_or_else(|| parts[1].to_string());
            g.venue.chat.note = match parts[2] {
                "offline" => format!("{name} is offline right now"),
                "notfriend" => format!("{name} needs to add you back first"),
                _ => "a little slower, please".to_string(),
            };
        }
        "hsaid" if parts.len() >= 5 => {
            let ch = parts[2].parse::<usize>().unwrap_or(0) % g.cast_list.len();
            let name = parts[3].replace('_', " ");
            let text = parts[4..].join(" ");
            room_line(g, name.clone(), ch, text.clone(), false);
            if g.venue.overlay != venues::Overlay::Talk {
                g.toasts.push((format!("{name}: {text}"), 4.0));
            }
        }
        "news" if parts.len() >= 3 => {
            if let Ok(id) = parts[1].parse::<i64>() {
                steps::notify::news(id, parts[2..].join(" "));
            }
        }
        "newsgone" if parts.len() >= 2 => {
            if let Ok(id) = parts[1].parse::<i64>() {
                steps::notify::news_gone(g, id);
            }
        }
        _ => return false,
    }
    true
}

thread_local! {
    static FACES: RefCell<Vec<RenderTarget>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn make_portraits(g: &Game) {
    if FACES.with(|f| !f.borrow().is_empty()) {
        return;
    }
    let mut faces = Vec::new();
    for (k, c) in g.cast_list.iter().enumerate() {
        let target = render_target_ex(48, 48, RenderTargetParams { sample_count: 1, depth: true });
        target.texture.set_filter(FilterMode::Nearest);
        set_camera(&Camera2D { render_target: Some(target.clone()), zoom: vec2(2.0 / 48.0, -2.0 / 48.0), target: vec2(24.0, 24.0), ..Default::default() });
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        let spot = vec2(6.5, 16.5);
        let p = Player::new(k, c.body, spot);
        let ground = p.ground(&g.world);
        let head = vec3(spot.x, ground + 0.62, spot.y);
        let eye = head + vec3(0.0, 0.12, 1.3);
        set_camera(&Camera3D { position: eye, target: head, up: Vec3::Y, fovy: 0.62, aspect: Some(1.0), render_target: Some(target.clone()), ..Default::default() });
        let light = Light { dusk: 0.1, fire: head + vec3(0.8, 1.2, 1.0), fire_strength: 0.35, tint: vec3(1.0, 0.85, 0.7), lamps: Vec::new() };
        let mut b = Batch::new(eye, light);
        p.draw(&mut b, &g.world, 0.0, 0.0);
        b.flush();
        faces.push(target);
    }
    set_default_camera();
    FACES.with(|f| *f.borrow_mut() = faces);
}

pub(crate) fn avatar(ch: usize, x: f32, y: f32, s: f32, tint: Color) {
    ui_round(x, y, s, s, 4.0, tint);
    let (kx, ky) = ui_scale();
    FACES.with(|f| {
        if let Some(t) = f.borrow().get(ch) {
            draw_texture_ex(&t.texture, (x * kx).round(), (y * ky).round(), WHITE, DrawTextureParams { dest_size: Some(vec2((s * kx).round(), (s * ky).round())), flip_y: true, ..Default::default() });
        }
    });
}

fn in_group(g: &Game) -> bool {
    steps::visits::visiting(g) || !g.venue.visits.peers.is_empty()
}

fn send_room(g: &mut Game, text: &str) {
    let text: String = text.chars().filter(|c| !c.is_control()).take(160).collect::<String>().trim().to_string();
    if text.is_empty() {
        return;
    }
    if in_group(g) && net::state() == 2 {
        net::send(&format!("hsay {text}"));
    }
    steps::partner::send(g, &text);
}

fn send_dm(g: &mut Game, code: &str, text: &str) {
    let text: String = text.chars().filter(|c| !c.is_control()).take(160).collect::<String>().trim().to_string();
    if text.is_empty() {
        return;
    }
    if net::state() != 2 {
        g.venue.chat.note = "you need to be online to message friends".to_string();
        return;
    }
    net::send(&format!("dm {code} {text}"));
    let (who, ch) = (g.venue.name.clone(), g.players[0].character);
    dm_line(g, code, Msg { who: if who.is_empty() { "you".to_string() } else { who }, ch, text, at: now_label(), mine: true });
    g.venue.chat.note.clear();
}

fn add_friend(g: &mut Game) {
    let Some(code) = venues::tidy_code(&g.venue.chat.add_draft) else {
        g.venue.chat.note = "a home code looks like ACD-EFG-HJK".to_string();
        return;
    };
    if code == steps::visits::my_code(g) {
        g.venue.chat.note = "that's your own code!".to_string();
        return;
    }
    if g.venue.chat.friends.iter().any(|f| f.code == code) {
        g.venue.chat.note = "already a friend".to_string();
        return;
    }
    if g.venue.chat.friends.len() >= 50 {
        g.venue.chat.note = "your friends list is full".to_string();
        return;
    }
    g.venue.chat.friends.push(Friend { code: code.clone(), name: code.clone(), ch: 0, st: 0 });
    g.venue.chat.sent_watch = false;
    g.venue.chat.add_draft.clear();
    g.venue.chat.adding = false;
    g.venue.chat.note = "added! once they add your code too, you can message each other".to_string();
    g.save_clock = 99.0;
}

struct Item {
    msg: Msg,
    lines: Vec<String>,
    h: f32,
}

fn layout(list: &[Msg], text_w: f32) -> Vec<Item> {
    list.iter()
        .map(|m| {
            let lines = ui::with_face(ui::Face::Notice, || ui_wrap(&m.text, 8.0, text_w));
            let h = 10.0 + lines.len() as f32 * ROW + 5.0;
            Item { msg: m.clone(), lines, h: h.max(22.0) }
        })
        .collect()
}

fn messages(g: &mut Game, fr: &mut Frame, list: &[Msg], x: f32, y: f32, w: f32, h: f32, fresh: bool, empty: &str) {
    if list.is_empty() {
        for (i, l) in ui_wrap(empty, 8.0, w - 16.0).iter().enumerate() {
            ui_text(l, x + 8.0, y + 14.0 + i as f32 * 10.0, 8.0, ui::skin::MUTED);
        }
        return;
    }
    let items = layout(list, w - 34.0);
    let total: f32 = items.iter().map(|i| i.h).sum();
    let max_scroll = if total <= h {
        0
    } else {
        let mut used = 0.0;
        let mut fit = 0;
        for it in items.iter() {
            if used + it.h > h {
                break;
            }
            used += it.h;
            fit += 1;
        }
        items.len() - fit.max(1)
    };
    if !fresh && hit(fr, x, y, w, h) {
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            g.venue.chat.scroll += 1;
        }
        if wheel < 0.0 {
            g.venue.chat.scroll = g.venue.chat.scroll.saturating_sub(1);
        }
    }
    g.venue.chat.scroll = g.venue.chat.scroll.min(max_scroll);
    let scroll = g.venue.chat.scroll;
    let mut bottom = y + h;
    let shown_w = if max_scroll > 0 { w - 8.0 } else { w };
    for it in items.iter().rev().skip(scroll) {
        let top = bottom - it.h;
        if top < y - 0.5 {
            break;
        }
        let m = &it.msg;
        avatar(m.ch, x + 4.0, top + 3.0, 18.0, if m.mine { Color::from_rgba(120, 90, 170, 255) } else { Color::from_rgba(250, 196, 120, 255) });
        let name_col = if m.mine { ui::skin::BLUE } else { ui::skin::PINK_EDGE };
        ui_text_fit(&m.who, x + 27.0, top + 10.0, 8.0, shown_w - 70.0, name_col);
        let nw = ui_measure(&m.who, 8.0).min(shown_w - 70.0);
        ui_text(&m.at, x + 31.0 + nw, top + 10.0, 7.0, ui::skin::MUTED);
        let mut ly = top + 21.0;
        for l in &it.lines {
            ui_text_as(ui::Face::Notice, l, x + 27.0, ly, 8.0, ui::skin::CREAM);
            ly += ROW;
        }
        bottom = top;
    }
    if max_scroll > 0 {
        let (bx, bh) = (x + w - 5.0, h);
        ui_round(bx, y, 3.0, bh, 1.5, Color::new(1.0, 1.0, 1.0, 0.1));
        let thumb = (bh * h / total).max(8.0);
        let top = y + (bh - thumb) * (1.0 - scroll as f32 / max_scroll as f32);
        ui_round(bx, top, 3.0, thumb, 1.5, ui::skin::PINK);
    }
}

fn field(fr: &Frame, text: &str, hint: &str, x: f32, y: f32, w: f32, h: f32) {
    ui_round_frame(x, y, w, h, 6.0, Color::new(0.1, 0.07, 0.17, 1.0), ui::skin::EDGE_STRONG);
    let caret = if (fr.time * 2.0).fract() < 0.6 { "|" } else { "" };
    if text.is_empty() {
        ui_text(&format!("{caret}{hint}"), x + 6.0, y + h / 2.0 + 3.0, 8.0, ui::skin::MUTED);
    } else {
        let mut view = text.to_string();
        while view.len() > 1 && ui_measure_as(ui::Face::Notice, &view, 8.0) > w - 14.0 {
            view.remove(0);
        }
        ui_text_as(ui::Face::Notice, &format!("{view}{caret}"), x + 6.0, y + h / 2.0 + 3.5, 8.0, ui::skin::CREAM);
    }
}

fn typing(g: &mut Game, adding: bool) {
    let settled = get_time() - g.venue.opened_at > 0.15;
    let target = if adding { &mut g.venue.chat.add_draft } else { &mut g.venue.chat.draft };
    let max = if adding { 12 } else { 160 };
    while let Some(ch) = get_char_pressed() {
        if settled && !ch.is_control() && target.chars().count() < max && !shortcut_held() {
            target.push(ch);
        }
    }
    paste_into(target, max, |c| !c.is_control());
    if held_repeat(KeyCode::Backspace) || is_key_pressed(KeyCode::Backspace) {
        target.pop();
    }
}

pub(crate) fn panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let w = 204f32.min(low_w() - 12.0);
    let top = 40.0f32.min(low_h() * 0.15);
    let h = (low_h() - top - 6.0).min(250.0);
    let x = (low_w() - w - 6.0).round();
    let y = top.round();
    ui_round(x + 2.0, y + 3.0, w, h, 9.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 9.0, ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG);
    ui_round(x + 8.0, y + 7.0, 13.0, 10.0, 3.0, ui::skin::PINK);
    ui_rect(x + 11.0, y + 16.0, 3.0, 3.0, ui::skin::PINK);
    for k in 0..3 {
        ui_rect(x + 11.0 + k as f32 * 3.0, y + 11.5, 1.6, 1.6, ui::skin::INK);
    }
    ui_text_as(ui::Face::Display, "chat", x + 26.0, y + 17.0, 11.0, ui::skin::GOLD);
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && !fresh;
    ui_rect(x + 6.0, y + 24.0, w - 12.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
    let tabs = ["room", "friends", "dm"];
    let tw = (w - 16.0 - 8.0) / 3.0;
    let unread = !g.venue.chat.unread.is_empty();
    for (i, t) in tabs.iter().enumerate() {
        let bx = x + 8.0 + i as f32 * (tw + 4.0);
        let clicked = button(fr, t, bx, y + 29.0, tw, 15.0, g.venue.chat.tab == i) && !fresh;
        if i == 2 && unread {
            ui_circle(bx + tw - 6.0, y + 33.0, 2.4, ui::skin::GOLD);
        }
        if clicked {
            g.venue.chat.tab = i;
            g.venue.chat.scroll = 0;
            g.venue.chat.note.clear();
            fr.click = false;
        }
    }
    if !fresh && is_key_pressed(KeyCode::Tab) {
        g.venue.chat.tab = (g.venue.chat.tab + 1) % 3;
        g.venue.chat.scroll = 0;
        g.venue.chat.note.clear();
    }
    let list_y = y + 50.0;
    let input_h = 18.0;
    let note_h = if g.venue.chat.note.is_empty() { 0.0 } else { 11.0 };
    let list_h = h - 50.0 - input_h - 12.0 - note_h;
    let tab = g.venue.chat.tab;
    let mut send = false;
    if tab == 0 {
        let list = g.venue.chat.room.clone();
        let noun = g.cast_list[g.players[1].character].noun;
        let empty = if in_group(g) { "say hi to everyone here!".to_string() } else { format!("call your {noun} (or \"buddy\") and it answers. friends visiting your home see this too.") };
        messages(g, fr, &list, x + 6.0, list_y, w - 12.0, list_h, fresh, &empty);
    } else if tab == 1 {
        friends_tab(g, fr, x, list_y, w, list_h + input_h + 6.0, fresh);
    } else {
        dm_tab(g, fr, x, list_y, w, list_h, fresh);
    }
    if !g.venue.chat.note.is_empty() {
        ui_text_fit(&g.venue.chat.note.clone(), x + 8.0, y + h - input_h - 12.0, 7.0, w - 16.0, ui::skin::GOLD);
    }
    let can_type = tab == 0 || (tab == 2 && g.venue.chat.peer.is_some());
    if can_type {
        let (fx, fy, fw) = (x + 6.0, y + h - input_h - 6.0, w - 12.0 - input_h - 4.0);
        field(fr, &g.venue.chat.draft, "type a message...", fx, fy, fw, input_h);
        let (sx, sy) = (fx + fw + 4.0, fy);
        let over = hit(fr, sx, sy, input_h, input_h);
        ui_round_frame(sx, sy, input_h, input_h, 6.0, if over { ui::skin::PINK_EDGE } else { ui::skin::PINK }, ui::skin::PINK_EDGE);
        for k in 0..5 {
            let half = 4.0 - k as f32 * 0.8;
            ui_rect(sx + 6.5 + k as f32 * 1.2, sy + input_h / 2.0 - half, 1.2, half * 2.0, ui::skin::INK);
        }
        if !fresh && over && fr.click {
            send = true;
            fr.click = false;
        }
        if !fresh && fr.click && hit(fr, fx, fy, fw, input_h) && radio::touch_ui() {
            if let Some(text) = radio::ask_text("say something", &g.venue.chat.draft) {
                g.venue.chat.draft = text.chars().filter(|c| !c.is_control()).take(160).collect();
            }
            fr.click = false;
        }
    }
    if fresh {
        return;
    }
    let adding = tab == 1 && g.venue.chat.adding;
    if can_type || adding {
        typing(g, adding);
    } else {
        while get_char_pressed().is_some() {}
    }
    let enter = get_time() - g.venue.opened_at > 0.15 && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter));
    if adding && enter {
        add_friend(g);
    } else if can_type && (enter || send) {
        let text = std::mem::take(&mut g.venue.chat.draft);
        if tab == 0 {
            send_room(g, &text);
        } else if let Some(code) = g.venue.chat.peer.clone() {
            send_dm(g, &code, &text);
        }
    }
    if close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h)) {
        if adding && is_key_pressed(KeyCode::Escape) {
            g.venue.chat.adding = false;
        } else {
            g.venue.overlay = venues::Overlay::None;
            fr.click = false;
        }
    }
}

fn status_label(st: u8) -> (&'static str, Color) {
    match st {
        1 => ("online", ui::skin::MINT),
        2 => ("visiting a home", ui::skin::CYAN),
        _ => ("offline", ui::skin::MUTED),
    }
}

fn friends_tab(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32, h: f32, fresh: bool) {
    let on = g.venue.chat.friends.iter().filter(|f| f.st > 0).count();
    let head = format!("{}/{} online", on, g.venue.chat.friends.len());
    ui_text_as(ui::Face::Label, &head, x + 10.0, y + 7.0, 8.0, ui::skin::CREAM);
    let mine = steps::visits::my_code(g);
    let row_h = 26.0;
    let list_y = y + 14.0;
    let bottom_h = if g.venue.chat.adding { 54.0 } else { 36.0 };
    let rows = (((h - 14.0 - bottom_h) / row_h).floor() as usize).max(1);
    let mut friends = g.venue.chat.friends.clone();
    friends.sort_by_key(|f| (f.st == 0, f.name.to_lowercase()));
    if friends.is_empty() {
        for (i, l) in ui_wrap("no friends yet. add a friend with their home code, and give them yours.", 8.0, w - 20.0).iter().enumerate() {
            ui_text(l, x + 10.0, list_y + 14.0 + i as f32 * 10.0, 8.0, ui::skin::MUTED);
        }
    }
    let mut remove: Option<String> = None;
    for (i, f) in friends.iter().take(rows).enumerate() {
        let ry = list_y + i as f32 * row_h;
        ui_round(x + 6.0, ry + 1.0, w - 12.0, row_h - 3.0, 5.0, Color::new(1.0, 1.0, 1.0, 0.04));
        avatar(f.ch, x + 10.0, ry + 3.0, 18.0, if f.st > 0 { Color::from_rgba(250, 196, 120, 255) } else { Color::from_rgba(90, 80, 110, 255) });
        ui_text_fit(&f.name, x + 33.0, ry + 11.0, 8.0, w - 100.0, if f.st > 0 { ui::skin::CREAM } else { ui::skin::MUTED });
        let (label, col) = status_label(f.st);
        ui_text(label, x + 33.0, ry + 20.0, 7.0, col);
        let bx = x + w - 54.0;
        if button(fr, "msg", bx, ry + 5.0, 20.0, 13.0, false) && !fresh {
            g.venue.chat.tab = 2;
            g.venue.chat.peer = Some(f.code.clone());
            g.venue.chat.unread.retain(|c| *c != f.code);
            g.venue.chat.scroll = 0;
            fr.click = false;
        }
        if f.st == 1 && button(fr, ">", bx + 22.0, ry + 5.0, 14.0, 13.0, false) && !fresh {
            g.venue.visits.draft = f.code.clone();
            steps::visits::knock(g);
            g.venue.chat.note = g.venue.visits.status.clone();
            fr.click = false;
        }
        if button(fr, "-", bx + 38.0, ry + 5.0, 10.0, 13.0, false) && !fresh {
            remove = Some(f.code.clone());
            fr.click = false;
        }
    }
    if friends.len() > rows {
        ui_text(&format!("+{} more", friends.len() - rows), x + 10.0, list_y + rows as f32 * row_h + 6.0, 7.0, ui::skin::MUTED);
    }
    if let Some(code) = remove {
        g.venue.chat.friends.retain(|f| f.code != code);
        g.venue.chat.sent_watch = false;
        g.save_clock = 99.0;
    }
    let code_y = y + h - bottom_h;
    let code_line = format!("your code {mine}");
    ui_text_fit(&code_line, x + 10.0, code_y + 9.0, 8.0, w - 56.0, ui::skin::MUTED);
    if button(fr, "copy", x + w - 40.0, code_y + 1.0, 32.0, 12.0, false) && !fresh {
        net::copy(&mine);
        g.venue.chat.note = "your code is copied - send it to a friend".to_string();
        fr.click = false;
    }
    let by = code_y + 15.0;
    if g.venue.chat.adding {
        field(fr, &g.venue.chat.add_draft, "their home code", x + 6.0, by, w - 60.0, 16.0);
        if button(fr, "add", x + w - 50.0, by, 44.0, 16.0, true) && !fresh {
            add_friend(g);
            fr.click = false;
        }
        ui_text("enter: add   esc: cancel", x + 8.0, by + 28.0, 7.0, ui::skin::MUTED);
    } else if button(fr, "+ add friend", x + 6.0, by + 2.0, w - 12.0, 16.0, false) && !fresh {
        g.venue.chat.adding = true;
        g.venue.chat.add_draft.clear();
        while get_char_pressed().is_some() {}
        fr.click = false;
    }
}

fn dm_tab(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32, h: f32, fresh: bool) {
    let friends = g.venue.chat.friends.clone();
    if friends.is_empty() {
        for (i, l) in ui_wrap("add a friend in the friends tab, and once you've both added each other you can message here.", 8.0, w - 20.0).iter().enumerate() {
            ui_text(l, x + 10.0, y + 14.0 + i as f32 * 10.0, 8.0, ui::skin::MUTED);
        }
        return;
    }
    if g.venue.chat.peer.as_ref().map(|p| !friends.iter().any(|f| &f.code == p)).unwrap_or(true) {
        g.venue.chat.peer = g.venue.chat.unread.first().cloned().filter(|c| friends.iter().any(|f| &f.code == c)).or_else(|| friends.iter().find(|f| f.st > 0).or(friends.first()).map(|f| f.code.clone()));
    }
    let peer = g.venue.chat.peer.clone().unwrap_or_default();
    let mut cx = x + 6.0;
    for f in friends.iter() {
        let label = f.name.split_whitespace().next().unwrap_or("friend").chars().take(8).collect::<String>();
        let bw = ui_measure_as(ui::Face::Label, &label, 8.0) + 12.0;
        if cx + bw > x + w - 6.0 {
            break;
        }
        let clicked = button(fr, &label, cx, y, bw, 13.0, f.code == peer) && !fresh;
        if g.venue.chat.unread.contains(&f.code) {
            ui_circle(cx + bw - 3.0, y + 2.0, 2.2, ui::skin::GOLD);
        }
        if clicked {
            g.venue.chat.peer = Some(f.code.clone());
            g.venue.chat.scroll = 0;
            fr.click = false;
        }
        cx += bw + 3.0;
    }
    g.venue.chat.unread.retain(|c| *c != peer);
    let list: Vec<Msg> = g.venue.chat.dms.iter().find(|(k, _)| *k == peer).map(|(_, l)| l.clone()).unwrap_or_default();
    let who = friends.iter().find(|f| f.code == peer).map(|f| f.name.clone()).unwrap_or_default();
    let empty = format!("say hello to {who}. messages only reach friends who are online and have added you too.");
    messages(g, fr, &list, x + 6.0, y + 17.0, w - 12.0, h - 17.0, fresh, &empty);
}
