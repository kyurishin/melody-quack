#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use crate::venues::*;

const PC_ROWS: [f32; 2] = [0.75, 2.9];
const PC_COLS: [f32; 4] = [1.0, 2.5, 4.0, 5.5];
const COUNTER: (f32, f32) = (7.75, 2.3);
const PILLARS: [(f32, f32); 6] = [(1.6, 2.6), (1.6, 4.6), (1.6, 6.6), (9.4, 2.6), (9.4, 4.6), (9.4, 6.6)];
const BOOTH: (f32, f32) = (5.5, 1.3);

fn ink() -> Color {
    Color::from_rgba(36, 26, 71, 255)
}

fn cream() -> Color {
    Color::from_rgba(255, 247, 224, 255)
}

fn hit(fr: &Frame, x: f32, y: f32, w: f32, h: f32) -> bool {
    fr.ux >= x && fr.ux < x + w && fr.uy >= y && fr.uy < y + h
}

fn bevel(x: f32, y: f32, w: f32, h: f32, raised: bool) {
    let grey = Color::from_rgba(198, 195, 189, 255);
    let dark = Color::from_rgba(110, 107, 102, 255);
    let (tl, br) = if raised { (WHITE, dark) } else { (dark, WHITE) };
    ui_rect(x, y, w, h, BLACK);
    ui_rect(x, y, w - 1.0, h - 1.0, tl);
    ui_rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, br);
    ui_rect(x + 1.0, y + 1.0, w - 3.0, h - 3.0, grey);
}

fn button(fr: &Frame, label: &str, x: f32, y: f32, w: f32, h: f32, on: bool) -> bool {
    let over = hit(fr, x, y, w, h);
    bevel(x, y, w, h, !(on || (over && is_mouse_button_down(MouseButton::Left))));
    if on {
        ui_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0, Color::from_rgba(27, 42, 138, 255));
    }
    let tw = ui_measure(label, 9.0);
    ui_text(label, x + (w - tw) / 2.0, y + h / 2.0 + 3.0, 9.0, if on { WHITE } else { BLACK });
    over && fr.click
}

fn link(fr: &Frame, label: &str, x: f32, y: f32, colour: Color) -> bool {
    let w = ui_measure(label, 10.0);
    let over = hit(fr, x, y - 9.0, w, 12.0);
    let c = if over { Color::from_rgba(255, 61, 242, 255) } else { colour };
    ui_text(label, x, y, 10.0, c);
    ui_rect(x, y + 1.5, w, 1.0, c);
    over && fr.click
}

fn pc_seats() -> Vec<Vec2> {
    let mut v = Vec::new();
    for z in PC_ROWS {
        for x in PC_COLS {
            v.push(vec2(x, z + 0.72));
        }
    }
    v
}

fn pc_solids() -> Vec<(f32, f32, f32, f32)> {
    let mut v = Vec::new();
    for z in PC_ROWS {
        for x in PC_COLS {
            v.push((x - 0.55, z - 0.3, x + 0.55, z + 0.3));
        }
    }
    v.push((COUNTER.0 - 0.8, COUNTER.1 - 1.6, COUNTER.0 + 0.8, COUNTER.1 + 0.3));
    v.push((8.1, 3.9, 8.8, 4.7));
    v
}

fn chapel_solids() -> Vec<(f32, f32, f32, f32)> {
    let mut v: Vec<(f32, f32, f32, f32)> = PILLARS.iter().map(|(x, z)| (x - 0.25, z - 0.25, x + 0.25, z + 0.25)).collect();
    v.push((BOOTH.0 - 1.3, 0.0, BOOTH.0 + 1.3, BOOTH.1 + 0.35));
    v.push((2.7, 0.6, 3.5, 1.5));
    v.push((7.5, 0.6, 8.3, 1.5));
    v
}

fn can_use(g: &Game) -> bool {
    !g.inside && !g.picking && !g.show_help && !g.show_inventory && !g.shop_open && !g.fishing.active && !g.sampler.open && !g.players[0].seated && !g.first_pick
}

fn enter(g: &mut Game, venue: Venue) {
    let (origin, size) = if venue == Venue::PcBang { (PC_ORIGIN, PC_SIZE) } else { (CH_ORIGIN, CH_SIZE) };
    let door = origin + vec2(size.0 / 2.0, size.1 - 0.7);
    for (who, off) in [(0usize, 0.0f32), (1, 0.8)] {
        g.players[who].seated = false;
        g.players[who].sleeping = false;
        g.players[who].place_at(door + vec2(off, 0.0));
        g.players[who].facing = vec2(0.0, -1.0);
    }
    g.venue.at = venue;
    g.venue.overlay = Overlay::None;
    g.venue.pc_open = false;
    g.venue.talk = 0.0;
    g.venue.peers.clear();
    g.venue.friends.clear();
    g.venue.joined = false;
    g.venue.on_air = None;
    g.venue.look = 0;
    g.venue.chat_open = false;
    g.venue.chat_draft.clear();
    g.venue.chat_log.clear();
    g.venue.asked = false;
    if venue == Venue::Chapel {
        if let Some(r) = g.venue.now.clone() {
            g.venue.bpm_now = r.bpm;
            g.venue.mix = (true, true, r.sound, r.bpm);
        }
        g.venue.rave_start = get_time();
        g.venue.last_step = -1;
        g.venue.crowd.clear();
        let solids = chapel_solids();
        let seed = g.venue.now.as_ref().map(|r| r.seed).unwrap_or(1) as i32 & 0xffff;
        let mut n = 0;
        while g.venue.crowd.len() < 14 && n < 200 {
            n += 1;
            let p = vec2(2.3 + hash(n, seed, 41) * 6.4, 2.3 + hash(n, seed, 42) * 5.2);
            let blocked = solids.iter().any(|s| p.x > s.0 - 0.3 && p.x < s.2 + 0.3 && p.y > s.1 - 0.3 && p.y < s.3 + 0.3);
            let crowded = g.venue.crowd.iter().any(|c| (c.pos - CH_ORIGIN).distance(p) < 0.75);
            if blocked || crowded {
                continue;
            }
            let who = (hash(n, seed, 43) * g.cast_list.len() as f32) as usize % g.cast_list.len();
            let mut c = Player::new(who, g.cast_list[who].body, CH_ORIGIN + p);
            c.facing = (vec2(BOOTH.0, BOOTH.1) - p).normalize();
            c.yaw = c.facing.x.atan2(c.facing.y);
            g.venue.crowd.push(c);
        }
    }
    g.audio.play_ch(3, &g.plink, 0.6);
    while get_char_pressed().is_some() {}
}

fn leave(g: &mut Game) {
    let door = match g.venue.at {
        Venue::PcBang => PCBANG_DOOR,
        Venue::Chapel => CHAPEL_DOOR + vec2(-0.5, 0.2),
        Venue::None => return,
    };
    for (who, off) in [(0usize, vec2(0.0, 0.25)), (1, vec2(0.8, 0.5))] {
        g.players[who].seated = false;
        let spot = rescue(&g.world, door + off);
        g.players[who].place_at(spot);
        g.players[who].facing = vec2(0.0, 1.0);
    }
    if g.venue.joined {
        net::send("leave");
    }
    g.venue.joined = false;
    g.venue.peers.clear();
    g.venue.friends.clear();
    if g.venue.on_air.take().map(|o| o.audio).unwrap_or(false) {
        net::listen("");
    }
    g.venue.at = Venue::None;
    g.venue.overlay = Overlay::None;
    g.venue.pc_open = false;
    g.audio.play_ch(3, &g.plink, 0.5);
}

fn rave_info(r: &Rave) -> String {
    format!("{}|{}|{}|{}", r.sound, r.bpm, r.seed, r.title)
}

fn rave_from(code: &str, info: &str) -> Option<Rave> {
    let f: Vec<&str> = info.splitn(4, '|').collect();
    if f.len() != 4 {
        return None;
    }
    Some(Rave {
        code: code.to_string(),
        title: f[3].chars().take(24).collect(),
        seed: f[2].parse().unwrap_or(1),
        sound: f[0].parse::<usize>().unwrap_or(0) % 3,
        bpm: f[1].parse::<f32>().unwrap_or(132.0).clamp(110.0, 160.0),
        expired: false,
    })
}

fn clock(ms: f64) -> String {
    let total = ms.abs() as i64;
    let (h, m, s, k) = (total / 3_600_000, total / 60_000 % 60, total / 1000 % 60, total % 1000);
    if h > 0 { format!("{h}:{m:02}:{s:02}.{k:03}") } else { format!("{m:02}:{s:02}.{k:03}") }
}

fn to_start(g: &Game) -> Option<f64> {
    g.venue.tonight.as_ref().map(|t| (t.starts - get_time()) * 1000.0)
}

fn here(g: &Game) -> u32 {
    1 + g.two_player as u32 + g.venue.friends.iter().map(|f| f.players).sum::<u32>()
}

fn online(g: &mut Game, dt: f32) {
    let st = net::state();
    g.venue.net_retry -= dt;
    if (st == 0 || st == -1) && g.venue.net_retry <= 0.0 {
        g.venue.net_retry = 10.0;
        if let Some(url) = net::server() {
            net::open(&url);
        }
    }
    if st != 2 {
        g.venue.peers.clear();
        g.venue.friends.clear();
        g.venue.joined = false;
        g.venue.announced = 0;
        g.venue.hello = false;
        g.venue.tonight = None;
        g.venue.key_sent = false;
        g.venue.resume_sent = false;
        g.venue.account = None;
        g.venue.providers.clear();
        g.venue.dj_name = None;
        g.venue.admin = false;
        g.venue.requests.clear();
        g.venue.granted.clear();
        g.venue.asked = false;
        if g.venue.on_air.take().map(|o| o.audio).unwrap_or(false) {
            net::listen("");
        }
    } else {
        if !g.venue.key_sent && !g.venue.dj_key.is_empty() {
            net::send(&format!("dj {}", g.venue.dj_key));
            g.venue.key_sent = true;
        }
        if !g.venue.hello {
            net::send(&format!("hello {}", g.venue.token));
            g.venue.hello = true;
        }
        if !g.venue.resume_sent && !g.venue.session.is_empty() {
            net::send(&format!("resume {}", g.venue.session));
            g.venue.resume_sent = true;
        }
        g.venue.cloud_clock += dt;
        while g.venue.announced < g.venue.raves.len() {
            let r = &g.venue.raves[g.venue.announced];
            net::send(&format!("host {} {}", r.code, rave_info(r)));
            g.venue.announced += 1;
        }
        if g.venue.at == Venue::Chapel && !g.venue.joined {
            if let Some(r) = g.venue.now.clone() {
                net::send(&format!("host {} {}", r.code, rave_info(&r)));
                net::send(&format!("join {} {} {} {} {}", r.code, 1 + g.two_player as u32, g.players[0].character, g.players[1].character, g.venue.name));
                g.venue.joined = true;
            }
        }
        if g.venue.ask {
            net::send("claim");
        }
    }
    g.venue.ask = false;
    g.venue.live_list = st == 2 && g.venue.tonight.is_some();
    if let Some((_, left)) = g.venue.sat.as_mut() {
        *left -= dt;
    }
    if g.venue.sat.as_ref().map(|s| s.1 <= 0.0).unwrap_or(false) {
        g.venue.sat = None;
    }
    let mut timed_out = false;
    if let Some((_, left)) = g.venue.find.as_mut() {
        *left -= dt;
        timed_out = *left <= 0.0;
    }
    if timed_out {
        g.venue.find = None;
        g.venue.door_msg = "can't reach the list right now. no code, no rave.".to_string();
    }
    while let Some(line) = net::recv() {
        let parts: Vec<&str> = line.split(' ').collect();
        let finding = |g: &Game, code: &str| g.venue.find.as_ref().map(|(c, _)| c == code).unwrap_or(false);
        match parts.first().copied().unwrap_or("") {
            "event" if parts.len() >= 7 => {
                let ms = parts[1].parse::<f64>().unwrap_or(0.0);
                let mut starts = get_time() + ms / 1000.0;
                if let Some(old) = &g.venue.tonight {
                    if (old.starts - starts).abs() < 0.25 {
                        starts = old.starts;
                    }
                }
                g.venue.tonight = Some(Tonight {
                    starts,
                    phase: parts[2].parse().unwrap_or(0),
                    mine: parts[3].parse().unwrap_or(0),
                    rank: parts[4].parse().unwrap_or(0),
                    left: parts[5].parse().unwrap_or(0),
                    info: parts[6..].join(" "),
                });
            }
            "sat" if parts.len() >= 2 => {
                let counting = g.venue.tonight.as_ref().map(|t| t.mine <= 1).unwrap_or(true);
                if let (Ok(off), true) = (parts[1].parse::<f64>(), counting) {
                    let text = if off == 0.0 {
                        "you sat down exactly on the dot. the goose is counting...".to_string()
                    } else {
                        format!("you sat down {:.3} s {}. the goose is counting...", off.abs() / 1000.0, if off < 0.0 { "early" } else { "late" })
                    };
                    g.venue.sat = Some((text, 9.0));
                    g.audio.play_ch(3, &g.bells[3], 0.5);
                }
            }
            "code" if parts.len() >= 2 => {
                let info = g.venue.tonight.as_ref().map(|t| t.info.clone()).unwrap_or_default();
                if let Some(r) = rave_from(parts[1], &info) {
                    g.venue.term.push(format!("tonight: {}", r.title));
                    g.venue.term.push(format!("code, scrambled: {}", rot13(&r.code)));
                    g.venue.term.push("every letter moved 13 along. rot13 it back.".to_string());
                    g.venue.secret = Some(r);
                }
            }
            "nocode" if parts.len() >= 2 => {
                let ms = parts.get(2).and_then(|n| n.parse::<f64>().ok()).unwrap_or(0.0);
                match parts[1] {
                    "early" => {
                        g.venue.term.push("the goose hasn't written tonight's code yet.".to_string());
                        g.venue.term.push("check your mail for when he does.".to_string());
                    }
                    "list" => {
                        g.venue.term.push("the first 30 have it. you weren't one of them.".to_string());
                        g.venue.term.push(format!("open to everyone else in {}.", clock(ms)));
                    }
                    _ => g.venue.term.push("the list is full. a hundred is a hundred.".to_string()),
                }
            }
            "early" if parts.len() >= 3 => {
                if finding(g, parts[1]) {
                    let ms = parts[2].parse::<f64>().unwrap_or(0.0);
                    g.venue.find = None;
                    g.venue.door_code.clear();
                    g.venue.door_msg = format!("right code. doors open in {}. come back then.", clock(ms));
                }
            }
            "full" if parts.len() >= 2 => {
                let inside = g.venue.at == Venue::Chapel && g.venue.now.as_ref().map(|r| r.code == parts[1]).unwrap_or(false);
                if inside {
                    g.venue.joined = false;
                    leave(g);
                }
                if inside || finding(g, parts[1]) {
                    g.venue.find = None;
                    g.venue.overlay = Overlay::Door;
                    g.venue.door_code.clear();
                    g.venue.door_msg = "we're full. a hundred in. one out, one in.".to_string();
                }
            }
            "room" if parts.len() >= 4 => {
                let code = parts[1];
                let elapsed = parts[2].parse::<f64>().unwrap_or(0.0) / 1000.0;
                if g.venue.at == Venue::Chapel && g.venue.now.as_ref().map(|r| r.code == code).unwrap_or(false) {
                    g.venue.rave_start = get_time() - elapsed;
                    g.venue.last_step = -1;
                } else if finding(g, code) {
                    if let Some(rave) = rave_from(code, &parts[3..].join(" ")) {
                        g.venue.find = None;
                        g.venue.learn(code);
                        g.venue.now = Some(rave);
                        enter(g, Venue::Chapel);
                    }
                }
            }
            "none" if parts.len() >= 2 => {
                if finding(g, parts[1]) {
                    g.venue.find = None;
                    g.venue.door_code.clear();
                    g.venue.door_msg = "never heard of it. no code, no rave.".to_string();
                    g.audio.play_ch(3, &g.tom, 0.6);
                }
            }
            "patreon" if parts.len() >= 2 => g.venue.patreon = parts[1].to_string(),
            "providers" => g.venue.providers = parts[1..].iter().map(|p| p.to_string()).collect(),
            "loginurl" if parts.len() >= 2 => {
                if parts[1].starts_with("https://") || parts[1].starts_with("http://") {
                    net::open_url(parts[1]);
                    g.toasts.push(("finish signing in in your browser, then come back".to_string(), 6.0));
                } else {
                    g.venue.signing_in = false;
                }
            }
            "nologin" => {
                g.venue.signing_in = false;
                g.toasts.push(("that sign-in isn't set up on this server".to_string(), 4.0));
            }
            "account" if parts.len() >= 5 => {
                g.venue.session = parts[1].chars().filter(|c| c.is_ascii_hexdigit()).take(64).collect();
                g.venue.account = Some((parts[2].to_string(), parts[3].to_string()));
                g.venue.resume_sent = true;
                g.save_clock = 99.0;
                if parts[4] == "new" {
                    g.toasts.push((format!("signed in: {}", parts[3]), 4.0));
                    if g.venue.signing_in {
                        net::send("cloudget");
                    }
                }
            }
            "noaccount" => {
                g.venue.account = None;
                g.venue.session.clear();
                g.venue.signing_in = false;
                g.save_clock = 99.0;
            }
            "cloud" if parts.len() >= 2 => {
                if g.venue.signing_in {
                    g.venue.signing_in = false;
                    if parts[1] == "-" {
                        g.venue.cloud_now = true;
                        g.save_clock = 99.0;
                    } else if g.venue.name.is_empty() || g.venue.welcome {
                        load_cloud(g, parts[1]);
                    } else {
                        g.venue.cloud_offer = Some(parts[1].to_string());
                        g.venue.overlay = Overlay::Account;
                    }
                }
            }
            "cloudok" => {}
            "dj" if parts.len() >= 2 => {
                if parts[1] == "ok" && parts.len() >= 4 {
                    g.venue.credit = parts[2].parse().unwrap_or(0);
                    let name = parts[3..].join(" ");
                    g.toasts.push((format!("you're on the dj list, {name}. credit: {} euro", g.venue.credit), 5.0));
                    g.venue.dj_name = Some(name);
                } else {
                    g.venue.dj_name = None;
                    g.toasts.push(("that dj key isn't on the list".to_string(), 5.0));
                }
            }
            "admin" => {
                g.venue.admin = true;
                g.toasts.push(("signed in as the admin. requests come by post.".to_string(), 5.0));
            }
            "request" if parts.len() >= 4 => {
                if let Ok(id) = parts[1].parse::<u32>() {
                    if !g.venue.requests.iter().any(|r| r.0 == id) {
                        g.venue.requests.push((id, parts[2].to_string(), parts[3..].join(" ")));
                        g.audio.play_ch(3, &g.bells[4], 0.4);
                    }
                }
            }
            "done" if parts.len() >= 2 => {
                if let Ok(id) = parts[1].parse::<u32>() {
                    g.venue.requests.retain(|r| r.0 != id);
                }
            }
            "balances" => g.venue.balances.clear(),
            "balance" if parts.len() >= 3 => {
                if let Ok(n) = parts[1].parse::<u32>() {
                    g.venue.balances.push((n, parts[2..].join(" ")));
                }
            }
            "asked" => g.venue.asked = true,
            "granted" if parts.len() >= 2 => {
                if !g.venue.granted.iter().any(|c| c == parts[1]) {
                    g.venue.granted.push(parts[1].to_string());
                    g.toasts.push(("the admin says yes. the decks are yours for a set.".to_string(), 5.0));
                    g.audio.play_ch(3, &g.bells[5], 0.5);
                }
                g.venue.asked = false;
            }
            "denied" => {
                g.venue.asked = false;
                g.toasts.push(("the admin says not this time.".to_string(), 5.0));
            }
            "credit" if parts.len() >= 2 => g.venue.credit = parts[1].parse().unwrap_or(g.venue.credit),
            "nolive" => g.toasts.push(("the decks aren't free, or you need a fresh yes from the admin.".to_string(), 5.0)),
            "live" if parts.len() >= 2 => {
                let was_audio = g.venue.on_air.as_ref().map(|o| o.audio).unwrap_or(false);
                g.venue.look = 0;
                if parts[1] == "0" || parts.len() < 5 {
                    g.venue.on_air = None;
                    if was_audio {
                        net::listen("");
                    }
                    if let Some(r) = g.venue.now.clone() {
                        retime(g, r.bpm);
                        g.venue.mix = (true, true, r.sound, r.bpm);
                    }
                } else {
                    let audio = parts[2] == "audio";
                    let url = if audio && parts[3].starts_with("https://") { parts[3].to_string() } else { String::new() };
                    let name = parts[4..].join(" ");
                    if audio && net::can_listen() {
                        net::listen(&url);
                    }
                    g.venue.granted.retain(|c| Some(c) != g.venue.now.as_ref().map(|r| &r.code));
                    g.toasts.push((format!("dj {name} is live"), 4.0));
                    g.venue.on_air = Some(OnAir { audio, url, name });
                }
            }
            "mix" if parts.len() >= 5 => {
                let bpm = parts[4].parse::<f32>().unwrap_or(132.0).clamp(100.0, 170.0);
                retime(g, bpm);
                g.venue.mix = (parts[1] == "1", parts[2] == "1", parts[3].parse::<usize>().unwrap_or(0) % 3, bpm);
            }
            "said" if parts.len() >= 3 => {
                let rest = parts[2..].join(" ");
                if let Some((name, text)) = rest.split_once('|') {
                    g.venue.chat_log.push((parts[1].parse().unwrap_or(0), name.to_string(), text.to_string(), get_time()));
                    if g.venue.chat_log.len() > 40 {
                        g.venue.chat_log.remove(0);
                    }
                }
            }
            "look" if parts.len() >= 2 => g.venue.look = parts[1].parse::<usize>().unwrap_or(0) % 4,
            "pad" if parts.len() >= 2 => {
                if let Ok(i) = parts[1].parse::<usize>() {
                    if g.venue.at == Venue::Chapel {
                        play_pad(g, i);
                    }
                }
            }
            "who" if parts.len() >= 7 => {
                if let Ok(id) = parts[1].parse::<u32>() {
                    let n = g.cast_list.len();
                    let friend = Friend {
                        id,
                        players: parts[2].parse::<u32>().unwrap_or(1).clamp(1, 2),
                        characters: [parts[3].parse::<usize>().unwrap_or(0) % n, parts[4].parse::<usize>().unwrap_or(0) % n],
                        dj: parts[5] == "1",
                        admin: parts[5] == "2",
                        name: parts[6..].join(" ").chars().take(12).collect(),
                    };
                    g.venue.friends.retain(|f| f.id != id);
                    if g.venue.friends.len() < 100 {
                        g.venue.friends.push(friend);
                    }
                }
            }
            "peer" if parts.len() >= 8 => {
                let nums: Vec<f32> = parts[3..8].iter().filter_map(|n| n.parse().ok()).collect();
                if let (Ok(id), Ok(slot), 5) = (parts[1].parse::<u32>(), parts[2].parse::<usize>(), nums.len()) {
                    let slot = slot.min(1);
                    let to = vec2(nums[0].clamp(0.0, CH_SIZE.0), nums[1].clamp(0.0, CH_SIZE.1 + 0.5));
                    let facing = vec2(nums[2], nums[3]);
                    let facing = if facing.length() > 0.1 { facing.normalize() } else { vec2(0.0, -1.0) };
                    let character = g.venue.friends.iter().find(|f| f.id == id).map(|f| f.characters[slot]).unwrap_or(0);
                    match g.venue.peers.iter_mut().find(|p| p.id == id && p.slot == slot) {
                        Some(p) => {
                            p.to = to;
                            p.facing = facing;
                            p.walking = nums[4] > 0.5;
                            p.character = character;
                        }
                        None => {
                            if g.venue.peers.len() < 100 {
                                g.venue.peers.push(Peer { id, slot, character, pos: to, to, facing, walking: nums[4] > 0.5, sing: 0.0 });
                            }
                        }
                    }
                }
            }
            "sing" if parts.len() >= 4 => {
                if let (Ok(id), Ok(slot), Ok(note)) = (parts[1].parse::<u32>(), parts[2].parse::<usize>(), parts[3].parse::<usize>()) {
                    if let Some(p) = g.venue.peers.iter_mut().find(|p| p.id == id && p.slot == slot.min(1)) {
                        p.sing = 0.25;
                        if note < 6 && g.venue.at == Venue::Chapel {
                            g.audio.play_ch(1, &g.voices[p.character][note], 0.8);
                        }
                    }
                }
            }
            "left" if parts.len() >= 2 => {
                if let Ok(id) = parts[1].parse::<u32>() {
                    g.venue.peers.retain(|p| p.id != id);
                    g.venue.friends.retain(|f| f.id != id);
                }
            }
            _ => {}
        }
    }
    for p in g.venue.peers.iter_mut() {
        let gap = p.to - p.pos;
        if gap.length() > 3.0 {
            p.pos = p.to;
        } else {
            p.pos += gap * (dt * 10.0).min(1.0);
        }
        p.sing = (p.sing - dt).max(0.0);
    }
    if g.venue.at == Venue::Chapel && g.venue.joined {
        g.venue.net_send -= dt;
        if g.venue.net_send <= 0.0 {
            g.venue.net_send = 0.08;
            for slot in 0..(1 + g.two_player as usize) {
                let p = &g.players[slot];
                let l = p.pos - CH_ORIGIN;
                net::send(&format!("at {slot} {:.2} {:.2} {:.2} {:.2} {}", l.x, l.y, p.facing.x, p.facing.y, p.walking as u8));
            }
        }
    }
}

