#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use ui::{skin, Face};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tab {
    Music,
    Chat,
    Items,
    Quests,
    Help,
    Settings,
}

pub(crate) struct Bar {
    logo: (f32, f32),
    logo_text: bool,
    tabs: Vec<(Tab, f32, f32, bool)>,
    group: (f32, f32),
    radio: (f32, f32),
    play: (f32, f32),
    label: (f32, f32),
    slider: Option<(f32, f32)>,
    speaker: Option<f32>,
}

const Y: f32 = 3.0;
const H: f32 = 18.0;
const TAB_SIZE: f32 = 8.5;
const LOGO: &str = "Melody Quack";

thread_local! {
    static DROP: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    static ICON: std::cell::RefCell<Option<Texture2D>> = const { std::cell::RefCell::new(None) };
}

fn tab_name(t: Tab) -> &'static str {
    match t {
        Tab::Music => "Music",
        Tab::Chat => "Chat",
        Tab::Items => "Items",
        Tab::Quests => "Quests",
        Tab::Help => "Help",
        Tab::Settings => "Settings",
    }
}

fn radio_label(g: &Game, time: f32, room: f32) -> String {
    if !g.now_playing.is_empty() {
        let name = g.now_playing.strip_prefix("LIVE ").unwrap_or(&g.now_playing);
        return ui::with_face(Face::Accent, || ui_ticker(name, 8.5, room, time));
    }
    if g.cooking {
        "cooking".to_string()
    } else {
        format!("q:{}", g.looper.quantise.label())
    }
}

pub(crate) const WIDGETS: [&str; 8] = ["radio", "counts", "recipes", "acorns", "clock", "fire", "mail", "bell"];
pub(crate) const RADIO: u8 = 0;
pub(crate) const COUNTS: u8 = 1;
pub(crate) const RECIPES: u8 = 2;
pub(crate) const ACORNS: u8 = 3;
pub(crate) const CLOCK: u8 = 4;
pub(crate) const FIRE: u8 = 5;
pub(crate) const MAIL: u8 = 6;
pub(crate) const BELL: u8 = 7;

thread_local! {
    static HIDE: std::cell::Cell<u16> = const { std::cell::Cell::new(0) };
}

pub(crate) fn hidden(w: u8) -> bool {
    HIDE.with(|h| h.get() & (1 << w) != 0)
}

pub(crate) fn hide_mask() -> u16 {
    HIDE.with(|h| h.get())
}

pub(crate) fn set_hide_mask(m: u16) {
    HIDE.with(|h| h.set(m & 0xff));
}

pub(crate) fn toggle_widget(w: u8) {
    HIDE.with(|h| h.set(h.get() ^ (1 << w)));
}

pub(crate) fn layout(g: &Game) -> Bar {
    let lw = low_w();
    let all = [Tab::Music, Tab::Chat, Tab::Items, Tab::Quests, Tab::Help, Tab::Settings];
    let tab_w = |t: Tab, words: bool| if words { 21.0 + ui_measure_as(Face::Label, tab_name(t), TAB_SIZE) } else { 18.0 };
    let dj = radio::flag(0);
    let label_w = if g.now_playing.is_empty() {
        ui_measure_as(Face::Label, &radio_label(g, 0.0, 999.0), 8.5).max(30.0)
    } else if dj {
        let name = g.now_playing.strip_prefix("LIVE ").unwrap_or(&g.now_playing);
        ui_measure_as(Face::Accent, name, 8.5).clamp(70.0, (lw * 0.42).max(70.0))
    } else {
        70.0
    } + if dj { 9.0 } else { 0.0 };
    let logo_w = |text: bool| if text { 24.0 + ui_measure_as(Face::Display, LOGO, 10.5) } else { 20.0 };
    let radio_w = |slider: bool| 4.0 + 14.0 + 5.0 + label_w + if slider { 6.0 + 40.0 + 4.0 + 10.0 } else { 0.0 } + 5.0;
    let mut pick = (true, true, true);
    for option in [(true, true, true), (false, true, true), (false, true, false), (false, false, false)] {
        pick = option;
        let tabs: f32 = all.iter().map(|t| tab_w(*t, option.1)).sum::<f32>() + 2.0 * (all.len() as f32 - 1.0) + 6.0;
        if 4.0 + logo_w(option.0) + 4.0 + tabs + 4.0 + radio_w(option.2) + 4.0 <= lw {
            break;
        }
    }
    let (logo_text, words, slider) = pick;
    let logo = (4.0, logo_w(logo_text));
    let mut x = logo.0 + logo.1 + 4.0;
    let group_x = x;
    x += 3.0;
    let mut tabs = Vec::new();
    for t in all {
        let w = tab_w(t, words);
        tabs.push((t, x, w, words));
        x += w + 2.0;
    }
    let group = (group_x, x - 2.0 + 3.0 - group_x);
    let rw = radio_w(slider);
    let rx = lw - 4.0 - rw;
    let play = (rx + 4.0, 14.0);
    let label = (play.0 + play.1 + 5.0, label_w);
    let (slider, speaker) = if slider { (Some((label.0 + label.1 + 6.0, 40.0)), Some(label.0 + label.1 + 6.0 + 40.0 + 4.0)) } else { (None, None) };
    Bar { logo, logo_text, tabs, group, radio: (rx, rw), play, label, slider, speaker }
}

