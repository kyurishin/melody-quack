#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use crate::strudel::{self, Program};
use steps::venues::{button, held_repeat, hit, shortcut_held};

pub(crate) const MAX_CODE: usize = 8000;

pub(crate) const SPOT: (f32, f32) = (5.5, 5.35);

#[derive(Default)]
pub(crate) struct CodeDesk {
    pub(crate) text: String,
    pub(crate) cursor: usize,
    pub(crate) mine: Option<Program>,
    pub(crate) room: Option<(String, Program)>,
    pub(crate) error: Option<(usize, String)>,
    pub(crate) last: f64,
    pub(crate) flash: Vec<f64>,
    pub(crate) sharing: bool,
    pub(crate) scroll: usize,
}

pub(crate) fn solid() -> (f32, f32, f32, f32) {
    (SPOT.0 - 0.45, SPOT.1 - 0.3, SPOT.0 + 0.45, SPOT.1 + 0.3)
}

pub(crate) fn near(local: Vec2) -> bool {
    local.distance(vec2(SPOT.0, SPOT.1 + 0.75)) < 0.8 || local.distance(vec2(SPOT.0, SPOT.1 - 0.75)) < 0.8
}

pub(crate) fn draw(b: &mut Batch, o: Vec3, time: f32, playing: bool) {
    let p = o + vec3(SPOT.0, 0.0, SPOT.1);
    let dark = Color::from_rgba(34, 30, 44, 255);
    b.cube(p + vec3(0.0, 0.36, 0.0), vec3(0.86, 0.06, 0.5), Color::from_rgba(60, 54, 74, 255));
    for (dx, dz) in [(-0.38, -0.2), (0.38, -0.2), (-0.38, 0.2), (0.38, 0.2)] {
        b.cube(p + vec3(dx, 0.17, dz), vec3(0.05, 0.34, 0.05), dark);
    }
    b.cube(p + vec3(0.0, 0.62, -0.08), vec3(0.5, 0.42, 0.34), Color::from_rgba(214, 206, 186, 255));
    let glow = if playing { 0.75 + 0.25 * (time * 8.0).sin().abs() } else { 0.55 };
    b.unlit = true;
    b.cube(p + vec3(0.0, 0.64, 0.095), vec3(0.4, 0.3, 0.01), Color::new(0.1 * glow, 0.9 * glow, 0.35 * glow, 1.0));
    for k in 0..3 {
        let w = 0.12 + ((time * 3.0 + k as f32).sin() * 0.5 + 0.5) * 0.18 * if playing { 1.0 } else { 0.3 };
        b.cube(p + vec3(-0.14 + w / 2.0, 0.72 - k as f32 * 0.07, 0.1), vec3(w, 0.025, 0.01), Color::from_rgba(200, 255, 140, 255));
    }
    b.unlit = false;
    b.cube(p + vec3(0.0, 0.405, 0.15), vec3(0.46, 0.03, 0.14), Color::from_rgba(196, 188, 170, 255));
}

