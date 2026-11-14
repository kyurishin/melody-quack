#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn help_and_escape(g: &mut Game, fr: &mut Frame) {
    fr.just_paused = false;
    fr.help_closed = false;
    if g.back_to_pause && !g.dex_open && g.venue.overlay == venues::Overlay::None {
        g.back_to_pause = false;
        g.paused = true;
        fr.just_paused = true;
        return;
    }
    if g.show_help && !g.paused {
        let (hx, hy, hw, hh) = help_box();
        let tab_hit = (0..3).find(|k| fr.click && fr.uy > hy + 20.0 && fr.uy < hy + 36.0 && fr.ux > hx + 10.0 + *k as f32 * (hw - 20.0) / 3.0 && fr.ux < hx + 10.0 + (*k + 1) as f32 * (hw - 20.0) / 3.0);
        if let Some(k) = tab_hit {
            g.help_page = k;
            fr.click = false;
        } else if is_key_pressed(KeyCode::Escape) || (fr.click && !(fr.ux > hx && fr.ux < hx + hw && fr.uy > hy && fr.uy < hy + hh)) {
            g.show_help = false;
            fr.help_closed = true;
            fr.click = false;
        } else if fr.click {
            fr.click = false;
        }
        if is_key_pressed(KeyCode::Right) {
            g.help_page = (g.help_page + 1) % 3;
        }
        if is_key_pressed(KeyCode::Left) {
            g.help_page = (g.help_page + 2) % 3;
        }
    }
    if g.sampler.open && !g.paused && is_key_pressed(KeyCode::Escape) {
        g.sampler.open = false;
        fr.help_closed = true;
    }
    if (g.ddr.on[0] || g.ddr.on[1]) && !g.paused && is_key_pressed(KeyCode::Escape) {
        g.ddr.on = [false; 2];
        fr.help_closed = true;
    }
    if !g.paused && !fr.help_closed && is_key_pressed(KeyCode::Escape) {
        let mut closed = true;
        if g.emoji_choose[0].is_some() || g.emoji_choose[1].is_some() {
            g.emoji_choose = [None, None];
        } else if g.shop_open {
            g.shop_open = false;
        } else if g.dj[0] {
            g.dj[0] = false;
        } else if g.show_inventory {
            g.show_inventory = false;
        } else {
            closed = false;
        }
        if closed {
            fr.help_closed = true;
        }
    }
    if !g.paused && !g.picking && !fr.help_closed {
        let tap_ok = if radio::touch_ui() && !g.two_player {
            if low_h() > low_w() { fr.uy < low_h() * 0.55 && fr.uy > 20.0 } else { fr.ux > low_w() * 0.235 && fr.ux < low_w() * 0.765 && fr.uy > 20.0 }
        } else {
            true
        };
        if (fr.click && tap_ok) || is_key_pressed(KeyCode::Escape) {
            g.paused = true;
            fr.just_paused = true;
            g.mix_drag = None;
        }
    }
}

pub(crate) fn hotkeys(g: &mut Game, fr: &mut Frame) {
    if is_key_pressed(KeyCode::P) {
        g.two_player = !g.two_player;
        g.brain = DogBrain::new();
        g.players[1].seated = false;
        g.players[1].sleeping = false;
        g.seat[1] = None;
        g.dj[1] = false;
        g.players[1].carrying = None;
        if !g.two_player {
            if g.shop_open && g.shop_owner == 1 {
                g.shop_owner = 0;
            }
            if g.holding.is_some() && g.holding_by == 1 {
                g.holding_by = 0;
            }
            if g.fishing.active && g.fishing.caster == 1 {
                g.fishing.active = false;
                g.fish_card = 0.0;
                g.toasts.push(("the fishing line was reeled in".to_string(), 1.5));
            }
        }
        if g.two_player {
            let start = if g.inside {
                home::world(g.house.free_spot_near(home::local(g.players[0].pos) + vec2(1.0, 0.0)))
            } else {
                rescue(&g.world, g.players[0].pos + vec2(-1.0, 0.0))
            };
            g.players[1].place_at(start);
        }
    }
    if is_key_pressed(KeyCode::Tab) {
        g.music_mode = !g.music_mode;
    }
    if is_key_pressed(KeyCode::R) {
        g.looper.recording = !g.looper.recording;
    }
    fr.shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    if is_key_pressed(KeyCode::C) && fr.shift_held {
        g.songbook.clear();
        g.partner_xp = 0;
        g.partner_phrase.clear();
        g.partner_soloing = false;
        g.toasts.push((format!("the {} forgot every tune - starting from scratch", g.cast_list[g.players[1].character].noun), 3.0));
    } else if is_key_pressed(KeyCode::C) || (g.two_player && is_key_pressed(KeyCode::Y)) {
        let only = if !g.two_player { None } else if is_key_pressed(KeyCode::C) { Some(0) } else { Some(1) };
        match only {
            None => {
                g.looper.notes.clear();
                g.looper.skip.clear();
            }
            Some(who) => {
                g.looper.notes.retain(|&(_, id)| note_owner(id) != who);
                g.looper.skip.retain(|&(_, id)| note_owner(id) != who);
                g.looper.pending.retain(|&(_, id)| note_owner(id) != who);
                g.toasts.push((format!("player {} cleared their loop", who + 1), 1.5));
            }
        }
    }
    if is_key_pressed(KeyCode::B) {
        g.looper.beat = !g.looper.beat;
    }
    if is_key_pressed(KeyCode::H) {
        g.show_help = !g.show_help;
    }
}

