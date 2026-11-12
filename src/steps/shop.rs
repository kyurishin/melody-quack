#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn draw_shop(g: &mut Game, _fr: &mut Frame) {
    if g.shop_open {
        let rows = home::shop_rows();
        let w = 330f32.min(low_w() - 10.0);
        let x = ((low_w() - w) / 2.0).round();
        let y = if low_h() > low_w() { (low_h() * 0.1).round() } else { ((low_h() - 214.0) / 2.0 - 4.0).round().max(18.0) };
        let h = 214.0f32.min(low_h() - y - 6.0);
        ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.2, 0.94));
        let tab_col = |on: bool| if on { Color::from_rgba(255, 226, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.45) };
        ui_text("catalogue", x + 8.0, y + 16.0, 14.0, tab_col(g.shop_tab == 0));
        let diary_x = x + 14.0 + ui_measure("catalogue", 14.0);
        ui_text("diary", diary_x, y + 16.0, 14.0, tab_col(g.shop_tab == 1));
        let fish_x = diary_x + ui_measure("diary", 14.0) + 6.0;
        ui_text("fishdex", fish_x, y + 16.0, 11.0, tab_col(false));
        ui_text("mushrooms", fish_x + ui_measure("fishdex", 11.0) + 5.0, y + 16.0, 11.0, tab_col(false));
        ui_rect(if g.shop_tab == 0 { x + 8.0 } else { diary_x }, y + 19.0, ui_measure(if g.shop_tab == 0 { "catalogue" } else { "diary" }, 14.0), 1.0, Color::from_rgba(255, 226, 150, 255));
        let wallet = format!("{}", g.inv.acorns);
        ui_rect(x + w - 14.0 - ui_measure(&wallet, 11.0), y + 8.0, 6.0, 6.0, ACORN_COLOR);
        ui_text(&wallet, x + w - 6.0 - ui_measure(&wallet, 11.0), y + 15.0, 11.0, WHITE);
        if g.shop_tab == 0 {
            let row = home::row(g.shop_row);
            let (name, price, extra) = shop_row_info(row, &g.house);
            let preview_x = x + 10.0;
            let preview_y = y + 28.0;
            let preview_w = w - 20.0;
            let preview_h = (preview_w * 150.0 / 400.0).min(118.0).round();
            let (preview_kx, preview_ky) = ui_scale();
            draw_texture_ex(
                &g.catalogue_view.texture,
                preview_x * preview_kx,
                preview_y * preview_ky,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(preview_w * preview_kx, preview_h * preview_ky)),
                    flip_y: true,
                    ..Default::default()
                },
            );
            ui_rect(preview_x, preview_y, preview_w, preview_h, Color::new(1.0, 1.0, 1.0, 0.06));
            let left_key = if g.shop_owner == 1 { "J" } else { "A" };
            let right_key = if g.shop_owner == 1 { "L" } else { "D" };
            let index = format!("{} / {}", g.shop_row + 1, rows);
            ui_text(&format!("{} <", left_key), preview_x + 6.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
            ui_text(&format!("> {}", right_key), preview_x + preview_w - 20.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
            ui_text(&index, preview_x + preview_w - ui_measure(&index, 9.0) - 6.0, preview_y + 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
            let title_y = preview_y + preview_h + 16.0;
            ui_text(&name, x + (w - ui_measure(&name, 15.0)) / 2.0, title_y, 15.0, Color::from_rgba(255, 240, 200, 255));
            let price_col = price.split_whitespace().next().and_then(|v| v.parse::<u32>().ok()).map(|p| if p <= g.inv.acorns { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) }).unwrap_or(Color::new(1.0, 1.0, 1.0, 0.75));
            ui_text(&price, x + (w - ui_measure(&price, 11.0)) / 2.0, title_y + 13.0, 11.0, price_col);
            let about = if extra.is_empty() { "a little room upgrade for your musical home" } else { &extra };
            let lines = ui_wrap(about, 10.0, w - 26.0);
            for (n, line) in lines.iter().take(2).enumerate() {
                ui_text(line, x + (w - ui_measure(line, 10.0)) / 2.0, title_y + 28.0 + n as f32 * 11.0, 10.0, Color::new(0.88, 0.94, 1.0, 0.82));
            }
        } else if g.diary.is_empty() {
            ui_text("no jams yet - play together by the fire", x + 10.0, y + 46.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
            ui_text("30s = 1 star, 1 min = 2, 3 min = 3", x + 10.0, y + 62.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.5));
        } else {
            let shown = 8usize;
            let row_h = 15.0;
            let total: u32 = g.diary.iter().map(|e| e.stars as u32).sum();
            let n = g.diary.len();
            let first = g.diary_row.saturating_sub(shown / 2).min(n.saturating_sub(shown));
            for r in first..(first + shown).min(n) {
                let entry = &g.diary[n - 1 - r];
                let ry = y + 30.0 + (r - first) as f32 * row_h;
                if r == g.diary_row {
                    ui_rect(x + 4.0, ry - 2.0, w - 8.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
                }
                let mins = format!("{}:{:02}", entry.seconds / 60, entry.seconds % 60);
                let who = format!("{} + {}", g.cast_list[entry.who.0.min(g.cast_list.len() - 1)].noun, g.cast_list[entry.who.1.min(g.cast_list.len() - 1)].noun);
                ui_text(&date_label(entry.when), x + 10.0, ry + 9.0, 11.0, WHITE);
                ui_text(&who, x + 56.0, ry + 9.0, 10.0, Color::new(0.85, 0.92, 1.0, 0.85));
                ui_text(&mins, x + w - 52.0 - ui_measure(&mins, 10.0), ry + 9.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
                for k in 0..3u8 {
                    let col = if k < entry.stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.18) };
                    ui_star(x + w - 42.0 + k as f32 * 11.0, ry + 1.0, 8.0, col);
                }
            }
            let line = format!("{} jams, {} stars", n, total);
            ui_text(&line, x + w - 8.0 - ui_measure(&line, 9.0), y + 26.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
        }
        let help = if g.shop_owner == 1 {
            if g.shop_tab == 0 { "I/K tabs - J/L browse - O buy - U close" } else { "I/K tabs - J/L choose - O play - U close" }
        } else if g.shop_tab == 0 {
            "W/S tabs - A/D browse - E buy - X close"
        } else {
            "W/S tabs - A/D choose - E play - X close"
        };
        ui_text(help, x + (w - ui_measure(help, 10.0)) / 2.0, y + h - 6.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
    }
}