fn sound_for<'a>(g: &'a Game, name: &str, note: Option<usize>) -> Option<(&'a audio::Sound, usize)> {
    let n = note.unwrap_or(0).min(5);
    let drum = |s: &'a audio::Sound| Some((s, 2usize));
    match name {
        "bd" | "kick" => drum(&g.kick),
        "sd" | "sn" | "cp" | "clap" => drum(&g.clap),
        "hh" | "hat" | "ch" | "sh" | "shaker" => drum(&g.hat),
        "oh" | "rd" | "ride" => drum(&g.open_hat),
        "lt" | "mt" | "ht" | "tom" => drum(&g.tom),
        "rim" | "rs" | "perc" | "cb" => drum(&g.rim),
        "cr" | "crash" => drum(&g.splash),
        "bass" => drum(&g.bass),
        "lid" => drum(&g.lid),
        "chop" => drum(&g.chop),
        "sizzle" => drum(&g.sizzle),
        "splash" => drum(&g.splash),
        "crackle" => drum(&g.crackle),
        "plink" => drum(&g.plink),
        "piano" | "keys" | "superpiano" | "rhodes" | "epiano" => Some((&g.piano[n], 3)),
        "marimba" | "sawtooth" | "saw" | "square" | "supersaw" | "pulse" | "vibraphone" | "xylophone" => Some((&g.marimba[n], 3)),
        "bell" | "bells" | "sine" | "triangle" | "tri" | "glockenspiel" | "music_box" => Some((&g.bells[n], 3)),
        other if other.starts_with("gm_") && other.contains("piano") => Some((&g.piano[n], 3)),
        other if other.starts_with("gm_") && (other.contains("bell") || other.contains("celesta") || other.contains("glock")) => Some((&g.bells[n], 3)),
        other if other.starts_with("gm_") => Some((&g.marimba[n], 3)),
        "stone" | "stones" => Some((&g.stones[n], 3)),
        "voice" | "me" => Some((&g.voices[g.players[0].character][n], 3)),
        other => {
            let k = g.cast_list.iter().position(|c| c.noun == other || c.name == other || voice_word(c.voice) == other)?;
            Some((&g.voices[k][n], 3))
        }
    }
}

fn cps(g: &Game, p: &Program) -> f64 {
    p.cps.unwrap_or(g.venue.bpm_now.max(60.0) as f64 / 240.0)
}

pub(crate) fn room_live(g: &Game) -> bool {
    g.venue.code.room.is_some()
}

pub(crate) fn tick(g: &mut Game) {
    if g.venue.at != venues::Venue::Chapel {
        g.venue.code.mine = None;
        return;
    }
    let now = get_time();
    let mut programs: Vec<Program> = Vec::new();
    if let Some((_, p)) = &g.venue.code.room {
        programs.push(p.clone());
    }
    if let Some(p) = &g.venue.code.mine {
        if g.venue.code.room.as_ref().map(|(_, r)| r != p).unwrap_or(true) {
            programs.push(p.clone());
        }
    }
    let last = g.venue.code.last;
    g.venue.code.last = now;
    if programs.is_empty() || last <= 0.0 || now - last > 0.5 {
        return;
    }
    for p in programs {
        let c = cps(g, &p);
        let from = (last - g.venue.rave_start) * c;
        let to = (now - g.venue.rave_start) * c;
        let mut cycle = from.floor() as i64;
        while (cycle as f64) < to {
            for h in strudel::hits(&p, cycle) {
                let t = cycle as f64 + h.at;
                if t >= from && t < to {
                    if let Some((sound, ch)) = sound_for(g, &h.sound, h.note) {
                        g.audio.play_ch(ch, sound, 0.8 * h.gain);
                    }
                    let fl = &mut g.venue.code.flash;
                    if fl.len() <= h.line {
                        fl.resize(h.line + 1, 0.0);
                    }
                    fl[h.line] = now;
                }
            }
            cycle += 1;
        }
    }
}

pub(crate) fn on_message(g: &mut Game, parts: &[&str], line: &str) -> bool {
    if parts.first().copied() != Some("strudel") {
        return false;
    }
    let text = strudel::unescape(line.strip_prefix("strudel ").unwrap_or(""));
    let who = g.venue.on_air.as_ref().map(|o| o.name.clone()).unwrap_or_else(|| "a dj".to_string());
    match strudel::parse(&text) {
        Ok(p) => {
            if g.venue.code.room.is_none() {
                g.toasts.push((format!("dj {who} is live coding"), 3.0));
            }
            g.venue.code.room = Some((text, p));
        }
        Err(_) => {}
    }
    true
}

pub(crate) fn open(g: &mut Game) {
    if g.venue.code.text.is_empty() {
        g.venue.code.text = strudel::EXAMPLE.to_string();
        g.venue.code.cursor = g.venue.code.text.len();
    }
    g.venue.overlay = venues::Overlay::Code;
    g.venue.opened_at = get_time();
    while get_char_pressed().is_some() {}
}