fn backdrop(b: &mut Batch, o: Vec3, look: usize, time: f32, pulse: f32) {
    let rgb = |r: f32, g: f32, bl: f32| Color::new(r / 255.0, g / 255.0, bl / 255.0, 1.0);
    let mut px = |x: f32, y: f32, w: f32, h: f32, z: f32, c: Color| {
        b.cube(o + vec3(x + w / 2.0, y + h / 2.0, z), vec3(w, h, 0.04), c);
    };
    let (left, wide) = (-0.3f32, CH_SIZE.0 + 0.6);
    match look {
        1 => {
            for k in 0..17 {
                let t = k as f32 / 16.0;
                let c = if t < 0.45 {
                    let u = t / 0.45;
                    rgb(255.0 - 10.0 * u, 176.0 - 80.0 * u, 70.0 + 40.0 * u)
                } else {
                    let u = (t - 0.45) / 0.55;
                    rgb(245.0 - 150.0 * u, 96.0 - 56.0 * u, 110.0 + 20.0 * u)
                };
                px(left, 0.4 + k as f32 * 0.2, wide, 0.2, -0.3, c);
            }
            for k in -5i32..6 {
                let y = 1.75 + k as f32 * 0.15;
                let half = (0.85f32 * 0.85 - (k as f32 * 0.15).powi(2)).max(0.0).sqrt();
                if k < 0 && k % 2 == 0 {
                    continue;
                }
                px(5.5 - half, y - 0.075, half * 2.0, 0.15, -0.26, rgb(255.0, 232.0 - (k + 5) as f32 * 6.0, 120.0));
            }
            for k in 0..30 {
                let x = left + k as f32 * 0.4;
                let h = 0.35 + 0.3 * hash(k, 3, 17) + 0.2 * ((k as f32) * 0.6).sin();
                px(x, 0.4, 0.4, h, -0.22, rgb(60.0, 40.0, 70.0));
            }
            for (i, (tx, th, lean)) in [(0.9f32, 2.6f32, 0.5f32), (2.6, 2.0, -0.3), (8.4, 2.2, 0.35), (10.3, 2.8, -0.5)].iter().enumerate() {
                let sway = (time * 0.6 + i as f32).sin() * 0.06;
                for k in 0..12 {
                    let t = k as f32 / 11.0;
                    px(tx + lean * t * t - 0.07, 0.4 + th * t, 0.14, th / 11.0 + 0.02, -0.18, rgb(24.0, 30.0, 34.0));
                }
                let (cx, cy) = (tx + lean + sway, 0.4 + th);
                for f in 0..7 {
                    let a = f as f32 / 6.0 * std::f32::consts::PI;
                    for s in 1..6 {
                        let r = s as f32 * 0.17;
                        px(cx + a.cos() * r - 0.09, cy + a.sin() * r * 0.55 - r * r * 0.5 - 0.05, 0.18, 0.1, -0.18, rgb(18.0, 52.0 + f as f32 * 4.0, 40.0));
                    }
                }
            }
            for k in 0..14 {
                let x = left + 0.4 + k as f32 * 0.82;
                let len = 3 + (hash(k, 9, 5) * 6.0) as i32;
                for s in 0..len {
                    let sway = (time * 0.9 + k as f32 + s as f32 * 0.4).sin() * 0.03 * s as f32;
                    px(x + sway, 3.7 - s as f32 * 0.12, 0.07, 0.12, 0.3, rgb(40.0, 120.0 + (s * 8) as f32, 60.0));
                }
            }
            for k in 0..12 {
                let on = ((time * 1.3 + k as f32 * 1.7).sin() * 0.5 + 0.5).powi(4);
                let x = 0.5 + hash(k, 1, 77) * 10.0 + (time * 0.3 + k as f32).sin() * 0.3;
                let y = 0.7 + hash(k, 2, 77) * 1.6;
                px(x, y, 0.06, 0.06, 0.4, Color::new(0.9, 1.0, 0.4, on));
            }
        }
        2 => {
            for k in 0..17 {
                let t = k as f32 / 16.0;
                let c = if t < 0.35 {
                    rgb(30.0, 58.0 + 30.0 * t, 104.0 + 40.0 * t)
                } else {
                    let u = (t - 0.35) / 0.65;
                    rgb(250.0 - 190.0 * u, 170.0 - 110.0 * u, 130.0 + 10.0 * u)
                };
                px(left, 0.4 + k as f32 * 0.2, wide, 0.2, -0.3, c);
            }
            for k in 0..22 {
                let x = left + hash(k, 4, 31) * wide;
                let y = 0.5 + hash(k, 5, 31) * 1.0;
                let on = (time * 1.5 + k as f32).sin() * 0.5 + 0.5;
                px(x, y, 0.3 + hash(k, 6, 31) * 0.4, 0.04, -0.27, Color::new(0.8, 0.9, 1.0, 0.15 + 0.35 * on));
            }
            for k in 0..16 {
                let x = 5.6 + k as f32 * 0.36;
                let hill = 0.5 + 0.5 * ((k as f32 - 8.0) / 8.0 * 1.4).cos();
                px(x, 1.75, 0.36, 0.15 + hill * 0.5, -0.25, rgb(52.0, 50.0, 84.0));
                let h = 0.25 + hash(k, 8, 13) * 0.75 + hill * 0.5;
                let bw = 0.3;
                px(x + 0.03, 1.8, bw, h, -0.23, rgb(34.0, 34.0, 62.0));
                for wy in 0..((h / 0.14) as i32) {
                    for wx in 0..2 {
                        if hash(k * 7 + wx, wy, 23) > 0.45 {
                            px(x + 0.07 + wx as f32 * 0.13, 1.84 + wy as f32 * 0.14, 0.06, 0.06, -0.21, rgb(255.0, 214.0, 120.0));
                        }
                    }
                }
            }
            for s in 0..9 {
                let w = 0.5 - s as f32 * 0.055;
                px(8.6 - w / 2.0, 1.8 + s as f32 * 0.2, w, 0.2, -0.2, rgb(214.0, 208.0, 196.0));
            }
            px(8.58, 3.6, 0.04, 0.3, -0.2, rgb(214.0, 208.0, 196.0));
            let red = rgb(206.0, 66.0, 44.0);
            let (t1, t2, deck, top) = (1.3f32, 4.4f32, 1.95f32, 3.25f32);
            px(left, deck, 6.2, 0.07, -0.16, red);
            px(left, deck - 0.05, 6.2, 0.04, -0.16, rgb(120.0, 36.0, 30.0));
            for tx in [t1, t2] {
                px(tx - 0.13, 1.5, 0.09, top - 1.5, -0.15, red);
                px(tx + 0.04, 1.5, 0.09, top - 1.5, -0.15, red);
                for k in 0..4 {
                    px(tx - 0.13, deck + 0.3 + k as f32 * 0.3, 0.26, 0.05, -0.15, red);
                }
            }
            for k in 0..32 {
                let t = k as f32 / 31.0;
                let x = t1 + (t2 - t1) * t;
                let y = deck + 0.12 + (top - deck - 0.12) * (2.0 * t - 1.0).powi(2);
                px(x - 0.05, y, 0.1, 0.035, -0.14, red);
                if k % 2 == 0 {
                    px(x - 0.01, deck, 0.02, y - deck, -0.14, rgb(150.0, 50.0, 40.0));
                }
            }
            for k in 0..14 {
                let t = k as f32 / 13.0;
                px(t1 - 1.6 * t - 0.05, top - (top - deck) * t, 0.12, 0.035, -0.14, red);
                px(t2 + 1.5 * t - 0.05, top - (top - deck) * t, 0.12, 0.035, -0.14, red);
            }
            for k in 0..9 {
                let drift = (time * 0.12 + k as f32 * 1.37).rem_euclid(wide + 3.0) - 1.5;
                let y = 1.7 + hash(k, 2, 61) * 0.7;
                let w = 1.2 + hash(k, 3, 61) * 1.6;
                px(left + drift, y, w, 0.12, -0.1, Color::new(0.95, 0.94, 1.0, 0.28));
                px(left + drift + 0.25, y + 0.1, w * 0.6, 0.1, -0.1, Color::new(0.95, 0.94, 1.0, 0.2));
            }
            px(left, 0.4, wide, 0.5, 0.02, rgb(96.0, 100.0, 116.0));
            for k in 0..24 {
                px(left + k as f32 * 0.5, 0.9, 0.06, 0.35, 0.02, rgb(60.0, 62.0, 80.0));
            }
            px(left, 1.25, wide, 0.06, 0.02, rgb(60.0, 62.0, 80.0));
        }
        3 => {
            let neon = Color::new(1.0, 0.1 + 0.2 * pulse, 0.25 + 0.2 * pulse, 0.45 + 0.55 * pulse);
            let pink = Color::new(1.0, 0.35, 0.75, 0.45 + 0.55 * pulse);
            for k in 0..15 {
                px(left + 0.15 + k as f32 * 0.8, 0.4, 0.4, 3.1, 0.02, rgb(96.0, 12.0, 28.0));
            }
            for k in 0..40 {
                let t = k as f32 / 40.0 * std::f32::consts::TAU;
                let hx = 16.0 * t.sin().powi(3);
                let hy = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
                px(5.5 + hx * 0.055 - 0.05, 2.25 + hy * 0.055, 0.1, 0.1, 0.06, neon);
            }
            for cx in [2.0f32, 9.0] {
                for k in 0..15 {
                    let t = k as f32 / 14.0 - 0.5;
                    px(cx + t * 1.1 - 0.05, 2.0 + t * 1.6, 0.1, 0.12, 0.06, pink);
                    px(cx - t * 1.1 - 0.05, 2.0 + t * 1.6, 0.1, 0.12, 0.06, pink);
                }
            }
            for k in 0..9 {
                let x = 0.9 + k as f32 * 1.15;
                let len = 6 + (hash(k, 4, 9) * 7.0) as i32;
                for s in 0..len {
                    let sway = (time * 1.2 + k as f32).sin() * 0.012 * s as f32;
                    let (w, h) = if s % 2 == 0 { (0.09, 0.06) } else { (0.04, 0.1) };
                    px(x + sway - w / 2.0, 3.55 - s as f32 * 0.09, w, h, 0.35, rgb(176.0, 178.0, 190.0));
                }
            }
            for k in 0..23 {
                px(left + 0.2 + k as f32 * 0.5, 0.52, 0.08, 0.08, 0.06, rgb(200.0, 200.0, 212.0));
            }
        }
        _ => {}
    }
}

