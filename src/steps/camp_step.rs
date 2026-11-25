#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn jam_and_fire(g: &mut Game, fr: &mut Frame) {
    for k in g.camp.glow.iter_mut() {
        *k = (*k - fr.dt * 2.5).max(0.0);
    }
    for who in 0..2 {
        if g.camp.play[who] && (!g.players[who].seated || g.inside || g.camp.ring == 0 || g.cooking) {
            g.camp.play[who] = false;
        }
    }
    let free = !g.inside && !g.picking && !g.paused && !g.shop_open && !g.show_help && !g.show_inventory && g.venue.overlay == crate::venues::Overlay::None && g.venue.at == crate::venues::Venue::None && !steps::venues::typing(g);
    for who in 0..(if g.two_player { 2 } else { 1 }) {
        let key = if who == 0 { KeyCode::X } else { KeyCode::U };
        if !free || !is_key_pressed(key) || !near_fire(&g.players[who]) {
            continue;
        }
        if g.players[who].seated && g.camp.ring > 0 && !g.cooking {
            g.camp.play[who] = !g.camp.play[who];
            g.music_mode = true;
            g.toasts.push((if g.camp.play[who] { "you tap the ring of stones - 1-6 play them".to_string() } else { "back to your own voice".to_string() }, 2.0));
        } else if !g.players[who].seated && !ring_full(&g.camp) && g.inv.stone > 0 {
            g.inv.stone -= 1;
            steps::friend::set_ring_stone(g);
        } else if !g.players[who].seated && !ring_full(&g.camp) {
            g.toasts.push(("find a stone and bring it to the fire".to_string(), 2.0));
        }
    }
    fr.was_jam = g.jam;
    g.jam = !g.inside && (g.players[0].seated || (g.two_player && g.players[1].seated));
    if g.jam && !fr.was_jam {
        g.music_mode = true;
        if g.first_steps == 2 {
            g.first_steps = 3;
            g.toasts.push(("play together to earn your first star".to_string(), 3.0));
        }
    }

    if g.camp.lit {
        g.camp.fuel -= fr.dt * if ring_full(&g.camp) { 0.6 } else { 1.0 };
        g.camp.pile = (g.camp.fuel / STICK_FUEL).ceil().max(0.0) as u32;
        if g.camp.fuel <= 0.0 {
            g.camp.fuel = 0.0;
            g.camp.lit = false;
            g.cooking = false;
            g.toasts.push(("the fire went out".to_string(), 3.0));
            for p in g.players.iter_mut() {
                p.seated = false;
            }
        }
        g.crackle_clock -= fr.dt;
        if g.crackle_clock <= 0.0 {
            g.crackle_clock = 0.6 + hash((fr.time * 10.0) as i32, 3, 9) * 1.2;
            let dist = g.players[0].pos.distance(FIRE_SPOT);
            let vol = (1.0 - dist / 7.0).max(0.0) * 0.35;
            if vol > 0.01 {
                g.audio.play_ch(3, &g.crackle, vol);
            }
        }
        if hash((fr.time * 60.0) as i32, 5, 1) > 0.7 {
            let c = fire_center();
            spawn(&mut g.particles, c + vec3(hash((fr.time * 97.0) as i32, 1, 2) * 0.4 - 0.2, 0.4, 0.1), vec3(hash((fr.time * 31.0) as i32, 2, 2) * 0.6 - 0.3, 1.6, 0.0), 0.9, Color::from_rgba(255, 180, 70, 255), 0.022, 0.05);
        }
    }
    if g.cooking && hash((fr.time * 40.0) as i32, 9, 2) > 0.8 {
        let c = fire_center();
        spawn(&mut g.particles, c + vec3(hash((fr.time * 53.0) as i32, 1, 3) * 0.3 - 0.15, 0.62, 0.06), vec3(0.0, 0.5, 0.0), 1.6, Color::from_rgba(240, 240, 245, 255), 0.05, -0.03);
    }
    if hash((fr.time * 30.0) as i32, 8, 1) > 0.85 {
        let (_, z0, x1, _) = COTTAGE;
        spawn(&mut g.particles, vec3(x1 as f32 - 0.6, 2.7, z0 as f32 + 0.6), vec3(0.15, 0.5, 0.0), 2.6, Color::from_rgba(220, 220, 228, 255), 0.06, -0.02);
    }
}

