#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use steps::venues::{button, hit};
use ui::skin;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Thing {
    Furniture(home::Item, u32),
    Food(usize, u32),
    Fish(usize, u32),
    Mush(usize, u32),
    Wood(u32),
    Stone(u32),
    Acorns(u32),
}

pub(crate) const TABS: [&str; 5] = ["all", "furniture", "music", "food", "gathered"];
const COLS: usize = 5;
const ROWS: usize = 3;
pub(crate) const CELL_PX: f32 = 64.0;

fn musical(item: home::Item) -> bool {
    use home::Item::*;
    matches!(item, Piano | Drums | WindChime | Speaker | RecordPlayer | DjBooth | Subwoofer | Sampler | Arcade)
}

const RARITY: [&str; 3] = ["common", "uncommon", "rare"];

pub(crate) fn things(g: &Game, tab: usize) -> Vec<Thing> {
    let mut furniture: Vec<(home::Item, u32)> = Vec::new();
    for it in &g.stored {
        match furniture.iter_mut().find(|(i, _)| i == it) {
            Some(e) => e.1 += 1,
            None => furniture.push((*it, 1)),
        }
    }
    let mut out = Vec::new();
    if tab == 0 || tab == 1 || tab == 2 {
        for (it, n) in &furniture {
            if tab == 0 || (tab == 2) == musical(*it) {
                out.push(Thing::Furniture(*it, *n));
            }
        }
    }
    if tab == 0 || tab == 3 {
        for d in 0..DISHES.len() {
            let n = g.meals.get(d).copied().unwrap_or(0);
            if n > 0 {
                out.push(Thing::Food(d, n));
            }
        }
    }
    if tab == 0 || tab == 4 {
        for k in 0..3 {
            if g.inv.fish[k] > 0 {
                out.push(Thing::Fish(k, g.inv.fish[k]));
            }
        }
        for k in 0..3 {
            if g.inv.mush[k] > 0 {
                out.push(Thing::Mush(k, g.inv.mush[k]));
            }
        }
        if g.inv.wood > 0 {
            out.push(Thing::Wood(g.inv.wood));
        }
        if g.inv.stone > 0 {
            out.push(Thing::Stone(g.inv.stone));
        }
        if g.inv.acorns > 0 {
            out.push(Thing::Acorns(g.inv.acorns));
        }
    }
    out
}

pub(crate) fn name(t: Thing) -> String {
    match t {
        Thing::Furniture(i, _) => i.name().to_string(),
        Thing::Food(d, _) => DISHES[d].to_string(),
        Thing::Fish(k, _) => format!("{} fish", RARITY[k]),
        Thing::Mush(k, _) => format!("{} mushroom", RARITY[k]),
        Thing::Wood(_) => "wood".to_string(),
        Thing::Stone(_) => "stone".to_string(),
        Thing::Acorns(_) => "acorns".to_string(),
    }
}

fn about(t: Thing) -> String {
    match t {
        Thing::Furniture(i, _) => i.note().to_string(),
        Thing::Food(d, _) => format!("eat it and your voice goes {} for 30 seconds.", crate::flavour::dish(d).0),
        Thing::Fish(..) => "cook it in the pot at the campfire, or try a new mix for a new dish.".to_string(),
        Thing::Mush(..) => "cook it in the pot at the campfire, alone or with fish.".to_string(),
        Thing::Wood(_) => "keeps the campfire burning.".to_string(),
        Thing::Stone(_) => "builds the stone ring round the fire.".to_string(),
        Thing::Acorns(_) => "spend them at the catalogue and the bars.".to_string(),
    }
}

fn count(t: Thing) -> u32 {
    match t {
        Thing::Furniture(_, n) | Thing::Food(_, n) | Thing::Fish(_, n) | Thing::Mush(_, n) | Thing::Wood(n) | Thing::Stone(n) | Thing::Acorns(n) => n,
    }
}