fn retime(g: &mut Game, bpm: f32) {
    let t = get_time();
    let beat = (t - g.venue.rave_start) * g.venue.bpm_now.max(1.0) as f64 / 60.0;
    g.venue.bpm_now = bpm;
    g.venue.rave_start = t - beat * 60.0 / bpm as f64;
}

fn play_pad(g: &Game, i: usize) {
    let s = [&g.kick, &g.clap, &g.hat, &g.open_hat, &g.tom, &g.rim][i.min(5)];
    g.audio.play_ch(2, s, 1.0);
}

fn my_set(g: &Game) -> bool {
    g.venue.joined && g.venue.dj_name.is_some() && g.venue.on_air.as_ref().map(|o| Some(&o.name) == g.venue.dj_name.as_ref()).unwrap_or(false)
}

fn send_mix(g: &mut Game) {
    let m = g.venue.mix;
    retime(g, m.3);
    net::send(&format!("mix {} {} {} {}", m.0 as u8, m.1 as u8, m.2, m.3));
}

fn load_cloud(g: &mut Game, data: &str) {
    let Some(text) = b64_decode(data).map(|b| String::from_utf8_lossy(&b).to_string()) else {
        return;
    };
    if !text.starts_with("melody-quack-save 1") {
        g.toasts.push(("the saved game on your account couldn't be read".to_string(), 5.0));
        return;
    }
    let mut text: String = text.lines().filter(|l| !l.starts_with("account ") && !l.starts_with("fresh")).map(|l| format!("{l}\n")).collect();
    text += &format!("account {}\n", g.venue.session);
    store::write(&text);
    store::restart();
}

fn provider_name(p: &str) -> &str {
    match p {
        "google" => "google / gmail",
        "apple" => "apple id",
        other => other,
    }
}

fn sign_in_buttons(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32) -> bool {
    let n = g.venue.providers.len().max(1) as f32;
    let bw = ((w - (n - 1.0) * 4.0) / n).floor();
    let mut pressed = false;
    for (i, p) in g.venue.providers.clone().iter().enumerate() {
        if button(fr, provider_name(p), x + i as f32 * (bw + 4.0), y, bw, 15.0, false) {
            g.venue.signing_in = true;
            net::send(&format!("login {p}"));
            fr.click = false;
            pressed = true;
        }
    }
    pressed
}

fn account_panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let (w, h) = (250.0f32.min(low_w() - 12.0), 120.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = ((low_h() - h) / 2.0).round();
    ui_rect(x + 3.0, y + 3.0, w, h, ink());
    ui_rect(x, y, w, h, Color::from_rgba(36, 26, 71, 255));
    ui_rect(x, y, w, 3.0, Color::from_rgba(255, 214, 110, 255));
    ui_text("your account", x + 10.0, y + 17.0, 12.0, cream());
    let soft = Color::new(1.0, 0.97, 0.88, 0.75);
    let say = |lines: &[&str]| {
        for (i, l) in lines.iter().enumerate() {
            ui_text(l, x + 10.0, y + 34.0 + i as f32 * 11.0, 9.0, soft);
        }
    };
    let mut close = false;
    if let Some(data) = g.venue.cloud_offer.clone() {
        say(&["your account already has a saved game.", "load it? the game on this device is replaced.", "or keep this one: it replaces the saved one."]);
        if button(fr, "load my saved game", x + 10.0, y + 74.0, 110.0, 16.0, false) && !fresh {
            g.venue.cloud_offer = None;
            load_cloud(g, &data);
        }
        if button(fr, "keep this one", x + 126.0, y + 74.0, 80.0, 16.0, false) && !fresh {
            g.venue.cloud_offer = None;
            g.venue.cloud_now = true;
            g.save_clock = 99.0;
            close = true;
        }
    } else if net::state() != 2 {
        say(&["signing in needs the game server,", "and you're offline right now."]);
    } else if let Some((provider, label)) = g.venue.account.clone() {
        say(&[&format!("signed in with {}:", provider_name(&provider)), &label, "your game is saved to your account as you play."]);
        if button(fr, "sign out", x + 10.0, y + 74.0, 60.0, 16.0, false) && !fresh {
            net::send("logout");
            fr.click = false;
        }
    } else if g.venue.signing_in {
        say(&["finish signing in in your browser.", "this screen updates by itself."]);
    } else if g.venue.providers.is_empty() {
        say(&["signing in isn't set up on this server yet."]);
    } else {
        say(&["sign in to keep your game safe and carry", "it to other computers."]);
        if !fresh {
            sign_in_buttons(g, fr, x + 10.0, y + 64.0, w - 20.0);
        }
    }
    if g.venue.cloud_offer.is_none() && button(fr, "close", x + w - 60.0, y + h - 24.0, 50.0, 16.0, false) {
        close = true;
    }
    if fresh {
        return;
    }
    if (close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h))) && g.venue.cloud_offer.is_none() {
        g.venue.overlay = Overlay::None;
        fr.click = false;
    }
}

fn decks(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let (w, h) = (270.0f32.min(low_w() - 12.0), 150.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = ((low_h() - h) / 2.0).round();
    let purple = Color::from_rgba(176, 92, 255, 255);
    ui_rect(x + 3.0, y + 3.0, w, h, ink());
    ui_rect(x, y, w, h, Color::from_rgba(52, 40, 102, 255));
    ui_rect(x + 10.0, y + 10.0, 6.0, 6.0, purple);
    ui_text("the decks", x + 21.0, y + 17.0, 12.0, cream());
    let code = g.venue.now.as_ref().map(|r| r.code.clone()).unwrap_or_default();
    let mut close = false;
    let say = |lines: &[&str]| {
        for (i, l) in lines.iter().enumerate() {
            ui_text(l, x + 10.0, y + 36.0 + i as f32 * 12.0, 9.0, cream());
        }
    };
    if net::state() != 2 {
        say(&["the decks need the server. you're offline."]);
    } else if g.venue.dj_name.is_none() {
        say(&["the decks are for djs.", "to become one: esc, restart, then 'become a dj'.", "it's 5 euros a month, and you get 1 euro back", "every time the admin lets you play here."]);
    } else if my_set(g) {
        let audio = g.venue.on_air.as_ref().map(|o| o.audio).unwrap_or(false);
        if audio {
            say(&["you're live with real audio.", "everyone hears your stream."]);
        } else {
            say(&["you're live on the in-game decks.", "1-6: pads, heard by everyone."]);
            let m = g.venue.mix;
            let mut changed = false;
            if button(fr, if m.0 { "beat: on" } else { "beat: off" }, x + 10.0, y + 64.0, 58.0, 14.0, m.0) || is_key_pressed(KeyCode::B) {
                g.venue.mix.0 = !m.0;
                changed = true;
            }
            if button(fr, if m.1 { "melody: on" } else { "melody: off" }, x + 72.0, y + 64.0, 66.0, 14.0, m.1) || is_key_pressed(KeyCode::M) {
                g.venue.mix.1 = !m.1;
                changed = true;
            }
            if button(fr, SOUNDS[m.2 % 3], x + 142.0, y + 64.0, 76.0, 14.0, false) {
                g.venue.mix.2 = (m.2 + 1) % 3;
                changed = true;
            }
            ui_text(&format!("{} bpm", m.3 as i32), x + 10.0, y + 96.0, 10.0, cream());
            if button(fr, "-", x + 60.0, y + 86.0, 16.0, 14.0, false) {
                g.venue.mix.3 = (m.3 - 2.0).max(100.0);
                changed = true;
            }
            if button(fr, "+", x + 80.0, y + 86.0, 16.0, 14.0, false) {
                g.venue.mix.3 = (m.3 + 2.0).min(170.0);
                changed = true;
            }
            if changed {
                send_mix(g);
            }
            if !fresh {
                for i in 0..6 {
                    if is_key_pressed(NOTE_KEYS[i]) {
                        play_pad(g, i);
                        net::send(&format!("pad {i}"));
                    }
                }
            }
            for (i, name) in DJ_PADS[0].iter().enumerate() {
                ui_text(&format!("{} {}", i + 1, name), x + 104.0 + (i % 3) as f32 * 54.0, y + 92.0 + (i / 3) as f32 * 11.0, 8.0, Color::new(1.0, 0.97, 0.88, 0.75));
            }
        }
        if button(fr, "go off air", x + 10.0, y + h - 34.0, 64.0, 16.0, false) {
            net::send("offair");
        }
        if button(fr, &format!("look: {}", LOOKS[g.venue.look % 4]), x + 78.0, y + h - 34.0, 124.0, 16.0, false) {
            g.venue.look = (g.venue.look + 1) % 4;
            net::send(&format!("look {}", g.venue.look));
        }
    } else if let Some(o) = g.venue.on_air.clone() {
        say(&[&format!("dj {} is on. wait for the set to end.", o.name)]);
    } else if g.venue.granted.iter().any(|c| *c == code) {
        say(&["the admin said yes. this counts as one set.", "how do you want to play?"]);
        if button(fr, "in-game decks", x + 10.0, y + 64.0, 84.0, 16.0, false) {
            net::send("golive decks");
        }
        ui_text("or real audio: your own live stream link (https)", x + 10.0, y + 94.0, 8.0, Color::new(1.0, 0.97, 0.88, 0.75));
        bevel(x + 10.0, y + 98.0, w - 20.0, 14.0, false);
        ui_rect(x + 11.0, y + 99.0, w - 22.0, 12.0, WHITE);
        let shown: String = g.venue.stream.chars().rev().take(46).collect::<Vec<char>>().into_iter().rev().collect();
        ui_text(if shown.is_empty() { "click to paste, or type" } else { &shown }, x + 14.0, y + 108.5, 8.0, if shown.is_empty() { Color::from_rgba(150, 146, 140, 255) } else { BLACK });
        if !fresh {
            while let Some(ch) = get_char_pressed() {
                if !ch.is_control() && ch != ' ' && g.venue.stream.len() < 190 {
                    g.venue.stream.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                g.venue.stream.pop();
            }
            if fr.click && hit(fr, x + 10.0, y + 98.0, w - 20.0, 14.0) {
                if let Some(text) = radio::ask_text("the https link to your live stream", &g.venue.stream) {
                    g.venue.stream = text.trim().chars().filter(|c| !c.is_control() && *c != ' ').take(190).collect();
                }
                fr.click = false;
            }
        }
        if button(fr, "go live with this link", x + 10.0, y + 116.0, 120.0, 16.0, false) {
            if g.venue.stream.starts_with("https://") {
                net::send(&format!("golive audio {}", g.venue.stream));
            } else {
                g.toasts.push(("the link has to start with https://".to_string(), 4.0));
            }
        }
    } else if g.venue.asked {
        say(&["asked. the admin will answer by post.", "you can keep dancing."]);
    } else {
        say(&[&format!("hello, dj {}. credit: {} euro.", g.venue.dj_name.clone().unwrap_or_default(), g.venue.credit), "each set needs a yes from the admin."]);
        if button(fr, "ask to play this rave", x + 10.0, y + 64.0, 120.0, 16.0, false) {
            net::send("ask");
        }
    }
    if button(fr, "close", x + w - 60.0, y + h - 34.0, 50.0, 16.0, false) {
        close = true;
    }
    for (k, (text, _)) in g.toasts.iter().rev().take(1).enumerate() {
        ui_text(text, x + 10.0, y + h - 6.0 - k as f32 * 10.0, 8.0, Color::from_rgba(255, 214, 110, 255));
    }
    if fresh {
        return;
    }
    if close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h)) {
        g.venue.overlay = Overlay::None;
        fr.click = false;
    }
}

fn tour_tick(g: &mut Game, dt: f32) {
    if g.venue.tour == 0 && g.first_steps >= 4 && !g.first_pick && !g.venue.name.is_empty() {
        g.venue.tour = 1;
    }
    match g.venue.tour {
        1 if g.venue.overlay == Overlay::Mail => g.venue.tour = 2,
        2 if g.venue.pc_open => g.venue.tour = 3,
        3 if !g.venue.known.is_empty() => g.venue.tour = 4,
        2 | 3 | 4 if g.venue.at == Venue::Chapel => {
            g.venue.tour = 5;
            g.venue.tour_clock = 0.0;
        }
        5 => {
            g.venue.tour_clock += dt;
            if g.venue.tour_clock > 25.0 || g.venue.chat_log.iter().any(|(_, n, _, _)| *n == g.venue.name) {
                g.venue.tour = 6;
                g.toasts.push(("that's the rave. see you on the dance floor.".to_string(), 4.0));
            }
        }
        _ => {}
    }
    let mark = match g.venue.tour {
        2 if g.venue.at == Venue::None => 1,
        4 if g.venue.at == Venue::None => 2,
        _ => 0,
    };
    TOUR_MARK.with(|m| m.set(mark));
}

fn tour_banner(g: &mut Game, fr: &mut Frame, y: f32) {
    if !(1..=5).contains(&g.venue.tour) || g.picking || g.show_help || g.paused || g.venue.overlay != Overlay::None {
        return;
    }
    let (heading, message) = match g.venue.tour {
        1 => ("the rave - 1/5", "post came. click the envelope at the top left"),
        2 if g.venue.at == Venue::PcBang => ("the pc bang - 2/5", "walk to the computer under the gold arrow and press E to sit down"),
        2 => ("the pc bang - 2/5", "follow the gold arrow to the pc bang and walk in through the door"),
        3 => ("a secret - 3/5", "tonight's code is hidden in this computer. try rave.exe or the terminal"),
        4 => ("the chapel - 4/5", "you have a code. follow the arrow to st. bassilica and give it at the door"),
        _ => ("you're in - 5/5", "press enter and say hello to everyone. 1-6 sings along. want to play here one day? esc, restart, become a dj."),
    };
    if g.venue.pc_open && g.venue.pc_app != 0 {
        return;
    }
    let w = (low_w() - 16.0).min(if g.venue.pc_open { 215.0 } else { 250.0 });
    let x = if g.venue.pc_open { (low_w() / 2.0 - w / 2.0 + 22.0).round() } else { ((low_w() - w) / 2.0).round() };
    let lines = ui_wrap(message, 9.0, w - 34.0);
    let h = 16.0 + lines.len() as f32 * 10.0 + 4.0;
    let y = if y < 0.0 { (low_h() + y - h).round() } else { y };
    ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.88));
    ui_rect(x, y, 3.0, h, Color::from_rgba(176, 92, 255, 255));
    ui_text(heading, x + 9.0, y + 12.0, 12.0, Color::from_rgba(255, 226, 150, 255));
    for (i, l) in lines.iter().enumerate() {
        ui_text(l, x + 9.0, y + 24.0 + i as f32 * 10.0, 9.0, WHITE);
    }
    let goal = match g.venue.tour {
        2 => Some(PCBANG_DOOR),
        4 => Some(CHAPEL_DOOR),
        _ => None,
    };
    if let (Some(goal), Venue::None, false) = (goal, g.venue.at, g.inside) {
        let d = goal - g.players[0].pos;
        let (cx, cy) = (x + w - 16.0, y + h - 11.0);
        if d.length() > 1.2 {
            let n = d.normalize();
            for k in -3..5 {
                let t = k as f32 * 1.4;
                ui_rect((cx + n.x * t).round() - 1.0, (cy + n.y * t).round() - 1.0, 2.0, 2.0, Color::from_rgba(255, 214, 110, 255));
            }
            let tip = vec2(cx, cy) + n * 7.0;
            let side = vec2(-n.y, n.x);
            for k in 1..4 {
                let back = tip - n * k as f32 * 1.4;
                for s in [-1.0f32, 1.0] {
                    let p = back + side * s * k as f32 * 1.4;
                    ui_rect(p.x.round() - 1.0, p.y.round() - 1.0, 2.0, 2.0, Color::from_rgba(255, 214, 110, 255));
                }
            }
        } else {
            ui_rect(cx - 2.0, cy - 2.0, 4.0, 4.0, Color::from_rgba(255, 214, 110, 255));
        }
    }
    let skip = "skip";
    let sw = ui_measure(skip, 8.0);
    let (sx, sy) = (x + w - sw - 8.0, y + 4.0);
    ui_text(skip, sx, sy + 7.0, 8.0, Color::new(1.0, 1.0, 1.0, 0.55));
    if fr.click && hit(fr, sx - 4.0, sy - 2.0, sw + 8.0, 12.0) {
        g.venue.tour = 6;
        TOUR_MARK.with(|m| m.set(0));
        fr.click = false;
    }
}

fn mail_rect(g: &Game) -> (f32, f32, f32, f32) {
    if radio::touch_ui() && !g.two_player {
        ((low_w() / 2.0 - 11.0).round(), 20.0, 22.0, 15.0)
    } else if g.two_player && g.venue.at == Venue::None {
        (6.0, 40.0, 22.0, 15.0)
    } else if g.venue.at == Venue::None {
        (6.0, 22.0, 22.0, 15.0)
    } else {
        (6.0, 26.0, 22.0, 15.0)
    }
}

fn mail_visible(g: &Game) -> bool {
    !g.venue.name.is_empty() && !g.picking && !g.show_help && !g.shop_open && !g.sampler.open && !g.venue.pc_open && g.venue.overlay == Overlay::None
}

fn mail_key(g: &Game) -> (String, u8, u8) {
    g.venue.tonight.as_ref().map(|t| (t.info.clone(), t.phase, t.mine)).unwrap_or_default()
}