fn active(g: &Game, t: Tab) -> bool {
    match t {
        Tab::Music => g.music_mode,
        Tab::Chat => g.venue.overlay == venues::Overlay::Talk,
        Tab::Items => g.show_inventory,
        Tab::Quests => g.venue.overlay == venues::Overlay::Quests,
        Tab::Help => g.show_help,
        Tab::Settings => g.paused,
    }
}

pub(crate) fn shown(g: &Game) -> bool {
    !g.two_player && g.venue.at == venues::Venue::None
}

pub(crate) fn click(g: &mut Game, fr: &mut Frame) -> bool {
    if !shown(g) || g.picking {
        return false;
    }
    let bar = layout(g);
    let radio_on = !hidden(RADIO);
    if let Some((sx, sw)) = bar.slider.filter(|_| radio_on) {
        if is_mouse_button_down(MouseButton::Left) && (g.mix_drag == Some(99) || (is_mouse_button_pressed(MouseButton::Left) && fr.uy > Y && fr.uy < Y + H && fr.ux > sx - 4.0 && fr.ux < sx + sw + 4.0)) {
            g.mix_drag = Some(99);
            g.mix[4] = ((fr.ux - sx) / sw).clamp(0.0, 1.0);
            g.audio.set_mix(g.mix);
            fr.click = false;
            return true;
        }
        if g.mix_drag == Some(99) {
            g.mix_drag = None;
            g.save_clock = 99.0;
        }
    }
    if !fr.click || fr.uy < Y || fr.uy > Y + H {
        return false;
    }
    if radio_on && fr.ux > bar.play.0 && fr.ux < bar.play.0 + bar.play.1 {
        g.looper.beat = !g.looper.beat;
        fr.click = false;
        return true;
    }
    if radio_on && fr.ux > bar.label.0 && fr.ux < bar.label.0 + bar.label.1 && g.now_playing.is_empty() && !g.cooking {
        g.looper.quantise = g.looper.quantise.next();
        g.save_clock = 99.0;
        fr.click = false;
        return true;
    }
    for (t, x, w, _) in bar.tabs.iter().copied() {
        if fr.ux > x && fr.ux < x + w {
            match t {
                Tab::Music => g.music_mode = !g.music_mode,
                Tab::Chat => {
                    if steps::partner::can_talk(g) {
                        steps::venues::open_talk(g);
                    }
                }
                Tab::Items => {
                    g.show_inventory = !g.show_inventory;
                    g.bag_owner = 0;
                    g.bag_sel = 0;
                }
                Tab::Quests => {
                    if g.venue.overlay == venues::Overlay::Quests {
                        g.venue.overlay = venues::Overlay::None;
                    } else if g.venue.overlay == venues::Overlay::None {
                        steps::quests_panel::open(g);
                    }
                }
                Tab::Help => g.show_help = !g.show_help,
                Tab::Settings => {
                    g.paused = true;
                    fr.just_paused = true;
                }
            }
            fr.click = false;
            return true;
        }
    }
    let right = if radio_on { bar.radio.0 + bar.radio.1 } else { bar.group.0 + bar.group.1 };
    if fr.ux > bar.logo.0 && fr.ux < right {
        fr.click = false;
        return true;
    }
    false
}

