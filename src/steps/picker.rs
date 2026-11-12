#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn character_picker(g: &mut Game, fr: &mut Frame) {
    fr.picker_closed = false;
    if is_key_pressed(KeyCode::V) {
        g.picking = !g.picking;
        fr.picker_closed = !g.picking;
    }
    if g.picking {
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            g.pick_row = 0;
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            g.pick_row = 1;
        }
        let step = if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            g.cast_list.len() - 1
        } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            1
        } else {
            0
        };
        if step > 0 {
            let idx = (g.players[g.pick_row].character + step) % g.cast_list.len();
            g.players[g.pick_row].character = idx;
            g.players[g.pick_row].body = g.cast_list[idx].body;
            g.players[g.pick_row].voice_timer = 0.4;
            g.audio.play_ch(g.pick_row, &g.voices[idx][2], 0.7);
        }
        if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            g.picking = false;
            fr.picker_closed = true;
        }
    }
    if fr.picker_closed && g.first_pick {
        g.first_pick = false;
        g.first_steps = 1;
        g.toasts.push((format!("the {} trots off to find sticks", g.cast_list[g.players[1].character].noun), 4.0));
    }
}
