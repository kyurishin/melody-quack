#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// First-steps guide, picker, sleep fade, sampler, arcade and help.
pub(crate) fn draw_overlays(g: &mut Game, fr: &mut Frame) {
    if !g.picking && !g.show_help && g.first_steps > 0 && g.first_steps < 4 {
        let (heading, message) = match g.first_steps {
            1 => ("your first song - 1/3", "press any note 1-6 to sing"),
            2 if g.camp.lit => ("find the fire - 2/3", "walk to the campfire and press E to sit"),
            2 => ("find the fire - 2/3", "explore while your friend brings firewood"),
            _ => ("make a memory - 3/3", "play music together to earn a star"),
        };
        let w = (low_w() - 16.0).min(250.0);
        let x = (low_w() - w) / 2.0;
        let y = if fr.touch_ui && fr.portrait { 27.0 } else { 21.0 };
        ui_rect(x, y, w, 30.0, Color::new(0.1, 0.08, 0.2, 0.83));
        ui_text(heading, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 226, 150, 255));
        let line = ui_wrap(message, 10.0, w - 16.0);
        if let Some(first) = line.first() {
            ui_text(first, x + 8.0, y + 24.0, 10.0, WHITE);
        }
    }

    if g.picking {
        let w = 230.0f32.min(low_w() - 10.0);
        let preview_h = (w - 12.0) * 130.0 / 300.0;
        let top = if g.first_pick { 24.0 } else { 30.0 };
        let x = (low_w() - w) / 2.0;
        let h = preview_h + if g.first_pick { 100.0 } else { 76.0 };
        ui_rect(x, top, w, h, Color::new(0.1, 0.08, 0.2, 0.94));
        let mut y = top + 16.0;
        if g.first_pick {
            let title = "melody quack";
            let tw = ui_measure(title, 22.0);
            ui_text(title, (low_w() - tw) / 2.0, y + 6.0, 22.0, Color::from_rgba(255, 226, 150, 255));
            y += 24.0;
        }
        {
            let (kx, ky) = ui_scale();
            let px = x + 6.0;
            let py = y - 8.0;
            draw_texture_ex(&g.pick_view.texture, px * kx, py * ky, WHITE, DrawTextureParams { dest_size: Some(vec2((w - 12.0) * kx, preview_h * ky)), flip_y: true, ..Default::default() });
            y += preview_h + 6.0;
        }
        let labels = if g.two_player { ["player 1", "player 2"] } else { ["you", "friend"] };
        for row in 0..2 {
            let chosen = row == g.pick_row;
            let col = if chosen { Color::from_rgba(255, 236, 170, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) };
            if chosen {
                ui_rect(x + 6.0, y - 9.0, w - 12.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.1));
            }
            ui_text(labels[row], x + 12.0, y, 12.0, col);
            let name = format!("<  {}  >", g.cast_list[g.players[row].character].name);
            ui_text(&name, x + 80.0, y, 12.0, col);
            y += 14.0;
        }
        ui_text("W/S row   A/D change   E start", x + 12.0, y + 6.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
    }

    if g.sleep_fade > 0.0 {
        g.sleep_fade -= get_frame_time();
        let a = (1.0 - (g.sleep_fade - 1.5).abs() / 1.5).clamp(0.0, 1.0);
        ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.03, 0.1, a * 0.92));
        let z = if (fr.time * 2.0) as i32 % 2 == 0 { "z z z" } else { "z Z z" };
        ui_text(z, low_w() / 2.0 - ui_measure(z, 16.0) / 2.0, low_h() / 2.0, 16.0, Color::new(1.0, 1.0, 1.0, a));
        if a > 0.6 {
            let line = "sweet dreams - game saved";
            ui_text(line, low_w() / 2.0 - ui_measure(line, 11.0) / 2.0, low_h() / 2.0 + 18.0, 11.0, Color::new(1.0, 1.0, 1.0, a * 0.8));
        }
    }
    if g.sampler.open {
        let w = (low_w() - 16.0).min(330.0);
        let x = ((low_w() - w) / 2.0).round();
        let cell = ((w - 60.0) / 16.0).floor();
        let h = 34.0 + cell * 4.0 + 34.0;
        let y = low_h() - h - 44.0;
        ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.94));
        ui_text("sampler", x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
        let bpm = format!("{:.0} bpm", g.looper.bpm);
        ui_text(&bpm, x + w - 8.0 - ui_measure(&bpm, 10.0), y + 13.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
        let gx = x + 52.0;
        let gy = y + 22.0;
        for row in 0..4 {
            let col = note_color(row);
            let label = SAMPLER_ROWS[row];
            ui_text(label, x + 8.0, gy + row as f32 * cell + cell * 0.7, 9.0, col);
            if g.sampler.pads[row].is_some() {
                ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 120, 150, 255));
                ui_text("mic", x + 35.0, gy + row as f32 * cell + cell * 0.68, 6.0, WHITE);
            }
            if matches!(g.sampler.recording, Some((r, _)) if r == row) && (fr.time * 4.0).fract() < 0.6 {
                ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 60, 70, 255));
            }
            for k in 0..16 {
                let cx = gx + k as f32 * cell;
                let cy = gy + row as f32 * cell;
                let on = g.sampler.grid[row][k];
                let beat = k % 4 == 0;
                let base = if on { col } else if beat { Color::new(1.0, 1.0, 1.0, 0.16) } else { Color::new(1.0, 1.0, 1.0, 0.08) };
                ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, base);
                if k == g.sampler.step {
                    ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, Color::new(1.0, 1.0, 1.0, if on { 0.45 } else { 0.12 }));
                }
                if (k, row) == g.sampler.cursor {
                    let c = Color::from_rgba(255, 226, 150, 255);
                    ui_rect(cx, cy, cell, 1.0, c);
                    ui_rect(cx, cy + cell - 1.0, cell, 1.0, c);
                    ui_rect(cx, cy, 1.0, cell, c);
                    ui_rect(cx + cell - 1.0, cy, 1.0, cell, c);
                }
            }
        }
        let hint = if g.sampler.recording.is_some() {
            "recording - make a sound into your mic"
        } else if g.sampler.owner == 0 {
            "arrows/WASD move  E step on/off  Space record your sound  X close"
        } else {
            "IJKL move  O step on/off  Space record your sound  U close"
        };
        ui_text(hint, x + (w - ui_measure(hint, 9.0)) / 2.0, y + h - 18.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.8));
        let note = "record only your own voice or sounds - recordings stay on this device and are left out of tapes";
        let mut shown = note.to_string();
        while ui_measure(&shown, 7.0) > w - 12.0 && shown.len() > 10 {
            shown.pop();
        }
        ui_text(&shown, x + (w - ui_measure(&shown, 7.0)) / 2.0, y + h - 6.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.5));
    }
    if g.ddr.on[0] || g.ddr.on[1] {
        let active: Vec<usize> = (0..2).filter(|w| g.ddr.on[*w]).collect();
        for (slot, &who) in active.iter().enumerate() {
            let cx = if active.len() == 2 { low_w() * (0.17 + slot as f32 * 0.66) } else { low_w() * 0.78 };
            let lane_w = 22.0;
            let x0 = cx - lane_w * 2.0;
            let top = 34.0;
            ui_rect(x0 - 4.0, top - 8.0, lane_w * 4.0 + 8.0, low_h() - top - 36.0, Color::new(0.04, 0.03, 0.1, 0.55));
            for lane in 0..4 {
                let lx = x0 + lane as f32 * lane_w + 2.0;
                let f = g.ddr.flash[who][lane];
                let ring: Vec<(usize, usize, Color)> = g.ddr_art[lane].iter().map(|(x, y, _)| (*x, *y, Color::new(0.7 + 0.3 * f, 0.7 + 0.3 * f, 0.75 + 0.25 * f, 0.35 + 0.65 * f))).collect();
                draw_pixel_grid(&ring, lx, top, 2.0, |c| c);
            }
            for a in g.ddr.arrows.iter().filter(|a| !a.hit[who]) {
                let y = top + (a.at - fr.time) * 75.0;
                if y < top - 20.0 || y > low_h() - 40.0 {
                    continue;
                }
                let fade = if a.missed[who] { 0.35 } else { 1.0 };
                draw_pixel_grid(&g.ddr_art[a.lane], x0 + a.lane as f32 * lane_w + 2.0, y, 2.0, |c| Color::new(c.r, c.g, c.b, fade));
            }
            let score = format!("{}", g.ddr.score[who]);
            ui_text(&score, cx - ui_measure(&score, 11.0) / 2.0, top - 12.0, 11.0, Color::from_rgba(255, 226, 150, 255));
            if let Some((word, col, left)) = g.ddr.judge[who] {
                let size = 13.0 + 3.0 * (left - 0.4).max(0.0) * 5.0;
                ui_text(word, cx - ui_measure(word, size) / 2.0, top + 38.0, size, Color::new(col.r, col.g, col.b, (left / 0.3).min(1.0)));
            }
            if g.ddr.combo[who] >= 4 {
                let combo = format!("{} combo", g.ddr.combo[who]);
                ui_text(&combo, cx - ui_measure(&combo, 10.0) / 2.0, top + 52.0, 10.0, WHITE);
            }
        }
        let source = if fr.time - g.ddr.last_mic < 3.0 { "arrows from the music on your mic" } else if g.microphone.is_some() { "listening... arrows follow the game beat for now" } else { "no mic - arrows follow the game beat and your loop" };
        ui_text(source, (low_w() - ui_measure(source, 9.0)) / 2.0, low_h() - 30.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
    }
    if g.show_help {
        let pages: [(&str, &[(&str, &str)]); 3] = [
            ("basics", &[
                ("WASD", "move  (player 2: IJKL)"),
                ("E", "act  (player 2: O)"),
                ("M", "bag, quests, furniture  (player 2: Backspace)"),
                ("Z", "emoji  (player 2: .)"),
                ("P", "1 or 2 players"),
                ("V", "choose characters"),
                ("Esc", "pause, mixer, fishdex"),
                ("H / Esc", "close this help"),
            ]),
            ("music", &[
                ("1 - 6", "sing  (player 2: 7 8 9 0 - =)"),
                ("Tab", "spells / music"),
                ("R / C", "record / clear  (player 2: Y)"),
                ("Q", "quantise"),
                ("B", "beat on / off"),
                ("F", "follow the mic tempo"),
                ("G", "echo on / off"),
                ("T", "tape: record a .wav"),
                ("shift+C", "friend forgets all tunes"),
            ]),
            ("world", &[
                ("E at fire", "sit, again: cook"),
                ("E at dock", "cast and fish"),
                ("E by trees", "stick, mushroom, acorns"),
                ("E in yard", "build walls and floors"),
                ("X", "change build tool"),
                ("E at door", "go inside"),
                ("E / X inside", "use / move furniture"),
                ("N twice", "fresh world"),
            ]),
        ];
        let (x, y, w, h) = help_box();
        ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.03, 0.02, 0.08, 0.55));
        ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.97));
        ui_rect(x, y, w, 3.0, Color::from_rgba(255, 214, 110, 255));
        ui_text("help", x + 10.0, y + 15.0, 13.0, Color::from_rgba(255, 226, 150, 255));
        let tab_w = (w - 20.0) / 3.0;
        for (k, (name, _)) in pages.iter().enumerate() {
            let tx = x + 10.0 + k as f32 * tab_w;
            let on = k == g.help_page;
            ui_rect(tx + 1.0, y + 20.0, tab_w - 2.0, 16.0, if on { Color::from_rgba(255, 214, 110, 255) } else { Color::new(1.0, 1.0, 1.0, 0.1) });
            ui_text(name, tx + (tab_w - ui_measure(name, 11.0)) / 2.0, y + 32.0, 11.0, if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE });
        }
        let rows = pages[g.help_page].1;
        let key_w = rows.iter().map(|(k, _)| ui_measure(k, 11.0)).fold(0.0, f32::max) + 10.0;
        for (n, (key, text)) in rows.iter().enumerate() {
            let ry = y + 44.0 + n as f32 * 16.0;
            ui_rect(x + 10.0, ry, ui_measure(key, 11.0) + 8.0, 13.0, Color::new(1.0, 0.84, 0.43, 0.2));
            ui_text(key, x + 14.0, ry + 10.0, 11.0, Color::from_rgba(255, 226, 150, 255));
            let mut t = text.to_string();
            while ui_measure(&t, 11.0) > w - key_w - 24.0 && t.len() > 4 {
                t.pop();
            }
            ui_text(&t, x + 16.0 + key_w, ry + 10.0, 11.0, WHITE);
        }
        let foot = if radio::touch_ui() { "tap a tab - tap outside to close" } else { "left/right: page   H or Esc: close" };
        ui_text(foot, x + (w - ui_measure(foot, 9.0)) / 2.0, y + h - 6.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.6));
    }
}
