#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// The dance arcade.
pub(crate) fn dance_arcade(g: &mut Game, fr: &mut Frame) {
    if let Some((who, i)) = g.ddr_request.take() {
        if !g.ddr.on[who] && i < g.house.pieces.len() {
            let piece = g.house.pieces[i];
            let front = rotate_y(vec3(0.0, 0.0, 0.95), piece.yaw());
            let side = if g.two_player { if who == 0 { -0.22 } else { 0.22 } } else { 0.0 };
            let across = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
            let spot = piece.center() + vec2(front.x + across.x, front.z + across.z);
            g.players[who].pos = home::world(spot);
            g.players[who].facing = vec2(-front.x, -front.z).normalize_or_zero();
            if !g.ddr.on[0] && !g.ddr.on[1] {
                g.ddr.arrows.clear();
            }
            g.ddr.on[who] = true;
            g.ddr.score[who] = 0;
            g.ddr.combo[who] = 0;
            if g.microphone.is_none() {
                g.microphone = mic::start();
            }
            if g.microphone.is_some() && !g.listening {
                g.listening = true;
                g.locked = false;
                g.bpm_history.clear();
                g.mic_message = None;
            }
            if !g.looper.beat && g.microphone.is_none() {
                g.looper.beat = true;
            }
            g.music_mode = true;
            g.audio.play_ch(3, &g.bells[4], 0.4);
            let keys = if who == 0 { "arrow keys or WASD" } else { "J K I L" };
            g.toasts.push((format!("dance! step with {keys} when the arrows reach the top - {} to stop", if who == 0 { "X" } else { "U" }), 3.5));
        }
    }
    if g.ddr.on[0] || g.ddr.on[1] {
        if !g.inside {
            g.ddr.on = [false; 2];
        }
        let step_s = g.looper.step_seconds();
        if !g.looper.beat && fr.time - g.ddr.last_mic > 4.0 && g.looper.notes.is_empty() {
            g.looper.beat = true;
        }
        if g.listening {
            if let Some(m) = &g.microphone {
                let hits: Vec<(usize, f32)> = m.detector.lock().map(|mut d| d.hits.drain(..).collect()).unwrap_or_default();
                for (band, _) in hits {
                    g.ddr.last_mic = fr.time;
                    let p = g.looper.position;
                    let snapped = (p / 2.0).round() * 2.0;
                    let target = fr.time + (snapped - p) * step_s + 16.0 * step_s;
                    g.ddr.add(DDR_LANE_OF_BAND[band], target, step_s * 2.0);
                }
            }
        }
        for who in 0..2 {
            if !g.ddr.on[who] {
                continue;
            }
            let keys: [bool; 4] = if who == 0 {
                [
                    is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
                    is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
                    is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
                    is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
                ]
            } else {
                [is_key_pressed(KeyCode::J), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::L)]
            };
            let quit = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
            if quit {
                g.ddr.on[who] = false;
                g.toasts.push((format!("player {} scored {}", who + 1, g.ddr.score[who]), 2.5));
                continue;
            }
            for lane in 0..4 {
                if !keys[lane] {
                    continue;
                }
                g.ddr.flash[who][lane] = 1.0;
                let best = g.ddr.arrows.iter().enumerate().filter(|(_, a)| a.lane == lane && !a.hit[who] && !a.missed[who] && (a.at - fr.time).abs() < 0.18).min_by(|a, b| (a.1.at - fr.time).abs().partial_cmp(&(b.1.at - fr.time).abs()).unwrap()).map(|(n, a)| (n, (a.at - fr.time).abs()));
                if let Some((n, off)) = best {
                    g.ddr.arrows[n].hit[who] = true;
                    g.ddr.combo[who] += 1;
                    let (word, col, points) = if off < 0.05 {
                        ("perfect", Color::from_rgba(255, 226, 120, 255), 300)
                    } else if off < 0.1 {
                        ("great", Color::from_rgba(140, 230, 170, 255), 200)
                    } else {
                        ("good", Color::from_rgba(170, 210, 255, 255), 100)
                    };
                    g.ddr.score[who] += points + g.ddr.combo[who].min(50) * 2;
                    g.ddr.judge[who] = Some((word, col, 0.6));
                    quest_event(&mut g.quests, QuestKind::Dance, 1, &mut g.inv, &mut g.toasts);
                    g.players[who].voice_timer = 0.15;
                    g.audio.play_ch(who, &g.voices[g.players[who].character][lane + 1], 0.18);
                }
            }
            for a in g.ddr.arrows.iter_mut() {
                if !a.hit[who] && !a.missed[who] && fr.time - a.at > 0.18 {
                    a.missed[who] = true;
                    g.ddr.combo[who] = 0;
                    g.ddr.judge[who] = Some(("miss", Color::from_rgba(255, 140, 140, 255), 0.6));
                }
            }
        }
        g.ddr.arrows.retain(|a| fr.time - a.at < 0.5);
    } else {
        g.ddr.arrows.clear();
    }
    for who in 0..2 {
        for lane in 0..4 {
            g.ddr.flash[who][lane] = (g.ddr.flash[who][lane] - fr.dt * 5.0).max(0.0);
        }
        if let Some(j) = g.ddr.judge[who].as_mut() {
            j.2 -= fr.dt;
        }
        if g.ddr.judge[who].map(|j| j.2 <= 0.0).unwrap_or(false) {
            g.ddr.judge[who] = None;
        }
    }
}