fn tab_icon(i: usize, x: f32, y: f32, c: Color) {
    match i {
        0 => {
            ui_text_as(ui::Face::Label, "all", x - 6.0, y + 3.0, 9.0, c);
        }
        1 => {
            ui_rect(x - 3.0, y - 4.0, 6.0, 1.6, c);
            ui_rect(x - 3.0, y - 4.0, 1.6, 8.0, c);
            ui_rect(x - 3.0, y, 6.5, 1.6, c);
            ui_rect(x + 2.0, y, 1.6, 4.0, c);
        }
        2 => {
            ui_rect(x + 1.0, y - 4.5, 1.4, 7.0, c);
            ui_rect(x + 1.0, y - 4.5, 3.4, 1.4, c);
            ui_circle(x - 0.4, y + 2.6, 2.0, c);
        }
        3 => {
            ui_round(x - 4.5, y - 0.5, 9.0, 4.5, 2.0, c);
            ui_rect(x - 5.0, y - 1.5, 10.0, 1.4, c);
            ui_rect(x - 1.0, y - 5.0, 1.0, 2.5, c);
            ui_rect(x + 1.5, y - 5.5, 1.0, 3.0, c);
        }
        _ => {
            ui_circle(x, y + 1.0, 3.2, c);
            ui_rect(x - 0.5, y - 4.5, 1.2, 3.0, c);
            ui_tri(vec2(x + 0.6, y - 4.0), vec2(x + 4.0, y - 4.5), vec2(x + 1.5, y - 2.0), c);
        }
    }
}

pub(crate) fn panel_rect(fr: &Frame) -> (f32, f32, f32, f32) {
    let w = 318f32.min(low_w() - 8.0);
    let h = 200f32.min(low_h() - 8.0);
    let x = ((low_w() - w) / 2.0).round();
    let top: f32 = if fr.touch_ui && fr.portrait { 42.0 } else { 40.0 };
    let y = top.max(((low_h() - h) / 2.0).round()).min(low_h() - h - 4.0);
    (x, y, w, h)
}

