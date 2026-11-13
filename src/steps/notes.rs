#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn play_notes(g: &mut Game, fr: &mut Frame) {
    fr.active = if g.two_player { 2 } else { 1 };
    fr.dog_jamming = g.jam && !g.two_player && g.players[1].seated;
    for who in 0..fr.active {
        let keys = if who == 0 { NOTE_KEYS } else { P2_NOTE_KEYS };
        for (i, key) in keys.iter().enumerate() {
            if !is_key_pressed(*key) {
                continue;
            }
            g.last_note_at = fr.time;
            if who == 0 && g.first_steps == 1 && g.music_mode && !g.cooking && !(g.inside && g.dj[0]) {
                g.first_steps = 2;
                g.toasts.push(("your first note! find the campfire next".to_string(), 3.0));
            }
            if g.inside && g.dj[who] {
                let id = 16 + who * 6 + i;
                if g.looper.press(id) {
                    g.play_pad(who, i, 0.85, g.players[1].character);
                }
                g.players[who].key_glow[i] = 1.0;
                g.players[who].voice_timer = 0.2;
                continue;
            }
            if g.inside && matches!(g.seat[who], Some(home::Item::Piano) | Some(home::Item::Drums)) {
                if g.seat[who] == Some(home::Item::Piano) {
                    g.audio.play_ch(who, &g.piano[i], 0.8);
                } else {
                    let drum = [&g.kick, &g.clap, &g.hat, &g.lid, &g.chop, &g.sizzle][i];
                    g.audio.play_ch(2, drum, 0.85);
                }
                g.players[who].key_glow[i] = 1.0;
                g.players[who].voice_timer = 0.3;
                g.players[1 - who].voice_timer = g.players[1 - who].voice_timer.max(0.15);
                continue;
            }
            if g.cooking && g.players[who].seated && i < 4 {
                let id = if who == 0 { 12 + i } else { 28 + i };
                let play_now = g.looper.press(id);
                if play_now {
                    g.audio.play_ch(2, g.perc()[i], 0.8);
                }
                let f = g.looper.position.fract();
                if f < 0.3 || f > 0.7 {
                    g.cook_progress += 1;
                }
                g.players[who].key_glow[i] = 1.0;
                let c = fire_center() + vec3(0.0, 0.6, 0.08);
                let col = [Color::from_rgba(255, 170, 80, 255), Color::from_rgba(220, 220, 230, 255), Color::from_rgba(120, 200, 90, 255), Color::from_rgba(255, 220, 120, 255)][i];
                for k in 0..6 {
                    let a = (k as f32 / 6.0 + fr.time) * 2.0 * PI;
                    spawn(&mut g.particles, c, vec3(a.cos() * 0.7, 1.2, a.sin() * 0.3), 0.5, col, 0.025, 0.5);
                }
                if g.cook_progress >= COOK_GOAL {
                    g.cook_progress = 0;
                    let fish_n = g.inv.fish[0] + g.inv.fish[1] + g.inv.fish[2];
                    let mush_n = g.inv.mush[0] + g.inv.mush[1] + g.inv.mush[2];
                    let take = |stock: &mut [u32; 3], n: u32| {
                        for _ in 0..n {
                            if let Some(k) = (0..3).find(|k| stock[*k] > 0) {
                                stock[k] -= 1;
                            }
                        }
                    };
                    let dish = if g.inv.mush[2] > 0 {
                        g.inv.mush[2] -= 1;
                        7
                    } else if g.inv.fish[2] > 0 {
                        g.inv.fish[2] -= 1;
                        3
                    } else if fish_n >= 1 && mush_n >= 1 {
                        take(&mut g.inv.fish, 1);
                        take(&mut g.inv.mush, 1);
                        6
                    } else if mush_n >= 2 {
                        take(&mut g.inv.mush, 2);
                        5
                    } else if fish_n >= 2 {
                        take(&mut g.inv.fish, 2);
                        2
                    } else if mush_n == 1 {
                        take(&mut g.inv.mush, 1);
                        4
                    } else if fish_n == 1 {
                        take(&mut g.inv.fish, 1);
                        1
                    } else {
                        0
                    };
                    let new = !g.dishes.contains(&dish);
                    if new {
                        g.dishes.push(dish);
                    }
                    let earned = if new { 10 } else { 5 };
                    g.inv.acorns += earned;
                    quest_event(&mut g.quests, QuestKind::Cook, 1, &mut g.inv, &mut g.toasts);
                    g.toasts.push((
                        if new { format!("new dish: {}! +{} acorns", DISHES[dish], earned) } else { format!("cooked {} again - yum, +{} acorns", DISHES[dish], earned) },
                        3.5,
                    ));
                    g.audio.play_ch(1, &g.voices[g.players[1].character][0], 0.6);
                    g.audio.play_ch(0, &g.voices[g.players[0].character][4], 0.6);
                    g.players[1].voice_timer = 0.5;
                    for k in 0..6 {
                        let who_pos = if k % 2 == 0 { g.players[0].pos } else { g.players[1].pos };
                        g.hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.2, 1.0 + k as f32 * 0.1, who_pos.y + 0.05), 1.6));
                    }
                }
                continue;
            }
            let id = who * 6 + i;
            let play_now = if g.music_mode { g.looper.press(id) } else { true };
            quest_event(&mut g.quests, QuestKind::Sing, 1, &mut g.inv, &mut g.toasts);
            if !g.music_mode {
                let (pos, facing) = (g.players[who].pos, g.players[who].facing);
                let (tx, tz) = g.players[who].ahead();
                cast(i, &mut g.world, pos, facing, &mut g.particles);
                if i == 0 && (tx, tz) == CAMPFIRE && g.camp.fuel > 0.0 && !g.camp.lit {
                    g.camp.lit = true;
                    g.toasts.push(("ember! the fire is lit".to_string(), 2.5));
                }
                if i == 2 && pos.distance(FIRE_SPOT) < 2.5 && g.camp.lit {
                    g.camp.lit = false;
                    g.cooking = false;
                    g.toasts.push(("the rain put the fire out".to_string(), 2.5));
                }
            }
            if play_now {
                let sound = &g.voices[g.players[who].character][i];
                g.audio.play_ch(who, sound, 0.8);
                g.players[who].key_glow[i] = 1.0;
                g.players[who].voice_timer = 0.4;
                if who == 0 && fr.dog_jamming {
                    g.cat_phrase_notes += 1;
                    g.recent_cat.push(i);
                    if g.recent_cat.len() > 8 {
                        g.recent_cat.remove(0);
                    }
                    if !g.partner_soloing && g.world.rand() < 0.6 {
                        g.echoes.push((g.looper.position + 8.0, i));
                    }
                }
            }
        }
    }

    for id in g.looper.tick(get_frame_time()) {
        if id >= 28 && id < 32 {
            g.audio.play_ch(2, g.perc()[id - 28], 0.7);
            continue;
        }
        if id >= 16 {
            g.play_pad((id - 16) / 6, (id - 16) % 6, 0.7, g.players[1].character);
            continue;
        }
        if id >= 12 {
            g.audio.play_ch(2, g.perc()[(id - 12).min(3)], 0.7);
            continue;
        }
        let (who, i) = (id / 6, id % 6);
        let sound = &g.voices[g.players[who].character][i];
        g.audio.play_ch(who, sound, 0.75);
        g.players[who].key_glow[i] = 1.0;
        g.players[who].voice_timer = 0.3;
    }
    fr.position = g.looper.position;
    fr.friend_voice = g.players[1].character;
    g.echoes.retain(|&(at, i)| {
        if fr.position >= at {
            g.audio.play_ch(1, &g.voices[fr.friend_voice][i], 0.45);
            g.players[1].voice_timer = 0.3;
            false
        } else {
            fr.position > at - 64.0
        }
    });
}

