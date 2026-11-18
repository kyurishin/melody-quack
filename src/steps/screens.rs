#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn bookshelf(g: &mut Game, fr: &mut Frame) {
        let left_k = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::J);
        let right_k = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::L);
        let pick_k = is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::O) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        let back_k = is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace);
        let close_hit = fr.click && fr.ux > low_w() - 30.0 && fr.uy < 26.0;
        let tap = fr.click && !close_hit;
        let (tap_left, tap_right) = (tap && fr.ux < low_w() * 0.33, tap && fr.ux > low_w() * 0.67);
        let tap_mid = tap && !tap_left && !tap_right;
        if g.open_book.is_none() || !g.spines_painted {
            for (i, book) in books::BOOKS.iter().enumerate() {
                paint_spine(&g.spine_rt[i], book);
            }
            g.spines_painted = g.painted.is_some();
        }
        let count = books::BOOKS.len();
        match g.open_book {
            None => {
                if left_k || tap_left {
                    g.shelf_sel = (g.shelf_sel + count - 1) % count;
                    g.audio.play_ch(3, &g.plink, 0.2);
                }
                if right_k || tap_right {
                    g.shelf_sel = (g.shelf_sel + 1) % count;
                    g.audio.play_ch(3, &g.plink, 0.2);
                }
                if pick_k || tap_mid {
                    g.open_book = Some(g.shelf_sel);
                    g.book_spread = 0;
                    g.page_turn = 0.0;
                    g.audio.play_ch(3, &g.bells[3], 0.25);
                }
                if back_k || close_hit {
                    g.shelf_open = false;
                    g.save_clock = 99.0;
                }
            }
            Some(_) => {
                if (left_k || tap_left) && g.book_spread > 0 && g.page_turn <= 0.0 {
                    g.book_spread -= 1;
                    g.page_turn = 1.0;
                    g.turn_dir = -1.0;
                    g.audio.play_ch(3, &g.plink, 0.25);
                }
                if (right_k || tap_right || pick_k || tap_mid) && g.book_spread + 1 < books::spreads(&books::BOOKS[g.open_book.unwrap_or(0)]) && g.page_turn <= 0.0 {
                    g.book_spread += 1;
                    g.page_turn = 1.0;
                    g.turn_dir = 1.0;
                    g.audio.play_ch(3, &g.plink, 0.25);
                }
                if back_k || close_hit {
                    g.open_book = None;
                }
            }
        }
        g.page_turn = (g.page_turn - get_frame_time() * 2.6).max(0.0);
        if let Some(b) = g.open_book {
            paint_page(&g.page_rt[0], &books::BOOKS[b], g.book_spread * 2, true);
            paint_page(&g.page_rt[1], &books::BOOKS[b], g.book_spread * 2 + 1, false);
            g.painted = Some((b, g.book_spread));
            draw_book_scene(&books::BOOKS[b], &g.page_rt, g.page_turn, g.turn_dir, fr.time);
        } else {
            draw_shelf_scene(g.shelf_sel, fr.time, &g.spine_rt);
        }
        set_default_camera();
        let touch = radio::touch_ui();
        ui_rect(low_w() - 26.0, 6.0, 20.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.16));
        ui_text("x", low_w() - 19.0, 19.0, 12.0, WHITE);
        match g.open_book {
            None => {
                let book = &books::BOOKS[g.shelf_sel];
                let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
                ui_text("the bookshelf", 10.0, 18.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text(book.title, (low_w() - ui_measure(book.title, 18.0)) / 2.0, low_h() - 30.0, 18.0, c);
                let hint = if touch { "tap a side to choose, the middle to open" } else { "A/D choose   E open   esc: close" };
                ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 12.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
            }
            Some(b) => {
                let book = &books::BOOKS[b];
                ui_text(book.title, 10.0, 18.0, 12.0, Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255));
                let at = format!("pages {}-{} of {}", g.book_spread * 2 + 1, g.book_spread * 2 + 2, book.pages.len());
                ui_text(&at, (low_w() - ui_measure(&at, 10.0)) / 2.0, low_h() - 24.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.8));
                let hint = if touch { "tap left or right to turn the page" } else { "A/D turn the page   esc: back to the shelf" };
                ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
        }
}