pub(crate) fn draw(g: &mut Game, fr: &mut Frame) {
    if !g.show_inventory || g.picking || g.shop_open || g.show_help {
        return;
    }
    let tab = g.venue.bag_tab.min(TABS.len() - 1);
    let list = things(g, tab);
    if !list.is_empty() {
        g.bag_sel = g.bag_sel.min(list.len() - 1);
    }
    let sel = list.get(g.bag_sel).copied();
    let per_page = COLS * ROWS;
    let page = g.bag_sel / per_page;
    let shown: Vec<Thing> = list.iter().skip(page * per_page).take(per_page).copied().collect();
    home_draw::render_bag_icons(&g.bag_icons, &shown, fr.time);
    if let Some(t) = sel {
        home_draw::render_bag_detail(&g.bag_detail, t, fr.time);
    }
    set_default_camera();

    let (x, y, w, h) = panel_rect(fr);
    ui_round(x + 2.0, y + 3.0, w, h, 10.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 10.0, skin::PANEL_SOLID, skin::EDGE_STRONG);
    ui_circle(x + 15.0, y + 13.0, 7.0, skin::PINK);
    steps::topbar::items_icon(x + 11.0, y + 13.5, skin::INK);
    let title = if g.two_player { if g.bag_owner == 0 { "items - player 1" } else { "items - player 2" } } else { "items" };
    ui_text_as(ui::Face::Display, title, x + 27.0, y + 18.0, 12.0, skin::GOLD);
    let close = button(fr, "x", x + w - 22.0, y + 6.0, 16.0, 15.0, false);

    let grid_w = w - 120.0;
    let tw = ((grid_w - 4.0 * 3.0) / 5.0).floor();
    let mut tab_click = None;
    for i in 0..TABS.len() {
        let tx = x + 8.0 + i as f32 * (tw + 3.0);
        let ty = y + 27.0;
        let on = i == tab;
        let over = hit(fr, tx, ty, tw, 16.0);
        ui_round_frame(tx, ty, tw, 16.0, 5.0, if on { skin::PINK } else if over { Color::new(0.32, 0.25, 0.52, 1.0) } else { skin::TAB }, if on { skin::PINK_EDGE } else { skin::EDGE });
        tab_icon(i, tx + tw / 2.0, ty + 8.0, if on { skin::INK } else { skin::CREAM });
        if over && fr.click {
            tab_click = Some(i);
        }
    }

    let (gx, gy) = (x + 8.0, y + 48.0);
    let gap = 3.0;
    let cell = ((grid_w - gap * (COLS as f32 - 1.0)) / COLS as f32).floor();
    ui_round(gx - 3.0, gy - 3.0, cell * COLS as f32 + gap * (COLS as f32 - 1.0) + 6.0, cell * ROWS as f32 + gap * (ROWS as f32 - 1.0) + 6.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.18));
    let (kx, ky) = ui_scale();
    let mut cell_click = None;
    for r in 0..ROWS {
        for c in 0..COLS {
            let k = r * COLS + c;
            let (cx, cy) = (gx + c as f32 * (cell + gap), gy + r as f32 * (cell + gap));
            let index = page * per_page + k;
            let selected = index == g.bag_sel && k < shown.len();
            ui_round_frame(cx, cy, cell, cell, 5.0, if k < shown.len() { Color::new(1.0, 1.0, 1.0, 0.07) } else { Color::new(1.0, 1.0, 1.0, 0.03) }, if selected { skin::PINK } else { Color::new(1.0, 1.0, 1.0, 0.08) });
            if let Some(t) = shown.get(k) {
                let src = Rect::new(c as f32 * CELL_PX, r as f32 * CELL_PX, CELL_PX, CELL_PX);
                draw_texture_ex(&g.bag_icons.texture, (cx + 1.0) * kx, (cy + 1.0) * ky, WHITE, DrawTextureParams { dest_size: Some(vec2((cell - 2.0) * kx, (cell - 2.0) * ky)), source: Some(src), flip_y: true, ..Default::default() });
                let n = count(*t);
                if n > 1 {
                    let s = format!("{n}");
                    let sw = ui_measure_as(ui::Face::Data, &s, 8.0) + 5.0;
                    ui_round(cx + cell - sw - 2.0, cy + cell - 11.0, sw, 9.0, 3.0, Color::new(0.08, 0.05, 0.16, 0.85));
                    ui_text_as(ui::Face::Data, &s, cx + cell - sw + 0.5, cy + cell - 4.0, 8.0, skin::CREAM);
                }
                if selected {
                    ui_round_frame(cx - 1.0, cy - 1.0, cell + 2.0, cell + 2.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.0), skin::PINK_EDGE);
                }
                if hit(fr, cx, cy, cell, cell) && fr.click {
                    cell_click = Some(index);
                }
            }
        }
    }
    let pages = list.len().div_ceil(per_page).max(1);
    if pages > 1 {
        let p = format!("page {}/{}", page + 1, pages);
        ui_text(&p, gx, gy + ROWS as f32 * (cell + gap) + 8.0, 7.0, skin::MUTED);
    }
    if list.is_empty() {
        let empty = match tab {
            1 | 2 => "no furniture in the bag. inside, M while carrying a piece puts it here.",
            3 => "no food yet. cook in the pot at the campfire.",
            4 => "nothing gathered yet. fish, pick mushrooms and fetch wood.",
            _ => "your bag is empty.",
        };
        for (i, l) in ui_wrap(empty, 8.0, grid_w - 20.0).iter().enumerate() {
            ui_text(l, gx + 6.0, gy + cell + 4.0 + i as f32 * 10.0, 8.0, skin::MUTED);
        }
    }

    let (dx, dy, dw, dh) = (x + w - 110.0, y + 27.0, 102.0, h - 42.0);
    ui_round_frame(dx, dy, dw, dh, 8.0, Color::new(1.0, 1.0, 1.0, 0.05), skin::EDGE_STRONG);
    let mut primary = false;
    let mut secondary = false;
    if let Some(t) = sel {
        let (px, py, pw, ph) = (dx + 6.0, dy + 6.0, dw - 12.0, 64.0);
        ui_round(px, py, pw, ph, 6.0, Color::new(0.0, 0.0, 0.0, 0.25));
        draw_texture_ex(&g.bag_detail.texture, px * kx, py * ky, WHITE, DrawTextureParams { dest_size: Some(vec2(pw * kx, ph * ky)), flip_y: true, ..Default::default() });
        let nm = name(t);
        let mut size = 10.0;
        let mut lines = ui::with_face(ui::Face::Label, || ui_wrap(&nm, size, dw - 12.0));
        while lines.len() > 2 && size > 7.0 {
            size -= 1.0;
            lines = ui::with_face(ui::Face::Label, || ui_wrap(&nm, size, dw - 12.0));
        }
        let mut ty = py + ph + 12.0;
        for l in lines.iter().take(2) {
            ui_text_as(ui::Face::Label, l, dx + 6.0, ty, size, skin::CREAM);
            ty += size + 1.0;
        }
        let (a, b) = actions(g, t);
        let no_action = a.is_none();
        let buttons_top = dy + dh - if a.is_some() && b.is_some() { 44.0 } else { 24.0 };
        let desc = ui_wrap(&about(t), 7.0, dw - 12.0);
        let room = (((buttons_top - ty + 4.0) / 8.5).floor().max(0.0)) as usize;
        for (i, l) in desc.iter().take(room).enumerate() {
            let shown = if i + 1 == room && desc.len() > room { format!("{}...", l.trim_end_matches('.')) } else { l.clone() };
            ui_text(&shown, dx + 6.0, ty, 7.0, skin::MUTED);
            ty += 8.5;
        }
        let bw = dw - 12.0;
        if let Some(label) = a {
            primary = gold_button(fr, &label, dx + 6.0, if b.is_some() { dy + dh - 40.0 } else { dy + dh - 21.0 }, bw, 16.0);
        }
        if let Some(label) = b {
            secondary = button(fr, &label, dx + 6.0, dy + dh - 20.0, bw, 15.0, false);
        }
        if no_action {
            let note = match t {
                Thing::Furniture(..) => "go home to place it",
                Thing::Fish(..) | Thing::Mush(..) => "E at the campfire pot",
                _ => "",
            };
            if !note.is_empty() {
                ui_round(dx + 6.0, dy + dh - 22.0, bw, 16.0, 5.0, Color::new(1.0, 1.0, 1.0, 0.06));
                ui_text(note, dx + 6.0 + (bw - ui_measure(note, 7.0)) / 2.0, dy + dh - 11.0, 7.0, skin::MUTED);
            }
        }
    } else {
        ui_text("nothing here", dx + 8.0, dy + 20.0, 8.0, skin::MUTED);
    }
    let hint = if fr.touch_ui {
        "tap an item - tap the bag to close"
    } else if g.bag_owner == 1 {
        "IJKL choose   O use   U second   Bksp close"
    } else {
        "WASD choose   E use   X second   tab: section   M close"
    };
    ui_text(hint, x + (w - ui_measure(hint, 7.0)) / 2.0, y + h - 5.0, 7.0, skin::MUTED);

    if let Some(i) = tab_click {
        g.venue.bag_tab = i;
        g.bag_sel = 0;
        fr.click = false;
    }
    if let Some(i) = cell_click {
        g.bag_sel = i;
        fr.click = false;
    }
    if primary || secondary {
        fr.click = false;
        if let Some(t) = sel {
            act(g, t, primary);
        }
    }
    if close {
        g.show_inventory = false;
        fr.click = false;
    }
}

