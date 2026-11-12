#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// The note bar, loop grid, status line, meters and hints.
pub(crate) fn draw_hud(g: &mut Game, fr: &mut Frame) {
    fr.ink = Color::from_rgba(40, 30, 60, 255);
    fr.touch_ui = radio::touch_ui() && !g.two_player;
    fr.portrait = low_h() > low_w();
    fr.edge = if fr.touch_ui && !fr.portrait { (low_w() * 0.235).round() } else { 0.0 };
    fr.right_edge = low_w() - fr.edge;
    fr.bar_y = if fr.touch_ui && fr.portrait { 58.0 } else if fr.touch_ui { low_h() - 2.0 } else { low_h() - 32.0 };
    
    
    if fr.touch_ui {
        fr.grid_w = 296f32.min(low_w() - 24.0);
        fr.grid_x = (low_w() - fr.grid_w) / 2.0;
    } else if g.two_player {
        for who in 0..2 {
            let box_w = 25.0;
            let start_x = who as f32 * fr.half + (fr.half - box_w * 6.0) / 2.0;
            let labels = if who == 0 { P1_KEY_LABELS } else { P2_KEY_LABELS };
            for i in 0..6 {
                let x = start_x + i as f32 * box_w;
                let glow = g.players[who].key_glow[i];
                let mut bg = note_color(i);
                bg.a = 0.55 + glow * 0.45;
                let lift = glow * 3.0;
                ui_rect(x + 1.0, fr.bar_y - lift, box_w - 2.0, 28.0, bg);
                ui_text(labels[i], x + 4.0, fr.bar_y + 12.0 - lift, 12.0, fr.ink);
                let name = if g.inside && g.dj[who] { &DJ_PADS[who][i][..DJ_PADS[who][i].len().min(4)] } else if g.cooking && g.players[who].seated && i < 4 { &PERC_NAMES[i][..3] } else if g.inside && g.seat[who] == Some(home::Item::Piano) { "piano" } else if g.inside && g.seat[who] == Some(home::Item::Drums) { "drum" } else { NOTE_NAMES[i] };
                ui_text(name, x + 4.0, fr.bar_y + 24.0 - lift, 12.0, fr.ink);
            }
        }
        fr.grid_x = 12.0;
        fr.grid_w = low_w() - 24.0;
    } else {
        let box_w = 50.0;
        let start_x = low_w() / 2.0 - box_w * 3.0;
        for i in 0..6 {
            let x = start_x + i as f32 * box_w;
            let glow = g.players[0].key_glow[i];
            let mut bg = note_color(i);
            bg.a = 0.55 + glow * 0.45;
            let lift = glow * 3.0;
            ui_rect(x + 2.0, fr.bar_y - lift, box_w - 4.0, 28.0, bg);
            ui_text(&format!("{} {}", i + 1, NOTE_NAMES[i]), x + 6.0, fr.bar_y + 12.0 - lift, 12.0, fr.ink);
            let label = if g.inside && g.dj[0] {
                DJ_PADS[0][i]
            } else if g.cooking && i < 4 {
                PERC_NAMES[i]
            } else if g.music_mode {
                voice_word(g.cast_list[g.players[0].character].voice)
            } else {
                SPELL_NAMES[i]
            };
            ui_text(label, x + 6.0, fr.bar_y + 24.0 - lift, 12.0, fr.ink);
        }
        fr.grid_x = start_x + 2.0;
        fr.grid_w = box_w * 6.0 - 4.0;
    }

    fr.show_grid = g.music_mode || g.looper.recording || !g.looper.notes.is_empty();
    fr.lanes = if g.two_player && !fr.touch_ui {
        (0..2).map(|who| (Some(who), who as f32 * fr.half + (fr.half - 150.0) / 2.0 + 1.0, 148.0)).collect()
    } else {
        vec![(None, fr.grid_x, fr.grid_w)]
    };
    fr.grid_y = fr.bar_y - 18.0;
    if fr.show_grid {
        for &(lane, lx, lw) in &fr.lanes {
            let cell = lw / LOOP_STEPS as f32;
            ui_rect(lx, fr.grid_y, lw, 14.0, Color::new(0.1, 0.08, 0.2, 0.5));
            for k in (0..LOOP_STEPS).step_by(4) {
                let a = if k % 16 == 0 { 0.6 } else { 0.25 };
                ui_rect(lx + k as f32 * cell, fr.grid_y, 0.34, 14.0, Color::new(1.0, 1.0, 1.0, a));
            }
            ui_rect(lx + fr.step as f32 * cell, fr.grid_y, cell, 14.0, Color::new(1.0, 1.0, 1.0, 0.3));
            for &(at, id) in &g.looper.notes {
                if lane.map(|who| note_owner(id) != who).unwrap_or(false) {
                    continue;
                }
                let slot = note_slot(id).min(5);
                let nx = lx + at * cell + 0.3;
                let ny = fr.grid_y + 1.0 + (5 - slot) as f32 * 2.0;
                let mut color = note_color(slot);
                if lane.is_none() && note_owner(id) == 1 {
                    color = Color::new(color.r * 0.75, color.g * 0.75, color.b * 0.75, 1.0);
                }
                ui_rect(nx, ny, (cell - 0.6).max(1.5), 1.7, color);
            }
        }
    }

    fr.status_y = if fr.show_grid { fr.grid_y - 4.0 } else { fr.bar_y - 4.0 };
    fr.status_x = fr.grid_x;
    if g.looper.recording {
        if (fr.time * 2.0).fract() < 0.6 {
            ui_rect(fr.status_x, fr.status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
        }
        ui_text("REC", fr.status_x + 9.0, fr.status_y, 12.0, WHITE);
        fr.status_x += 34.0;
    }
    if g.listening {
        let dot = if g.locked {
            Color::from_rgba(90, 230, 120, 255)
        } else {
            let blink = 0.35 + 0.65 * (fr.time * 4.0).sin().abs();
            Color::new(1.0, 0.75, 0.25, blink)
        };
        ui_rect(fr.status_x + 1.0, fr.status_y - 7.0, 6.0, 6.0, dot);
        let text = if g.locked { format!("synced {:.0} bpm", g.looper.bpm) } else { "syncing...".to_string() };
        ui_text(&text, fr.status_x + 12.0, fr.status_y, 12.0, if g.locked { Color::from_rgba(150, 255, 170, 255) } else { WHITE });
    } else if g.looper.beat {
        ui_text(&format!("beat {:.0} bpm", g.looper.bpm), fr.status_x, fr.status_y, 12.0, WHITE);
    }
    if let Some(msg) = g.mic_message {
        g.mic_message_clock += get_frame_time();
        if g.mic_message_clock > 3.0 {
            g.mic_message = None;
        } else {
            ui_rect(fr.grid_x - 1.0, fr.status_y - 21.0, 7.0, 7.0, Color::from_rgba(255, 90, 90, 255));
            ui_text(msg, fr.grid_x + 9.0, fr.status_y - 14.0, 12.0, Color::from_rgba(255, 210, 160, 255));
        }
    } else {
        g.mic_message_clock = 0.0;
    }
    if let Some(t) = g.session {
        let label = format!("together {}:{:02}", t as u32 / 60, t as u32 % 60);
        let tw = ui_measure(&label, 11.0) + 36.0;
        let sx = fr.grid_x + (fr.grid_w - tw) / 2.0;
        ui_text(&label, sx, fr.status_y, 11.0, Color::from_rgba(255, 236, 200, 255));
        for k in 0..3u8 {
            let col = if k < g.session_stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.22) };
            ui_star(sx + ui_measure(&label, 11.0) + 4.0 + k as f32 * 11.0, fr.status_y - 8.0, 8.0, col);
        }
    }
    fr.right_x = fr.grid_x + fr.grid_w;
    if g.ringing {
        let glow = 0.75 + 0.25 * (fr.time * 4.0).sin();
        fr.right_x -= ui_measure("~echo~", 12.0);
        ui_text("~echo~", fr.right_x, fr.status_y, 12.0, Color::new(0.75, 0.95, 1.0, glow));
        fr.right_x -= 10.0;
    }
    if let Some(t) = g.audio.tape_seconds() {
        let label = format!("tape {}:{:02}", t as u32 / 60, t as u32 % 60);
        fr.right_x -= ui_measure(&label, 12.0);
        ui_text(&label, fr.right_x, fr.status_y, 12.0, Color::from_rgba(255, 216, 216, 255));
        if (fr.time * 2.0).fract() < 0.6 {
            ui_rect(fr.right_x - 9.0, fr.status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
        }
    }

    ui_rect(0.0, 0.0, low_w(), 17.0, Color::new(0.1, 0.08, 0.2, 0.4));
    fr.mode = if g.cooking { "cooking".to_string() } else if g.music_mode { format!("music  q:{}", g.looper.quantise.label()) } else { "spells".to_string() };
    if g.two_player {
        ui_text(g.cast_list[g.players[0].character].name, 6.0, 12.0, 12.0, Color::from_rgba(255, 226, 190, 255));
        ui_text(g.cast_list[g.players[1].character].name, fr.half + 6.0, 12.0, 12.0, Color::from_rgba(240, 206, 140, 255));
        ui_text(&fr.mode, 76.0, 12.0, 12.0, WHITE);
    } else {
        ui_text(&fr.mode, fr.edge + 6.0, 12.0, 12.0, WHITE);
        if !g.now_playing.is_empty() && fr.right_edge - 144.0 - fr.edge - ui_measure(&fr.mode, 12.0) > 40.0 {
            let left = fr.edge + 6.0 + ui_measure(&fr.mode, 12.0) + 12.0;
            let room = fr.right_edge - 118.0 - 8.0 - left;
            let line = ui_ticker(&format!("~ {now_playing}", now_playing = g.now_playing), 12.0, room, fr.time);
            ui_text(&line, left, 12.0, 12.0, Color::from_rgba(255, 214, 110, 255));
        }
    }
    (fr.meter_x, fr.meter_y) = if g.two_player { (6.0, 27.0) } else { (fr.right_edge - 118.0, 12.0) };
    ui_text("fire", fr.meter_x, fr.meter_y, 12.0, Color::from_rgba(255, 200, 140, 255));
    ui_rect(fr.meter_x + 24.0, fr.meter_y - 7.0, 40.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.35));
    fr.fill = (g.camp.fuel / 180.0).min(1.0);
    fr.fire_col = if g.camp.lit { Color::from_rgba(255, 150, 60, 255) } else { Color::from_rgba(120, 110, 110, 255) };
    ui_rect(fr.meter_x + 24.0, fr.meter_y - 7.0, 40.0 * fr.fill, 6.0, fr.fire_col);
    {
        let hx = fr.right_edge - 46.0;
        ui_rect(hx - 2.0, 2.0, 12.0, 13.0, Color::from_rgba(255, 214, 110, 255));
        ui_text("H", hx + 1.0, 12.0, 12.0, Color::from_rgba(36, 26, 71, 255));
        ui_text("help", hx + 14.0, 12.0, 12.0, WHITE);
    }
    {
        let sk = sky();
        let temp = sk.temp.map(|t| format!(" {:.0}C", t)).unwrap_or_default();
        let line = format!("{:02}:{:02}  {}  {}{}", sk.hour as u32, ((sk.hour.fract()) * 60.0) as u32, SEASONS[sk.season], WEATHERS[sk.weather], temp);
        let wx = if g.two_player { fr.half - ui_measure(&line, 9.0) - 6.0 } else { fr.right_edge - ui_measure(&line, 9.0) - 4.0 };
        ui_text(&line, wx, 38.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
    }
    if g.ring_flash > 0.0 {
        let msg = if g.ringing { "echo on  (G)" } else { "echo off  (G)" };
        let a = g.ring_flash.min(1.0);
        let w = ui_measure(msg, 12.0) + 12.0;
        ui_rect((low_w() - w) / 2.0, 26.0, w, 16.0, Color::new(0.1, 0.08, 0.2, 0.7 * a));
        ui_text(msg, (low_w() - w) / 2.0 + 6.0, 38.0, 12.0, Color::new(0.8, 0.95, 1.0, a));
    }

    fr.fish_total = g.inv.fish.iter().sum();
    fr.items = [
        (Color::from_rgba(176, 124, 76, 255), format!("{}", g.inv.wood)),
        (Color::from_rgba(168, 170, 184, 255), format!("{}", g.inv.stone)),
        (Color::from_rgba(140, 190, 210, 255), format!("{}", fr.fish_total)),
        (Color::from_rgba(255, 140, 160, 255), format!("{}/{}", g.dishes.len(), DISHES.len())),
        (ACORN_COLOR, format!("{}", g.inv.acorns)),
    ];
    fr.ix = fr.right_edge - 6.0 - fr.items.len() as f32 * 24.0;
    for (col, text) in fr.items.iter() {
        ui_rect(fr.ix, 21.0, 6.0, 6.0, *col);
        ui_text(text, fr.ix + 8.0, 27.0, 11.0, WHITE);
        fr.ix += 24.0;
    }

    fr.inside_hint = if g.inside {
        let ahead = ahead_local(&g.players[0]);
        let here = home::local(g.players[0].pos);
        Some(if g.shop_open || g.dj[0] {
            String::new()
        } else if let Some(piece) = g.holding {
            if ahead == home::CATALOGUE {
                format!("E: sell the {} for {} acorns", piece.item.name(), piece.item.price() / 2)
            } else {
                format!("E: place the {} - X: turn it - M: put in bag", piece.item.name())
            }
        } else if let Some(item) = g.seat[0] {
            match item {
                home::Item::Piano | home::Item::Drums => format!("1-6: play the {} - move to get up", item.name()),
                home::Item::Bed => "zzz - move to get up".to_string(),
                _ => format!("sitting on the {} - move to get up", item.name()),
            }
        } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
            format!("E: catalogue & diary ({} acorns)", g.inv.acorns)
        } else if let Some(i) = g.house.piece_at(ahead.0, ahead.1) {
            let item = g.house.pieces[i].item;
            match item {
                home::Item::Piano | home::Item::Drums => format!("E: play the {} - X: move it", item.name()),
                home::Item::RecordPlayer => format!("E: record {} - X: move it", if g.house.record_on { "off" } else { "on" }),
                home::Item::DjBooth => "E: get on the decks - X: move it".to_string(),
                home::Item::WindChime => "E: ring the chime - X: move it".to_string(),
                home::Item::Lamp | home::Item::RedLight => format!("E: light {} - X: move it", if g.house.pieces[i].state == 0 { "off" } else { "on" }),
                home::Item::Speaker | home::Item::Subwoofer => format!("E: {} {} - X: move it", item.name(), if g.house.pieces[i].state == 0 { "off" } else { "on" }),
                home::Item::Bed => "E: sleep (saves the game) - X: move it".to_string(),
                home::Item::Sofa | home::Item::ConcreteBench => format!("E: sit on the {} - X: move it", item.name()),
                home::Item::Plant => {
                    let piece = &g.house.pieces[i];
                    if piece.state == home::WITHERED {
                        "withered - E: water it to bring it back".to_string()
                    } else if piece.day == sky().day as i32 {
                        "watered today - X: move it".to_string()
                    } else {
                        "thirsty - E: water the plant - X: move it".to_string()
                    }
                }
                home::Item::Bookshelf => "E: read a book - X: move it".to_string(),
                home::Item::Table => "E: serve a dish - X: move it".to_string(),
                home::Item::SteelPillar => "E: knock on it - X: move it".to_string(),
                home::Item::MirrorBall => format!("E: mirror ball {} - X: move it", if g.house.pieces[i].state == 0 { "off" } else { "on" }),
                home::Item::Tv => format!("E: next channel ({}) - X: move it", if g.house.pieces[i].state == 0 { "off" } else { TV_CHANNELS[g.house.pieces[i].state as usize - 1] }),
                home::Item::Arcade => "E: dance! - X: move it".to_string(),
                home::Item::Sampler => "E: open the sampler - X: move it".to_string(),
                _ => format!("X: move the {}", item.name()),
            }
        } else if here.y > g.house.d() as f32 - 1.1 && (here.x - g.house.door().x).abs() < 0.9 {
            "walk out the door to go outside".to_string()
        } else {
            String::new()
        })
    } else {
        None
    };
    fr.hint = if let Some(h) = fr.inside_hint.take() {
        h
    } else if g.players[0].carrying.is_some() && !g.players[0].seated {
        format!("bring the {} to the logs by the fire", carried_name(g.players[0].carrying))
    } else if g.two_player && g.players[1].carrying.is_some() {
        format!("player 2: bring the {} to the logs by the fire", carried_name(g.players[1].carrying))
    } else if g.fishing.active {
        let coop = g.two_player && g.players[0].pos.distance(DOCK_END) < 2.2 && g.players[1].pos.distance(DOCK_END) < 2.2;
        let who = if coop { "E and O" } else if g.fishing.caster == 1 { "O" } else { "E" };
        if g.fish_card > 0.0 {
            String::new()
        } else if g.fishing.lesson {
            format!("{} when the ring touches the bobber - catch {}/3", who, g.fishing.hits.min(3))
        } else {
            format!("{} when the ring closes - catch {}/3 - tries {}", who, g.fishing.hits.min(3), "o".repeat(g.fishing.tries as usize))
        }
    } else if g.cooking {
        format!("cook on the beat {}/{} - {}: done", g.cook_progress, COOK_GOAL, if g.two_player { "E / O" } else { "E" })
    } else if g.jam {
        if fr.dog_jamming && g.partner_soloing {
            format!("the {} knows {} tune{} - answer back!", g.cast_list[g.players[1].character].noun, g.songbook.len(), if g.songbook.len() == 1 { "" } else { "s" })
        } else if fr.dog_jamming || (g.two_player && g.players[1].seated) {
            "playing together by the fire - E / O: cook, move to stand up".to_string()
        } else {
            format!("playing by the fire - E: cook, the {} is on the way", g.cast_list[g.players[1].character].noun)
        }
    } else if g.players[0].pos.distance(DOOR_OUT) < 0.9 && !g.players[0].seated {
        if g.two_player { "E / O: go inside".to_string() } else { "E: go inside".to_string() }
    } else if g.camp.lit && near_fire(&g.players[0]) && !g.players[0].seated {
        "E: sit by the fire".to_string()
    } else if at_dock_end(&g.players[0]) {
        if g.two_player { "E / O: cast your fishing line".to_string() } else { "E: cast your fishing line".to_string() }
    } else if fr.can_build_here {
        format!("E: build {}  X: change", g.tools[0].name())
    } else if !g.camp.lit && g.camp.fuel <= 0.0 {
        if g.two_player { "find a tree with a stick at its foot - E or O picks it up".to_string() } else { format!("the {} is looking for sticks - or grab one yourself with E", g.cast_list[g.players[1].character].noun) }
    } else {
        String::new()
    };
    if g.two_player && g.inside && !g.picking && !g.shop_open {
        let ahead = ahead_local(&g.players[1]);
        let here = home::local(g.players[1].pos);
        let mut p2_hint = if g.dj[1] {
            "O: leave DJ booth".to_string()
        } else if let Some(piece) = g.holding {
            if g.holding_by == 1 {
                format!("O: place {} - U: turn - Bksp: bag", piece.item.name())
            } else {
                String::new()
            }
        } else if let Some(item) = g.seat[1] {
            if item == home::Item::Bed { "O: get out of bed".to_string() } else { format!("O: get up from {}", item.name()) }
        } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
            "O: catalogue and diary".to_string()
        } else if let Some(i) = g.house.piece_at(ahead.0, ahead.1) {
            format!("O: use {} - U: move", g.house.pieces[i].item.name())
        } else {
            String::new()
        };
        while ui_measure(&p2_hint, 11.0) > fr.half - 12.0 && p2_hint.len() > 7 {
            p2_hint.pop();
        }
        if !p2_hint.is_empty() {
            let w = ui_measure(&p2_hint, 11.0);
            let x = fr.half + (fr.half - w) * 0.5;
            let y = if fr.show_grid { fr.grid_y - 31.0 } else { fr.bar_y - 26.0 };
            ui_rect(x - 4.0, y - 9.0, w + 8.0, 13.0, Color::new(0.1, 0.08, 0.2, 0.68));
            ui_text(&p2_hint, x, y, 11.0, Color::from_rgba(255, 236, 200, 255));
        }
    }
    if !fr.hint.is_empty() && !g.picking {
        let w = ui_measure(&fr.hint, 12.0);
        let hy = if fr.touch_ui && fr.portrait { (low_h() * 0.56).round() } else if fr.show_grid { fr.grid_y - 16.0 } else { fr.bar_y - 10.0 };
        ui_rect((low_w() - w) / 2.0 - 4.0, hy - 9.0, w + 8.0, 12.0, Color::new(0.1, 0.08, 0.2, 0.5));
        ui_text(&fr.hint, (low_w() - w) / 2.0, hy, 12.0, Color::from_rgba(255, 236, 200, 255));
    }
    for (k, (text, life)) in g.toasts.iter().enumerate() {
        let a = life.min(1.0);
        let toast_y = if fr.touch_ui && fr.portrait { 70.0 } else { 40.0 };
        ui_text(text, fr.edge + 8.0, toast_y + k as f32 * 11.0, 12.0, Color::new(1.0, 0.95, 0.85, a));
    }
}