fn icon(t: Tab, x: f32, y: f32, c: Color) {
    match t {
        Tab::Music => {
            for (k, h) in [3.0, 6.0, 4.0, 7.0, 3.0].iter().enumerate() {
                ui_round(x + k as f32 * 1.7, y - h / 2.0, 1.0, *h, 0.5, c);
            }
        }
        Tab::Items => {
            ui_tri(vec2(x + 4.0, y - 4.5), vec2(x + 8.0, y - 2.3), vec2(x, y - 2.3), c);
            ui_round(x, y - 2.3, 8.0, 6.3, 1.0, c);
            ui_line(vec2(x + 4.0, y - 2.0), vec2(x + 4.0, y + 3.6), 0.6, skin::TAB);
            ui_line(vec2(x + 0.4, y - 2.0), vec2(x + 7.6, y - 2.0), 0.6, skin::TAB);
        }
        Tab::Chat => {
            ui_round(x, y - 4.0, 8.5, 6.0, 2.2, c);
            ui_tri(vec2(x + 1.5, y + 1.5), vec2(x + 4.0, y + 1.5), vec2(x + 1.0, y + 4.2), c);
            for k in 0..3 {
                ui_circle(x + 2.3 + k as f32 * 2.0, y - 1.0, 0.6, skin::TAB);
            }
        }
        Tab::Quests => {
            ui_round(x, y - 4.5, 8.0, 9.0, 1.5, c);
            for k in 0..3 {
                ui_rect(x + 1.6, y - 2.6 + k as f32 * 2.4, 1.2, 1.2, skin::TAB);
                ui_rect(x + 3.4, y - 2.4 + k as f32 * 2.4, 3.2, 0.8, skin::TAB);
            }
        }
        Tab::Help => {
            ui_round(x - 0.5, y - 4.5, 9.0, 9.0, 1.5, skin::GOLD);
            ui_rect(x + 1.7, y - 2.7, 1.4, 5.4, skin::INK);
            ui_rect(x + 5.4, y - 2.7, 1.4, 5.4, skin::INK);
            ui_rect(x + 1.7, y - 0.7, 5.1, 1.3, skin::INK);
        }
        Tab::Settings => {
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::PI / 4.0;
                ui_circle(x + 4.0 + a.cos() * 3.6, y + a.sin() * 3.6, 1.3, c);
            }
            ui_circle(x + 4.0, y, 3.3, c);
            ui_circle(x + 4.0, y, 1.4, skin::TAB);
        }
    }
}

fn speaker(x: f32, y: f32, c: Color) {
    ui_round(x, y - 1.6, 2.4, 3.2, 0.4, c);
    ui_tri(vec2(x + 1.6, y), vec2(x + 4.6, y - 3.4), vec2(x + 4.6, y + 3.4), c);
    for (k, r) in [2.4f32, 4.2].iter().enumerate() {
        let a0 = -0.8f32;
        for s in 0..4 {
            let a = a0 + s as f32 * 0.4;
            let b = a + 0.4;
            ui_line(vec2(x + 4.6 + a.cos() * r, y + a.sin() * r), vec2(x + 4.6 + b.cos() * r, y + b.sin() * r), 0.5, Color::new(c.r, c.g, c.b, c.a * (1.0 - k as f32 * 0.3)));
        }
    }
}

