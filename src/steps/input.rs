#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// Frame timing, the first click, emoji pickers and bubbles.
pub(crate) fn begin_frame(g: &mut Game, fr: &mut Frame) {
    fr.dt = get_frame_time().min(0.05);
    fr.time = get_time() as f32;

    fr.click = is_mouse_button_pressed(MouseButton::Left);
    if fr.click && !g.first_click_done {
        g.first_click_done = true;
        if cfg!(target_arch = "wasm32") {
            fr.click = false;
        }
    }
    (fr.ux, fr.uy) = {
        let (kx, ky) = ui_scale();
        let (mx, my) = mouse_position();
        (mx / kx, my / ky)
    };
    fr.emoji_send = [None, None];
    for who in 0..2 {
        if who == 1 && !g.two_player {
            g.emoji_choose[1] = None;
            continue;
        }
        let pressed = if who == 0 { is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Apostrophe) } else { is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Slash) };
        if pressed && !g.dex_open {
            g.emoji_choose[who] = Some(match g.emoji_choose[who] {
                None => (g.emoji_last[who], 0.0),
                Some((id, _)) => ((id + 1) % emoji::COUNT, 0.0),
            });
        }
        if let Some((id, t)) = g.emoji_choose[who].as_mut() {
            *t += fr.dt;
            let (sx, sy, cell) = emoji_strip(who, g.two_player);
            if fr.click && fr.uy > sy - 3.0 && fr.uy < sy + cell + 3.0 && fr.ux > sx && fr.ux < sx + cell * emoji::COUNT as f32 {
                *id = (((fr.ux - sx) / cell) as usize).min(emoji::COUNT - 1);
                *t = 99.0;
                fr.click = false;
            }
            if *t > 1.1 {
                fr.emoji_send[who] = Some(*id);
            }
        }
        if let Some(id) = fr.emoji_send[who] {
            g.emoji_choose[who] = None;
            g.emoji_last[who] = id;
            g.players[who].emote = Some((id, 3.0));
            quest_event(&mut g.quests, QuestKind::Emoji, 1, &mut g.inv, &mut g.toasts);
            if who == 0 && !g.two_player {
                g.emoji_reply = Some((emoji::reply(id, hash((fr.time * 17.0) as i32, 8, 3)), 0.8));
            }
        }
    }
    if let Some((id, t)) = g.emoji_reply.as_mut() {
        *t -= fr.dt;
        if *t <= 0.0 {
            g.players[1].emote = Some((*id, 3.0));
            g.players[1].voice_timer = 0.3;
            g.emoji_reply = None;
        }
    }
    for p in g.players.iter_mut() {
        if let Some((_, left)) = p.emote.as_mut() {
            *left -= fr.dt;
        }
        if p.emote.map(|e| e.1 <= 0.0).unwrap_or(false) {
            p.emote = None;
        }
    }

    radio::overlay(if g.paused || g.dex_open || g.picking || g.shelf_open { 0.0 } else if g.inside && (g.dj[0] || g.dj[1]) { 0.075 } else { 0.52 });
}