thread_local! {
    static CHOICE: std::cell::RefCell<Vec<(u8, (f32, f32, f32, f32))>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn choice_shown(g: &Game) -> bool {
    !g.inside && !g.picking && !g.paused && !g.show_inventory && !g.shop_open && !g.show_help && !g.two_player && g.venue.overlay == crate::venues::Overlay::None && g.venue.at == crate::venues::Venue::None && g.players[0].seated && g.camp.lit && near_fire(&g.players[0]) && !g.cooking
}

pub(crate) fn draw_choice(g: &Game, hint_top: f32) {
    let mut rects = Vec::new();
    if choice_shown(g) {
        let stones = g.camp.ring > 0;
        let playing = g.camp.play[0];
        let second = if !stones { format!("stones: build the ring first ({}/{})", g.camp.ring, RING_MAX) } else if playing { "my voice".to_string() } else { "play the stones".to_string() };
        let labels = [("cook", true), (second.as_str(), stones)];
        let widths: Vec<f32> = labels.iter().map(|(l, _)| ui_measure_as(ui::Face::Label, l, 9.0) + 22.0).collect();
        let total = widths.iter().sum::<f32>() + 6.0;
        let mut x = (low_w() - total) / 2.0;
        let y = hint_top - 22.0;
        for (k, ((label, on), w)) in labels.iter().zip(widths.iter()).enumerate() {
            let active = k == 1 && playing;
            let (fill, edge, ink) = if !on {
                (Color::new(1.0, 1.0, 1.0, 0.08), ui::skin::EDGE, ui::skin::MUTED)
            } else if k == 0 {
                (Color::from_rgba(255, 214, 110, 255), Color::from_rgba(200, 150, 60, 255), ui::skin::INK)
            } else if active {
                (ui::skin::PINK, ui::skin::PINK_EDGE, ui::skin::INK)
            } else {
                (ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG, ui::skin::CREAM)
            };
            ui_round_frame(x, y, *w, 17.0, 6.0, fill, edge);
            if k == 0 {
                ui_round(x + 7.0, y + 7.5, 8.0, 5.0, 2.0, ink);
                ui_rect(x + 6.0, y + 6.5, 10.0, 1.2, ink);
            } else {
                for j in 0..3 {
                    ui_circle(x + 8.0 + j as f32 * 3.0, y + 9.5 - (j % 2) as f32 * 1.5, 1.4, ink);
                }
            }
            ui_text_as(ui::Face::Label, label, x + 19.0, y + 12.0, 9.0, ink);
            if *on {
                rects.push((k as u8, (x, y, *w, 17.0)));
            }
            x += w + 6.0;
        }
    }
    CHOICE.with(|c| *c.borrow_mut() = rects);
}

pub(crate) fn choice_click(g: &mut Game, fr: &mut Frame) -> bool {
    if !fr.click || !choice_shown(g) {
        return false;
    }
    let hit = CHOICE.with(|c| c.borrow().iter().find(|(_, (x, y, w, h))| fr.ux >= *x && fr.ux <= x + w && fr.uy >= *y && fr.uy <= y + h).map(|(k, _)| *k));
    let Some(k) = hit else { return false };
    fr.click = false;
    if k == 0 {
        g.camp.play[0] = false;
        let pot = auto_pot(g.inv.fish, g.inv.mush);
        g.venue.kitchen.pot = pot;
        g.venue.kitchen.cook = 0;
        g.venue.kitchen.row = 0;
        g.venue.overlay = crate::venues::Overlay::Pot;
        g.venue.opened_at = get_time();
    } else {
        g.camp.play[0] = !g.camp.play[0];
        g.music_mode = true;
        g.toasts.push((if g.camp.play[0] { "you tap the ring of stones - 1-6 play them".to_string() } else { "back to your own voice".to_string() }, 2.0));
    }
    true
}