fn logo_icon(x: f32, y: f32, size: f32) {
    let tex = ICON.with(|i| {
        let mut i = i.borrow_mut();
        if i.is_none() {
            let t = Texture2D::from_file_with_format(include_bytes!("../../assets/icon64.png"), Some(ImageFormat::Png));
            t.set_filter(FilterMode::Nearest);
            *i = Some(t);
        }
        i.clone()
    });
    if let Some(t) = tex {
        let (kx, ky) = ui_scale();
        let s = size * kx.min(ky);
        draw_texture_ex(&t, (x * kx).round(), (y * ky).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
    }
}

pub(crate) fn draw(g: &mut Game, fr: &mut Frame) {
    let bar = layout(g);
    let cy = Y + H / 2.0;
    ui_round_frame(bar.logo.0, Y, bar.logo.1, H, 5.0, skin::PANEL, skin::EDGE);
    logo_icon(bar.logo.0 + 2.0, Y + 1.5, 15.0);
    if bar.logo_text {
        ui_text_bold(Face::Display, LOGO, bar.logo.0 + 20.0, Y + 13.5, 10.5, skin::GOLD);
    }
    ui_round_frame(bar.group.0, Y, bar.group.1, H, 6.0, skin::PANEL, skin::EDGE);
    for (t, x, w, words) in bar.tabs.iter().copied() {
        let on = active(g, t);
        let hover = fr.uy > Y && fr.uy < Y + H && fr.ux > x && fr.ux < x + w;
        let (fill, ink) = if on { (skin::PINK, skin::INK) } else if hover { (Color::new(0.32, 0.25, 0.52, 1.0), skin::CREAM) } else { (skin::TAB, skin::CREAM) };
        ui_round(x, Y + 2.5, w, H - 5.0, 4.0, fill);
        if on {
            let pts = (x, Y + 2.5, w, H - 5.0);
            ui_round_frame(pts.0, pts.1, pts.2, pts.3, 4.0, fill, skin::PINK_EDGE);
        }
        icon(t, x + 5.0, cy, ink);
        if words {
            ui_text_as(Face::Label, tab_name(t), x + 16.0, cy + 3.0, TAB_SIZE, ink);
        }
    }
    if !hidden(RADIO) {
    let (rx, rw) = bar.radio;
    ui_round_frame(rx, Y, rw, H, 6.0, skin::PANEL, skin::EDGE);
    let (px, pw) = bar.play;
    ui_round(px, Y + 2.0, pw, H - 4.0, 3.5, if g.looper.beat { skin::PINK } else { Color::new(0.75, 0.38, 0.55, 1.0) });
    if g.looper.beat {
        ui_round(px + 4.3, cy - 3.0, 2.0, 6.0, 0.6, skin::INK);
        ui_round(px + 7.7, cy - 3.0, 2.0, 6.0, 0.6, skin::INK);
    } else {
        ui_tri(vec2(px + 4.8, cy - 3.5), vec2(px + 10.0, cy), vec2(px + 4.8, cy + 3.5), skin::INK);
    }
    let dj = radio::flag(0);
    let (lx, lw) = if dj {
        let on = radio::online();
        ui_circle(bar.label.0 + 3.0, cy, 2.6, if on { Color::from_rgba(120, 230, 140, 255) } else { Color::from_rgba(255, 90, 110, 255) });
        (bar.label.0 + 9.0, bar.label.1 - 9.0)
    } else {
        (bar.label.0, bar.label.1)
    };
    let label = radio_label(g, fr.time, lw);
    ui_text_as(if g.now_playing.is_empty() { Face::Label } else { Face::Accent }, &label, lx, cy + 3.0, 8.5, if g.now_playing.is_empty() { skin::CREAM } else { skin::GOLD });
    if let Some((sx, sw)) = bar.slider {
        let v = g.mix[4];
        ui_round(sx, cy - 1.2, sw, 2.4, 1.2, Color::new(1.0, 1.0, 1.0, 0.25));
        ui_round(sx, cy - 1.2, sw * v, 2.4, 1.2, skin::PINK_EDGE);
        ui_circle(sx + sw * v, cy, 3.2, skin::CREAM);
    }
    if let Some(spx) = bar.speaker {
        speaker(spx, cy, skin::CREAM);
    }
    }
    stats(g, fr);
}

fn pill_w(parts: &[(Option<Color>, String)]) -> f32 {
    parts.iter().map(|(c, t)| if c.is_some() { 8.0 } else { 0.0 } + ui_measure_as(Face::Data, t, 9.0) + 5.0).sum::<f32>() + 5.0
}

fn pill(x: f32, y: f32, parts: &[(Option<Color>, String)]) {
    let w = pill_w(parts);
    ui_round_frame(x, y, w, 14.0, 5.0, skin::PANEL, skin::EDGE);
    let mut px = x + 5.0;
    for (c, t) in parts {
        if let Some(c) = c {
            ui_round(px, y + 4.0, 6.0, 6.0, 1.5, *c);
            px += 8.0;
        }
        ui_text_as(Face::Data, t, px, y + 10.5, 9.0, skin::CREAM);
        px += ui_measure_as(Face::Data, t, 9.0) + 5.0;
    }
}

fn stats(g: &Game, fr: &mut Frame) {
    let y = Y + H + 3.0;
    let fish: u32 = g.inv.fish.iter().map(|n| *n as u32).sum();
    let res = [(Some(Color::from_rgba(255, 128, 74, 255)), format!("{}", g.inv.wood)), (Some(Color::from_rgba(200, 200, 214, 255)), format!("{}", g.inv.stone)), (Some(Color::from_rgba(120, 200, 240, 255)), format!("{}", fish))];
    let dishes = [(None, format!("{}/{}", g.dishes.len(), DISHES.len()))];
    let acorns = [(None, format!("{}", g.inv.acorns))];
    let dish_w = pill_w(&dishes) + 9.0;
    let acorn_w = pill_w(&acorns) + 9.0;
    let res_w = pill_w(&res);
    let fire_w = 44.0;
    let mut x = low_w() - 4.0;
    if !hidden(ACORNS) {
        x -= acorn_w;
        let ax = x;
        ui_round_frame(ax, y, acorn_w, 14.0, 5.0, skin::PANEL, skin::EDGE);
        ui_acorn(ax + 8.5, y + 7.4, 1.0);
        ui_text_as(Face::Data, &acorns[0].1, ax + 15.0, y + 10.5, 9.0, skin::CREAM);
        x -= 3.0;
    }
    if !hidden(RECIPES) {
        x -= dish_w;
        let dx = x;
        ui_round_frame(dx, y, dish_w, 14.0, 5.0, skin::PANEL, skin::EDGE);
        ui_circle(dx + 8.5, y + 8.0, 3.6, Color::from_rgba(255, 150, 180, 255));
        ui_round(dx + 4.0, y + 6.5, 9.0, 1.6, 0.8, Color::from_rgba(255, 200, 220, 255));
        ui_text_as(Face::Data, &dishes[0].1, dx + 15.0, y + 10.5, 9.0, skin::CREAM);
        x -= 3.0;
    }
    if !hidden(COUNTS) {
        x -= res_w;
        pill(x, y, &res);
        x -= 3.0;
    }
    let row1 = !(hidden(ACORNS) && hidden(RECIPES) && hidden(COUNTS));
    let row2_left = x + 3.0;
    let y3 = if row1 { y + 17.0 } else { y };
    let sk = sky();
    let temp = sk.temp.map(|t| format!("  {:.0}C", t)).unwrap_or_default();
    let line = format!("{:02}:{:02}  {}  {}{}", sk.hour as u32, ((sk.hour.fract()) * 60.0) as u32, SEASONS[sk.season], WEATHERS[sk.weather], temp);
    let tw = ui_measure(&line, 9.0) + 20.0;
    let row2 = !(hidden(CLOCK) && hidden(FIRE));
    let fx = low_w() - 4.0 - fire_w;
    let tx = if hidden(FIRE) { low_w() - 4.0 - tw } else { fx - 3.0 - tw };
    DROP.with(|d| d.set(match (row1, row2) { (true, true) => 17.0, (false, false) => -17.0, _ => 0.0 }));
    if !hidden(FIRE) {
        ui_round_frame(fx, y3, fire_w, 14.0, 5.0, skin::PANEL, skin::EDGE);
        let lit = g.camp.lit;
        let flame = if lit { Color::from_rgba(255, 140, 50, 255) } else { Color::from_rgba(140, 120, 130, 255) };
        ui_tri(vec2(fx + 6.5, y3 + 2.5), vec2(fx + 10.0, y3 + 10.5), vec2(fx + 3.0, y3 + 10.5), flame);
        ui_circle(fx + 6.5, y3 + 9.0, 3.2, flame);
        if lit {
            ui_circle(fx + 6.5, y3 + 9.6, 1.6, Color::from_rgba(255, 220, 110, 255));
        }
        let fill = (g.camp.fuel / 180.0).clamp(0.0, 1.0);
        ui_round(fx + 13.0, y3 + 5.5, 26.0, 3.0, 1.5, Color::new(0.0, 0.0, 0.0, 0.35));
        if fill > 0.0 {
            ui_round(fx + 13.0, y3 + 5.5, 26.0 * fill, 3.0, 1.5, flame);
        }
    }
    if !hidden(CLOCK) {
    let ty = y3;
    ui_round_frame(tx, ty, tw, 14.0, 5.0, skin::PANEL, skin::EDGE);
    let night = sk.hour < 6.0 || sk.hour >= 19.0;
    if night {
        ui_circle(tx + 8.0, ty + 7.0, 4.0, skin::GOLD);
        ui_circle(tx + 10.0, ty + 5.6, 3.6, skin::PANEL_SOLID);
    } else {
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::PI / 4.0;
            ui_line(vec2(tx + 8.0 + a.cos() * 3.4, ty + 7.0 + a.sin() * 3.4), vec2(tx + 8.0 + a.cos() * 5.0, ty + 7.0 + a.sin() * 5.0), 0.5, skin::GOLD);
        }
        ui_circle(tx + 8.0, ty + 7.0, 2.8, skin::GOLD);
    }
    ui_text(&line, tx + 15.0, ty + 10.5, 9.0, skin::CREAM);
    }
    fr.ix = if row2 { row2_left.min(if hidden(CLOCK) { fx } else { tx }) } else { row2_left };
}

pub(crate) fn drop(g: &Game) -> f32 {
    if shown(g) { DROP.with(|d| d.get()) } else { 0.0 }
}