fn can_share(g: &Game) -> bool {
    g.venue.admin || g.venue.dj_name.is_some()
}

fn run(g: &mut Game) {
    match strudel::parse(&g.venue.code.text) {
        Ok(p) => {
            g.venue.code.error = None;
            g.venue.code.mine = Some(p);
            if g.venue.code.sharing {
                net::send(&format!("strudel {}", strudel::escape(&g.venue.code.text)));
            }
        }
        Err(e) => g.venue.code.error = Some(e),
    }
}

fn stop(g: &mut Game) {
    g.venue.code.mine = None;
    if g.venue.code.sharing {
        g.venue.code.sharing = false;
        net::send("offair");
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            v.push(i + 1);
        }
    }
    v
}

fn prev_char(text: &str, i: usize) -> usize {
    text[..i].char_indices().last().map(|(k, _)| k).unwrap_or(0)
}

fn next_char(text: &str, i: usize) -> usize {
    text[i..].chars().next().map(|c| i + c.len_utf8()).unwrap_or(i)
}

fn edit(g: &mut Game, settled: bool) {
    let d = &mut g.venue.code;
    let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
    d.cursor = d.cursor.min(d.text.len());
    while let Some(ch) = get_char_pressed() {
        if settled && !ch.is_control() && !shortcut_held() && d.text.len() < MAX_CODE {
            d.text.insert(d.cursor, ch);
            d.cursor += ch.len_utf8();
        }
    }
    let before = d.text.len();
    if let Some(text) = net::pasted() {
        let clean: String = text.replace("\r\n", "\n").replace('\r', "\n").replace('\t', "  ").chars().filter(|c| *c == '\n' || !c.is_control()).collect();
        if d.text.trim() == strudel::EXAMPLE.trim() && !clean.trim().is_empty() {
            d.text = clean;
            d.cursor = d.text.len();
            d.scroll = 0;
        } else if before + clean.len() <= MAX_CODE {
            d.text.insert_str(d.cursor, &clean);
            d.cursor += clean.len();
        } else if clean.len() <= MAX_CODE {
            d.text = clean;
            d.cursor = d.text.len();
        } else {
            d.error = Some((0, format!("that's too long to paste - {MAX_CODE} characters at most")));
        }
    }
    if !settled {
        return;
    }
    let starts = line_starts(&d.text);
    let row = starts.iter().rposition(|s| *s <= d.cursor).unwrap_or(0);
    let col = d.text[starts[row]..d.cursor].chars().count();
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter)) && !ctrl && !shift {
        d.text.insert(d.cursor, '\n');
        d.cursor += 1;
    }
    if is_key_pressed(KeyCode::Tab) {
        d.text.insert_str(d.cursor, "  ");
        d.cursor += 2;
    }
    if (held_repeat(KeyCode::Backspace) || is_key_pressed(KeyCode::Backspace)) && d.cursor > 0 {
        let p = prev_char(&d.text, d.cursor);
        d.text.replace_range(p..d.cursor, "");
        d.cursor = p;
    }
    if (held_repeat(KeyCode::Delete) || is_key_pressed(KeyCode::Delete)) && d.cursor < d.text.len() {
        let n = next_char(&d.text, d.cursor);
        d.text.replace_range(d.cursor..n, "");
    }
    if held_repeat(KeyCode::Left) || is_key_pressed(KeyCode::Left) {
        d.cursor = prev_char(&d.text, d.cursor);
    }
    if held_repeat(KeyCode::Right) || is_key_pressed(KeyCode::Right) {
        d.cursor = next_char(&d.text, d.cursor);
    }
    let goto = |text: &str, starts: &[usize], r: usize, col: usize| -> usize {
        let end = starts.get(r + 1).map(|e| e - 1).unwrap_or(text.len());
        let line = &text[starts[r]..end];
        starts[r] + line.char_indices().nth(col).map(|(k, _)| k).unwrap_or(line.len())
    };
    let starts = line_starts(&d.text);
    if (held_repeat(KeyCode::Up) || is_key_pressed(KeyCode::Up)) && row > 0 {
        d.cursor = goto(&d.text, &starts, row - 1, col);
    }
    if (held_repeat(KeyCode::Down) || is_key_pressed(KeyCode::Down)) && row + 1 < starts.len() {
        d.cursor = goto(&d.text, &starts, row + 1, col);
    }
    if is_key_pressed(KeyCode::Home) {
        d.cursor = starts[row.min(starts.len() - 1)];
    }
    if is_key_pressed(KeyCode::End) {
        d.cursor = starts.get(row + 1).map(|e| e - 1).unwrap_or(d.text.len());
    }
}

