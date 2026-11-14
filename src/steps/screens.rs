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
                if (right_k || tap_right || pick_k || tap_mid) && g.book_spread + 1 < books::SPREADS && g.page_turn <= 0.0 {
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
        let labels: [String; 5] = if g.two_player {
            [
                g.cast_list[g.players[0].character].name.to_string(),
                g.cast_list[g.players[1].character].name.to_string(),
                "beat & pot".to_string(),
                "world".to_string(),
                "master".to_string(),
            ]
        } else {
            [
                "you".to_string(),
                format!("the {}", g.cast_list[g.players[1].character].noun),
                "beat & pot".to_string(),
                "world".to_string(),
                "master".to_string(),
            ]
        };
        let w = 230f32.min(low_w() - 10.0);
        let row_h = 24.0;
        let h = 40.0 + row_h * 5.0 + 98.0;
        let x = ((low_w() - w) / 2.0).round();
        let y = if radio::touch_ui() && low_h() > low_w() { (low_h() * 0.16).round() } else { ((low_h() - h) / 2.0).round() };
        let label_w = 70.0;
        let bar_x = x + 10.0 + label_w;
        let bar_w = w - 20.0 - label_w - 30.0;
        let row_y = |r: usize| y + 40.0 + r as f32 * row_h + row_h / 2.0;
        if is_mouse_button_down(MouseButton::Left) {
            if g.mix_drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
                for r in 0..5 {
                    if (fr.uy - row_y(r)).abs() < row_h / 2.0 && fr.ux > bar_x - 10.0 && fr.ux < bar_x + bar_w + 10.0 {
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
        let inside = fr.ux > x && fr.ux < x + w && fr.uy > y && fr.uy < y + h;
        let (dex_x, btn_y, dex_w, btn_h) = (x + 10.0, y + 40.0 + row_h * 5.0 + 4.0, ((w - 26.0) * 0.42).round(), 20.0);
        let (btn_x, btn_w) = (dex_x + dex_w + 6.0, w - 26.0 - dex_w);
        let on_button = fr.ux > btn_x && fr.ux < btn_x + btn_w && fr.uy > btn_y && fr.uy < btn_y + btn_h;
        let on_dex = fr.ux > dex_x && fr.ux < dex_x + dex_w && fr.uy > btn_y && fr.uy < btn_y + btn_h;
        if !fr.just_paused && ((fr.click && on_dex) || is_key_pressed(KeyCode::F)) {
            g.paused = false;
            g.back_to_pause = true;
            g.dex_open = true;
            g.dex_from_shop = false;
            g.dex_query.clear();
            g.dex_sel = 0;
            while get_char_pressed().is_some() {}
            fr.click = false;
        }
        let live_y = btn_y + btn_h + 4.0;
        let live_w = ((w - 26.0) / 2.0).round();
        let live_x = [dex_x, dex_x + live_w + 6.0];
        let on_live = |i: usize| fr.ux > live_x[i] && fr.ux < live_x[i] + live_w && fr.uy > live_y && fr.uy < live_y + btn_h;
        for (i, key) in [KeyCode::J, KeyCode::K].into_iter().enumerate() {
            if !fr.just_paused && ((fr.click && on_live(i) && g.mix_drag.is_none()) || is_key_pressed(key)) {
                radio::set_flag(i as i32, !radio::flag(i as i32));
                fr.click = false;
            }
        }
        g.restart_confirm = (g.restart_confirm - get_frame_time()).max(0.0);
        if !fr.just_paused && ((fr.click && on_button && g.mix_drag.is_none()) || is_key_pressed(KeyCode::R)) {
            if g.restart_confirm > 0.0 {
                g.restart_now = true;
                g.paused = false;
            } else {
                g.restart_confirm = 3.0;
            }
        }
        if !fr.just_paused && ((fr.click && !inside && g.mix_drag.is_none()) || is_key_pressed(KeyCode::Escape)) {
            g.paused = false;
            g.restart_confirm = 0.0;
            g.save_clock = 99.0;
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
        ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.06, 0.04, 0.14, 0.55));
        ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.92));
        let title = "paused";
        ui_text(title, x + (w - ui_measure(title, 18.0)) / 2.0, y + 22.0, 18.0, Color::from_rgba(255, 226, 150, 255));
        ui_text("mixer", x + 10.0, y + 34.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.55));
        for r in 0..5 {
            let cy = row_y(r);
            let mut name = labels[r].clone();
            while ui_measure(&name, 11.0) > label_w - 4.0 && name.len() > 3 {
                name.pop();
            }
            ui_text(&name, x + 10.0, cy + 4.0, 11.0, WHITE);
            ui_rect(bar_x, cy - 2.0, bar_w, 4.0, Color::new(1.0, 1.0, 1.0, 0.18));
            let fill = if r == 4 { Color::from_rgba(255, 214, 110, 255) } else { note_color([0, 2, 4, 3][r]) };
            ui_rect(bar_x, cy - 2.0, bar_w * g.mix[r], 4.0, fill);
            ui_rect(bar_x + bar_w * g.mix[r] - 4.0, cy - 6.0, 8.0, 12.0, if g.mix_drag == Some(r) { WHITE } else { Color::new(0.95, 0.92, 0.85, 1.0) });
            ui_text(&format!("{:.0}", g.mix[r] * 100.0), bar_x + bar_w + 8.0, cy + 4.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        ui_rect(dex_x, btn_y, dex_w, btn_h, if on_dex { Color::from_rgba(90, 160, 230, 255) } else { Color::from_rgba(60, 110, 170, 255) });
        let dex_label = if radio::touch_ui() { "fishdex" } else { "fishdex (F)" };
        ui_text(dex_label, dex_x + (dex_w - ui_measure(dex_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
        let armed = g.restart_confirm > 0.0;
        ui_rect(btn_x, btn_y, btn_w, btn_h, if armed { Color::from_rgba(220, 70, 80, 255) } else if on_button { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) });
        let btn_label = if armed {
            if radio::touch_ui() { "tap: erase, new game" } else { "R: erase, new game" }
        } else if radio::touch_ui() {
            "restart"
        } else {
            "restart (R)"
        };
        let mut btn_label = btn_label.to_string();
        while ui_measure(&btn_label, 11.0) > btn_w - 6.0 && btn_label.len() > 4 {
            btn_label.pop();
        }
        ui_text(&btn_label, btn_x + (btn_w - ui_measure(&btn_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
        for i in 0..2 {
            let on = radio::flag(i as i32);
            let name = if i == 0 { "dj set" } else { "karaoke" };
            let key = if radio::touch_ui() { "" } else if i == 0 { " (J)" } else { " (K)" };
            let mut label = format!("{}: {}{}", name, if on { "on" } else { "off" }, key);
            while ui_measure(&label, 10.0) > live_w - 6.0 && label.len() > 4 {
                label.pop();
            }
            let fill = if on { Color::from_rgba(255, 214, 110, 255) } else if on_live(i) { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) };
            ui_rect(live_x[i], live_y, live_w, btn_h, fill);
            let ink = if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE };
            ui_text(&label, live_x[i] + (live_w - ui_measure(&label, 10.0)) / 2.0, live_y + 14.0, 10.0, ink);
        }
        let acc_y = live_y + btn_h + 4.0;
        let comp_w = live_w;
        let on_comp = fr.ux > dex_x && fr.ux < dex_x + comp_w && fr.uy > acc_y && fr.uy < acc_y + btn_h;
        let noun = g.cast_list[g.players[1].character].noun;
        let mut comp_label = format!("{}: {}{}", noun, if g.companion { "on" } else { "off" }, if radio::touch_ui() { "" } else { " (C)" });
        while ui_measure(&comp_label, 10.0) > comp_w - 6.0 && comp_label.len() > 4 {
            comp_label.pop();
        }
        let comp_fill = if g.companion { Color::from_rgba(140, 222, 168, 255) } else if on_comp { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) };
        ui_rect(dex_x, acc_y, comp_w, btn_h, comp_fill);
        ui_text(&comp_label, dex_x + (comp_w - ui_measure(&comp_label, 10.0)) / 2.0, acc_y + 14.0, 10.0, if g.companion { Color::from_rgba(36, 26, 71, 255) } else { WHITE });
        if !fr.just_paused && ((fr.click && on_comp && g.mix_drag.is_none()) || is_key_pressed(KeyCode::C)) {
            g.companion = !g.companion;
            g.save_clock = 99.0;
            let line = if g.companion { format!("the {noun} is back") } else { format!("the {noun} is resting. you play on your own") };
            g.toasts.push((line, 3.0));
            fr.click = false;
        }
        let acc_x = dex_x + comp_w + 6.0;
        let acc_w = w - 20.0 - comp_w - 6.0;
        let on_acc = fr.ux > acc_x && fr.ux < acc_x + acc_w && fr.uy > acc_y && fr.uy < acc_y + btn_h;
        let mut acc_label = match &g.venue.account {
            Some((_, label)) => label.clone(),
            None => "sign in to save".to_string(),
        };
        if !radio::touch_ui() {
            acc_label += " (A)";
        }
        while ui_measure(&acc_label, 10.0) > acc_w - 6.0 && acc_label.len() > 4 {
            acc_label.pop();
        }
        ui_rect(acc_x, acc_y, acc_w, btn_h, if on_acc { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) });
        ui_text(&acc_label, acc_x + (acc_w - ui_measure(&acc_label, 10.0)) / 2.0, acc_y + 14.0, 10.0, WHITE);
        if !fr.just_paused && ((fr.click && on_acc && g.mix_drag.is_none()) || is_key_pressed(KeyCode::A)) {
            g.paused = false;
            g.restart_confirm = 0.0;
            g.venue.overlay = venues::Overlay::Account;
            g.venue.key_draft.clear();
            g.venue.acct_key = false;
            while get_char_pressed().is_some() {}
            g.back_to_pause = true;
            fr.click = false;
        }
        let hint = if radio::touch_ui() { "tap outside to play" } else { "click outside or esc to play" };
        ui_text(hint, x + (w - ui_measure(hint, 10.0)) / 2.0, y + h - 8.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
}