pub(crate) fn mail_icon(g: &mut Game, fr: &mut Frame) {
    if g.venue.at == Venue::None && !g.inside {
        tour_banner(g, fr, if fr.touch_ui && fr.portrait { 52.0 } else { 44.0 });
    }
    if !mail_visible(g) {
        return;
    }
    let (x, y, w, h) = mail_rect(g);
    let paper = Color::from_rgba(255, 204, 72, 255);
    let fold = Color::from_rgba(240, 160, 60, 255);
    ui_rect(x, y, w, h, BLACK);
    ui_rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, paper);
    ui_rect(x + 1.0, y + h - 2.0, w - 2.0, 1.0, fold);
    let mid = (w / 2.0).floor();
    for k in 0..(mid as i32 - 1) {
        let fy = (k as f32 * 0.72).floor();
        ui_rect(x + 1.0 + k as f32, y + 1.0 + fy, 1.0, 1.0, BLACK);
        ui_rect(x + w - 2.0 - k as f32, y + 1.0 + fy, 1.0, 1.0, BLACK);
        if k < 6 {
            ui_rect(x + 1.0 + k as f32, y + h - 2.0 - (k as f32 * 0.8).floor(), 1.0, 1.0, BLACK);
            ui_rect(x + w - 2.0 - k as f32, y + h - 2.0 - (k as f32 * 0.8).floor(), 1.0, 1.0, BLACK);
        }
    }
    ui_rect(x + mid - 2.0, y + 8.0, 4.0, 1.0, BLACK);
    if (g.venue.tonight.is_some() && g.venue.mail_seen != mail_key(g)) || (g.venue.admin && !g.venue.requests.is_empty()) {
        let (bx, by) = (x + w - 6.0, y - 5.0);
        ui_rect(bx + 1.0, by, 7.0, 9.0, BLACK);
        ui_rect(bx, by + 1.0, 9.0, 7.0, BLACK);
        ui_rect(bx + 1.0, by + 1.0, 7.0, 7.0, Color::from_rgba(226, 54, 54, 255));
        ui_rect(bx + 4.0, by + 2.0, 1.0, 5.0, WHITE);
        ui_rect(bx + 3.0, by + 3.0, 1.0, 1.0, WHITE);
    }
}

fn mail(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let (w, h) = (268.0f32.min(low_w() - 12.0), if g.venue.admin { 166.0 } else { 150.0 });
    let x = ((low_w() - w) / 2.0).round();
    let y = ((low_h() - h) / 2.0).round();
    ui_rect(x + 3.0, y + 3.0, w, h, ink());
    ui_rect(x, y, w, h, cream());
    ui_rect(x, y, w, 3.0, Color::from_rgba(255, 204, 72, 255));
    let mut lines: Vec<(String, f32, Color)> = Vec::new();
    let grey = Color::from_rgba(110, 100, 130, 255);
    match (g.venue.tonight.clone(), to_start(g)) {
        (Some(t), Some(ms)) => {
            let title = t.info.splitn(4, '|').nth(3).unwrap_or("tonight").to_string();
            let mark = ms - 30.0 * 60_000.0;
            lines.push(("from: the goose".to_string(), 8.0, grey));
            lines.push((title, 16.0, ink()));
            lines.push(("st. bassilica. a hundred in, no more.".to_string(), 9.0, ink()));
            if ms > 0.0 {
                lines.push((format!("doors in {}", clock(ms)), 14.0, Color::from_rgba(200, 60, 90, 255)));
            } else {
                lines.push(("doors are open".to_string(), 14.0, Color::from_rgba(40, 150, 90, 255)));
            }
            let body: Vec<String> = match (t.phase, t.mine) {
                (_, 2) => vec![format!("you're number {} on my list of 30.", t.rank), "the terminal at the pc bang will give you the code.".to_string()],
                (_, 3) => vec!["you have a place. the code is yours.".to_string(), "bring it to the door when it opens.".to_string()],
                (0, _) | (1, 0) => vec![
                    "i write the code down exactly 30 minutes before doors.".to_string(),
                    "the 30 who sit down at a pc bang computer closest to".to_string(),
                    "that moment get the first go at it.".to_string(),
                    if mark >= 0.0 { format!("sit down in {}", clock(mark)) } else { format!("now. {} gone.", clock(mark)) },
                ],
                (1, _) => vec!["i saw you sit down. counting...".to_string()],
                (2, _) => vec!["the first 30 are on the list. you aren't.".to_string(), format!("the terminal opens to everyone else in {}.", clock(mark + 5.0 * 60_000.0)), "70 more places.".to_string()],
                _ => vec![format!("the terminal is open to everyone: {} of 70 places left.", t.left), "pc bang, terminal, find the code.".to_string()],
            };
            for b in body {
                lines.push((b, 9.0, ink()));
            }
        }
        _ => {
            lines.push(("no post today".to_string(), 16.0, ink()));
            lines.push(("the goose's list can't be reached from here.".to_string(), 9.0, ink()));
            lines.push(("tonight's secret is yours alone: the terminal".to_string(), 9.0, ink()));
            lines.push(("at the pc bang still has it.".to_string(), 9.0, ink()));
        }
    }
    let admin = g.venue.admin;
    if admin {
        let n = g.venue.requests.len();
        let label = if n > 0 { format!("dj requests ({n})") } else { "dj requests".to_string() };
        if button(fr, "post", x + w - 150.0, y + 8.0, 40.0, 14.0, !g.venue.post_tab) {
            g.venue.post_tab = false;
        }
        if button(fr, &label, x + w - 106.0, y + 8.0, 96.0, 14.0, g.venue.post_tab) {
            g.venue.post_tab = true;
        }
    }
    if admin && g.venue.post_tab {
        lines.clear();
        ui_text("dj requests", x + 12.0, y + 20.0, 14.0, ink());
        let mut ry = y + 30.0;
        if g.venue.requests.is_empty() {
            ui_text("nobody is asking right now.", x + 12.0, ry + 9.0, 9.0, grey);
            ry += 14.0;
        }
        let mut act: Option<String> = None;
        for (id, code, name) in g.venue.requests.iter().take(4) {
            ui_text(&format!("dj {name} wants {code}"), x + 12.0, ry + 10.0, 9.0, ink());
            if button(fr, "yes +1", x + w - 96.0, ry, 44.0, 14.0, false) {
                act = Some(format!("grant {id}"));
            }
            if button(fr, "no", x + w - 48.0, ry, 36.0, 14.0, false) {
                act = Some(format!("deny {id}"));
            }
            ry += 17.0;
        }
        ry += 4.0;
        ui_text("credit owed", x + 12.0, ry + 8.0, 8.0, grey);
        ry += 12.0;
        if g.venue.balances.is_empty() {
            ui_text("nothing owed.", x + 12.0, ry + 9.0, 9.0, grey);
        }
        for (n, name) in g.venue.balances.iter().take(3) {
            ui_text(&format!("dj {name}: {n} euro"), x + 12.0, ry + 10.0, 9.0, ink());
            if button(fr, "paid", x + w - 48.0, ry, 36.0, 14.0, false) {
                act = Some(format!("settle {name}"));
            }
            ry += 17.0;
        }
        if let Some(line) = act {
            if !fresh {
                net::send(&line);
                fr.click = false;
            }
        }
    }
    let mut ty = y + if admin { 30.0 } else { 16.0 };
    for (text, size, col) in lines {
        ty += size * 0.35;
        ui_text(&text, x + 12.0, ty, size, col);
        ty += size * 0.75 + 4.0;
    }
    let foot = "esc or click outside to close";
    ui_text(foot, x + (w - ui_measure(foot, 8.0)) / 2.0, y + h - 6.0, 8.0, grey);
    if fresh {
        return;
    }
    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::E) || (fr.click && !hit(fr, x, y, w, h)) {
        g.venue.overlay = Overlay::None;
        fr.click = false;
    }
}

fn name_screen(g: &mut Game, fr: &mut Frame) {
    set_default_camera();
    clear_background(Color::from_rgba(36, 26, 71, 255));
    let cx = low_w() / 2.0;
    let gold = Color::from_rgba(255, 214, 110, 255);
    let purple = Color::from_rgba(176, 92, 255, 255);
    let soft = Color::new(1.0, 0.97, 0.88, 0.7);
    let dj = g.venue.dj_page;
    let top = (low_h() * if dj { 0.12 } else { 0.3 }).round();
    let title = "melody quack";
    ui_text(title, cx - ui_measure(title, 26.0) / 2.0 + 2.0, top + 2.0, 26.0, BLACK);
    ui_text(title, cx - ui_measure(title, 26.0) / 2.0, top, 26.0, gold);
    let ask = if g.venue.name.is_empty() { "what's your name?" } else { "starting again. still you?" };
    ui_text(ask, cx - ui_measure(ask, 12.0) / 2.0, top + 34.0, 12.0, cream());
    let (w, h) = (170.0, 24.0);
    let (x, y) = ((cx - w / 2.0).round(), top + 44.0);
    let ky = y + h + 74.0;
    if !dj {
        g.venue.field = 0;
    }
    if is_key_pressed(KeyCode::Tab) && dj {
        g.venue.field = 1 - g.venue.field.min(1);
    }
    if fr.click && hit(fr, x, y, w, h) {
        g.venue.field = 0;
    }
    if fr.click && dj && hit(fr, x, ky, w, h) {
        g.venue.field = 1;
    }
    let on_key = g.venue.field == 1;
    while let Some(ch) = get_char_pressed() {
        if on_key {
            if !ch.is_control() && ch != ' ' && g.venue.key_draft.chars().count() < 40 {
                g.venue.key_draft.push(ch);
            }
        } else if (ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_') && g.venue.name_draft.chars().count() < 12 {
            g.venue.name_draft.push(ch.to_ascii_lowercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        if on_key {
            g.venue.key_draft.pop();
        } else {
            g.venue.name_draft.pop();
        }
    }
    if fr.click && hit(fr, x, y, w, h) && radio::touch_ui() {
        if let Some(text) = radio::ask_text("what's your name?", &g.venue.name_draft) {
            g.venue.name_draft = text.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').take(12).collect::<String>().to_lowercase();
        }
    }
    if fr.click && dj && hit(fr, x, ky, w, h) && radio::touch_ui() {
        if let Some(text) = radio::ask_text("your dj key", &g.venue.key_draft) {
            g.venue.key_draft = text.chars().filter(|c| !c.is_control() && *c != ' ').take(40).collect();
        }
    }
    let caret = (fr.time * 2.0).fract() < 0.6;
    ui_rect(x + 3.0, y + 3.0, w, h, BLACK);
    ui_rect(x, y, w, h, if on_key { ink() } else { gold });
    ui_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0, cream());
    ui_text(&g.venue.name_draft, x + 8.0, y + 16.5, 14.0, ink());
    if caret && !on_key {
        ui_rect(x + 9.0 + ui_measure(&g.venue.name_draft, 14.0), y + 5.0, 2.0, h - 10.0, ink());
    }
    let toggle = if dj { "just play, no dj key" } else { "become a dj" };
    let tw = ui_measure(toggle, 9.0) + 22.0;
    let (tx, ty) = ((cx - tw / 2.0).round(), y + h + 8.0);
    ui_rect(tx + 2.0, ty + 2.0, tw, 15.0, BLACK);
    ui_rect(tx, ty, tw, 15.0, if dj { Color::from_rgba(70, 56, 120, 255) } else { purple });
    ui_rect(tx + 5.0, ty + 5.0, 5.0, 5.0, if dj { purple } else { WHITE });
    ui_text(toggle, tx + 15.0, ty + 10.5, 9.0, WHITE);
    if fr.click && hit(fr, tx, ty, tw, 15.0) {
        g.venue.dj_page = !dj;
        g.venue.field = if dj { 0 } else { 1 };
        fr.click = false;
    }
    let mut bottom = ty + 15.0;
    if dj {
        let lines = ["5 euros a month. you get 1 euro back as credit", "every time the admin lets you play the arena.", "members get a dj key. type it in here:"];
        for (i, l) in lines.iter().enumerate() {
            ui_text(l, cx - ui_measure(l, 9.0) / 2.0, ty + 28.0 + i as f32 * 11.0, 9.0, if i == 2 { cream() } else { soft });
        }
        ui_rect(x + 3.0, ky + 3.0, w, h, BLACK);
        ui_rect(x, ky, w, h, if on_key { purple } else { ink() });
        ui_rect(x + 2.0, ky + 2.0, w - 4.0, h - 4.0, cream());
        let shown: String = g.venue.key_draft.chars().rev().take(20).collect::<Vec<char>>().into_iter().rev().collect();
        if shown.is_empty() && !on_key {
            ui_text("dj key", x + 8.0, ky + 16.0, 12.0, Color::from_rgba(150, 146, 140, 255));
        }
        ui_text(&shown, x + 8.0, ky + 16.0, 12.0, ink());
        if caret && on_key {
            ui_rect(x + 9.0 + ui_measure(&shown, 12.0), ky + 5.0, 2.0, h - 10.0, ink());
        }
        bottom = ky + h;
        let join = if g.venue.patreon.is_empty() { "no key yet? ask the admin for the patreon page" } else { "no key yet? join on patreon" };
        let jw = ui_measure(join, 9.0);
        let jy = bottom + 14.0;
        ui_text(join, cx - jw / 2.0, jy, 9.0, if g.venue.patreon.is_empty() { soft } else { Color::from_rgba(196, 150, 255, 255) });
        if !g.venue.patreon.is_empty() {
            ui_rect(cx - jw / 2.0, jy + 2.0, jw, 1.0, Color::from_rgba(196, 150, 255, 255));
            if fr.click && hit(fr, cx - jw / 2.0 - 3.0, jy - 9.0, jw + 6.0, 14.0) {
                net::open_url(&g.venue.patreon);
                fr.click = false;
            }
        }
        bottom = jy + 4.0;
    }
    let ready = !g.venue.name_draft.trim().is_empty();
    let hint = if ready { "enter to start" } else { "friends see it above your head when you rave together" };
    let hy = bottom + 18.0;
    let ay = hy + 16.0;
    if net::state() == 2 && !g.venue.providers.is_empty() {
        let aw = 230.0f32.min(low_w() - 16.0);
        let ax = (cx - aw / 2.0).round();
        if let Some((provider, label)) = g.venue.account.clone() {
            let t = format!("signed in with {}: {label}", provider_name(&provider));
            ui_text(&t, cx - ui_measure(&t, 9.0) / 2.0, ay + 10.0, 9.0, soft);
        } else if g.venue.signing_in {
            let t = "finish signing in in your browser...";
            ui_text(t, cx - ui_measure(t, 9.0) / 2.0, ay + 10.0, 9.0, soft);
        } else {
            let t = "have a saved game? sign in to load it";
            ui_text(t, cx - ui_measure(t, 9.0) / 2.0, ay + 8.0, 9.0, soft);
            sign_in_buttons(g, fr, ax, ay + 14.0, aw);
        }
    }
    let bw = ui_measure(hint, 10.0) + 16.0;
    if ready {
        ui_rect(cx - bw / 2.0, hy - 11.0, bw, 16.0, gold);
    }
    ui_text(hint, cx - ui_measure(hint, 10.0) / 2.0, hy, 10.0, if ready { ink() } else { soft });
    let tapped = ready && fr.click && hit(fr, cx - bw / 2.0, hy - 11.0, bw, 16.0);
    if ready && (is_key_pressed(KeyCode::Enter) || tapped) {
        g.venue.name = g.venue.name_draft.trim().to_string();
        if g.venue.hard() {
            g.venue.term = vec!["ravenet dev box. hard mode. type help.".to_string()];
        }
        let key = if g.venue.dj_page { g.venue.key_draft.trim().to_string() } else { String::new() };
        if key != g.venue.dj_key {
            g.venue.dj_key = key;
            g.venue.key_sent = false;
            g.venue.dj_name = None;
            g.venue.admin = false;
            if g.venue.dj_key.is_empty() && net::state() == 2 {
                net::send("dj -");
            }
        }
        g.venue.welcome = false;
        g.save_clock = 99.0;
        g.audio.play_ch(3, &g.bells[2], 0.5);
        g.audio.play_ch(3, &g.bells[4], 0.4);
        fr.click = false;
    }
}

pub(crate) fn frame(g: &mut Game, fr: &mut Frame) -> bool {
    online(g, fr.dt);
    tour_tick(g, fr.dt);
    if g.venue.name.is_empty() || g.venue.welcome {
        if is_quit_requested() {
            return false;
        }
        name_screen(g, fr);
        return true;
    }
    FLYER_COLOURS.with(|f| {
        let mut f = f.borrow_mut();
        f.clear();
        for r in g.venue.all_flyers().iter().take(4) {
            let (a, b, _, _) = flyer_palette(r.seed);
            f.push((a, b));
        }
    });
    if is_quit_requested() && (g.venue.at != Venue::None || g.venue.overlay != Overlay::None) {
        leave(g);
        g.venue.overlay = Overlay::None;
        return false;
    }
    let mut fresh = false;
    if mail_visible(g) && fr.click {
        let (mx, my, mw, mh) = mail_rect(g);
        if hit(fr, mx - 2.0, my - 6.0, mw + 8.0, mh + 8.0) {
            g.venue.overlay = Overlay::Mail;
            g.venue.post_tab = g.venue.admin && !g.venue.requests.is_empty();
            g.venue.mail_seen = mail_key(g);
            g.audio.play_ch(3, &g.plink, 0.4);
            fr.click = false;
            fresh = true;
        }
    }
    if g.venue.at == Venue::None && g.venue.overlay == Overlay::None {
        if !can_use(g) || !is_key_pressed(KeyCode::E) {
            return false;
        }
        let p = g.players[0].pos;
        if p.distance(PCBANG_DOOR) < 0.8 {
            enter(g, Venue::PcBang);
        } else if p.distance(BOARD) < 0.9 {
            g.venue.overlay = Overlay::Flyer(0);
        } else if p.distance(CHAPEL_DOOR) < 0.9 || p.distance(GOOSE) < 0.9 {
            g.venue.overlay = Overlay::Door;
            g.venue.door_code.clear();
            g.venue.door_msg = "evening. code?".to_string();
            while get_char_pressed().is_some() {}
        } else {
            return false;
        }
        fresh = true;
    }
    g.audio.tick();
    match g.venue.at {
        Venue::None => {
            blit(g);
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.07, 0.05, 0.15, 0.6));
        }
        Venue::PcBang => pcbang(g, fr, fresh),
        Venue::Chapel => chapel(g, fr, fresh),
    }
    match g.venue.overlay {
        Overlay::Flyer(i) => flyer_view(g, fr, i, fresh),
        Overlay::Door => door_talk(g, fr, fresh),
        Overlay::Mail => mail(g, fr, fresh),
        Overlay::Decks => decks(g, fr, fresh),
        Overlay::Account => account_panel(g, fr, fresh),
        Overlay::None => {}
    }
    true
}