pub(crate) fn fishdex(g: &mut Game, fr: &mut Frame) {
        let shrooms_mode = g.dex_mode == 1;
        let counts: &Vec<u32> = if shrooms_mode { &g.shroom_counts } else { &g.dex_counts };
        let total = if shrooms_mode { mushroom::COUNT } else { fish::COUNT };
        let name_of = |i: usize| if shrooms_mode { mushroom::SPECIES[i] } else { fish::SPECIES[i] };
        let code_of = |i: usize| if shrooms_mode { mushroom::code(i) } else { fish::code(i) };
        let rarity_of = |i: usize| if shrooms_mode { mushroom::rarity(i) } else { fish::rarity(i) };
        let hint_of = |i: usize| if shrooms_mode { mushroom::hint(i) } else { fish::hint(i) };
        let acorns_of = |i: usize| if shrooms_mode { mushroom::acorns(i) } else { fish::acorns(i) };
        let art_of = |i: usize| if shrooms_mode { &g.shroom_art[i] } else { &g.fish_art[i] };
        let (art_w, art_h) = if shrooms_mode { (mushroom::ART, mushroom::ART) } else { (fish::ART_W, fish::ART_H) };
        let verb = if shrooms_mode { "found" } else { "caught" };
        let numeric = g.dex_query.trim().trim_start_matches('#').trim_start_matches('m').parse::<usize>().is_ok();
        let found: Vec<usize> = (if shrooms_mode { mushroom::search(&g.dex_query) } else { fish::search(&g.dex_query) }).into_iter().filter(|i| g.dex_query.trim().is_empty() || numeric || counts[*i] > 0).collect();
        let tab_names = ["catalogue", "diary", "fishdex", "mushrooms"];
        let mut tab_go: Option<usize> = None;
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Right) {
            tab_go = Some(if g.dex_mode == 0 { 3 } else if g.dex_from_shop { 0 } else { 2 });
        }
        if is_key_pressed(KeyCode::Left) {
            tab_go = Some(if g.dex_mode == 1 { 2 } else if g.dex_from_shop { 1 } else { 3 });
        }
        if is_key_pressed(KeyCode::Up) && g.dex_sel == 0 && g.dex_from_shop {
            tab_go = Some(if g.dex_mode == 1 { 2 } else { 1 });
        }
        while let Some(ch) = get_char_pressed() {
            if (ch.is_ascii_alphanumeric() || ch == ' ' || ch == '#') && g.dex_query.len() < 20 {
                g.dex_query.push(ch.to_ascii_lowercase());
                g.dex_sel = 0;
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            g.dex_query.pop();
            g.dex_sel = 0;
        }
        let n = found.len();
        if n > 0 {
            if is_key_pressed(KeyCode::Down) {
                g.dex_sel = (g.dex_sel + 1).min(n - 1);
            }
            if is_key_pressed(KeyCode::Up) && tab_go.is_none() {
                g.dex_sel = g.dex_sel.saturating_sub(1);
            }
            let wheel = mouse_wheel().1;
            if wheel < 0.0 {
                g.dex_sel = (g.dex_sel + 1).min(n - 1);
            } else if wheel > 0.0 {
                g.dex_sel = g.dex_sel.saturating_sub(1);
            }
        }
        g.dex_sel = g.dex_sel.min(n.saturating_sub(1));
        let caught = counts.iter().filter(|c| **c > 0).count();
        let w = (low_w() - 12.0).min(390.0);
        let h = (low_h() - 16.0).min(260.0);
        let x = ((low_w() - w) / 2.0).round();
        let y = ((low_h() - h) / 2.0).round().max(8.0);
        let wide = w >= 300.0;
        let detail_h = if wide { 0.0 } else { 74.0 };
        let list_w = if wide { (w * 0.52).round() } else { w - 16.0 };
        let search_y = y + 24.0 + detail_h;
        let list_y = search_y + 20.0;
        let row_h = 14.0;
        let shown = (((y + h - 16.0) - list_y) / row_h).floor().max(1.0) as usize;
        let first = g.dex_sel.saturating_sub(shown / 2).min(n.saturating_sub(shown));
        let on_search = fr.ux > x + 8.0 && fr.ux < x + 8.0 + list_w && fr.uy > search_y && fr.uy < search_y + 15.0;
        let on_list = fr.ux > x + 8.0 && fr.ux < x + 8.0 + list_w && fr.uy > list_y && fr.uy < list_y + shown as f32 * row_h;
        let inside = fr.ux > x && fr.ux < x + w && fr.uy > y && fr.uy < y + h;
        if fr.click && on_search && radio::touch_ui() {
            if let Some(text) = radio::ask_text(if shrooms_mode { "search the mushrooms: a name or a number" } else { "search the fishdex: a name or a number" }, &g.dex_query) {
                g.dex_query = text.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '#').take(20).collect();
                g.dex_sel = 0;
            }
        } else if fr.click && on_list {
            let r = first + ((fr.uy - list_y) / row_h) as usize;
            if r < n {
                g.dex_sel = r;
            }
            g.dex_drag = Some((fr.uy, g.dex_sel));
        } else if fr.click && !inside {
            g.dex_open = false;
        }
        {
            let mut tx = x + 8.0;
            for (k, name) in tab_names.iter().enumerate() {
                let tw = ui_measure(name, 11.0);
                let usable = k >= 2 || g.dex_from_shop;
                if fr.click && usable && fr.ux > tx - 2.0 && fr.ux < tx + tw + 2.0 && fr.uy > y + 4.0 && fr.uy < y + 20.0 {
                    tab_go = Some(k);
                }
                tx += tw + 10.0;
            }
        }
        if let Some(k) = tab_go {
            if k >= 2 {
                if k - 2 != g.dex_mode {
                    g.dex_mode = k - 2;
                    g.dex_sel = 0;
                    g.dex_query.clear();
                }
            } else {
                g.dex_open = false;
                g.shop_open = true;
                g.shop_tab = k;
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            if let Some((start, from)) = g.dex_drag {
                let moved = ((start - fr.uy) / row_h) as i32;
                g.dex_sel = (from as i32 + moved).clamp(0, n.saturating_sub(1) as i32) as usize;
            }
        } else {
            g.dex_drag = None;
        }
        if is_key_pressed(KeyCode::Escape) {
            g.dex_open = false;
        }
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &g.full_view.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.06, 0.14, 0.6));
        ui_rect(x, y, w, h, Color::new(0.08, 0.12, 0.22, 0.96));
        ui_rect(x, y, w, 3.0, Color::from_rgba(110, 190, 255, 255));
        {
            let mut tx = x + 8.0;
            for (k, name) in tab_names.iter().enumerate() {
                let active = k == g.dex_mode + 2;
                let usable = k >= 2 || g.dex_from_shop;
                let col = if active { Color::from_rgba(150, 215, 255, 255) } else if usable { Color::new(1.0, 1.0, 1.0, 0.55) } else { Color::new(1.0, 1.0, 1.0, 0.2) };
                ui_text(name, tx, y + 16.0, 11.0, col);
                if active {
                    ui_rect(tx, y + 18.0, ui_measure(name, 11.0), 1.0, col);
                }
                tx += ui_measure(name, 11.0) + 10.0;
            }
        }
        let count = format!("{}/{} {}", caught, total, verb);
        ui_text(&count, x + w - 8.0 - ui_measure(&count, 11.0), y + 16.0, 11.0, WHITE);
        ui_rect(x + 8.0, search_y, list_w, 15.0, Color::new(1.0, 1.0, 1.0, if on_search { 0.16 } else { 0.1 }));
        let placeholder = if radio::touch_ui() { "tap to search a name or #number" } else { "type a name or #number" };
        let cursor = if (fr.time * 2.0).fract() < 0.5 && !radio::touch_ui() { "_" } else { "" };
        if g.dex_query.is_empty() {
            ui_text(placeholder, x + 13.0, search_y + 11.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.45));
        } else {
            ui_text(&format!("{}{}", g.dex_query, cursor), x + 13.0, search_y + 11.0, 11.0, WHITE);
        }
        if n == 0 {
            ui_text(if shrooms_mode { "no mushrooms found" } else { "no fish found" }, x + 13.0, list_y + 11.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.6));
        }
        for r in first..(first + shown).min(n) {
            let i = found[r];
            let known = counts[i] > 0;
            let ry = list_y + (r - first) as f32 * row_h;
            if r == g.dex_sel {
                ui_rect(x + 6.0, ry - 1.0, list_w + 4.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
            }
            ui_text(&code_of(i), x + 10.0, ry + 9.5, 9.0, Color::new(0.75, 0.85, 1.0, 0.8));
            draw_fish_art(art_of(i), x + 34.0 + if shrooms_mode { 2.0 } else { 0.0 }, ry + if shrooms_mode { 0.0 } else { 1.0 }, if shrooms_mode { 1.05 } else { 0.95 }, known);
            let name = if known { name_of(i) } else { "???" };
            ui_text(name, x + 54.0, ry + 9.5, 10.0, if known { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) });
            if known {
                let c = format!("x{}", counts[i]);
                ui_text(&c, x + 6.0 + list_w - ui_measure(&c, 9.0), ry + 9.5, 9.0, fish::RARITY_COLORS[rarity_of(i)]);
            }
        }
        if let Some(&i) = found.get(g.dex_sel) {
            let known = counts[i] > 0;
            let (dx, dy, dw) = if wide { (x + list_w + 18.0, y + 30.0, w - list_w - 26.0) } else { (x + 8.0, y + 24.0, w - 16.0) };
            let px = if wide { (dw / art_w as f32).min(5.5) } else { 3.6 };
            let art_w = px * art_w as f32;
            let art_x = if wide { dx + (dw - art_w) / 2.0 } else { dx };
            ui_rect(art_x - 4.0, dy - 2.0, art_w + 8.0, px * art_h as f32 + 6.0, Color::new(0.3, 0.55, 0.8, 0.25));
            draw_fish_art(art_of(i), art_x, dy + 1.0, px, known);
            let tx = if wide { dx } else { dx + art_w + 12.0 };
            let mut ty = if wide { dy + px * art_h as f32 + 18.0 } else { dy + 10.0 };
            let rarity = rarity_of(i);
            ui_text(&code_of(i), tx, ty, 11.0, Color::new(0.75, 0.85, 1.0, 0.9));
            ty += 14.0;
            ui_text(if known { name_of(i) } else { "???" }, tx, ty, 13.0, WHITE);
            ty += 13.0;
            ui_text(fish::RARITY_NAMES[rarity], tx, ty, 10.0, fish::RARITY_COLORS[rarity]);
            ty += 12.0;
            let line = if known { format!("{} {} - {} acorns", verb, counts[i], acorns_of(i)) } else { format!("not {} yet", verb) };
            ui_text(&line, tx, ty, 10.0, Color::new(1.0, 1.0, 1.0, 0.75));
            ty += 12.0;
            ui_text(hint_of(i), tx, ty, 9.0, Color::new(0.8, 0.9, 1.0, 0.6));
        }
        let help = if radio::touch_ui() { "tap a row - tap the tabs - tap outside to close" } else { "type to search - up/down - left/right tabs - esc to close" };
        ui_text(help, x + (w - ui_measure(help, 9.0)) / 2.0, y + h - 5.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
}