pub(crate) fn bag(g: &mut Game, fr: &mut Frame) {
    fr.bag_keys = [is_key_pressed(KeyCode::M), g.two_player && is_key_pressed(KeyCode::Backspace)];
    for who in 0..2 {
        if !fr.bag_keys[who] || g.picking {
            continue;
        }
        if g.inside && g.holding.is_some() && g.holding_by == who {
            if let Some(piece) = g.holding.take() {
                g.stored.push(piece.item);
                g.audio.play_ch(3, &g.plink, 0.35);
                g.toasts.push((format!("the {} goes into your bag", piece.item.name()), 2.0));
                g.save_clock = 99.0;
            }
        } else if g.show_inventory && g.bag_owner != who {
            g.bag_owner = who;
        } else {
            g.show_inventory = !g.show_inventory;
            g.bag_owner = who;
            g.bag_sel = 0;
            g.bag_clear_armed = 0.0;
        }
    }
    g.bag_clear_armed = (g.bag_clear_armed - fr.dt).max(0.0);
    fr.bag_busy = g.show_inventory && g.inside && !g.shop_open;
    if fr.bag_busy && g.holding.is_none() {
        let owner = g.bag_owner;
        let (up, down, act, clear) = if owner == 0 {
            (is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up), is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down), is_key_pressed(KeyCode::E), is_key_pressed(KeyCode::X))
        } else {
            (is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::O), is_key_pressed(KeyCode::U))
        };
        if up {
            g.bag_sel = g.bag_sel.saturating_sub(1);
        }
        if down && g.bag_sel + 1 < g.stored.len() {
            g.bag_sel += 1;
        }
        if act && g.bag_sel < g.stored.len() {
            let item = g.stored.remove(g.bag_sel);
            let ahead = ahead_local(&g.players[owner]);
            g.holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
            g.holding_by = owner;
            g.show_inventory = false;
            g.bag_sel = g.bag_sel.min(g.stored.len().saturating_sub(1));
            g.toasts.push((format!("took the {} out - {} to place it", item.name(), if owner == 0 { "E" } else { "O" }), 2.5));
        }
        if clear && !g.house.pieces.is_empty() {
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
                g.toasts.push((format!("packed away all {} pieces into your bag", n), 2.5));
            } else {
                g.bag_clear_armed = 3.0;
            }
        }
    }
}

pub(crate) fn tape_and_fresh_start(g: &mut Game, fr: &mut Frame) {
    if is_key_pressed(KeyCode::T) {
        if g.audio.tape_seconds().is_some() {
            let msg = match g.audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
                Some(place) => format!("jam saved: {place}"),
                None => "could not save the jam".to_string(),
            };
            g.toasts.push((msg, 4.0));
        } else {
            g.audio.start_tape();
            g.toasts.push(("tape rolling - T again to stop and save".to_string(), 2.5));
        }
    }
    if g.audio.tape_seconds().map_or(false, |t| t > TAPE_LIMIT) {
        if let Some(place) = g.audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
            g.toasts.push((format!("tape full (5 min) - jam saved: {place}"), 4.0));
        }
    }
    g.fresh_confirm = (g.fresh_confirm - get_frame_time()).max(0.0);
    fr.start_fresh = false;
    if is_key_pressed(KeyCode::N) && !g.picking {
        if g.fresh_confirm > 0.0 {
            fr.start_fresh = true;
        } else {
            g.fresh_confirm = 3.0;
            g.toasts.push(("press N again to start a fresh world".to_string(), 3.0));
        }
    }
    if fr.start_fresh {
        store::clear();
        g.world = World::new();
        venues::stamp(&mut g.world);
        g.inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
        g.camp = Camp { fuel: 0.0, lit: false, pile: 0 };
        g.dishes.clear();
        g.cook_progress = 0;
        g.cooking = false;
        g.fishing.active = false;
        g.fish_lesson = false;
        g.cook_lesson = false;
        g.gather_lesson = false;
        g.fish_card = 0.0;
        g.cook_card = 0.0;
        g.gather_card = 0.0;
        g.looper.notes.clear();
        g.looper.skip.clear();
        g.songbook.clear();
        g.partner_xp = 0;
        g.house = home::Home::new();
        g.dj = [false, false];
        g.stick_trees.clear();
        g.diary.clear();
        g.playback = None;
        g.inside = false;
        g.holding = None;
        g.shop_open = false;
        g.seat = [None, None];
        g.brain = DogBrain::new();
        for (who, spot) in [vec2(6.5, 16.5), vec2(7.5, 16.5)].into_iter().enumerate() {
            g.players[who].place_at(spot);
            g.players[who].seated = false;
            g.players[who].carrying = None;
        }
        g.fresh_confirm = 0.0;
        g.picking = true;
        g.first_pick = true;
        g.first_steps = 0;
        g.pick_row = 0;
    }
}