fn begin_room(g: &mut Game, center: Vec3, scale: f32, back: Color) -> Mat4 {
    if low_w() as u32 != g.view_w || low_h() as u32 != g.view_h {
        g.view_w = low_w() as u32;
        g.view_h = low_h() as u32;
        g.full_view = make_view(g.view_w, g.view_h);
    }
    let (width, height) = (low_w(), low_h());
    set_camera(&Camera2D {
        render_target: Some(g.full_view.clone()),
        zoom: vec2(2.0 / g.full_view.texture.width(), -2.0 / height),
        target: vec2(g.full_view.texture.width() / 2.0, height / 2.0),
        ..Default::default()
    });
    clear_background(back);
    draw_rectangle(0.0, 0.0, width, height, back);
    let eye = center + vec3(0.0, 5.6 * scale, 6.2 * scale);
    let look_at = center + vec3(0.0, 0.1, 0.2);
    let aspect = width / height;
    let base = 21.0_f32.to_radians().tan();
    let fovy = if aspect < 4.0 / 3.0 { 2.0 * (base * (4.0 / 3.0) / aspect).min(48f32.to_radians().tan()).atan() } else { 2.0 * base.atan() };
    set_camera(&Camera3D {
        position: eye,
        target: look_at,
        up: Vec3::Y,
        fovy,
        aspect: Some(aspect),
        render_target: Some(g.full_view.clone()),
        viewport: Some((0, 0, width as i32, height as i32)),
        ..Default::default()
    });
    Mat4::perspective_rh_gl(fovy, aspect, 0.01, 10000.0) * Mat4::look_at_rh(eye, look_at, Vec3::Y)
}

fn name_tag(view: Mat4, at: Vec3, text: &str, colour: Color) {
    let p = view.project_point3(at);
    if p.z < -1.0 || p.z > 1.0 {
        return;
    }
    let (x, y) = ((p.x + 1.0) * 0.5 * low_w(), (1.0 - p.y) * 0.5 * low_h());
    let w = ui_measure(text, 8.0);
    ui_rect(x - w / 2.0 - 2.0, y - 8.0, w + 4.0, 10.0, Color::new(0.14, 0.1, 0.28, 0.75));
    ui_text(text, x - w / 2.0, y, 8.0, colour);
}

fn blit(g: &Game) {
    set_default_camera();
    clear_background(BLACK);
    draw_texture_ex(
        &g.full_view.texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), flip_y: true, ..Default::default() },
    );
}

fn walk_room(g: &mut Game, dt: f32, origin: Vec2, size: (f32, f32), solids: &[(f32, f32, f32, f32)], frozen: bool) {
    let walk = |p: Vec2| {
        let l = p - origin;
        let in_door = (l.x - size.0 / 2.0).abs() < 0.45;
        l.x > 0.3 && l.x < size.0 - 0.3 && l.y > 0.45 && (l.y < size.1 - 0.3 || (in_door && l.y < size.1 + 0.4)) && !solids.iter().any(|s| l.x > s.0 - 0.17 && l.x < s.2 + 0.17 && l.y > s.1 - 0.17 && l.y < s.3 + 0.17)
    };
    let save = |_p: Vec2| origin + vec2(size.0 / 2.0, size.1 - 1.0);
    let d0 = if frozen {
        Vec2::ZERO
    } else if g.two_player {
        direction(&[KeyCode::W], &[KeyCode::S], &[KeyCode::A], &[KeyCode::D])
    } else {
        direction(&[KeyCode::W, KeyCode::Up], &[KeyCode::S, KeyCode::Down], &[KeyCode::A, KeyCode::Left], &[KeyCode::D, KeyCode::Right])
    };
    if !g.players[0].seated || d0 != Vec2::ZERO {
        g.players[0].step_with(d0, dt, &walk, &save);
    }
    let d1 = if g.two_player {
        if frozen { Vec2::ZERO } else { direction(&[KeyCode::I, KeyCode::Up], &[KeyCode::K, KeyCode::Down], &[KeyCode::J, KeyCode::Left], &[KeyCode::L, KeyCode::Right]) }
    } else {
        let to = g.players[0].pos - g.players[1].pos;
        if to.length() > 1.3 { to.normalize() } else { Vec2::ZERO }
    };
    g.players[1].step_with(d1, dt, &walk, &save);
    if frozen {
        return;
    }
    for who in 0..2 {
        if who == 1 && !g.two_player {
            break;
        }
        let keys = if who == 0 { NOTE_KEYS } else { P2_NOTE_KEYS };
        for i in 0..6 {
            if is_key_pressed(keys[i]) {
                let voice = g.players[who].character;
                g.audio.play_ch(who, &g.voices[voice][i], 1.0);
                g.players[who].voice_timer = 0.25;
                g.players[who].key_glow[i] = 1.0;
                if g.venue.joined {
                    net::send(&format!("sing {who} {i}"));
                }
            }
        }
    }
}

fn place_label(text: &str, sub: &str) {
    let w = ui_measure(text, 12.0) + 12.0;
    ui_rect(6.0, 6.0, w, 16.0, Color::new(0.14, 0.1, 0.28, 0.85));
    ui_text(text, 12.0, 17.0, 12.0, Color::from_rgba(255, 214, 110, 255));
    if !sub.is_empty() {
        let sw = ui_measure(sub, 9.0);
        ui_rect((low_w() - sw) / 2.0 - 4.0, low_h() - 16.0, sw + 8.0, 12.0, Color::new(0.1, 0.08, 0.2, 0.6));
        ui_text(sub, (low_w() - sw) / 2.0, low_h() - 7.0, 9.0, Color::from_rgba(255, 236, 200, 255));
    }
}

fn pcbang(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let dt = fr.dt;
    let time = fr.time;
    let solids = pc_solids();
    let seats = pc_seats();
    let busy = g.venue.pc_open || g.venue.overlay != Overlay::None;
    walk_room(g, dt, PC_ORIGIN, PC_SIZE, &solids, busy);
    let local = g.players[0].pos - PC_ORIGIN;
    let near_seat = seats.iter().position(|s| s.distance(local) < 0.5);
    let near_kim = local.distance(vec2(COUNTER.0, COUNTER.1 + 0.6)) < 0.8;
    g.venue.talk = (g.venue.talk - dt).max(0.0);
    if !busy && !fresh {
        if local.y > PC_SIZE.1 + 0.05 || is_key_pressed(KeyCode::Escape) {
            leave(g);
            return;
        }
        if is_key_pressed(KeyCode::E) {
            if let Some(i) = near_seat {
                g.players[0].place_at(PC_ORIGIN + seats[i]);
                g.players[0].facing = vec2(0.0, -1.0);
                g.players[0].seated = true;
                g.venue.pc_open = true;
                if net::state() == 2 {
                    net::send("sit");
                }
                g.venue.pc_boot = 0.0;
                g.venue.pc_app = 0;
                g.audio.play_ch(3, &g.bells[2], 0.5);
                g.audio.play_ch(3, &g.bells[4], 0.4);
                while get_char_pressed().is_some() {}
            } else if near_kim {
                g.venue.talk = 5.0;
                g.audio.play_ch(1, &g.voices[g.players[1].character][1], 0.6);
            }
        }
    }

    let center = vec3(PC_ORIGIN.x + PC_SIZE.0 / 2.0, 0.5, PC_ORIGIN.y + PC_SIZE.1 / 2.0);
    begin_room(g, center, 1.28, Color::from_rgba(24, 18, 40, 255));
    let mut lamps = Vec::new();
    for z in PC_ROWS {
        for x in PC_COLS {
            lamps.push((vec3(PC_ORIGIN.x + x, 1.3, PC_ORIGIN.y + z + 0.5), 0.5, vec3(0.2, 0.9, 0.95)));
        }
    }
    lamps.push((vec3(center.x, 2.2, PC_ORIGIN.y + 0.4), 1.3, vec3(1.0, 0.2, 0.9)));
    let light = Light { dusk: 0.62, fire: center + vec3(0.0, 2.0, 0.0), fire_strength: 0.25, tint: vec3(0.6, 0.6, 1.0), lamps };
    let mut b = Batch::new(center + vec3(0.0, 5.6, -1.8), light);
    let o = vec3(PC_ORIGIN.x, 0.5, PC_ORIGIN.y);
    for z in 0..PC_SIZE.1 as i32 {
        for x in 0..PC_SIZE.0 as i32 {
            let c = if (x + z) % 2 == 0 { Color::from_rgba(52, 44, 96, 255) } else { Color::from_rgba(44, 38, 84, 255) };
            b.top(o + vec3(x as f32 + 0.5, 0.0, z as f32 + 0.5), c);
        }
    }
    let wall = Color::from_rgba(40, 34, 64, 255);
    b.cube(o + vec3(PC_SIZE.0 / 2.0, 1.1, -0.1), vec3(PC_SIZE.0 + 0.4, 2.2, 0.2), wall);
    b.cube(o + vec3(-0.1, 0.8, PC_SIZE.1 / 2.0), vec3(0.2, 1.6, PC_SIZE.1), wall);
    b.cube(o + vec3(PC_SIZE.0 + 0.1, 0.8, PC_SIZE.1 / 2.0), vec3(0.2, 1.6, PC_SIZE.1), wall);
    b.unlit = true;
    b.cube(o + vec3(PC_SIZE.0 / 2.0, 2.0, 0.02), vec3(PC_SIZE.0, 0.05, 0.04), Color::from_rgba(255, 61, 242, 255));
    b.cube(o + vec3(PC_SIZE.0 / 2.0, 1.86, 0.02), vec3(PC_SIZE.0, 0.03, 0.04), Color::from_rgba(61, 242, 255, 255));
    for (i, hue) in [0.02f32, 0.6, 0.38].iter().enumerate() {
        let c = hsv(*hue, 0.6, 0.9);
        b.cube(o + vec3(6.9 + i as f32 * 0.62, 1.35, 0.03), vec3(0.46, 0.6, 0.03), c);
    }
    b.unlit = false;
    let beige = Color::from_rgba(226, 216, 190, 255);
    let beige_dark = Color::from_rgba(186, 176, 150, 255);
    let desk = Color::from_rgba(74, 66, 110, 255);
    for (zi, z) in PC_ROWS.iter().enumerate() {
        for (xi, x) in PC_COLS.iter().enumerate() {
            let d = o + vec3(*x, 0.0, *z);
            b.cube(d + vec3(0.0, 0.34, 0.0), vec3(1.1, 0.06, 0.56), desk);
            b.cube(d + vec3(-0.48, 0.16, 0.0), vec3(0.06, 0.32, 0.5), desk);
            b.cube(d + vec3(0.48, 0.16, 0.0), vec3(0.06, 0.32, 0.5), desk);
            b.cube(d + vec3(-0.12, 0.59, -0.06), vec3(0.44, 0.38, 0.4), beige);
            b.cube(d + vec3(-0.12, 0.39, 0.0), vec3(0.3, 0.05, 0.3), beige_dark);
            b.cube(d + vec3(0.34, 0.56, -0.04), vec3(0.18, 0.4, 0.4), beige);
            b.cube(d + vec3(0.34, 0.64, 0.165), vec3(0.12, 0.03, 0.01), beige_dark);
            b.cube(d + vec3(-0.12, 0.39, 0.2), vec3(0.4, 0.03, 0.12), beige);
            b.unlit = true;
            let flick = 0.85 + 0.15 * (time * 3.0 + (xi + zi * 4) as f32 * 1.3).sin();
            b.cube(d + vec3(-0.12, 0.6, 0.145), vec3(0.34, 0.26, 0.01), Color::new(0.06 * flick, 0.56 * flick, 0.54 * flick, 1.0));
            b.cube(d + vec3(-0.12, 0.49, 0.15), vec3(0.34, 0.035, 0.01), Color::from_rgba(198, 195, 189, 255));
            b.cube(d + vec3(0.34, 0.42, 0.165), vec3(0.02, 0.02, 0.01), Color::from_rgba(198, 255, 61, 255));
            b.unlit = false;
            let seat = d + vec3(0.0, 0.0, 0.72);
            b.cube(seat + vec3(0.0, 0.1, 0.0), vec3(0.34, 0.08, 0.34), Color::from_rgba(176, 42, 74, 255));
            b.cube(seat + vec3(0.0, 0.3, 0.19), vec3(0.34, 0.4, 0.06), Color::from_rgba(217, 74, 106, 255));
        }
    }
    let k = o + vec3(COUNTER.0, 0.0, COUNTER.1);
    b.cube(k + vec3(0.0, 0.25, 0.0), vec3(1.6, 0.5, 0.5), desk);
    b.cube(k + vec3(0.0, 0.52, 0.0), vec3(1.7, 0.05, 0.6), Color::from_rgba(107, 99, 144, 255));
    b.unlit = true;
    b.cube(k + vec3(0.0, 0.3, 0.26), vec3(1.5, 0.04, 0.01), Color::from_rgba(255, 61, 242, 255));
    b.unlit = false;
    for (i, c) in [rgba(255, 128, 98), rgba(255, 214, 110), rgba(140, 222, 168), rgba(255, 168, 204)].iter().enumerate() {
        b.cube(k + vec3(-0.6 + i as f32 * 0.2, 0.62, 0.05), vec3(0.14, 0.16, 0.14), *c);
    }
    b.cube(k + vec3(0.5, 0.63, 0.0), vec3(0.3, 0.18, 0.26), Color::from_rgba(198, 195, 189, 255));
    let vend = o + vec3(8.45, 0.0, 4.3);
    b.cube(vend + vec3(0.0, 0.6, 0.0), vec3(0.6, 1.2, 0.6), Color::from_rgba(217, 67, 61, 255));
    b.unlit = true;
    b.cube(vend + vec3(-0.31, 0.7, 0.0), vec3(0.01, 0.7, 0.44), Color::from_rgba(180, 230, 255, 255));
    b.unlit = false;
    let kim_at = PC_ORIGIN + vec2(COUNTER.0, COUNTER.1 - 0.75);
    let who = g.cast_list.iter().position(|c| matches!(c.body, Body::Otter)).unwrap_or(0);
    let mut kim = Player::new(who, g.cast_list[who].body, kim_at);
    kim.yaw = 0.0;
    kim.voice_timer = if g.venue.talk > 4.6 { 0.2 } else { 0.0 };
    kim.draw(&mut b, &g.world, time, 0.0);
    if g.venue.tour == 2 && !g.venue.pc_open {
        let me = g.players[0].pos - PC_ORIGIN;
        if let Some(seat) = pc_seats().into_iter().min_by(|a, c| a.distance(me).total_cmp(&c.distance(me))) {
            let bob = (time * 4.0).sin().abs() * 0.12;
            guide_arrow(&mut b, vec3(PC_ORIGIN.x + seat.x, 1.05 + bob, PC_ORIGIN.y + seat.y), 0.05);
        }
    }
    for p in g.players.iter() {
        p.draw(&mut b, &g.world, time, 0.0);
    }
    b.flush();
    blit(g);

    if g.venue.pc_open {
        desktop(g, fr);
        tour_banner(g, fr, 34.0);
        return;
    }
    tour_banner(g, fr, -22.0);
    let hint = if near_seat.is_some() {
        "E: use the computer"
    } else if near_kim {
        "E: talk to mr. kim"
    } else {
        "walk out of the door or press esc to leave   1-6: sing"
    };
    place_label("pc bang  24h", hint);
    mail_icon(g, fr);
    if let (Some(t), Some(ms)) = (g.venue.tonight.clone(), to_start(g)) {
        let mark = ms - 30.0 * 60_000.0;
        if t.phase <= 1 && t.mine == 0 && mark < 10.0 * 60_000.0 {
            let line = if mark >= 0.0 { format!("the goose writes the code in {}. sit down on the dot.", clock(mark)) } else { format!("now! {} gone. sit down.", clock(mark)) };
            let w = ui_measure(&line, 10.0) + 14.0;
            ui_rect((low_w() - w) / 2.0, 26.0, w, 15.0, Color::new(0.78, 0.2, 0.3, 0.9));
            ui_text(&line, (low_w() - w) / 2.0 + 7.0, 36.5, 10.0, WHITE);
        }
    }
    if g.venue.talk > 0.0 {
        let lines = if g.venue.raves.is_empty() {
            ["mr. kim: first hour's free. the snacks are not.", "the kids use rave.exe on those machines, get a code,", "and go to the old chapel in the forest. i don't ask."]
        } else {
            ["mr. kim: you again. busy night?", "the goose at the chapel wants your code.", "don't spill ramyeon on the keyboards."]
        };
        let w = lines.iter().map(|l| ui_measure(l, 10.0)).fold(0.0, f32::max) + 16.0;
        let x = (low_w() - w) / 2.0;
        ui_rect(x, 30.0, w, 44.0, Color::new(1.0, 0.97, 0.88, 0.95));
        ui_rect(x, 30.0, w, 2.0, ink());
        for (i, l) in lines.iter().enumerate() {
            ui_text(l, x + 8.0, 45.0 + i as f32 * 12.0, 10.0, ink());
        }
    }
}