pub(crate) fn on_step(g: &mut Game, fr: &mut Frame) {
    fr.step = g.looper.current_step();
    if g.looper.last_step != Some(fr.step) {
        let crossed: Vec<usize> = match g.looper.last_step {
            Some(prev) => {
                let mut v = Vec::new();
                let mut k = (prev + 1) % LOOP_STEPS;
                while k != fr.step && v.len() < LOOP_STEPS {
                    v.push(k);
                    k = (k + 1) % LOOP_STEPS;
                }
                v.push(fr.step);
                v
            }
            None => vec![fr.step],
        };
        g.looper.last_step = Some(fr.step);
        g.sampler.step = fr.step % 16;
        if g.inside && g.house.has(home::Item::Sampler) {
            for row in 0..4 {
                if g.sampler.grid[row][fr.step % 16] {
                    if let Some(pad) = &g.sampler.pads[row] {
                        g.audio.play_private(pad, 0.8);
                    } else {
                        match row {
                            0 => g.audio.play_ch(2, &g.kick, 0.8),
                            1 => g.audio.play_ch(2, &g.clap, 0.55),
                            2 => g.audio.play_ch(2, &g.hat, 0.45),
                            _ => g.audio.play_ch(2, &g.voices[g.players[g.sampler.owner].character][(fr.step / 4) % 6], 0.45),
                        }
                    }
                }
            }
        }
        if (g.ddr.on[0] || g.ddr.on[1]) && fr.time - g.ddr.last_mic > 3.0 && fr.step % 2 == 0 {
            let step_s = g.looper.step_seconds();
            let frac = (g.looper.position - fr.step as f32).rem_euclid(LOOP_STEPS as f32);
            let target = fr.time + (16.0 - frac) * step_s;
            let slot_len = step_s * 2.0;
            if g.looper.beat {
                let style = &BEAT_STYLES[g.beat_style.min(BEAT_STYLES.len() - 1)];
                if velocity(style.kick, fr.step).is_some() {
                    g.ddr.add(0, target, slot_len);
                }
                if velocity(style.clap, fr.step).is_some() {
                    g.ddr.add(1, target, slot_len);
                }
                if velocity(style.open, fr.step).is_some() {
                    g.ddr.add(3, target, slot_len);
                } else if velocity(style.hat, fr.step).is_some() && fr.step % 4 == 2 {
                    g.ddr.add(2, target, slot_len);
                }
            }
            let notes: Vec<usize> = g.looper.notes.iter().filter(|(at, _)| (at.round() as usize) % LOOP_STEPS == (fr.step + 16) % LOOP_STEPS).map(|(_, id)| *id).collect();
            for id in notes {
                g.ddr.add(note_slot(id) % 4, target, slot_len);
            }
        }
        if g.inside && g.house.record_on && g.house.has(home::Item::RecordPlayer) && g.playback.is_none() {
            if crossed.contains(&0) || g.record_tune.is_empty() {
                let tunes = if g.songbook.is_empty() { vec![vec![(0, 0), (4, 2), (8, 4), (12, 3), (16, 2), (20, 1), (24, 0)]] } else { g.songbook.clone() };
                g.record_pick = (g.record_pick + 1) % tunes.len();
                g.record_tune = tunes[g.record_pick].clone();
            }
            for &(at, note) in &g.record_tune {
                if crossed.contains(&at) {
                    g.audio.play_ch(1, &g.voices[g.players[1].character][note], 0.3);
                }
            }
        }
        if g.looper.beat {
            let style = &BEAT_STYLES[g.beat_style.min(BEAT_STYLES.len() - 1)];
            if let Some(v) = velocity(style.kick, fr.step) {
                let sub = if g.house.switched_on(home::Item::Subwoofer) { 1.4 } else { 1.0 };
                g.audio.play_ch(2, &g.kick, (0.75 * v * sub).min(1.0));
            }
            if let Some(v) = velocity(style.clap, fr.step) {
                g.audio.play_ch(2, &g.clap, 0.5 * v);
            }
            if let Some(v) = velocity(style.hat, fr.step) {
                g.audio.play_ch(2, &g.hat, 0.45 * v);
            }
            if let Some(v) = velocity(style.open, fr.step) {
                g.audio.play_ch(2, &g.open_hat, 0.4 * v);
            }
        } else if (g.looper.recording || g.fishing.active) && fr.step % 4 == 0 {
            let accent = if fr.step % 16 == 0 { 0.5 } else { 0.25 };
            g.audio.play_ch(2, &g.hat, accent);
        }
        if fr.dog_jamming && crossed.contains(&0) && g.cat_phrase_notes >= 3 {
            g.inv.acorns += 1;
            g.toasts.push(("nice jam: +1 acorn".to_string(), 1.6));
        }
        if fr.dog_jamming && crossed.contains(&0) {
            let chance = if g.partner_solo_count >= 2 {
                0.15
            } else if g.cat_phrase_notes >= 2 {
                0.75
            } else if g.partner_soloing {
                0.3
            } else {
                0.45
            };
            g.partner_soloing = g.world.rand() < chance;
            g.partner_solo_count = if g.partner_soloing { g.partner_solo_count + 1 } else { 0 };
            g.partner_phrase.clear();
            if g.partner_soloing {
                let partner_mood: &str;
                let pick = |world: &mut World, n: usize| (world.rand() * n as f32) as usize % n.max(1);
                let longest = (2 + g.partner_xp as usize / 2).min(8);
                let roll = g.world.rand();
                let fresh = |world: &mut World, recent: &[usize]| {
                    let mut phrase = Vec::new();
                    let mut at = if world.rand() < 0.5 { 0 } else { 2 };
                    let length = 1 + (world.rand() * longest as f32) as usize;
                    let mut note = recent.last().copied().unwrap_or((world.rand() * 6.0) as usize % 6) as i32;
                    for k in 0..length {
                        if at >= LOOP_STEPS - 1 {
                            break;
                        }
                        let borrow = !recent.is_empty() && world.rand() < 0.35;
                        note = if borrow {
                            recent[k % recent.len()] as i32
                        } else {
                            let moves = [-2, -1, -1, 0, 1, 1, 2];
                            (note + moves[(world.rand() * 7.0) as usize % 7]).clamp(0, 5)
                        };
                        phrase.push((at, note as usize));
                        let gaps = [2, 2, 3, 4, 4, 6, 1];
                        at += gaps[(world.rand() * 7.0) as usize % 7];
                    }
                    phrase
                };
                if g.songbook.is_empty() || (roll < 0.3 && g.songbook.len() < 8) {
                    g.partner_phrase = fresh(&mut g.world, &g.recent_cat);
                    g.songbook.push(g.partner_phrase.clone());
                    partner_mood = if g.songbook.len() == 1 { "a first little tune" } else { "a new tune" };
                } else if roll < 0.7 {
                    let k = pick(&mut g.world, g.songbook.len());
                    let mut tune = g.songbook[k].clone();
                    let change = g.world.rand();
                    if change < 0.4 && tune.len() < longest {
                        let last = tune.last().copied().unwrap_or((0, 2));
                        let gaps = [2, 3, 4];
                        let at = last.0 + gaps[pick(&mut g.world, 3)];
                        if at < LOOP_STEPS - 1 {
                            let moves = [-1, 1, 2, -2];
                            tune.push((at, (last.1 as i32 + moves[pick(&mut g.world, 4)]).clamp(0, 5) as usize));
                        }
                    } else if change < 0.75 && !tune.is_empty() {
                        let j = pick(&mut g.world, tune.len());
                        let moves = [-1, 1];
                        tune[j].1 = (tune[j].1 as i32 + moves[pick(&mut g.world, 2)]).clamp(0, 5) as usize;
                    } else {
                        let shift = if g.world.rand() < 0.5 { -1 } else { 1 };
                        for n in tune.iter_mut() {
                            n.1 = (n.1 as i32 + shift).clamp(0, 5) as usize;
                        }
                    }
                    g.songbook[k] = tune.clone();
                    g.partner_phrase = tune;
                    partner_mood = "growing a tune";
                } else {
                    let k = pick(&mut g.world, g.songbook.len());
                    g.partner_phrase = g.songbook[k].clone();
                    partner_mood = "its favourite tune";
                }
                g.partner_xp += 1;
                g.toasts.push((format!("the {} plays {}", g.cast_list[g.players[1].character].noun, partner_mood), 2.0));
            }
            g.cat_phrase_notes = 0;
        }
        if !fr.dog_jamming {
            g.partner_soloing = false;
            g.partner_phrase.clear();
        }
        if fr.dog_jamming && g.partner_soloing {
            for &(at, note) in &g.partner_phrase {
                if crossed.contains(&at) {
                    g.audio.play_ch(1, &g.voices[g.players[1].character][note], 0.62);
                    g.players[1].voice_timer = 0.3;
                    g.players[1].key_glow[note] = 1.0;
                }
            }
            if fr.step % 4 == 2 && !g.looper.beat && !g.cooking {
                g.audio.play_ch(2, &g.hat, 0.16);
            }
        } else if fr.dog_jamming {
            let chord = [0, 4, 3, 1][(fr.step / 8) % 4];
            if fr.step % 8 == 0 {
                g.audio.play_ch(1, &g.voices[g.players[1].character][chord], 0.42);
                g.players[1].voice_timer = 0.25;
            } else if fr.step % 16 == 11 {
                g.audio.play_ch(1, &g.voices[g.players[1].character][chord], 0.26);
            }
            if fr.step % 4 == 2 && !g.looper.beat && !g.cooking {
                g.audio.play_ch(2, &g.hat, 0.16);
            }
        }
    }
}