fn gold_button(fr: &Frame, label: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
    let over = hit(fr, x, y, w, h);
    let fill = if over { Color::from_rgba(255, 226, 140, 255) } else { Color::from_rgba(255, 214, 110, 255) };
    ui_round_frame(x, y, w, h, 5.0, fill, Color::from_rgba(200, 150, 60, 255));
    let lw = ui_measure_as(ui::Face::Label, label, 9.0);
    ui_text_as(ui::Face::Label, label, x + (w - lw) / 2.0, y + h / 2.0 + 3.5, 9.0, skin::INK);
    over && fr.click
}

fn empty_table(g: &Game) -> Option<usize> {
    g.house.pieces.iter().position(|p| p.item == home::Item::Table && p.state == 0)
}

fn actions(g: &Game, t: Thing) -> (Option<String>, Option<String>) {
    let at_home = g.inside && !steps::visits::visiting(g);
    match t {
        Thing::Furniture(..) => (
            at_home.then(|| "place".to_string()),
            (at_home && !g.house.pieces.is_empty()).then(|| if g.bag_clear_armed > 0.0 { "again: store all".to_string() } else { "store all".to_string() }),
        ),
        Thing::Food(..) => (Some("eat".to_string()), (at_home && empty_table(g).is_some()).then(|| "serve".to_string())),
        _ => (None, None),
    }
}