fn desktop(g: &mut Game, fr: &mut Frame) {
    let grey = Color::from_rgba(198, 195, 189, 255);
    let navy = Color::from_rgba(27, 42, 138, 255);
    let teal = Color::from_rgba(15, 143, 138, 255);
    g.venue.pc_boot += fr.dt;
    ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.07, 0.05, 0.15, 0.7));
    let (mw, mh) = (304.0, 214.0);
    let mx = ((low_w() - mw) / 2.0).round();
    let my = ((low_h() - mh) / 2.0 - 8.0).round().max(2.0);
    ui_rect(mx + 3.0, my + 3.0, mw, mh, ink());
    ui_rect(mx, my, mw, mh, Color::from_rgba(232, 220, 192, 255));
    ui_rect(mx, my + mh - 14.0, mw, 14.0, Color::from_rgba(191, 178, 148, 255));
    ui_text("quacktron 15", mx + 10.0, my + mh - 4.0, 8.0, Color::from_rgba(120, 108, 84, 255));
    ui_rect(mx + mw - 24.0, my + mh - 9.0, 8.0, 3.0, Color::from_rgba(198, 255, 61, 255));
    let (sx, sy, sw, sh) = (mx + 10.0, my + 8.0, mw - 20.0, mh - 28.0);
    ui_rect(sx - 2.0, sy - 2.0, sw + 4.0, sh + 4.0, Color::from_rgba(27, 26, 34, 255));
    let mut stand = false;
    let mut typed: Vec<char> = Vec::new();
    while let Some(ch) = get_char_pressed() {
        typed.push(ch);
    }
    if g.venue.pc_boot < 1.4 {
        ui_rect(sx, sy, sw, sh, BLACK);
        ui_text("quackbios v4.20", sx + 6.0, sy + 14.0, 10.0, grey);
        if g.venue.pc_boot > 0.3 {
            ui_text("640k ram ok   cd-rom ok   modem ok", sx + 6.0, sy + 26.0, 10.0, grey);
        }
        if g.venue.pc_boot > 0.7 {
            ui_rect(sx, sy, sw, sh, teal);
            let t = "quack 95";
            ui_text(t, sx + (sw - ui_measure(t, 24.0)) / 2.0, sy + sh / 2.0, 24.0, WHITE);
            let t = "starting up...";
            ui_text(t, sx + (sw - ui_measure(t, 9.0)) / 2.0, sy + sh / 2.0 + 16.0, 9.0, cream());
        }
        return;
    }
    ui_rect(sx, sy, sw, sh, teal);
    let icons = [("quacknet", 1u8), ("rave.exe", 2), ("readme", 3), ("terminal", 4)];
    for (i, (label, app)) in icons.iter().enumerate() {
        let (ix, iy) = (sx + 8.0, sy + 8.0 + i as f32 * 40.0);
        let over = hit(fr, ix - 4.0, iy - 2.0, 44.0, 36.0) && g.venue.pc_app == 0;
        if over {
            ui_rect(ix - 4.0, iy - 2.0, 44.0, 36.0, Color::new(0.1, 0.16, 0.54, 0.4));
        }
        ui_rect(ix + 7.0, iy, 20.0, 20.0, ink());
        match app {
            1 => {
                ui_rect(ix + 8.0, iy + 1.0, 18.0, 18.0, Color::from_rgba(116, 172, 255, 255));
                ui_rect(ix + 11.0, iy + 4.0, 7.0, 6.0, Color::from_rgba(140, 222, 168, 255));
                ui_rect(ix + 16.0, iy + 11.0, 6.0, 5.0, Color::from_rgba(140, 222, 168, 255));
            }
            2 => {
                ui_rect(ix + 8.0, iy + 1.0, 18.0, 18.0, Color::from_rgba(255, 226, 60, 255));
                ui_rect(ix + 12.0, iy + 6.0, 2.0, 4.0, ink());
                ui_rect(ix + 20.0, iy + 6.0, 2.0, 4.0, ink());
                ui_rect(ix + 12.0, iy + 13.0, 10.0, 2.0, ink());
            }
            4 => {
                ui_rect(ix + 8.0, iy + 1.0, 18.0, 18.0, BLACK);
                ui_text(">_", ix + 10.0, iy + 13.0, 9.0, Color::from_rgba(198, 255, 61, 255));
            }
            _ => {
                ui_rect(ix + 8.0, iy + 1.0, 18.0, 18.0, WHITE);
                for k in 0..4 {
                    ui_rect(ix + 11.0, iy + 4.0 + k as f32 * 3.5, 12.0, 1.0, Color::from_rgba(110, 107, 102, 255));
                }
            }
        }
        ui_text(label, ix + 17.0 - ui_measure(label, 8.0) / 2.0, iy + 30.0, 8.0, WHITE);
        if over && fr.click {
            g.venue.pc_app = *app;
            g.venue.pc_page = 0;
            g.audio.play_ch(3, &g.plink, 0.5);
            fr.click = false;
        }
    }
    let bar_y = sy + sh - 14.0;
    if g.venue.pc_app != 0 {
        let (wx, wy, ww, wh) = (sx + 46.0, sy + 6.0, sw - 54.0, sh - 26.0);
        bevel(wx, wy, ww, wh, true);
        for k in 0..8 {
            let t = k as f32 / 8.0;
            ui_rect(wx + 2.0 + (ww - 4.0) * t, wy + 2.0, (ww - 4.0) / 8.0 + 0.5, 11.0, mix(navy, Color::from_rgba(74, 106, 216, 255), t));
        }
        let title = ["", "quacknet explorer", "rave.exe", "readme.txt - notepad", "terminal"][g.venue.pc_app.min(4) as usize];
        ui_text(title, wx + 5.0, wy + 10.5, 9.0, WHITE);
        if button(fr, "x", wx + ww - 13.0, wy + 3.0, 10.0, 9.0, false) {
            g.venue.pc_app = 0;
            fr.click = false;
        }
        let (bx, by, bw, bh) = (wx + 3.0, wy + 15.0, ww - 6.0, wh - 18.0);
        match g.venue.pc_app {
            1 => browser(g, fr, bx, by, bw, bh),
            2 => rave_exe(g, fr, bx, by, bw, bh, &typed),
            4 => terminal(g, fr, bx, by, bw, bh, &typed),
            _ => {
                ui_rect(bx, by, bw, bh, WHITE);
                let lines = [
                    "melody quack - readme.txt",
                    "",
                    "rave.exe books a rave and gives you a code.",
                    "the goose at st. bassilica, the old chapel",
                    "at the end of the forest path, wants it.",
                    "",
                    "flyers carry codes too. old ones are no use.",
                    "",
                    "in the chapel: wasd to move, 1-6 to sing.",
                    "",
                    "bored? the terminal hides a secret rave.",
                ];
                for (i, l) in lines.iter().enumerate() {
                    ui_text(l, bx + 5.0, by + 12.0 + i as f32 * 11.0, 9.0, BLACK);
                }
            }
        }
    }
    ui_rect(sx, bar_y, sw, 14.0, grey);
    ui_rect(sx, bar_y, sw, 1.0, WHITE);
    if button(fr, "shut down", sx + 2.0, bar_y + 2.0, 54.0, 11.0, false) {
        stand = true;
    }
    let s = sky();
    let clock = format!("{:02}:{:02}", s.hour as u32, (s.hour.fract() * 60.0) as u32);
    bevel(sx + sw - 38.0, bar_y + 2.0, 36.0, 11.0, false);
    ui_text(&clock, sx + sw - 32.0, bar_y + 10.5, 9.0, BLACK);
    let mut y = sy;
    while y < sy + sh {
        ui_rect(sx, y, sw, 0.5, Color::new(0.0, 0.0, 0.0, 0.1));
        y += 2.0;
    }
    if let Some((text, _)) = &g.venue.sat {
        let w = ui_measure(text, 9.0) + 12.0;
        ui_rect((low_w() - w) / 2.0, my - 1.0, w, 13.0, Color::from_rgba(255, 204, 72, 255));
        ui_text(text, (low_w() - w) / 2.0 + 6.0, my + 8.5, 9.0, ink());
    }
    let t = "esc: close a window, or stand up";
    ui_text(t, (low_w() - ui_measure(t, 9.0)) / 2.0, (my + mh + 12.0).min(low_h() - 3.0), 9.0, Color::new(1.0, 0.97, 0.88, 0.7));
    if g.venue.overlay == Overlay::None && is_key_pressed(KeyCode::Escape) {
        if g.venue.pc_app != 0 {
            g.venue.pc_app = 0;
        } else {
            stand = true;
        }
    }
    if stand {
        g.venue.pc_open = false;
        g.players[0].seated = false;
        let p = g.players[0].pos + vec2(0.0, 0.25);
        g.players[0].place_at(p);
        g.audio.play_ch(3, &g.plink, 0.4);
    }
}

fn browser(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32, h: f32) {
    let blue = Color::from_rgba(26, 47, 208, 255);
    let url = ["quack://home", "quack://news", "quack://weather", "quack://underground"][g.venue.pc_page.min(3) as usize];
    ui_rect(x, y, w, 13.0, Color::from_rgba(198, 195, 189, 255));
    if button(fr, "home", x + 1.0, y + 1.0, 26.0, 11.0, false) {
        g.venue.pc_page = 0;
        fr.click = false;
    }
    bevel(x + 29.0, y + 1.0, w - 30.0, 11.0, false);
    ui_rect(x + 30.0, y + 2.0, w - 32.0, 9.0, WHITE);
    ui_text(url, x + 33.0, y + 9.5, 9.0, BLACK);
    let (px, py, ph) = (x, y + 13.0, h - 13.0);
    match g.venue.pc_page {
        0 => {
            ui_rect(px, py, w, ph, Color::from_rgba(253, 243, 208, 255));
            ui_text("quackville online", px + 7.0, py + 17.0, 16.0, Color::from_rgba(255, 128, 98, 255));
            let msg = "*** welcome to the information superpondway ***   ";
            ui_rect(px, py + 22.0, w, 11.0, ink());
            ui_text(&ui_ticker(msg, 9.0, w - 6.0, fr.time), px + 3.0, py + 30.0, 9.0, Color::from_rgba(198, 255, 61, 255));
            ui_text(&format!("you are visitor no. {:06}", 1337 + g.venue.raves.len() * 7), px + 7.0, py + 47.0, 9.0, ink());
            if link(fr, "> town news", px + 7.0, py + 62.0, blue) {
                g.venue.pc_page = 1;
            }
            if link(fr, "> weather", px + 7.0, py + 76.0, blue) {
                g.venue.pc_page = 2;
            }
            for k in 0..10 {
                ui_rect(px + 7.0 + k as f32 * 7.0, py + 86.0, 7.0, 7.0, if k % 2 == 0 { Color::from_rgba(255, 214, 110, 255) } else { BLACK });
            }
            ui_text("under construction", px + 82.0, py + 93.0, 9.0, ink());
            ui_text("best viewed at 640x480", px + 7.0, py + ph - 5.0, 8.0, Color::from_rgba(120, 116, 110, 255));
            if link(fr, ".", px + w - 9.0, py + ph - 5.0, Color::from_rgba(214, 200, 160, 255)) {
                g.venue.pc_page = 3;
                g.audio.play_ch(3, &g.bells[0], 0.5);
            }
        }
        1 => {
            ui_rect(px, py, w, ph, WHITE);
            ui_text("town news", px + 7.0, py + 16.0, 14.0, ink());
            let lines = [
                "* pc bang now open 24h. first hour free.",
                "* strange bass heard from the old chapel",
                "  every night. council 'looking into it'.",
                "* flyers keep appearing on the board by",
                "  the chapel. nobody admits to pasting them.",
                "* campfire: still on fire.",
            ];
            for (i, l) in lines.iter().enumerate() {
                ui_text(l, px + 7.0, py + 31.0 + i as f32 * 11.0, 9.0, ink());
            }
        }
        2 => {
            ui_rect(px, py, w, ph, Color::from_rgba(186, 230, 255, 255));
            let s = sky();
            ui_text("weather", px + 7.0, py + 16.0, 14.0, ink());
            ui_text(&format!("quackville: {}, {}.", SEASONS[s.season], WEATHERS[s.weather]), px + 7.0, py + 32.0, 9.0, ink());
            ui_text("the chapel: fog, lasers, 100% chance of bass.", px + 7.0, py + 44.0, 9.0, ink());
        }
        _ => {
            ui_rect(px, py, w, ph, BLACK);
            let glitch = if (fr.time * 7.0).sin() > 0.9 { 1.5 } else { 0.0 };
            ui_text("u n d e r g r o u n d", px + 7.0 - glitch, py + 15.0, 13.0, Color::from_rgba(61, 242, 255, 255));
            ui_text("u n d e r g r o u n d", px + 7.0 + glitch, py + 15.0, 13.0, Color::from_rgba(255, 61, 242, 255));
            ui_text("the code on the flyer is the way in. tell no one.", px + 7.0, py + 27.0, 8.0, Color::from_rgba(198, 255, 61, 255));
            let flyers: Vec<Rave> = g.venue.all_flyers().into_iter().take(6).cloned().collect();
            for (i, r) in flyers.iter().enumerate() {
                let (fx, fy) = (px + 7.0 + i as f32 * 35.0, py + 34.0);
                draw_flyer(r, fx, fy, 1.0);
                if hit(fr, fx, fy, 30.0, 42.0) {
                    ui_rect(fx - 1.0, fy - 1.0, 32.0, 1.0, Color::from_rgba(198, 255, 61, 255));
                    ui_rect(fx - 1.0, fy + 42.0, 32.0, 1.0, Color::from_rgba(198, 255, 61, 255));
                    if fr.click {
                        g.venue.overlay = Overlay::Flyer(i);
                        fr.click = false;
                    }
                }
            }
            ui_text("throw your own: rave.exe on the desktop.", px + 7.0, py + ph - 14.0, 8.0, Color::from_rgba(198, 195, 189, 255));
            ui_text("<!-- goose: never change the house tempo. 132. -->", px + 7.0, py + ph - 4.0, 7.0, Color::from_rgba(70, 66, 80, 255));
        }
    }
}

