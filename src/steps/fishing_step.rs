#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn fishing(g: &mut Game, fr: &mut Frame) {
    if g.fishing.active && g.fish_card <= 0.0 {
        let fish_window = 0.22 * g.fishing.tight / g.looper.step_seconds();
        let perfect_window = 0.09 * g.fishing.tight / g.looper.step_seconds();
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            if !fr.fish_press[who] {
                continue;
            }
            let d = g.looper.position - g.fishing.cue_at;
            if d.abs() <= fish_window && !g.fishing.hit_by[who] && !g.fishing.early[who] {
                g.fishing.hit_by[who] = true;
                g.fishing.dip = 1.0;
                g.audio.play_ch(3, &g.plink, 0.7);
                for k in 0..10 {
                    let a = k as f32 / 10.0 * 2.0 * PI;
                    spawn(&mut g.particles, BOBBER + vec3(0.0, 0.05, 0.0), vec3(a.cos() * 0.8, 1.6, a.sin() * 0.8), 0.6, Color::from_rgba(200, 230, 255, 255), 0.025, 0.5);
                }
                let both = g.fishing.hit_by[0] && g.fishing.hit_by[1];
                let (word, col) = if both {
                    ("together!", Color::from_rgba(255, 150, 190, 255))
                } else if d.abs() <= perfect_window {
                    g.fishing.perfects += 1;
                    ("perfect!", Color::from_rgba(255, 226, 120, 255))
                } else {
                    ("nice!", Color::from_rgba(170, 230, 255, 255))
                };
                g.fish_say = Some((word.to_string(), col, 1.0));
            } else if d < -fish_window && d > -4.0 && !g.fishing.hit_by[who] {
                g.fishing.early[who] = true;
                g.fish_say = Some(("too early - wait for the ring".to_string(), Color::from_rgba(255, 190, 150, 255), 1.2));
            }
        }
        if g.looper.position > g.fishing.cue_at + fish_window {
            let hit = g.fishing.hit_by[0] || g.fishing.hit_by[1];
            let both = g.fishing.hit_by[0] && g.fishing.hit_by[1];
            let friend_on_dock = !g.two_player && g.players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8;
            let helped = !hit && !g.fishing.lesson && friend_on_dock && hash((fr.time * 29.0) as i32, 4, 8) < 0.5;
            if !hit && g.fishing.lesson {
                let key = if g.fishing.caster == 0 { "E" } else { "O" };
                g.fish_say = Some((format!("almost! press {key} as the ring touches the bobber"), Color::from_rgba(255, 210, 160, 255), 1.6));
            } else if hit || helped {
                g.fishing.hits += if both { 2 } else { 1 };
                if helped {
                    g.fishing.dip = 1.0;
                    g.audio.play_ch(1, &g.voices[g.players[1].character][2], 0.5);
                    g.players[1].voice_timer = 0.4;
                    let line = format!("the {} tugged the line for you!", g.cast_list[g.players[1].character].noun);
                    g.fish_say = Some((line, Color::from_rgba(200, 240, 170, 255), 1.6));
                }
                if both {
                    for k in 0..3 {
                        g.hearts.push((vec3(g.players[k % 2].pos.x, 1.0 + k as f32 * 0.12, g.players[k % 2].pos.y), 1.4));
                    }
                }
            } else {
                g.fishing.tries = g.fishing.tries.saturating_sub(1);
                g.audio.play_ch(3, &g.splash, 0.35);
                if g.fishing.tries > 0 {
                    g.fish_say = Some((format!("missed - {} {} left", g.fishing.tries, if g.fishing.tries == 1 { "try" } else { "tries" }), Color::from_rgba(255, 190, 150, 255), 1.4));
                }
            }
            if g.fishing.hits >= 3 {
                let roll = hash((fr.time * 100.0) as i32, 7, 7);
                let roll2 = hash((fr.time * 61.0) as i32, 3, 13);
                let luck = (g.fishing.perfects as f32 / 3.0).min(1.0) + if g.fishing.hit_by[0] && g.fishing.hit_by[1] { 0.3 } else { 0.0 };
                let species = if g.fishing.lesson { fish::pick(roll * 0.5, roll2, 0.0) } else { fish::pick(roll, roll2, luck) };
                let kind = fish::kind(species);
                let is_new = g.dex_counts[species] == 0;
                g.dex_counts[species] += 1;
                g.fish_pop = Some((species, 3.0, is_new, false));
                g.inv.fish[kind] += 1;
                quest_event(&mut g.quests, QuestKind::Fish, 1, &mut g.inv, &mut g.toasts);
                g.inv.acorns += fish::acorns(species);
                g.fishing.active = false;
                g.fishing.jump = Some((kind, 0.0));
                g.audio.play_ch(3, &g.splash, 0.6);
                let singer = if g.fishing.hit_by[1] { 1 } else { g.fishing.caster };
                g.audio.play_ch(singer, &g.voices[g.players[singer].character][5], 0.7);
                g.players[singer].voice_timer = 0.5;
                let who = if g.fishing.hit_by[0] && g.fishing.hit_by[1] { "you both caught" } else if g.fishing.hit_by[1] || (g.fishing.caster == 1 && !g.fishing.hit_by[0]) { "player 2 caught" } else { "caught" };
                g.toasts.push((format!("{who} a {}! +{} acorns", fish::SPECIES[species], fish::acorns(species)), 3.0));
                if g.fishing.lesson {
                    g.fish_lesson = true;
                    g.save_clock = 99.0;
                    g.fish_say = Some(("your first fish!".to_string(), Color::from_rgba(255, 226, 120, 255), 2.0));
                }
                if friend_on_dock && hash((fr.time * 37.0) as i32, 3, 11) < 0.55 {
                    let bonus = fish::pick(hash((fr.time * 53.0) as i32, 5, 2) * 0.8, hash((fr.time * 71.0) as i32, 2, 9), 0.0);
                    g.dex_counts[bonus] += 1;
                    g.inv.fish[fish::kind(bonus)] += 1;
                    g.inv.acorns += 2;
                    g.audio.play_ch(1, &g.voices[g.players[1].character][4], 0.6);
                    g.players[1].voice_timer = 0.5;
                    g.toasts.push((format!("the {} caught a {} too! +2 acorns", g.cast_list[g.players[1].character].noun, fish::SPECIES[bonus]), 3.0));
                }
            } else if g.fishing.tries == 0 {
                g.fishing.active = false;
                g.audio.play_ch(3, &g.splash, 0.5);
                g.toasts.push(("the fish got away - cast again!".to_string(), 2.5));
            } else {
                let roll = hash((fr.time * 47.0) as i32, 9, 3);
                let gap = if g.fishing.lesson { FISH_CUE_GAP } else if roll < 0.25 { FISH_CUE_GAP - 2.0 } else if roll > 0.8 { FISH_CUE_GAP + 2.0 } else { FISH_CUE_GAP };
                g.fishing.cue_at += gap;
                g.fishing.hit_by = [false; 2];
                g.fishing.early = [false; 2];
                if !g.fishing.lesson {
                    g.fishing.tight = 0.72 + 0.2 * hash((fr.time * 43.0) as i32, 1, 8);
                }
            }
        }
    }
    g.fish_card = (g.fish_card - fr.dt).max(0.0);
    g.cook_card = (g.cook_card - fr.dt).max(0.0);
    g.gather_card = (g.gather_card - fr.dt).max(0.0);
    if let Some(say) = g.fish_say.as_mut() {
        say.2 -= fr.dt;
    }
    if g.fish_say.as_ref().map(|s| s.2 <= 0.0).unwrap_or(false) {
        g.fish_say = None;
    }
    g.fishing.dip = (g.fishing.dip - fr.dt * 4.0).max(0.0);
    if let Some((kind, t)) = g.fishing.jump {
        g.fishing.jump = if t + fr.dt > 1.0 { None } else { Some((kind, t + fr.dt)) };
    }
}