pub(crate) fn panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let w = (low_w() - 16.0).min(440.0);
    let h = (low_h() - 16.0).min(250.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = ((low_h() - h) / 2.0).round();
    let green = Color::from_rgba(198, 255, 61, 255);
    ui_round(x + 2.0, y + 3.0, w, h, 8.0, Color::new(0.0, 0.0, 0.0, 0.5));
    ui_round_frame(x, y, w, h, 8.0, Color::from_rgba(14, 11, 22, 250), ui::skin::EDGE_STRONG);
    ui_text_as(ui::Face::Display, "live code", x + 10.0, y + 16.0, 11.0, green);
    let playing = g.venue.code.mine.is_some();
    let status = if g.venue.code.sharing { "playing for the room" } else if playing { "playing for you" } else if room_live(g) { "a dj is live coding" } else { "stopped" };
    ui_text(status, x + 16.0 + ui_measure_as(ui::Face::Display, "live code", 11.0), y + 15.0, 8.0, if playing || g.venue.code.sharing { green } else { ui::skin::MUTED });
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && !fresh;
    let (ex, ey, ew, eh) = (x + 8.0, y + 24.0, w - 16.0, h - 24.0 - 44.0);
    ui_round(ex, ey, ew, eh, 4.0, Color::from_rgba(6, 5, 12, 255));
    let before = pixel_text(true);
    let row_h = 10.0;
    let rows = ((eh - 6.0) / row_h).floor().max(1.0) as usize;
    let starts = line_starts(&g.venue.code.text);
    let cursor = g.venue.code.cursor.min(g.venue.code.text.len());
    let crow = starts.iter().rposition(|s| *s <= cursor).unwrap_or(0);
    if crow < g.venue.code.scroll {
        g.venue.code.scroll = crow;
    }
    if crow >= g.venue.code.scroll + rows {
        g.venue.code.scroll = crow + 1 - rows;
    }
    let now = get_time();
    let err_line = g.venue.code.error.as_ref().map(|e| e.0);
    for (k, line) in g.venue.code.text.split('\n').enumerate().skip(g.venue.code.scroll).take(rows) {
        let ly = ey + 4.0 + (k - g.venue.code.scroll) as f32 * row_h;
        let flash = g.venue.code.flash.get(k).map(|t| (1.0 - (now - t) / 0.18).clamp(0.0, 1.0) as f32).unwrap_or(0.0);
        if flash > 0.0 && playing {
            ui_rect(ex + 2.0, ly, ew - 4.0, row_h, Color::new(0.78, 1.0, 0.24, 0.12 * flash));
        }
        if err_line == Some(k) {
            ui_rect(ex + 2.0, ly, ew - 4.0, row_h, Color::new(1.0, 0.3, 0.35, 0.18));
        }
        ui_text(&format!("{:>2}", k + 1), ex + 3.0, ly + 8.0, 8.0, Color::new(1.0, 1.0, 1.0, 0.25));
        let trimmed = line.trim_start();
        let col = if trimmed.starts_with("//") {
            Color::from_rgba(120, 112, 140, 255)
        } else if trimmed.starts_with("set") {
            Color::from_rgba(255, 196, 110, 255)
        } else {
            Color::from_rgba(230, 236, 222, 255)
        };
        let mut shown = line.to_string();
        while shown.chars().count() > 1 && ui_measure(&shown, 8.0) > ew - 22.0 {
            shown.pop();
        }
        ui_text(&shown, ex + 18.0, ly + 8.0, 8.0, col);
        if k == crow && (fr.time * 2.0).fract() < 0.6 && g.venue.overlay == venues::Overlay::Code {
            let before_cursor: String = g.venue.code.text[starts[k]..cursor].to_string();
            let cx = ex + 18.0 + ui_measure(&before_cursor, 8.0);
            ui_rect(cx, ly + 1.0, 1.0, row_h - 1.0, green);
        }
    }
    pixel_text(before);
    let by = y + h - 38.0;
    if let Some((line, msg)) = g.venue.code.error.clone() {
        ui_text_fit(&format!("line {}: {msg}", line + 1), x + 10.0, by - 2.0, 8.0, w - 20.0, Color::from_rgba(255, 120, 130, 255));
    } else if let Some(p) = g.venue.code.mine.as_ref().filter(|p| !p.skipped.is_empty()) {
        let list: Vec<String> = p.skipped.iter().take(6).map(|m| format!(".{m}")).collect();
        ui_text_fit(&format!("playing - skipped what this computer can't do yet: {}", list.join(" ")), x + 10.0, by - 2.0, 7.0, w - 20.0, ui::skin::GOLD);
    } else {
        ui_text_fit("s(\"bd sd\")  note(\"d e f# a\").s(\"piano\")   ~ rest  [ ] group  < > take turns  *2 faster", x + 10.0, by - 2.0, 7.0, w - 20.0, ui::skin::MUTED);
    }
    let bw = 60.0;
    let play = button(fr, "play", x + 8.0, by + 4.0, bw, 16.0, playing && !g.venue.code.sharing) && !fresh;
    let halt = button(fr, "stop", x + 12.0 + bw, by + 4.0, bw, 16.0, false) && !fresh;
    let mut share = false;
    if can_share(g) {
        let label = if g.venue.code.sharing { "update the room" } else { "play for the room" };
        share = button(fr, label, x + 16.0 + bw * 2.0, by + 4.0, 104.0, 16.0, g.venue.code.sharing) && !fresh;
    }
    let keys = if radio::touch_ui() { "tap the code to edit" } else { "ctrl+enter: play   ctrl+.: stop   esc: close" };
    ui_text(keys, x + w - 10.0 - ui_measure(keys, 7.0), y + h - 8.0, 7.0, ui::skin::MUTED);
    if fresh {
        return;
    }
    let settled = get_time() - g.venue.opened_at > 0.2;
    let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    let run_keys = settled && (ctrl || shift) && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter));
    let stop_keys = settled && ctrl && is_key_pressed(KeyCode::Period);
    if fr.click && hit(fr, ex, ey, ew, eh) && radio::touch_ui() {
        let shown = g.venue.code.text.replace('\n', " ;; ");
        if let Some(t) = radio::ask_text("live code (;; starts a new line)", &shown) {
            g.venue.code.text = t.replace(" ;; ", "\n").replace(";;", "\n").chars().take(MAX_CODE).collect();
            g.venue.code.cursor = g.venue.code.text.len();
            run(g);
        }
        fr.click = false;
    }
    edit(g, settled);
    if play || run_keys {
        run(g);
    }
    if halt || stop_keys {
        stop(g);
    }
    if share {
        if !g.venue.code.sharing {
            net::send("golive code");
            g.venue.code.sharing = true;
        }
        run(g);
        if g.venue.code.error.is_some() && g.venue.code.mine.is_none() {
            g.venue.code.sharing = false;
        }
    }
    if close || is_key_pressed(KeyCode::Escape) {
        g.venue.overlay = venues::Overlay::None;
        fr.click = false;
    }
}