fn rave_exe(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32, h: f32, typed: &[char]) {
    ui_rect(x, y, w, h, Color::from_rgba(198, 195, 189, 255));
    if let Some(i) = g.venue.pc_made {
        if let Some(r) = g.venue.raves.get(i).cloned() {
            ui_text("your rave is booked. the code:", x + 6.0, y + 12.0, 9.0, BLACK);
            bevel(x + 6.0, y + 16.0, 118.0, 24.0, false);
            ui_rect(x + 8.0, y + 18.0, 114.0, 20.0, BLACK);
            ui_text(&r.code, x + 13.0, y + 33.0, 16.0, Color::from_rgba(198, 255, 61, 255));
            draw_flyer(&r, x + w - 56.0, y + 6.0, 1.6);
            let tips = ["1. give the code to the goose", "   at the chapel door, north", "   along the forest path.", "2. tell your friends the code,", "   or let them find the flyer."];
            for (k, t) in tips.iter().enumerate() {
                ui_text(t, x + 6.0, y + 54.0 + k as f32 * 10.0, 9.0, BLACK);
            }
            if button(fr, "host another", x + 6.0, y + h - 18.0, 70.0, 13.0, false) {
                g.venue.pc_made = None;
                g.venue.pc_title.clear();
            }
            return;
        }
        g.venue.pc_made = None;
    }
    for ch in typed {
        if (ch.is_ascii_alphanumeric() || *ch == ' ') && g.venue.pc_title.len() < 16 {
            g.venue.pc_title.push(ch.to_ascii_lowercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        g.venue.pc_title.pop();
    }
    ui_text("event name (just type)", x + 6.0, y + 12.0, 9.0, BLACK);
    bevel(x + 6.0, y + 15.0, w - 12.0, 14.0, false);
    ui_rect(x + 7.0, y + 16.0, w - 14.0, 12.0, WHITE);
    let shown = if g.venue.pc_title.is_empty() { "acid mass".to_string() } else { g.venue.pc_title.clone() };
    ui_text(&shown, x + 10.0, y + 25.5, 9.0, if g.venue.pc_title.is_empty() { Color::from_rgba(150, 146, 140, 255) } else { BLACK });
    if (fr.time * 2.0).fract() < 0.6 {
        let cx = x + 11.0 + if g.venue.pc_title.is_empty() { 0.0 } else { ui_measure(&g.venue.pc_title, 9.0) };
        ui_rect(cx, y + 18.0, 1.0, 8.0, BLACK);
    }
    ui_text("sound", x + 6.0, y + 42.0, 9.0, BLACK);
    let bw = (w - 12.0) / 3.0;
    for (i, s) in SOUNDS.iter().enumerate() {
        if button(fr, s, x + 6.0 + i as f32 * bw, y + 45.0, bw - 2.0, 13.0, g.venue.pc_sound == i) {
            g.venue.pc_sound = i;
            g.venue.pc_bpm = [136.0, 130.0, 142.0][i];
        }
    }
    ui_text(&format!("tempo: {} bpm", g.venue.pc_bpm as i32), x + 6.0, y + 72.0, 9.0, BLACK);
    if button(fr, "-", x + 86.0, y + 63.0, 14.0, 12.0, false) {
        g.venue.pc_bpm = (g.venue.pc_bpm - 2.0).max(110.0);
    }
    if button(fr, "+", x + 102.0, y + 63.0, 14.0, 12.0, false) {
        g.venue.pc_bpm = (g.venue.pc_bpm + 2.0).min(160.0);
    }
    if button(fr, "generate code", x + 6.0, y + 84.0, 90.0, 16.0, false) || is_key_pressed(KeyCode::Enter) {
        let title = g.venue.pc_title.clone();
        let i = g.venue.host(&title, g.venue.pc_sound, g.venue.pc_bpm);
        g.venue.pc_made = Some(i);
        for n in [0usize, 2, 4, 5] {
            g.audio.play_ch(3, &g.bells[n], 0.4);
        }
    }
    if !g.venue.raves.is_empty() {
        ui_text("your raves:", x + 6.0, y + h - 16.0, 8.0, BLACK);
        let codes: Vec<String> = g.venue.raves.iter().rev().take(2).map(|r| r.code.clone()).collect();
        ui_text(&codes.join("  "), x + 6.0, y + h - 6.0, 9.0, Color::from_rgba(27, 42, 138, 255));
    }
}

fn terminal(g: &mut Game, fr: &mut Frame, x: f32, y: f32, w: f32, h: f32, typed: &[char]) {
    let green = Color::from_rgba(198, 255, 61, 255);
    ui_rect(x, y, w, h, Color::from_rgba(8, 8, 14, 255));
    for ch in typed {
        if !ch.is_control() && g.venue.term_in.chars().count() < if g.venue.hard() { 200 } else { 30 } {
            g.venue.term_in.push(*ch);
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        g.venue.term_in.pop();
    }
    if is_key_pressed(KeyCode::Enter) {
        let line = std::mem::take(&mut g.venue.term_in);
        if g.venue.run(&line) {
            for n in [0usize, 2, 4, 5] {
                g.audio.play_ch(3, &g.bells[n], 0.45);
            }
        } else {
            g.audio.play_ch(3, &g.plink, 0.25);
        }
    }
    let rows = ((h - 16.0) / 10.0) as usize;
    let mut lines: Vec<(String, Color)> = Vec::new();
    for l in g.venue.term.iter() {
        let c = if l.starts_with("flag captured") { Color::from_rgba(255, 214, 110, 255) } else if l.contains("@pc7") { WHITE } else { green };
        let mut rest: Vec<char> = l.chars().collect();
        while !rest.is_empty() {
            let mut n = rest.len();
            while n > 1 && ui_measure(&rest[..n].iter().collect::<String>(), 9.0) > w - 8.0 {
                n -= 1;
            }
            lines.push((rest[..n].iter().collect(), c));
            rest.drain(..n);
        }
    }
    let start = lines.len().saturating_sub(rows);
    for (i, (l, c)) in lines[start..].iter().enumerate() {
        ui_text(l, x + 4.0, y + 10.0 + i as f32 * 10.0, 9.0, *c);
    }
    let who = if g.venue.hard() { "leon" } else if g.venue.term_goose { "goose" } else { "guest" };
    let prompt = if g.venue.term_pw { "password: ".to_string() } else { format!("{who}@pc7:~$ ") };
    let mut shown = if g.venue.term_pw { "*".repeat(g.venue.term_in.len()) } else { g.venue.term_in.clone() };
    while shown.chars().count() > 1 && ui_measure(&format!("{prompt}{shown}"), 9.0) > w - 14.0 {
        shown = shown.chars().skip(1).collect();
    }
    let line = format!("{prompt}{shown}");
    ui_text(&line, x + 4.0, y + h - 4.0, 9.0, WHITE);
    if (fr.time * 2.0).fract() < 0.6 {
        ui_rect(x + 5.0 + ui_measure(&line, 9.0), y + h - 11.0, 4.0, 8.0, green);
    }
}

fn flyer_view(g: &mut Game, fr: &mut Frame, i: usize, fresh: bool) {
    let flyers: Vec<Rave> = g.venue.all_flyers().into_iter().cloned().collect();
    if flyers.is_empty() {
        g.venue.overlay = Overlay::None;
        return;
    }
    let i = i.min(flyers.len() - 1);
    let r = &flyers[i];
    g.venue.learn(&r.code);
    ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.07, 0.05, 0.15, 0.72));
    let cell = ((low_h() - 44.0) / FLYER_H as f32).floor().max(3.0);
    let (w, h) = (FLYER_W as f32 * cell, FLYER_H as f32 * cell);
    let x = ((low_w() - w) / 2.0 - 58.0).max(6.0).round();
    let y = ((low_h() - h) / 2.0).round();
    draw_flyer(r, x, y, cell);
    let cx = x + w + 12.0;
    ui_rect(cx + 2.0, y + 12.0, 112.0, 76.0, ink());
    ui_rect(cx, y + 10.0, 112.0, 76.0, cream());
    ui_text("the code", cx + 8.0, y + 24.0, 9.0, Color::from_rgba(110, 100, 130, 255));
    ui_text(&r.code, cx + 8.0, y + 42.0, 15.0, ink());
    let note = if r.expired { "old and torn. that night's over." } else { "take it to the goose." };
    for (k, l) in ui_wrap(note, 9.0, 98.0).iter().enumerate() {
        ui_text(l, cx + 8.0, y + 56.0 + k as f32 * 10.0, 9.0, ink());
    }
    ui_text(&format!("flyer {} of {}", i + 1, flyers.len()), cx + 8.0, y + 80.0, 8.0, Color::from_rgba(110, 100, 130, 255));
    let hint = "a/d: next flyer   esc: close";
    ui_text(hint, (low_w() - ui_measure(hint, 9.0)) / 2.0, low_h() - 6.0, 9.0, Color::new(1.0, 0.97, 0.88, 0.8));
    if fresh {
        return;
    }
    let n = flyers.len();
    if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
        g.venue.overlay = Overlay::Flyer((i + 1) % n);
        g.audio.play_ch(3, &g.plink, 0.3);
    } else if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        g.venue.overlay = Overlay::Flyer((i + n - 1) % n);
        g.audio.play_ch(3, &g.plink, 0.3);
    } else if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::E) || (fr.click && !hit(fr, x, y, w + 130.0, h)) {
        g.venue.overlay = Overlay::None;
        fr.click = false;
    }
}