/// The bag panel.
pub(crate) fn draw_bag(g: &mut Game, fr: &mut Frame) {
    if g.show_inventory && !g.picking && !g.shop_open && !g.show_help {
        let lines = [
            format!("acorns  {}", g.inv.acorns),
            format!("wood    {}   stone  {}", g.inv.wood, g.inv.stone),
            format!("fish    {}   mushrooms  {}", g.inv.fish[0] + g.inv.fish[1] + g.inv.fish[2], g.inv.mush[0] + g.inv.mush[1] + g.inv.mush[2]),
            format!("fishdex {}/{}   shrooms {}/{}", g.dex_counts.iter().filter(|n| **n > 0).count(), fish::COUNT, g.shroom_counts.iter().filter(|n| **n > 0).count(), mushroom::COUNT),
            format!("dishes  {}/{}   tunes  {}", g.dishes.len(), DISHES.len(), g.songbook.len()),
        ];
        let w = 210f32.min(low_w() - 16.0);
        let shown_items = 5usize;
        let furn_rows = if g.stored.is_empty() { 1 } else { g.stored.len().min(shown_items) };
        let h = 30.0 + lines.len() as f32 * 11.0 + 16.0 + g.quests.list.len() as f32 * 11.0 + 16.0 + furn_rows as f32 * 11.0 + 18.0;
        let x = 8.0;
        let y = if fr.touch_ui && fr.portrait { 86.0 } else { 22.0 };
        let h = h.min(low_h() - y - 4.0);
        ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.92));
        ui_text(if g.two_player { if g.bag_owner == 0 { "bag - player 1" } else { "bag - player 2" } } else { "bag" }, x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
        ui_rect(x + 8.0, y + 17.0, w - 16.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
        let mut ty = y + 29.0;
        for (n, line) in lines.iter().enumerate() {
            let col = if n == 0 { Color::from_rgba(255, 236, 200, 255) } else { WHITE };
            ui_text(line, x + 8.0, ty, 10.0, col);
            ty += 11.0;
        }
        ty += 5.0;
        ui_text("today's quests", x + 8.0, ty, 10.0, Color::from_rgba(150, 215, 255, 255));
        ty += 11.0;
        for q in &g.quests.list {
            let mark = if q.done { "[x]" } else { "[ ]" };
            let line = format!("{} {}  {}/{}", mark, q.label(), q.progress, q.target);
            let reward = format!("+{}", q.reward());
            let col = if q.done { Color::new(0.6, 0.9, 0.6, 0.9) } else { WHITE };
            let mut shown = line.clone();
            while ui_measure(&shown, 9.0) > w - 40.0 && shown.len() > 6 {
                shown.pop();
            }
            ui_text(&shown, x + 8.0, ty, 9.0, col);
            ui_text(&reward, x + w - 8.0 - ui_measure(&reward, 9.0), ty, 9.0, ACORN_COLOR);
            ty += 11.0;
        }
        ty += 5.0;
        ui_text(&format!("furniture in the bag ({})", g.stored.len()), x + 8.0, ty, 10.0, Color::from_rgba(255, 190, 210, 255));
        ty += 11.0;
        if g.stored.is_empty() {
            ui_text(if g.inside { "empty - M while carrying a piece puts it here" } else { "empty" }, x + 8.0, ty, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
        } else {
            let first = g.bag_sel.saturating_sub(shown_items / 2).min(g.stored.len().saturating_sub(shown_items));
            for r in first..(first + shown_items).min(g.stored.len()) {
                if g.inside && r == g.bag_sel {
                    ui_rect(x + 5.0, ty - 8.0, w - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.14));
                }
                ui_text(g.stored[r].name(), x + 10.0, ty, 9.0, WHITE);
                ty += 11.0;
            }
        }
        let bag_hint = if !g.inside {
            if fr.touch_ui { "bag to close" } else { "M to close" }
        } else if g.bag_clear_armed > 0.0 {
            "X again: pack ALL furniture away"
        } else if g.bag_owner == 0 {
            "W/S choose  E take out  X pack all  M close"
        } else {
            "I/K choose  O take out  U pack all  Bksp close"
        };
        ui_text(bag_hint, x + (w - ui_measure(bag_hint, 9.0)) / 2.0, y + h - 6.0, 9.0, if g.bag_clear_armed > 0.0 { Color::from_rgba(255, 150, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) });
    }
}