pub(crate) fn act(g: &mut Game, t: Thing, primary: bool) {
    let owner = g.bag_owner;
    let at_home = g.inside && !steps::visits::visiting(g);
    match (t, primary) {
        (Thing::Furniture(item, _), true) if at_home => {
            if let Some(pos) = g.stored.iter().position(|i| *i == item) {
                g.stored.remove(pos);
                let ahead = crate::player::ahead_local(&g.players[owner]);
                g.holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                g.holding_by = owner;
                g.show_inventory = false;
                g.toasts.push((format!("took the {} out - {} to place it", item.name(), if owner == 0 { "E" } else { "O" }), 2.5));
            }
        }
        (Thing::Furniture(..), false) if at_home && !g.house.pieces.is_empty() => {
            if g.bag_clear_armed > 0.0 {
                let n = g.house.pieces.len();
                for piece in g.house.pieces.drain(..) {
                    g.stored.push(piece.item);
                }
                for k in 0..2 {
                    g.seat[k] = None;
                    g.players[k].seated = false;
                    g.players[k].sleeping = false;
                    g.dj[k] = false;
                    g.ddr.on[k] = false;
                }
                g.bag_clear_armed = 0.0;
                g.save_clock = 99.0;
                g.toasts.push((format!("stored all {} pieces in your bag", n), 2.5));
            } else {
                g.bag_clear_armed = 3.0;
            }
        }
        (Thing::Food(d, _), true) => {
            crate::flavour::eat(g, d);
        }
        (Thing::Food(d, _), false) if at_home => {
            if let Some(i) = empty_table(g) {
                if g.meals[d] > 0 {
                    g.meals[d] -= 1;
                    g.house.pieces[i].state = (d + 1) as u8;
                    g.audio.play_ch(3, &g.plink, 0.4);
                    g.toasts.push((format!("you serve the {} on the table", DISHES[d]), 2.5));
                    g.save_clock = 99.0;
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn keys(g: &mut Game, fr: &mut Frame) {
    fr.bag_busy = g.show_inventory && !g.shop_open && !g.picking;
    if !fr.bag_busy || g.holding.is_some() {
        return;
    }
    let owner = g.bag_owner;
    let p = |k: KeyCode| is_key_pressed(k);
    let (up, down, left, right, use1, use2) = if owner == 0 {
        (p(KeyCode::W) || p(KeyCode::Up), p(KeyCode::S) || p(KeyCode::Down), p(KeyCode::A) || p(KeyCode::Left), p(KeyCode::D) || p(KeyCode::Right), p(KeyCode::E) || p(KeyCode::Enter), p(KeyCode::X))
    } else {
        (p(KeyCode::I), p(KeyCode::K), p(KeyCode::J), p(KeyCode::L), p(KeyCode::O), p(KeyCode::U))
    };
    if p(KeyCode::Tab) {
        let n = TABS.len();
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        g.venue.bag_tab = (g.venue.bag_tab + if shift { n - 1 } else { 1 }) % n;
        g.bag_sel = 0;
    }
    let list = things(g, g.venue.bag_tab.min(TABS.len() - 1));
    let n = list.len();
    if n == 0 {
        return;
    }
    let s = g.bag_sel.min(n - 1);
    g.bag_sel = if left {
        s.saturating_sub(1)
    } else if right {
        (s + 1).min(n - 1)
    } else if up {
        s.saturating_sub(COLS)
    } else if down {
        (s + COLS).min(n - 1)
    } else {
        s
    };
    if use1 || use2 {
        let t = list[g.bag_sel];
        act(g, t, use1);
        let left_now = things(g, g.venue.bag_tab.min(TABS.len() - 1)).len();
        g.bag_sel = g.bag_sel.min(left_now.saturating_sub(1));
    }
}
