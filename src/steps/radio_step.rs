#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn report_and_radio(g: &mut Game, _fr: &mut Frame) {
    {
        let labels: Vec<&str> = (0..6)
            .map(|i| {
                if g.inside && g.dj[0] {
                    DJ_PADS[0][i]
                } else if g.seat[0] == Some(home::Item::Piano) {
                    "piano"
                } else if g.seat[0] == Some(home::Item::Drums) {
                    ["kick", "clap", "hat", "lid", "chop", "sizzle"][i]
                } else if g.cooking && g.players[0].seated && i < 4 {
                    PERC_NAMES[i]
                } else if g.music_mode {
                    voice_word(g.cast_list[g.players[0].character].voice)
                } else {
                    SPELL_NAMES[i]
                }
            })
            .collect();
        g.audio.set_boost(g.house.switched_on(home::Item::Speaker));
        let tool_label = if g.inside {
            let ahead = ahead_local(&g.players[0]);
            if g.shop_open {
                "close".to_string()
            } else if g.dj[0] {
                "leave".to_string()
            } else if g.holding.is_some() {
                "turn".to_string()
            } else if g.seat[0].is_some() {
                "get up".to_string()
            } else if g.house.piece_at(ahead.0, ahead.1).is_some() {
                "move".to_string()
            } else {
                "-".to_string()
            }
        } else {
            g.tools[0].name().to_string()
        };
        let report = format!(
            "mode={}|rec={}|echo={}|tape={}|tool={}|notes={}",
            if g.cooking { "cooking" } else if g.music_mode { "music" } else { "spells" },
            g.looper.recording as u8,
            g.ringing as u8,
            g.audio.tape_seconds().map(|t| t as u32).map_or(-1, |t| t as i64),
            tool_label,
            labels.join(",")
        );
        if report != g.last_report {
            radio::report(&report);
            g.last_report = report;
        }
    }
    g.radio_clock += get_frame_time();
    if g.radio_clock > 0.5 {
        g.radio_clock = 0.0;
        g.live_decks = radio::decks();
        g.live_next = radio::next();
        let title = radio::now_playing();
        if title != g.now_playing {
            if !title.is_empty() {
                g.toasts.push((format!("now playing: {title}"), 4.0));
            }
            g.now_playing = title;
        }
        let bpm = radio::bpm();
        if bpm > 0.0 && (bpm - g.radio_bpm).abs() > 0.05 {
            g.looper.bpm = bpm.clamp(60.0, 200.0);
            g.toasts.push((format!("the beat follows the mix: {:.0} bpm", g.looper.bpm), 2.5));
        }
        g.radio_bpm = bpm;
    }
}