pub(crate) fn pause_menu(g: &mut Game, fr: &mut Frame) {
    use steps::venues::{button, hit};
    let w = 300f32.min(low_w() - 10.0);
    let h = 206f32.min(low_h() - 10.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = if radio::touch_ui() && low_h() > low_w() { (low_h() * 0.12).round() } else { ((low_h() - h) / 2.0).round() };
    let inside = hit(fr, x, y, w, h);
    let tabs = ["general", "audio", "controls", "graphics", "account"];
    let nav_w = 78f32.min(w * 0.3);
    let (cx, cy, cw) = (x + nav_w + 16.0, y + 30.0, w - nav_w - 24.0);
    let noun = g.cast_list[g.players[1].character].noun;
    let sing_here = radio::karaoke_here();
    let keys = !fr.just_paused;
    let tab = g.venue.settings_tab.min(4);
    if keys && is_key_pressed(KeyCode::Tab) {
        g.venue.settings_tab = (tab + 1) % 5;
    }
    if keys && (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)) {
        g.venue.settings_tab = (tab + 1).min(4);
    }
    if keys && (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) {
        g.venue.settings_tab = tab.saturating_sub(1);
    }
    let tab = g.venue.settings_tab.min(4);

    crate::glow::present(&g.full_view);
    ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.06, 0.04, 0.14, 0.55));
    ui_round(x + 2.0, y + 3.0, w, h, 9.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 9.0, ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG);
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::PI / 4.0;
        ui_circle(x + 14.5 + a.cos() * 4.6, y + 12.5 + a.sin() * 4.6, 1.6, ui::skin::PINK);
    }
    ui_circle(x + 14.5, y + 12.5, 4.2, ui::skin::PINK);
    ui_circle(x + 14.5, y + 12.5, 1.8, ui::skin::PANEL_SOLID);
    ui_text_as(ui::Face::Display, "settings", x + 25.0, y + 17.0, 11.0, ui::skin::GOLD);
    let ver = format!("v{}", update::VERSION);
    ui_text(&ver, x + w - 28.0 - ui_measure(&ver, 7.0), y + 14.0, 7.0, ui::skin::MUTED);
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && keys;
    ui_round(x + 6.0, y + 26.0, nav_w + 4.0, h - 44.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.14));
    for (i, t) in tabs.iter().enumerate() {
        let ty = y + 30.0 + i as f32 * 20.0;
        let on = tab == i;
        let over = hit(fr, x + 8.0, ty, nav_w, 17.0);
        if on {
            ui_round_frame(x + 8.0, ty, nav_w, 17.0, 5.0, ui::skin::PINK, ui::skin::PINK_EDGE);
        } else if over {
            ui_round(x + 8.0, ty, nav_w, 17.0, 5.0, Color::new(1.0, 1.0, 1.0, 0.08));
        }
        ui_text_as(ui::Face::Label, t, x + 16.0, ty + 12.0, 9.0, if on { ui::skin::INK } else { ui::skin::CREAM });
        if over && fr.click && keys {
            g.venue.settings_tab = i;
            fr.click = false;
        }
    }
    let tab = g.venue.settings_tab.min(4);
    ui_text_as(ui::Face::Label, tabs[tab], cx, cy + 4.0, 10.0, ui::skin::CREAM);
    let key = |k: &str| if radio::touch_ui() { String::new() } else { format!(" ({k})") };
    let toggle = |fr: &Frame, label: &str, note: &str, ry: f32, on: bool| -> bool {
        ui_text_fit(label, cx, ry + 9.0, 9.0, cw - 34.0, ui::skin::CREAM);
        if !note.is_empty() {
            ui_text_fit(note, cx, ry + 18.0, 7.0, cw - 34.0, ui::skin::MUTED);
        }
        let (tx, ty) = (cx + cw - 26.0, ry + 2.0);
        ui_round(tx, ty, 24.0, 12.0, 6.0, if on { ui::skin::PINK } else { Color::new(1.0, 1.0, 1.0, 0.16) });
        ui_circle(if on { tx + 18.0 } else { tx + 6.0 }, ty + 6.0, 4.6, if on { ui::skin::CREAM } else { ui::skin::MUTED });
        hit(fr, cx, ry, cw, 18.0) && fr.click
    };
    let clicked_toggle = |c: bool| c && keys;
    let mut open_dex = keys && is_key_pressed(KeyCode::F);
    let mut flip_dj = keys && is_key_pressed(KeyCode::J);
    let mut flip_karaoke = keys && sing_here && is_key_pressed(KeyCode::K);
    let mut flip_companion = keys && is_key_pressed(KeyCode::C);
    let mut open_homes = keys && is_key_pressed(KeyCode::O);
    let mut open_account = keys && is_key_pressed(KeyCode::A);
    let mut login = keys && is_key_pressed(KeyCode::L);
    let mut restart = keys && is_key_pressed(KeyCode::R);
    let mut open_help = false;
    g.restart_confirm = (g.restart_confirm - get_frame_time()).max(0.0);
    let mut ry = cy + 12.0;
    match tab {
        0 => {
            if clicked_toggle(toggle(fr, &format!("companion{}", key("C")), &format!("the {noun} plays, fishes and chats with you"), ry, g.companion)) {
                flip_companion = true;
            }
            ry += 26.0;
            if clicked_toggle(toggle(fr, &format!("dj set{}", key("J")), "show the tracks on your serato decks", ry, radio::flag(0))) {
                flip_dj = true;
            }
            ry += 26.0;
            if sing_here {
                if clicked_toggle(toggle(fr, &format!("karaoke{}", key("K")), "lyrics from your serato bridge", ry, radio::flag(1))) {
                    flip_karaoke = true;
                }
                ry += 26.0;
            }
            let bw = (cw - 6.0) / 2.0;
            if button(fr, &format!("fishdex{}", key("F")), cx, ry + 4.0, bw, 16.0, false) && keys {
                open_dex = true;
            }
            let guests = g.venue.visits.peers.len();
            let homes = if steps::visits::visiting(g) { "visiting".to_string() } else if guests > 0 { format!("{guests} visiting") } else { "friends' homes".to_string() };
            if button(fr, &format!("{homes}{}", key("O")), cx + bw + 6.0, ry + 4.0, bw, 16.0, false) && keys {
                open_homes = true;
            }
        }
        1 => {
            let labels: [String; 5] = if g.two_player {
                [g.cast_list[g.players[0].character].name.to_string(), g.cast_list[g.players[1].character].name.to_string(), "beat & pot".to_string(), "world".to_string(), "master".to_string()]
            } else {
                ["you".to_string(), format!("the {noun}"), "beat & pot".to_string(), "world".to_string(), "master".to_string()]
            };
            let label_w = 52.0;
            let bar_x = cx + label_w;
            let bar_w = cw - label_w - 22.0;
            let row_y = |r: usize| ry + 8.0 + r as f32 * 24.0;
            if is_mouse_button_down(MouseButton::Left) && keys {
                if g.mix_drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
                    for r in 0..5 {
                        if (fr.uy - row_y(r)).abs() < 10.0 && fr.ux > bar_x - 8.0 && fr.ux < bar_x + bar_w + 8.0 {
                            g.mix_drag = Some(r);
                        }
                    }
                }
                if let Some(r) = g.mix_drag {
                    g.mix[r] = ((fr.ux - bar_x) / bar_w).clamp(0.0, 1.0);
                    g.audio.set_mix(g.mix);
                }
            } else {
                g.mix_drag = None;
            }
            for r in 0..5 {
                let yy = row_y(r);
                ui_text_fit(&labels[r], cx, yy + 3.0, 8.0, label_w - 4.0, ui::skin::CREAM);
                ui_round(bar_x, yy - 2.0, bar_w, 4.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.14));
                let fill = if r == 4 { ui::skin::GOLD } else { note_color([0, 2, 4, 3][r]) };
                ui_round(bar_x, yy - 2.0, (bar_w * g.mix[r]).max(4.0), 4.0, 2.0, fill);
                ui_circle(bar_x + bar_w * g.mix[r], yy, 4.4, if g.mix_drag == Some(r) { WHITE } else { ui::skin::CREAM });
                ui_text_as(ui::Face::Data, &format!("{:.0}", g.mix[r] * 100.0), bar_x + bar_w + 6.0, yy + 3.0, 8.0, ui::skin::MUTED);
            }
        }
        2 => {
            let lines = [("wasd", "move"), ("1-6", "sing a note"), ("e", "act, sit, fish, cook"), ("x", "move furniture"), ("m", "your bag"), ("enter", "chat"), ("tab", "music or spells"), ("r / c", "record / clear the loop"), ("p", "two players"), ("esc", "close, or pause")];
            for (i, (k, what)) in lines.iter().enumerate() {
                let yy = ry + 4.0 + i as f32 * 12.5;
                if yy > y + h - 40.0 {
                    break;
                }
                let kw = ui_measure_as(ui::Face::Label, k, 7.0) + 8.0;
                ui_round(cx, yy - 1.0, kw, 11.0, 3.0, ui::skin::TAB);
                ui_text_as(ui::Face::Label, k, cx + 4.0, yy + 7.5, 7.0, ui::skin::CREAM);
                ui_text(what, cx + 46.0, yy + 7.5, 8.0, ui::skin::MUTED);
            }
            if button(fr, &format!("every key{}", key("H")), cx + cw - 70.0, y + h - 36.0, 70.0, 15.0, false) && keys {
                open_help = true;
            }
        }
        3 => {
            if clicked_toggle(toggle(fr, "high resolution", "off: the old chunky pixel look", ry, !crate::render::pixel_look())) {
                crate::render::set_pixel_look(!crate::render::pixel_look());
                g.save_clock = 99.0;
            }
            ry += 26.0;
            if clicked_toggle(toggle(fr, "soft glow", "lamps, fire and neon glow a little", ry, crate::glow::enabled())) {
                crate::glow::set_enabled(!crate::glow::enabled());
                g.save_clock = 99.0;
            }
        }
        _ => {
            let name = if g.venue.name.is_empty() { "-".to_string() } else { g.venue.name.clone() };
            ui_text("name", cx, ry + 9.0, 8.0, ui::skin::MUTED);
            ui_text_fit(&name, cx + 40.0, ry + 9.0, 9.0, cw - 40.0, ui::skin::CREAM);
            ry += 16.0;
            let acc = match &g.venue.account {
                Some((_, label)) => label.clone(),
                None => "sign in to save".to_string(),
            };
            if button(fr, &format!("{acc}{}", key("A")), cx, ry, cw, 16.0, false) && keys {
                open_account = true;
            }
            ry += 20.0;
            let keyed = g.venue.admin || g.venue.dj_name.is_some() || !g.venue.dj_key.is_empty();
            if button(fr, &format!("{}{}", if keyed { "log out" } else { "dj or admin key: login" }, key("L")), cx, ry, cw, 16.0, keyed) && keys {
                login = true;
            }
            ry += 26.0;
            let armed = g.restart_confirm > 0.0;
            let label = if armed { if radio::touch_ui() { "tap again: erase everything" } else { "R again: erase everything" } } else if radio::touch_ui() { "restart" } else { "restart (R)" };
            let over = hit(fr, cx, ry, cw, 16.0);
            ui_round_frame(cx, ry, cw, 16.0, 5.0, if armed { Color::from_rgba(220, 70, 80, 255) } else if over { Color::new(0.5, 0.2, 0.3, 1.0) } else { Color::new(0.33, 0.14, 0.22, 1.0) }, Color::from_rgba(255, 120, 130, 255));
            let lw = ui_measure_as(ui::Face::Label, label, 9.0);
            ui_text_as(ui::Face::Label, label, cx + (cw - lw) / 2.0, ry + 11.5, 9.0, ui::skin::CREAM);
            if over && fr.click && keys {
                restart = true;
            }
            ui_text("restart erases your world, home and save", cx, ry + 26.0, 7.0, ui::skin::MUTED);
        }
    }
    let hint = if radio::touch_ui() { "tap outside to play" } else { "tab or w/s: sections   esc: play" };
    ui_text(hint, x + (w - ui_measure(hint, 7.0)) / 2.0, y + h - 7.0, 7.0, ui::skin::MUTED);

    if open_dex {
        g.paused = false;
        g.back_to_pause = true;
        g.dex_open = true;
        g.dex_from_shop = false;
        g.dex_query.clear();
        g.dex_sel = 0;
        while get_char_pressed().is_some() {}
        fr.click = false;
    }
    if flip_dj {
        radio::set_flag(0, !radio::flag(0));
        fr.click = false;
    }
    if flip_karaoke {
        radio::set_flag(1, !radio::flag(1));
        fr.click = false;
    }
    if flip_companion {
        g.companion = !g.companion;
        g.save_clock = 99.0;
        let line = if g.companion { format!("the {noun} is back") } else { format!("the {noun} is resting. you play on your own") };
        g.toasts.push((line, 3.0));
        fr.click = false;
    }
    if restart {
        if g.restart_confirm > 0.0 {
            g.restart_now = true;
            g.paused = false;
        } else {
            g.restart_confirm = 3.0;
        }
        fr.click = false;
    }
    if open_account {
        g.paused = false;
        g.restart_confirm = 0.0;
        g.venue.overlay = venues::Overlay::Account;
        g.venue.key_draft.clear();
        g.venue.acct_key = false;
        while get_char_pressed().is_some() {}
        g.back_to_pause = true;
        fr.click = false;
    }
    if login {
        let keyed = g.venue.admin || g.venue.dj_name.is_some() || !g.venue.dj_key.is_empty();
        if keyed {
            steps::venues::log_out(g);
        } else {
            g.paused = false;
            g.restart_confirm = 0.0;
            g.venue.overlay = venues::Overlay::Account;
            g.venue.key_draft.clear();
            g.venue.acct_key = true;
            g.venue.opened_at = get_time();
            while get_char_pressed().is_some() {}
            g.back_to_pause = true;
        }
        fr.click = false;
    }
    if open_homes {
        g.paused = false;
        g.restart_confirm = 0.0;
        steps::venues::open_homes(g);
        g.back_to_pause = true;
        fr.click = false;
    }
    if open_help {
        g.paused = false;
        g.show_help = true;
        fr.click = false;
    }
    if keys && (close || (fr.click && !inside && g.mix_drag.is_none()) || is_key_pressed(KeyCode::Escape)) {
        g.paused = false;
        g.restart_confirm = 0.0;
        g.save_clock = 99.0;
    }
}
