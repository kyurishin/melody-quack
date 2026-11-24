#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use steps::venues::{button, hit};

pub(crate) fn open(g: &mut Game) {
    g.venue.overlay = venues::Overlay::Recipes;
    g.venue.opened_at = get_time();
    if let Some(d) = g.dishes.last() {
        g.venue.recipe_sel = *d;
    }
}

pub(crate) fn panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let n = DISHES.len();
    let w = 300f32.min(low_w() - 12.0);
    let h = 214f32.min(low_h() - 50.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = (44.0f32).max(((low_h() - h) / 2.0).round()).min(low_h() - h - 4.0);
    ui_round(x + 2.0, y + 3.0, w, h, 9.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 9.0, ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG);
    steps::topbar::widget_icon(steps::topbar::RECIPES, x + 9.0, y + 6.0);
    ui_text_as(ui::Face::Display, "recipes", x + 26.0, y + 17.0, 11.0, ui::skin::GOLD);
    let count = format!("{}/{} found", g.dishes.len(), n);
    ui_text(&count, x + w - 28.0 - ui_measure(&count, 8.0), y + 15.0, 8.0, ui::skin::MUTED);
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && !fresh;
    let sel = g.venue.recipe_sel.min(n - 1);
    let known = g.dishes.contains(&sel);
    let (px, py, pw) = (x + 8.0, y + 24.0, w - 16.0);
    let ph = (pw * 150.0 / 400.0).round();
    home_draw::render_dish_preview(&g.recipe_view, sel, known, fr.time);
    set_default_camera();
    let (kx, ky) = ui_scale();
    draw_texture_ex(&g.recipe_view.texture, px * kx, py * ky, WHITE, DrawTextureParams { dest_size: Some(vec2(pw * kx, ph * ky)), flip_y: true, ..Default::default() });
    let prev = button(fr, "<", px + 4.0, py + ph / 2.0 - 8.0, 16.0, 16.0, false) && !fresh;
    let next = button(fr, ">", px + pw - 20.0, py + ph / 2.0 - 8.0, 16.0, 16.0, false) && !fresh;
    let idx = format!("{} / {}", sel + 1, n);
    ui_text(&idx, px + pw - 6.0 - ui_measure(&idx, 8.0), py + 10.0, 8.0, Color::new(1.0, 1.0, 1.0, 0.6));
    let ty = py + ph + 13.0;
    let name = if known { DISHES[sel] } else { "???" };
    ui_text_as(ui::Face::Label, name, x + (w - ui_measure_as(ui::Face::Label, name, 11.0)) / 2.0, ty, 11.0, if known { ui::skin::CREAM } else { ui::skin::MUTED });
    let effect = if known { format!("your voice: {}", crate::flavour::dish(sel).0) } else { "cook it to find out what it does to your voice".to_string() };
    ui_text_fit(&effect, x + 12.0, ty + 13.0, 8.0, w - 24.0, Color::from_rgba(255, 214, 110, 255));
    ui_text_fit(&format!("in the pot: {}", home_draw::recipe_hint(sel)), x + 12.0, ty + 24.0, 8.0, w - 24.0, ui::skin::MUTED);
    let have = g.meals.get(sel).copied().unwrap_or(0);
    let mut eat = false;
    if have > 0 {
        ui_text(&format!("in your bag: {have}"), x + 12.0, ty + 37.0, 8.0, ui::skin::CREAM);
        eat = button(fr, "eat one", x + w - 66.0, ty + 28.0, 54.0, 14.0, true) && !fresh;
    }
    let keys = if radio::touch_ui() { "tap < > to browse" } else { "A/D browse   E eat   esc: close" };
    ui_text(keys, x + (w - ui_measure(keys, 7.0)) / 2.0, y + h - 6.0, 7.0, ui::skin::MUTED);
    if fresh {
        return;
    }
    if prev || is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        g.venue.recipe_sel = (sel + n - 1) % n;
        fr.click = false;
    }
    if next || is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
        g.venue.recipe_sel = (sel + 1) % n;
        fr.click = false;
    }
    if (eat || (is_key_pressed(KeyCode::E) && get_time() - g.venue.opened_at > 0.2)) && have > 0 {
        crate::flavour::eat(g, sel);
        fr.click = false;
    }
    if close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h)) {
        g.venue.overlay = venues::Overlay::None;
        fr.click = false;
    }
}
