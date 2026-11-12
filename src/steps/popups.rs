#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// DJ pads, fish pop-ups, emoji strips and lesson cards.
pub(crate) fn draw_popups(g: &mut Game, fr: &mut Frame) {
    for who in 0..2 {
        if !(g.inside && g.dj[who]) || (who == 1 && !g.two_player) {
            continue;
        }
        let style = BEAT_STYLES[g.beat_style.min(BEAT_STYLES.len() - 1)].name;
        let title = format!("{}  {:.0} bpm  {}{}", if g.dj[0] && g.dj[1] { "b2b" } else { "on the decks" }, g.looper.bpm, if g.looper.beat { style } else { "beat off" }, if g.looper.recording { "  rec" } else { "" });
        let area = if g.two_player { fr.half } else { low_w() };
        let left = if g.two_player && who == 1 { fr.half } else { 0.0 };
        let hint = if who == 0 { "W/S tempo  A/D style  R rec  X out" } else { "7 8 9 0 - = pads  O out" };
        let track = if g.now_playing.is_empty() { String::new() } else { format!("now playing: {now_playing}", now_playing = g.now_playing) };
        let two_decks = g.live_decks.len() >= 2;
        let widest = ui_measure(&title, 12.0).max(ui_measure(hint, 9.0)).max(ui_measure(&track, 10.0));
        let w = if two_decks { (area - 8.0).min(400.0) } else { (widest + 16.0).max(150.0).min(area - 8.0) };
        let col_w = (w - 24.0) / 2.0;
        let columns: Vec<Vec<String>> = if two_decks {
            g.live_decks.iter().take(2).map(|(_, name)| ui_wrap(name, 10.0, col_w).into_iter().take(4).collect()).collect()
        } else {
            Vec::new()
        };
        let lines = if two_decks || track.is_empty() { Vec::new() } else { ui_wrap(&track, 10.0, w - 16.0) };
        let body = if two_decks { 13.0 + columns.iter().map(|c| c.len()).max().unwrap_or(1) as f32 * 12.0 + 4.0 } else { lines.len() as f32 * 12.0 };
        let w = if g.live_next.is_empty() { w } else { w.max(220.0f32.min(area - 8.0)) };
        let heard_line = g.live_next.iter().find(|t| t.starts_with('=')).map(|t| t[1..].to_string()).filter(|t| two_decks || !track.contains(t.as_str()));
        let picks: Vec<String> = g.live_next.iter().filter(|t| !t.starts_with('=')).take(4).map(|t| {
            let mut t = format!("+ {}", t);
            while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                t.pop();
            }
            t
        }).collect();
        let heard_line = heard_line.map(|t| {
            let mut t = format!("heard: {}", t);
            while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                t.pop();
            }
            t
        });
        let heard_h = if heard_line.is_some() { 13.0 } else { 0.0 };
        let extra = heard_h + if picks.is_empty() { 0.0 } else { 14.0 + picks.len() as f32 * 11.0 };
        let h = 30.0 + body + extra;
        let x = left + (area - w) / 2.0;
        let y = if low_h() > low_w() { 84.0 } else if g.two_player { 64.0 } else { 52.0 };
        ui_rect(x, y, w, h, Color::new(0.06, 0.04, 0.1, 0.78));
        ui_text(&title, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 90, 100, 255));
        for (n, line) in lines.iter().enumerate() {
            ui_text(line, x + 8.0, y + 25.0 + n as f32 * 12.0, 10.0, Color::from_rgba(255, 214, 110, 255));
        }
        let deck_colors = [Color::from_rgba(220, 60, 70, 255), Color::from_rgba(240, 200, 80, 255)];
        for (c, column) in columns.iter().enumerate() {
            let cx = x + 8.0 + c as f32 * (col_w + 8.0);
            ui_rect(cx, y + 17.0, 2.0, body - 4.0, deck_colors[c]);
            ui_text(&format!("deck {}", g.live_decks[c].0), cx + 6.0, y + 26.0, 9.0, deck_colors[c]);
            for (n, line) in column.iter().enumerate() {
                ui_text(line, cx + 6.0, y + 38.0 + n as f32 * 12.0, 10.0, WHITE);
            }
        }
        let lines = if two_decks { vec![String::new(); ((body - 4.0) / 12.0).round() as usize] } else { lines };
        let mut shown = hint.to_string();
        while ui_measure(&shown, 9.0) > w - 14.0 && shown.len() > 4 {
            shown.pop();
        }
        ui_text(&shown, x + 8.0, y + 25.0 + lines.len() as f32 * 12.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.75));
        if let Some(line) = &heard_line {
            ui_text(line, x + 8.0, y + 30.0 + body + 6.0, 9.0, Color::from_rgba(150, 200, 255, 255));
        }
        if !picks.is_empty() {
            let top = y + 30.0 + body + 6.0 + heard_h;
            ui_text("goes well with", x + 8.0, top, 9.0, Color::from_rgba(120, 220, 160, 255));
            for (n, pick) in picks.iter().enumerate() {
                ui_text(pick, x + 8.0, top + 12.0 + n as f32 * 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.9));
            }
        }
    }
    if g.fishing.active && g.fish_card <= 0.0 && g.fishing.lesson && (g.looper.position - g.fishing.cue_at).abs() <= 0.22 / g.looper.step_seconds() {
        let pop = 1.0 + 0.15 * (fr.time * 16.0).sin();
        let size = 22.0 * pop;
        let word = if g.fishing.caster == 1 { "O!" } else { "E!" };
        let cx = if g.two_player { fr.half * (g.fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
        ui_text(word, cx - ui_measure(word, size) / 2.0, low_h() * 0.38, size, Color::from_rgba(255, 226, 120, 255));
    }
    if let Some((i, left)) = g.fish_pop.map(|(i, l, _, _)| (i, l)) {
        let is_new = g.fish_pop.map(|f| f.2).unwrap_or(false);
        let shroom = g.fish_pop.map(|f| f.3).unwrap_or(false);
        let a = (left.min(0.4) / 0.4).min(1.0) * ((3.0 - left) * 6.0).min(1.0);
        let w = 150.0;
        let cx = if g.two_player { fr.half / 2.0 } else { low_w() / 2.0 };
        let x = (cx - w / 2.0).round();
        let y = if low_h() > low_w() { 120.0 } else { 46.0 };
        ui_rect(x, y, w, 48.0, Color::new(0.08, 0.12, 0.22, 0.9 * a));
        let rarity_now = if shroom { mushroom::rarity(i) } else { fish::rarity(i) };
        let rc = fish::RARITY_COLORS[rarity_now];
        ui_rect(x, y, w, 2.0, Color::new(rc.r, rc.g, rc.b, a));
        if a > 0.5 {
            if shroom {
                draw_fish_art(&g.shroom_art[i], x + 8.0, y + 7.0, 2.8, true);
            } else {
                draw_fish_art(&g.fish_art[i], x + 6.0, y + 8.0, 2.2, true);
            }
        }
        let tx = x + 50.0;
        ui_text(&if shroom { mushroom::code(i) } else { fish::code(i) }, tx, y + 14.0, 9.0, Color::new(0.75, 0.85, 1.0, a));
        if is_new {
            ui_rect(tx + 26.0, y + 6.0, 26.0, 10.0, Color::new(1.0, 0.45, 0.55, a));
            ui_text("new!", tx + 29.0, y + 14.0, 9.0, Color::new(1.0, 1.0, 1.0, a));
        }
        let mut name = if shroom { mushroom::SPECIES[i] } else { fish::SPECIES[i] }.to_string();
        while ui_measure(&name, 11.0) > w - 56.0 && name.len() > 3 {
            name.pop();
        }
        ui_text(&name, tx, y + 28.0, 11.0, Color::new(1.0, 1.0, 1.0, a));
        let r = rarity_now;
        let c = fish::RARITY_COLORS[r];
        ui_text(fish::RARITY_NAMES[r], tx, y + 41.0, 9.0, Color::new(c.r, c.g, c.b, a));
    }
    if let Some(pop) = g.fish_pop.as_mut() {
        pop.1 -= get_frame_time();
        if pop.1 <= 0.0 {
            g.fish_pop = None;
        }
    }
    for who in 0..2 {
        if let Some((id, t)) = g.emoji_choose[who] {
            let (sx, sy, cell) = emoji_strip(who, g.two_player);
            ui_rect(sx - 4.0, sy - 4.0, cell * emoji::COUNT as f32 + 8.0, cell + 8.0, Color::new(0.97, 0.94, 0.87, 0.92));
            for k in 0..emoji::COUNT {
                let ex = sx + k as f32 * cell;
                if k == id {
                    ui_rect(ex - 1.0, sy - 2.0, cell, cell + 2.0, Color::from_rgba(255, 214, 110, 255));
                }
                draw_emoji_ui(k, ex + 1.5, sy + 0.5, 1.15, 1.0);
            }
            let fill = (t / 1.1).min(1.0);
            ui_rect(sx, sy + cell + 1.5, cell * emoji::COUNT as f32 * fill, 1.5, Color::from_rgba(255, 214, 110, 255));
        }
    }
    fr.small_pop = g.fish_say.as_ref().map(|(w, _, _)| matches!(w.as_str(), "perfect!" | "nice!" | "together!")).unwrap_or(false);
    if let (true, Some((word, col, left))) = (fr.small_pop, g.fish_say.as_ref()) {
        let view = if g.two_player { g.fishing.caster.min(1) } else { 0 };
        let (bx, by) = BOBBER_AT.with(|cell| cell.get()[view]);
        let age = 1.0 - left.clamp(0.0, 1.0);
        let a = (left / 0.35).min(1.0);
        let pop = 1.0 + 0.45 * (1.0 - (age * 6.0).min(1.0));
        let size = 9.0 * pop;
        let w = ui_measure(word, size);
        let y = by - 4.0 - age * 10.0;
        ui_text(word, bx - w / 2.0, y, size, Color::new(col.r, col.g, col.b, a));
        if word == "perfect!" {
            for k in 0..3 {
                let ang = age * 5.0 + k as f32 * 2.1;
                ui_star(bx + ang.cos() * (w * 0.5 + 5.0), y - 3.0 + ang.sin() * 5.0, 2.5, Color::new(1.0, 0.95, 0.6, a));
            }
        }
    } else if let Some((word, col, left)) = g.fish_say.as_ref() {
        let a = left.min(0.4) / 0.4;
        let rise = (1.0 - left.min(1.0)) * 6.0;
        let cx = if g.two_player { fr.half * (g.fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
        let size = if word.len() < 12 { 16.0 } else { 11.0 };
        let w = ui_measure(word, size);
        ui_rect(cx - w / 2.0 - 6.0, low_h() * 0.3 - rise - size, w + 12.0, size + 6.0, Color::new(0.08, 0.06, 0.16, 0.7 * a));
        ui_text(word, cx - w / 2.0, low_h() * 0.3 - rise, size, Color::new(col.r, col.g, col.b, a));
    }
    if g.fish_card > 0.0 {
        let a = (g.fish_card.min(0.4) / 0.4).min(1.0);
        let w = 250f32.min(low_w() - 16.0);
        let h = 128.0;
        let x = ((low_w() - w) / 2.0).round();
        let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
        ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
        ui_rect(x, y, w, 3.0, Color::new(0.45, 0.75, 1.0, a));
        let friend = g.cast_list[g.players[1].character].noun;
        ui_text("fishing on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
        let cx = x + 34.0;
        let cy = y + 62.0;
        let t = (fr.time * 0.5).fract();
        let r = 6.0 + (1.0 - t) * 18.0;
        let ring_col = if t > 0.85 { Color::new(1.0, 0.86, 0.3, a) } else { Color::new(0.95, 0.97, 1.0, a) };
        for k in 0..20 {
            let ang = k as f32 / 20.0 * 2.0 * PI;
            ui_rect(cx + ang.cos() * r - 1.0, cy + ang.sin() * r * 0.6 - 1.0, 2.0, 2.0, ring_col);
        }
        ui_rect(cx - 3.0, cy - 5.0, 6.0, 4.0, Color::new(0.9, 0.24, 0.24, a));
        ui_rect(cx - 3.0, cy - 1.0, 6.0, 3.0, Color::new(0.98, 0.98, 0.95, a));
        if t > 0.85 {
            ui_text(if g.fishing.caster == 1 { "O!" } else { "E!" }, cx - 6.0, cy - 22.0, 12.0, Color::new(1.0, 0.86, 0.3, a));
        }
        let lines = [
            if g.two_player { "the river whispers:".to_string() } else { format!("the {} whispers:", friend) },
            "watch the ring around the bobber.".to_string(),
            format!("when it shrinks onto it, press {}.", if g.fishing.caster == 0 { "E" } else { "O" }),
            "three good ones and the fish is yours!".to_string(),
            if g.two_player { "player 2 can press O too - together is better".to_string() } else { format!("miss? you get 3 tries, and the {} helps", friend) },
        ];
        for (n, line) in lines.iter().enumerate() {
            let col = if n == 0 { Color::new(0.75, 0.9, 0.7, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
            let mut shown = line.clone();
            while ui_measure(&shown, 9.5) > w - 78.0 && shown.len() > 4 {
                shown.pop();
            }
            ui_text(&shown, x + 68.0, y + 38.0 + n as f32 * 14.0, 9.5, col);
        }
        let go = if g.fishing.caster == 0 { "E: got it" } else { "O: got it" };
        ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
    }
    if g.cook_card > 0.0 {
        let a = (g.cook_card.min(0.4) / 0.4).min(1.0);
        let w = 250f32.min(low_w() - 16.0);
        let h = 128.0;
        let x = ((low_w() - w) / 2.0).round();
        let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
        ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
        ui_rect(x, y, w, 3.0, Color::new(1.0, 0.62, 0.34, a));
        ui_text("cooking on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
        let lines = [
            "sit by the fire, then press E or O again".to_string(),
            "to put the pot on.".to_string(),
            "tap the first four notes on the beat.".to_string(),
            "fish makes bigger dishes, and new dishes".to_string(),
            "earn more acorns.".to_string(),
        ];
        for (n, line) in lines.iter().enumerate() {
            let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
            let mut shown = line.clone();
            while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                shown.pop();
            }
            ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
        }
        let go = if g.two_player { "E / O: got it" } else { "E: got it" };
        ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
    }
    if g.gather_card > 0.0 {
        let a = (g.gather_card.min(0.4) / 0.4).min(1.0);
        let w = 250f32.min(low_w() - 16.0);
        let h = 128.0;
        let x = ((low_w() - w) / 2.0).round();
        let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
        ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
        ui_rect(x, y, w, 3.0, Color::new(0.5, 0.88, 0.52, a));
        ui_text("gathering around the woods", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
        let lines = [
            "face a tree or stone and press E or O.".to_string(),
            "sparkly trees give sticks to carry.".to_string(),
            "other trees shake loose acorns.".to_string(),
            "stones can be picked up too.".to_string(),
            "bring your finds back to the fire.".to_string(),
        ];
        for (n, line) in lines.iter().enumerate() {
            let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
            let mut shown = line.clone();
            while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                shown.pop();
            }
            ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
        }
        let go = if g.two_player { "E / O: got it" } else { "E: got it" };
        ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
    }
}