fn door_talk(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let (w, h) = (250.0, 128.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = ((low_h() - h) / 2.0).round();
    ui_rect(x + 3.0, y + 3.0, w, h, ink());
    ui_rect(x, y, w, h, Color::from_rgba(52, 40, 102, 255));
    let (gx, gy) = (x + 12.0, y + 14.0);
    ui_rect(gx + 4.0, gy + 44.0, 44.0, 44.0, Color::from_rgba(30, 28, 44, 255));
    ui_rect(gx + 18.0, gy + 12.0, 16.0, 40.0, WHITE);
    ui_rect(gx + 12.0, gy, 28.0, 22.0, WHITE);
    ui_rect(gx + 34.0, gy + 12.0, 16.0, 7.0, Color::from_rgba(255, 154, 60, 255));
    ui_rect(gx + 12.0, gy + 6.0, 30.0, 6.0, Color::from_rgba(10, 10, 20, 255));
    let tx = x + 76.0;
    ui_text("the goose", tx, y + 18.0, 9.0, Color::from_rgba(255, 168, 204, 255));
    for (k, l) in ui_wrap(&g.venue.door_msg, 11.0, w - 88.0).iter().take(2).enumerate() {
        ui_text(l, tx, y + 34.0 + k as f32 * 12.0, 11.0, cream());
    }
    let raw: String = g.venue.door_code.clone();
    let mut shown = String::new();
    for (k, c) in raw.chars().enumerate() {
        if k == 3 || k == 6 {
            shown.push('-');
        }
        shown.push(c);
    }
    ui_rect(tx, y + 58.0, 150.0, 22.0, ink());
    ui_rect(tx + 2.0, y + 60.0, 146.0, 18.0, cream());
    ui_text(if shown.is_empty() { "type the code" } else { &shown }, tx + 8.0, y + 74.0, 14.0, if shown.is_empty() { Color::from_rgba(170, 160, 150, 255) } else { ink() });
    let mut submit: Option<String> = None;
    if !g.venue.known.is_empty() {
        ui_text("codes you've seen (click one)", x + 12.0, y + 100.0, 8.0, Color::new(1.0, 0.97, 0.88, 0.6));
        let mut cx = x + 12.0;
        for c in g.venue.known.clone().iter().take(3) {
            let cw = ui_measure(c, 9.0) + 10.0;
            let over = hit(fr, cx, y + 104.0, cw, 14.0);
            ui_rect(cx, y + 104.0, cw, 14.0, if over { Color::from_rgba(255, 214, 110, 255) } else { ink() });
            ui_text(c, cx + 5.0, y + 114.0, 9.0, if over { ink() } else { cream() });
            if over && fr.click {
                submit = Some(c.clone());
                fr.click = false;
            }
            cx += cw + 5.0;
        }
    }
    if fresh {
        return;
    }
    while let Some(ch) = get_char_pressed() {
        if ch.is_ascii_alphanumeric() && g.venue.door_code.len() < 9 {
            g.venue.door_code.push(ch.to_ascii_uppercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        g.venue.door_code.pop();
    }
    if fr.click && hit(fr, tx, y + 58.0, 150.0, 22.0) && radio::touch_ui() {
        if let Some(text) = radio::ask_text("the code on the flyer", &shown) {
            g.venue.door_code = text.chars().filter(|c| c.is_ascii_alphanumeric()).take(9).collect::<String>().to_uppercase();
        }
    }
    if is_key_pressed(KeyCode::Enter) {
        submit = Some(g.venue.door_code.clone());
    }
    if is_key_pressed(KeyCode::Escape) {
        g.venue.overlay = Overlay::None;
        return;
    }
    if let Some(text) = submit {
        match tidy_code(&text) {
            None => {
                g.venue.door_msg = "nine letters and numbers, mate.".to_string();
                g.audio.play_ch(3, &g.tom, 0.6);
            }
            Some(code) => {
                let found = g.venue.raves.iter().chain(g.venue.secret.iter()).find(|r| r.code == code).cloned();
                if let Some(r) = found {
                    g.venue.now = Some(r);
                    enter(g, Venue::Chapel);
                } else if g.venue.decoys.iter().any(|r| r.code == code) {
                    g.venue.door_msg = "that flyer's from last summer. that night's long over.".to_string();
                    g.venue.door_code.clear();
                    g.audio.play_ch(3, &g.tom, 0.6);
                } else if net::state() == 2 {
                    net::send(&format!("find {code}"));
                    g.venue.find = Some((code, 4.0));
                    g.venue.door_msg = "let me check the list...".to_string();
                } else {
                    g.venue.door_msg = "never heard of it. no code, no rave.".to_string();
                    g.venue.door_code.clear();
                    g.audio.play_ch(3, &g.tom, 0.6);
                }
            }
        }
    }
}

fn rave_tick(g: &mut Game, dt: f32) -> (f32, usize, bool) {
    g.venue.live_clock += dt;
    if g.venue.live_clock > 0.5 {
        g.venue.live_clock = 0.0;
        g.venue.live = radio::now_playing();
        let bpm = radio::bpm();
        if bpm > 0.0 {
            g.radio_bpm = bpm;
        }
    }
    let Some(r) = g.venue.now.clone() else {
        return (0.0, 0, false);
    };
    let decks_live = g.venue.on_air.as_ref().map(|o| !o.audio).unwrap_or(false);
    let live = !g.venue.live.is_empty() || g.venue.on_air.as_ref().map(|o| o.audio).unwrap_or(false);
    let (beat_on, melody_on, sound) = if decks_live { (g.venue.mix.0, g.venue.mix.1, g.venue.mix.2) } else { (true, true, r.sound) };
    let bpm = if !g.venue.live.is_empty() && g.radio_bpm > 0.0 { g.radio_bpm } else { g.venue.bpm_now.max(60.0) };
    let beat = (get_time() - g.venue.rave_start) * bpm as f64 / 60.0;
    let step = (beat * 4.0).floor() as i64;
    let bar = (step / 16).rem_euclid(32) as usize;
    let breakdown = (16..20).contains(&bar);
    let drop = bar >= 20;
    if step != g.venue.last_step {
        g.venue.last_step = step;
        if !live {
            let s = step.rem_euclid(16) as i32;
            let seed = (r.seed & 0xfff) as u32;
            let intro = bar < 4;
            if beat_on && !breakdown && s % 4 == 0 {
                g.audio.play_ch(2, &g.kick, 1.0);
            }
            if beat_on && !breakdown && !intro && (s == 4 || s == 12) {
                g.audio.play_ch(2, &g.clap, 0.7);
            }
            if beat_on && s % 4 == 2 {
                g.audio.play_ch(2, if drop { &g.open_hat } else { &g.hat }, 0.5);
            } else if beat_on && s % 2 == 1 && !intro {
                g.audio.play_ch(2, &g.hat, 0.22);
            }
            if beat_on && !breakdown && !intro && s % 4 != 0 {
                g.audio.play_ch(2, &g.bass, 0.5);
            }
            let on = hash(s, seed as i32, 21) > 0.42;
            let note = ((hash(s, seed as i32, 22) * 6.0) as usize).min(5);
            if !intro && melody_on {
                match sound {
                    0 => {
                        if on {
                            g.audio.play_ch(3, &g.marimba[note], 0.5);
                        }
                    }
                    1 => {
                        if on && s % 2 == 0 {
                            g.audio.play_ch(3, &g.bells[note], 0.38);
                        }
                        if s == 0 {
                            for n in [0usize, 2, 4] {
                                g.audio.play_ch(3, &g.piano[n], 0.26);
                            }
                        }
                    }
                    _ => {
                        if s % 2 == 0 {
                            g.audio.play_ch(3, &g.piano[[0usize, 2, 4, 5, 4, 2, 3, 1][(s / 2) as usize % 8]], 0.32);
                        }
                    }
                }
            }
            if breakdown && s == 0 && bar == 19 {
                g.audio.play_ch(2, &g.tom, 0.8);
            }
        }
    }
    let frac = beat.rem_euclid(1.0) as f32;
    let pulse = if breakdown { 0.2 * (1.0 - frac) } else { (1.0 - frac).powi(3) };
    (pulse, bar, drop)
}

fn chapel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let dt = fr.dt;
    let time = fr.time;
    let solids = chapel_solids();
    let can_chat = g.venue.joined && net::state() == 2;
    let mut chat_done = false;
    if g.venue.chat_open && !fresh {
        while let Some(ch) = get_char_pressed() {
            if !ch.is_control() && g.venue.chat_draft.chars().count() < 120 {
                g.venue.chat_draft.push(ch);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            g.venue.chat_draft.pop();
        }
        if is_key_pressed(KeyCode::Enter) {
            let text = g.venue.chat_draft.trim().to_string();
            if !text.is_empty() && can_chat {
                net::send(&format!("say {text}"));
            }
            chat_done = true;
        }
        if is_key_pressed(KeyCode::Escape) || !can_chat {
            chat_done = true;
        }
    } else if !fresh && can_chat && g.venue.overlay == Overlay::None && is_key_pressed(KeyCode::Enter) {
        g.venue.chat_open = true;
        g.venue.chat_draft.clear();
        while get_char_pressed().is_some() {}
    }
    let busy = g.venue.overlay != Overlay::None || g.venue.chat_open;
    if chat_done {
        g.venue.chat_open = false;
        g.venue.chat_draft.clear();
    }
    walk_room(g, dt, CH_ORIGIN, CH_SIZE, &solids, busy);
    let local = g.players[0].pos - CH_ORIGIN;
    if !fresh && !busy && (local.y > CH_SIZE.1 + 0.05 || is_key_pressed(KeyCode::Escape)) {
        leave(g);
        return;
    }
    let at_decks = local.distance(vec2(BOOTH.0, BOOTH.1 + 0.85)) < 1.0;
    if !fresh && !busy && at_decks && is_key_pressed(KeyCode::E) {
        g.venue.overlay = Overlay::Decks;
        while get_char_pressed().is_some() {}
        return;
    }
    let (pulse, bar, drop) = rave_tick(g, dt);
    g.venue.pulse = pulse;
    let rave = g.venue.now.clone();
    let center = vec3(CH_ORIGIN.x + CH_SIZE.0 / 2.0, 0.5, CH_ORIGIN.y + CH_SIZE.1 / 2.0);
    let view = begin_room(g, center + vec3(0.0, 0.0, -1.2), 1.75, Color::from_rgba(10, 7, 20, 255));
    let o = vec3(CH_ORIGIN.x, 0.5, CH_ORIGIN.y);
    let booth = o + vec3(BOOTH.0, 0.0, BOOTH.1);
    let mut lamps = Vec::new();
    for n in 0..4 {
        let a = time * (0.5 + n as f32 * 0.13) + n as f32 * 1.7;
        let c = hsv((0.82 + n as f32 * 0.17 + time * 0.02).fract(), 0.8, 1.0);
        lamps.push((o + vec3(5.5 + a.cos() * 3.2, 0.9, 4.6 + (a * 0.8).sin() * 2.2), 0.7 + pulse * 0.9, vec3(c.r, c.g, c.b)));
    }
    lamps.push((booth + vec3(0.0, 1.2, 0.3), 0.9 + pulse * 0.8, vec3(1.0, 0.25, 0.95)));
    let look = if g.venue.on_air.is_some() { g.venue.look % 4 } else { 0 };
    let (dusk, tint) = match look {
        1 => (0.72, vec3(1.0, 0.55, 0.25)),
        2 => (0.82, vec3(0.55, 0.65, 1.0)),
        3 => (0.97, vec3(1.0, 0.12, 0.2)),
        _ => (0.96, vec3(0.9, 0.3, 1.0)),
    };
    if look == 3 {
        for l in lamps.iter_mut() {
            l.2 = vec3(1.0, 0.1 + 0.25 * l.2.z, 0.25 + 0.3 * l.2.z);
        }
    }
    let light = Light { dusk, fire: booth + vec3(0.0, 1.5, 0.0), fire_strength: 0.15 + pulse * 0.35, tint, lamps };
    let mut b = Batch::new(center + vec3(0.0, 5.6, -1.8), light);
    for z in 0..CH_SIZE.1 as i32 {
        for x in 0..CH_SIZE.0 as i32 {
            let even = (x + z) % 2 == 0;
            let c = match look {
                1 => {
                    let k = hash(x, z, 91);
                    if k > 0.8 { Color::from_rgba(120, 96, 60, 255) } else if even { Color::from_rgba(70, 118, 62, 255) } else { Color::from_rgba(60, 104, 58, 255) }
                }
                2 => if x % 2 == 0 { Color::from_rgba(150, 112, 78, 255) } else { Color::from_rgba(136, 100, 70, 255) },
                3 => if even { Color::from_rgba(22, 18, 24, 255) } else { Color::from_rgba(120, 18, 34, 255) },
                _ => if even { Color::from_rgba(96, 92, 108, 255) } else { Color::from_rgba(86, 82, 98, 255) },
            };
            b.top(o + vec3(x as f32 + 0.5, 0.0, z as f32 + 0.5), c);
        }
    }
    let (conc, conc_dark, conc_deep) = match look {
        1 => (Color::from_rgba(122, 90, 60, 255), Color::from_rgba(102, 74, 50, 255), Color::from_rgba(46, 84, 50, 255)),
        2 => (Color::from_rgba(196, 190, 180, 255), Color::from_rgba(170, 164, 156, 255), Color::from_rgba(96, 100, 116, 255)),
        3 => (Color::from_rgba(40, 34, 44, 255), Color::from_rgba(26, 22, 30, 255), Color::from_rgba(110, 16, 30, 255)),
        _ => (Color::from_rgba(150, 146, 162, 255), Color::from_rgba(118, 114, 132, 255), Color::from_rgba(78, 74, 94, 255)),
    };
    let open_back = look == 1 || look == 2;
    for level in 0..if open_back { 1 } else { 9 } {
        let c = if level % 2 == 0 { conc } else { conc_dark };
        b.cube(o + vec3(CH_SIZE.0 / 2.0, 0.2 + level as f32 * 0.4, -0.15), vec3(CH_SIZE.0 + 0.6, 0.4, 0.3), c);
    }
    for level in 0..5 {
        let c = if level % 2 == 0 { conc_dark } else { conc };
        b.cube(o + vec3(-0.15, 0.2 + level as f32 * 0.4, CH_SIZE.1 / 2.0), vec3(0.3, 0.4, CH_SIZE.1), c);
        b.cube(o + vec3(CH_SIZE.0 + 0.15, 0.2 + level as f32 * 0.4, CH_SIZE.1 / 2.0), vec3(0.3, 0.4, CH_SIZE.1), c);
    }
    if !open_back {
        b.cube(o + vec3(CH_SIZE.0 / 2.0, 3.7, -0.05), vec3(CH_SIZE.0 + 0.8, 0.3, 0.5), conc_deep);
    }
    for (x, z) in PILLARS {
        for level in 0..7 {
            let c = if level % 2 == 0 { conc } else { conc_dark };
            b.cube(o + vec3(x, 0.2 + level as f32 * 0.4, z), vec3(0.42, 0.4, 0.42), c);
        }
        b.cube(o + vec3(x, 2.9, z), vec3(0.6, 0.16, 0.6), conc_deep);
        if look == 1 {
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU + x;
                b.cube(o + vec3(x + a.cos() * 0.5, 2.95 - (k % 2) as f32 * 0.14, z + a.sin() * 0.5), vec3(0.5, 0.1, 0.5), Color::from_rgba(40, 110 + (k * 9) as u8, 56, 255));
            }
        }
        if look == 3 {
            b.cube(o + vec3(x, 1.4, z), vec3(0.46, 0.14, 0.46), Color::from_rgba(150, 20, 36, 255));
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_2;
                b.cube(o + vec3(x + a.cos() * 0.23, 1.4, z + a.sin() * 0.23), vec3(0.07, 0.07, 0.07), Color::from_rgba(200, 200, 210, 255));
            }
        }
    }
    b.unlit = true;
    let glow = 0.35 + 0.65 * pulse;
    backdrop(&mut b, o, look, time, pulse);
    for (i, (wx, ww, wh)) in [(2.0f32, 0.8f32, 1.7f32), (5.5, 1.1, 2.1), (9.0, 0.8, 1.7)].iter().enumerate() {
        if look != 0 {
            break;
        }
        let c = hsv((0.86 - i as f32 * 0.16 + (bar / 4) as f32 * 0.09).rem_euclid(1.0), 0.75, glow);
        lancet(&mut b, o.x + wx, o.y + 0.9, o.z + 0.02, *ww, *wh, c, 0.04);
        b.cube(o + vec3(*wx, 0.9 + wh * 0.5, 0.05), vec3(0.05, *wh, 0.02), Color::from_rgba(14, 10, 26, 255));
        for k in 1..4 {
            b.cube(o + vec3(*wx, 0.9 + wh * k as f32 / 4.0, 0.05), vec3(*ww, 0.04, 0.02), Color::from_rgba(14, 10, 26, 255));
        }
    }
    for k in 0..if look == 0 { 10 } else { 0 } {
        let a = k as f32 / 10.0 * std::f32::consts::TAU + time * 0.3;
        let c = hsv((0.8 + k as f32 * 0.03).fract(), 0.7, glow);
        b.cube(o + vec3(3.75 + a.cos() * 0.32, 2.6 + a.sin() * 0.32, 0.03), vec3(0.13, 0.13, 0.03), c);
        b.cube(o + vec3(7.25 + a.cos() * 0.32, 2.6 + a.sin() * 0.32, 0.03), vec3(0.13, 0.13, 0.03), c);
    }
    b.unlit = false;
    b.cube(booth + vec3(0.0, 0.3, 0.0), vec3(2.4, 0.6, 0.6), conc_dark);
    b.cube(booth + vec3(0.0, 0.62, 0.0), vec3(2.6, 0.06, 0.7), conc_deep);
    for dx in [-0.75f32, 0.75] {
        b.cube(booth + vec3(dx, 0.69, 0.0), vec3(0.6, 0.06, 0.46), Color::from_rgba(26, 24, 38, 255));
        let spin = time * 4.0 * dx.signum();
        b.box_at(booth + vec3(dx - 0.06, 0.74, 0.0), spin, Vec3::ZERO, vec3(0.34, 0.03, 0.34), Color::from_rgba(50, 46, 70, 255));
        b.box_at(booth + vec3(dx - 0.06, 0.76, 0.0), spin, vec3(0.1, 0.0, 0.0), vec3(0.06, 0.02, 0.06), Color::from_rgba(255, 61, 242, 255));
    }
    b.cube(booth + vec3(0.0, 0.7, 0.0), vec3(0.5, 0.08, 0.44), Color::from_rgba(26, 24, 38, 255));
    b.unlit = true;
    b.cube(booth + vec3(0.0, 0.3, 0.31), vec3(2.3, 0.05, 0.01), Color::new(1.0, 0.24, 0.95, 1.0));
    for k in 0..4 {
        let lvl = (0.3 + 0.7 * pulse * hash(k, (time * 8.0) as i32, 5)).min(1.0);
        b.cube(booth + vec3(-0.15 + k as f32 * 0.1, 0.76, 0.1), vec3(0.04, 0.02, 0.12 * lvl), Color::from_rgba(61, 242, 255, 255));
    }
    b.unlit = false;
    for sx in [3.1f32, 7.9] {
        for level in 0..3 {
            let sp = o + vec3(sx, 0.3 + level as f32 * 0.6, 1.05);
            b.cube(sp, vec3(0.7, 0.58, 0.6), Color::from_rgba(30, 28, 44, 255));
            let throb = 0.3 + 0.08 * pulse;
            b.cube(sp + vec3(0.0, 0.0, 0.31), vec3(throb, throb, 0.02), Color::from_rgba(58, 54, 82, 255));
        }
    }
    let duck = g.cast_list.iter().position(|c| matches!(c.body, Body::Duck)).unwrap_or(0);
    let mut dj = Player::new(duck, g.cast_list[duck].body, CH_ORIGIN + vec2(BOOTH.0, BOOTH.1 - 0.75));
    dj.yaw = 0.0;
    dj.lift = pulse * 0.05;
    dj.draw(&mut b, &g.world, time, pulse);
    for (i, c) in g.venue.crowd.iter_mut().enumerate() {
        let off = hash(i as i32, 7, 44);
        c.lift = (pulse * (0.04 + off * 0.07)).max(0.0);
        c.voice_timer = if drop && off > 0.6 && pulse > 0.7 { 0.1 } else { 0.0 };
    }
    for c in g.venue.crowd.iter() {
        c.draw(&mut b, &g.world, time, pulse);
    }
    for p in g.players.iter() {
        p.draw(&mut b, &g.world, time, pulse);
    }
    let mut tags: Vec<(Vec3, String, Color)> = Vec::new();
    for peer in g.venue.peers.iter() {
        let mut friend = Player::new(peer.character, g.cast_list[peer.character].body, CH_ORIGIN + peer.pos);
        friend.facing = peer.facing;
        friend.yaw = peer.facing.x.atan2(peer.facing.y);
        friend.walking = peer.walking;
        friend.voice_timer = peer.sing;
        friend.lift = if peer.walking { 0.0 } else { pulse * 0.05 };
        friend.draw(&mut b, &g.world, time, pulse);
        let name = g.venue.friends.iter().find(|f| f.id == peer.id).map(|f| f.name.clone()).unwrap_or_default();
        let is_dj = g.venue.friends.iter().find(|f| f.id == peer.id).map(|f| f.dj).unwrap_or(false);
        let is_admin = g.venue.friends.iter().find(|f| f.id == peer.id).map(|f| f.admin).unwrap_or(false);
        let name = if peer.slot == 1 { format!("{name} +1") } else if is_admin { format!("* {name}") } else if is_dj { format!("dj {name}") } else { name };
        let colour = if is_admin { Color::from_rgba(255, 214, 110, 255) } else if is_dj { Color::from_rgba(196, 130, 255, 255) } else { Color::from_rgba(61, 242, 255, 255) };
        tags.push((vec3(CH_ORIGIN.x + peer.pos.x, 1.75, CH_ORIGIN.y + peer.pos.y), name, colour));
    }
    if net::state() == 2 {
        let mine = if g.venue.dj_name.is_some() { format!("dj {}", g.venue.name) } else { g.venue.name.clone() };
        tags.push((vec3(g.players[0].pos.x, 1.75, g.players[0].pos.y), mine, if g.venue.dj_name.is_some() { Color::from_rgba(196, 130, 255, 255) } else { Color::from_rgba(255, 214, 110, 255) }));
        if g.two_player {
            tags.push((vec3(g.players[1].pos.x, 1.75, g.players[1].pos.y), format!("{} +1", g.venue.name), Color::from_rgba(255, 214, 110, 255)));
        }
    }
    b.unlit = true;
    let beams = if drop { 9 } else { 5 };
    for i in 0..beams {
        let k = i as f32 / (beams - 1) as f32 - 0.5;
        let yaw = k * 2.1 + (time * (0.7 + 0.2 * i as f32) + i as f32).sin() * 0.25;
        let al = 0.3 + 0.4 * pulse;
        let c = match (look, i % 2 == 0) {
            (1, true) => Color::new(1.0, 0.75, 0.2, al),
            (1, false) => Color::new(0.3, 1.0, 0.45, al),
            (2, true) => Color::new(1.0, 0.5, 0.3, al),
            (2, false) => Color::new(0.4, 0.8, 1.0, al),
            (3, true) => Color::new(1.0, 0.1, 0.2, al),
            (3, false) => Color::new(1.0, 0.3, 0.7, al),
            (_, true) => Color::new(0.78, 1.0, 0.24, al),
            _ => Color::new(1.0, 0.24, 0.95, al),
        };
        b.box_at(booth + vec3(0.0, 1.75, 0.2), yaw, vec3(0.0, 0.0, 4.6), vec3(0.03, 0.03, 9.2), c);
    }
    b.flush();
    blit(g);
    for (at, text, colour) in tags.iter() {
        name_tag(view, *at, text, *colour);
    }
    if drop && pulse > 0.85 {
        ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(1.0, 1.0, 1.0, 0.06));
    }
    let Some(r) = rave else {
        return;
    };
    let on_air = g.venue.on_air.clone();
    let live = !g.venue.live.is_empty() || on_air.is_some();
    let section = if live { "live" } else if bar < 4 { "intro" } else if (16..20).contains(&bar) { "breakdown" } else if drop { "drop" } else { "groove" };
    let line = if let Some(o) = &on_air {
        format!("live: dj {} - {}", o.name, if o.audio { "real audio" } else { "on the decks" })
    } else if live {
        format!("live: {}", g.venue.live)
    } else { format!("generated set - {} - {} bpm - {}", SOUNDS[r.sound % 3], g.venue.bpm_now as i32, section)
    };
    let w = (ui_measure(&line, 9.0) + 40.0).max(ui_measure(&r.title, 14.0) + 40.0).min(low_w() - 12.0);
    let x = (low_w() - w) / 2.0;
    let slim = look != 0;
    let hud_y = if slim { -17.0 } else { 0.0 };
    ui_rect(x, 5.0, w, 34.0 + hud_y, Color::new(0.2, 0.16, 0.4, 0.88));
    if !slim {
        ui_text(&r.title, (low_w() - ui_measure(&r.title, 14.0)) / 2.0, 19.0, 14.0, cream());
    }
    let shown = ui_ticker(&line, 9.0, w - 36.0, time);
    ui_text(&shown, x + 8.0, 33.0 + hud_y, 9.0, if live { Color::from_rgba(255, 110, 200, 255) } else { Color::from_rgba(198, 255, 61, 255) });
    let beat_now = ((get_time() - g.venue.rave_start) * g.venue.bpm_now as f64 / 60.0).rem_euclid(4.0) as usize;
    for k in 0..4 {
        ui_rect(x + w - 26.0 + k as f32 * 5.0, 28.0 + hud_y, 4.0, 4.0, if k == beat_now { Color::from_rgba(255, 214, 110, 255) } else { ink() });
    }
    let code = format!("code {}", r.code);
    let cw = ui_measure(&code, 9.0) + 10.0;
    let (code_x, code_y) = if slim { (6.0, low_h() - 18.0) } else { ((low_w() - cw) / 2.0, 42.0) };
    ui_rect(code_x, code_y, cw, 12.0, Color::new(0.14, 0.1, 0.28, 0.85));
    ui_text(&code, code_x + 5.0, code_y + 9.0, 9.0, cream());
    tour_banner(g, fr, if look != 0 { 26.0 } else { 58.0 });
    place_label("st. bassilica", if g.venue.chat_open {
        "enter: send   esc: close"
    } else if at_decks {
        "E: the decks   enter: chat"
    } else if can_chat {
        "wasd: move   1-6: sing   enter: chat   esc or the door: leave"
    } else {
        "wasd: move   1-6: sing over the set   esc or the door: leave"
    });
    if can_chat && g.venue.overlay == Overlay::None {
        let now = get_time();
        let open = g.venue.chat_open;
        let cw = 190.0f32.min(low_w() - 12.0);
        let base = low_h() - 36.0;
        let mut rows: Vec<(Color, String, String, f32)> = Vec::new();
        for (kind, name, text, at) in g.venue.chat_log.iter().rev() {
            let age = (now - *at) as f32;
            let alpha = if open { 1.0 } else { (1.0 - (age - 14.0) / 3.0).clamp(0.0, 1.0) };
            if alpha <= 0.0 || rows.len() >= if open { 9 } else { 5 } {
                break;
            }
            let (colour, shown) = match kind {
                2 => (Color::from_rgba(255, 214, 110, 255), format!("* {name}")),
                1 => (Color::from_rgba(196, 130, 255, 255), format!("dj {name}")),
                _ => (Color::from_rgba(61, 242, 255, 255), name.clone()),
            };
            let lead = ui_measure(&shown, 8.0) + 5.0;
            let wrapped = ui_wrap(text, 8.0, cw - lead - 6.0);
            for (i, part) in wrapped.iter().enumerate().rev() {
                rows.push((colour, if i == 0 { shown.clone() } else { String::new() }, part.clone(), alpha));
            }
        }
        for (i, (colour, name, text, alpha)) in rows.iter().enumerate() {
            let ry = base - 12.0 - i as f32 * 10.0;
            let lead = if name.is_empty() { 8.0 } else { ui_measure(name, 8.0) + 5.0 };
            ui_rect(6.0, ry, cw, 10.0, Color::new(0.08, 0.06, 0.16, 0.6 * alpha));
            ui_text(name, 9.0, ry + 7.5, 8.0, Color::new(colour.r, colour.g, colour.b, *alpha));
            ui_text(text, 9.0 + lead, ry + 7.5, 8.0, Color::new(1.0, 0.97, 0.88, *alpha));
        }
        if open {
            ui_rect(6.0, base, cw, 13.0, ink());
            ui_rect(7.0, base + 1.0, cw - 2.0, 11.0, cream());
            let tail: String = g.venue.chat_draft.chars().rev().take(40).collect::<Vec<char>>().into_iter().rev().collect();
            ui_text(&tail, 10.0, base + 9.5, 8.0, ink());
            if (time * 2.0).fract() < 0.6 {
                ui_rect(11.0 + ui_measure(&tail, 8.0), base + 3.0, 1.0, 7.0, ink());
            }
        } else if radio::touch_ui() {
            if button(fr, "chat", 6.0, base, 34.0, 13.0, false) {
                if let Some(text) = radio::ask_text("say something to the room", "") {
                    let text: String = text.chars().filter(|c| !c.is_control()).take(120).collect();
                    if !text.trim().is_empty() {
                        net::send(&format!("say {}", text.trim()));
                    }
                }
                fr.click = false;
            }
        }
    }
    if let Some(o) = &on_air {
        if o.audio && !net::can_listen() && !o.url.is_empty() && g.venue.overlay == Overlay::None {
            let label = "listen in your browser";
            let bw = ui_measure(label, 9.0) + 14.0;
            if button(fr, label, (low_w() - bw) / 2.0, 58.0, bw, 14.0, false) {
                net::open_url(&o.url);
                fr.click = false;
            }
        }
    }
    let status = match net::state() {
        2 if g.venue.joined => format!("online - {} of 100 here", here(g)),
        2 => "online".to_string(),
        1 => "connecting...".to_string(),
        _ => "offline".to_string(),
    };
    let sw = ui_measure(&status, 9.0) + 18.0;
    ui_rect(low_w() - sw - 6.0, 6.0, sw, 14.0, Color::new(0.14, 0.1, 0.28, 0.85));
    ui_rect(low_w() - sw, 11.0, 4.0, 4.0, if net::state() == 2 { Color::from_rgba(140, 222, 168, 255) } else { Color::from_rgba(150, 146, 162, 255) });
    ui_text(&status, low_w() - sw + 8.0, 16.5, 9.0, cream());
    mail_icon(g, fr);
}