pub(crate) fn session_and_playback(g: &mut Game, fr: &mut Frame) {
    fr.together = !g.inside && if g.two_player { g.players[0].seated && g.players[1].seated } else if g.companion { fr.dog_jamming } else { g.jam && g.players[0].seated };
    fr.playing = fr.together && fr.time - g.last_note_at < 8.0;
    match g.session {
        None => {
            if fr.playing {
                g.session = Some(0.0);
                g.session_stars = 0;
                g.audio.start_jam();
            }
        }
        Some(t) => {
            if fr.playing {
                let t = t + fr.dt;
                g.session = Some(t);
                let stars = stars_for(t);
                if stars > g.session_stars {
                    g.session_stars = stars;
                    quest_event(&mut g.quests, QuestKind::Star, 1, &mut g.inv, &mut g.toasts);
                    let label = ["", "30 seconds together - 1 star!", "1 minute together - 2 stars!", "3 minutes together - 3 stars!!"][stars as usize];
                    g.toasts.push((label.to_string(), 3.0));
                    for k in 0..3 {
                        g.audio.play_ch(3, &g.bells[(k * 2 + stars as usize) % 6], 0.35);
                    }
                    for k in 0..(4 * stars as usize) {
                        let who_pos = if k % 2 == 0 { g.players[0].pos } else { g.players[1].pos };
                        g.hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.25, 1.0 + k as f32 * 0.08, who_pos.y + 0.05), 1.8));
                    }
                }
            } else {
                let events = g.audio.stop_jam();
                if t >= 30.0 {
                    g.diary.push(DiaryEntry {
                        when: miniquad::date::now(),
                        seconds: t as u32,
                        stars: stars_for(t),
                        who: (g.players[0].character, g.players[1].character),
                        events,
                    });
                    g.toasts.push((format!("the jam was written in the diary - {} star{}", stars_for(t), if stars_for(t) == 1 { "" } else { "s" }), 3.5));
                }
                g.session = None;
                g.session_stars = 0;
            }
        }
    }
    if g.first_steps == 3 && g.session_stars > 0 {
        g.first_steps = 4;
        g.toasts.push(("your first jam! explore the house and make more music".to_string(), 4.0));
    }
    if let Some((idx, start, next)) = g.playback {
        if !g.inside || idx >= g.diary.len() {
            g.playback = None;
        } else {
            let events = &g.diary[idx].events;
            let t = (get_time() - start) as f32;
            let mut k = next;
            while k < events.len() && events[k].0 <= t {
                g.audio.play_id(events[k].1, events[k].2 * 0.85);
                k += 1;
            }
            if k >= events.len() && t > events.last().map_or(0.0, |e| e.0) + 2.0 {
                g.playback = None;
                g.toasts.push(("the record ends".to_string(), 2.0));
            } else {
                g.playback = Some((idx, start, k));
            }
        }
    }
}
