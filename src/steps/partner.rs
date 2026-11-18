#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub(crate) enum Job {
    #[default]
    None,
    Fish,
    Cook,
    Sit,
}

#[derive(Default)]
pub(crate) struct Partner {
    pub(crate) log: Vec<(bool, String)>,
    pub(crate) shown: f32,
    pub(crate) waiting: f32,
    pub(crate) draft: String,
    pub(crate) noticed: bool,
    pub(crate) job: Job,
    pub(crate) job_t: f32,
    pub(crate) walk_t: f32,
    pub(crate) want: Option<(Carry, u8)>,
    pub(crate) later: Option<String>,
    pub(crate) hum: Vec<(f32, usize)>,
    pub(crate) hum_t: f32,
    pub(crate) hum_wait: f32,
    pub(crate) scroll: usize,
    pub(crate) drag: bool,
}

pub(crate) const LOG_KEEP: usize = 300;

pub(crate) fn names(name: &str, noun: &str) -> Vec<String> {
    let mut out = vec![noun.to_lowercase(), name.to_lowercase()];
    if let Some(last) = name.split_whitespace().last() {
        out.push(last.to_lowercase());
    }
    out.dedup();
    out
}

pub(crate) fn called(text: &str, name: &str, noun: &str) -> bool {
    let t = format!(" {} ", text.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect::<String>());
    names(name, noun).iter().any(|n| {
        let n = n.trim();
        !n.is_empty() && (t.contains(&format!(" {n} ")) || t.contains(&format!(" {n}s ")) || t.contains(&format!(" {n}y ")) || t.contains(&format!(" {n}ie ")))
    })
}

pub(crate) const CALLS: [&str; 4] = ["partner", "buddy", "friend", "pal"];

pub(crate) fn called_any(text: &str, cast: &[Character]) -> bool {
    cast.iter().any(|c| called(text, c.name, c.noun)) || CALLS.iter().any(|w| called(text, w, w))
}

fn friend(g: &Game) -> (&'static str, &'static str) {
    let ch = &g.cast_list[g.players[1].character];
    (ch.name, ch.noun)
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub(crate) fn context(g: &Game) -> String {
    let sk = sky();
    let mut bits = vec![format!("{:02}:{:02}, {}, {}", sk.hour as u32, (sk.hour.fract() * 60.0) as u32, SEASONS[sk.season], WEATHERS[sk.weather])];
    bits.push(if g.inside { "inside the house".to_string() } else { "outside by the campfire clearing".to_string() });
    bits.push(if g.camp.lit { format!("the campfire is lit ({}% fuel)", ((g.camp.fuel / 180.0).min(1.0) * 100.0) as u32) } else { "the campfire is out".to_string() });
    let fish: u32 = g.inv.fish.iter().sum();
    let mush: u32 = g.inv.mush.iter().sum();
    bits.push(format!("bag: {} fish, {} mushrooms, {} wood, {} stones, {} acorns", fish, mush, g.inv.wood, g.inv.stone, g.inv.acorns));
    bits.push(format!("dishes discovered {}/{}, tunes you know {}", g.dishes.len(), DISHES.len(), g.songbook.len()));
    if g.jam {
        bits.push("the player is sitting by the fire playing music".to_string());
    }
    if g.fishing.active {
        bits.push("the player is fishing".to_string());
    }
    let p = &g.venue.partner;
    bits.push(match (p.job, p.want) {
        (Job::Fish, _) => "you are off fishing".to_string(),
        (Job::Cook, _) => "you are cooking".to_string(),
        (Job::Sit, _) => "you are sitting with the player".to_string(),
        (_, Some((c, n))) => format!("you are collecting {} ({} left)", carry_word(c), n),
        _ => "you are hanging out".to_string(),
    });
    bits.join("; ")
}

fn carry_word(c: Carry) -> &'static str {
    match c {
        Carry::Stick => "sticks",
        Carry::Stone => "stones",
        Carry::Acorn => "acorns",
    }
}

pub(crate) fn can_talk(g: &Game) -> bool {
    !g.picking && !g.show_help && !g.shop_open && !g.sampler.open && !g.dj[0] && !g.fishing.active && !g.paused && !g.dex_open && !g.shelf_open && !(g.ddr.on[0] || g.ddr.on[1])
}

fn push(g: &mut Game, otter: bool, text: String) {
    let p = &mut g.venue.partner;
    p.log.push((otter, text));
    if p.log.len() > LOG_KEEP {
        p.log.remove(0);
    }
    p.shown = 14.0;
    p.scroll = 0;
}

pub(crate) fn send(g: &mut Game, text: &str) {
    let text: String = text.chars().filter(|c| !c.is_control()).take(160).collect::<String>().trim().to_string();
    if text.is_empty() {
        return;
    }
    let history: Vec<(bool, String)> = g.venue.partner.log.iter().rev().take(8).rev().cloned().collect();
    push(g, false, text.clone());
    g.venue.partner.noticed = true;
    let (_, noun) = friend(g);
    if !called_any(&text, &g.cast_list) {
        return;
    }
    if !g.companion && !g.two_player {
        push(g, true, format!("(the {noun} is resting. esc, then C, brings it along)"));
        return;
    }
    if g.two_player {
        push(g, true, format!("(player 2 is the {noun} right now, so they can answer you)"));
        return;
    }
    if net::state() != 2 {
        push(g, true, format!("(the {noun} can't hear you while you're offline)"));
        return;
    }
    let hist = history.iter().map(|(o, t)| format!("[{},{}]", esc(if *o { "otter" } else { "you" }), esc(t))).collect::<Vec<_>>().join(",");
    let who = "player".to_string();
    let line = format!("otter {{\"friend\":{},\"who\":{},\"ctx\":{},\"history\":[{}],\"text\":{}}}", esc(g.cast_list[g.players[1].character].name), esc(&who), esc(&context(g)), hist, esc(&text));
    net::send(&line);
    g.venue.partner.waiting = 25.0;
}

pub(crate) fn reply(g: &mut Game, action: &str, say: &str) {
    g.venue.partner.waiting = 0.0;
    push(g, true, say.to_string());
    let c = g.players[1].character;
    g.audio.play_ch(1, &g.voices[c][2], 0.45);
    g.players[1].voice_timer = 0.4;
    act(g, action);
}

pub(crate) fn refused(g: &mut Game, why: &str) {
    g.venue.partner.waiting = 0.0;
    let noun = friend(g).1;
    let line = match why {
        "busy" => format!("(one thing at a time! give the {noun} a second)"),
        "tired" => format!("(the {noun} has chatted a lot today. it'll be chatty again tomorrow)"),
        "closed" => format!("(the {noun} is napping. back tomorrow)"),
        _ => format!("(the {noun} can't talk right now)"),
    };
    push(g, true, line);
}

pub(crate) fn act(g: &mut Game, action: &str) {
    if g.inside && matches!(action, "cook" | "fish" | "sit" | "collect_sticks" | "collect_stones" | "collect_acorns") {
        g.venue.partner.later = Some(action.to_string());
        return;
    }
    let p = &mut g.venue.partner;
    let job = match action {
        "fish" => Job::Fish,
        "cook" => Job::Cook,
        "sit" => Job::Sit,
        _ => Job::None,
    };
    match action {
        "fish" | "cook" | "sit" => {
            p.job = job;
            p.job_t = 0.0;
            p.walk_t = 0.0;
            p.want = None;
            g.brain.task = Task::Idle;
            g.players[1].carrying = None;
            g.players[1].seated = false;
        }
        "collect_sticks" | "collect_stones" | "collect_acorns" => {
            let carry = match action {
                "collect_sticks" => Carry::Stick,
                "collect_stones" => Carry::Stone,
                _ => Carry::Acorn,
            };
            p.job = Job::None;
            p.want = Some((carry, 3));
            g.brain.rest = 0.0;
            if !matches!(g.brain.task, Task::Seek { .. } | Task::Sniff { .. } | Task::Return) {
                g.brain.task = Task::Idle;
            }
            g.players[1].seated = false;
        }
        "follow" => {
            p.job = Job::None;
            p.want = None;
            g.brain.task = Task::Idle;
            g.brain.rest = 40.0;
            g.players[1].seated = false;
        }
        "hum" => start_hum(g),
        _ => {}
    }
}

pub(crate) fn start_hum(g: &mut Game) {
    if !g.venue.partner.hum.is_empty() {
        return;
    }
    let tune: Vec<(f32, usize)> = if !g.songbook.is_empty() {
        let k = (g.world.rand() * g.songbook.len() as f32) as usize % g.songbook.len();
        g.songbook[k].iter().map(|&(at, n)| (at as f32 * 0.16, n.min(5))).collect()
    } else {
        let mut t = 0.0;
        let mut note = 2usize;
        (0..7)
            .map(|_| {
                let step = [0usize, 1, 2, 4, 5][(g.world.rand() * 5.0) as usize % 5];
                note = (note + step) % 6;
                let at = t;
                t += 0.28 + g.world.rand() * 0.2;
                (at, note)
            })
            .collect()
    };
    g.venue.partner.hum = tune;
    g.venue.partner.hum_t = 0.0;
}

pub(crate) fn tick(g: &mut Game, dt: f32) {
    {
        let p = &mut g.venue.partner;
        p.shown = (p.shown - dt).max(0.0);
        if p.waiting > 0.0 {
            p.waiting -= dt;
            if p.waiting <= 0.0 {
                p.waiting = 0.0;
                p.log.push((true, "(no answer. maybe ask again?)".to_string()));
                p.shown = 8.0;
            }
        }
    }
    if !g.inside {
        if let Some(a) = g.venue.partner.later.take() {
            act(g, &a);
        }
    }
    if !g.venue.partner.hum.is_empty() {
        g.venue.partner.hum_t += dt;
        let now = g.venue.partner.hum_t;
        let due: Vec<usize> = g.venue.partner.hum.iter().filter(|(t, _)| *t <= now).map(|(_, n)| *n).collect();
        g.venue.partner.hum.retain(|(t, _)| *t > now);
        let c = g.players[1].character;
        for n in due {
            g.audio.play_ch(1, &g.voices[c][n], 0.3);
            g.players[1].voice_timer = 0.25;
            g.players[1].key_glow[n] = 0.6;
            let at = vec3(g.players[1].pos.x, 1.0 + g.players[1].lift, g.players[1].pos.y);
            spawn(&mut g.particles, at, vec3(0.2, 0.9, 0.0), 1.2, note_color(n), 0.05, -0.2);
        }
    }
    let p = &mut g.venue.partner;
    if p.hum_wait <= 0.0 {
        p.hum_wait = 60.0 + g.world.rand() * 90.0;
    }
    p.hum_wait -= dt;
    let near = g.players[1].pos.distance(g.players[0].pos) < 4.0;
    if p.hum_wait <= 0.0 {
        p.hum_wait = 70.0 + g.world.rand() * 90.0;
        if g.companion && !g.two_player && near && !g.jam && !g.cooking && !g.fishing.active && !g.players[0].sleeping {
            start_hum(g);
        }
    }
}

pub(crate) fn job_step(g: &mut Game, dt: f32) -> Option<Vec2> {
    if g.inside || g.two_player || !g.companion {
        return None;
    }
    let job = g.venue.partner.job;
    if job == Job::None {
        return None;
    }
    let noun = friend(g).1;
    let pos = g.players[1].pos;
    let goal = match job {
        Job::Fish => DOCK_END - vec2(0.7, 0.0),
        Job::Cook => DOG_SEAT,
        Job::Sit => g.players[0].pos + vec2(0.75, 0.25),
        Job::None => return None,
    };
    if job == Job::Cook && !g.camp.lit {
        g.venue.partner.job = Job::None;
        push(g, true, format!("(the fire is out, so the {noun} can't cook. light it first)"));
        return None;
    }
    if job == Job::Sit && g.players[1].seated {
        let far = g.players[1].pos.distance(g.players[0].pos) > 2.2;
        g.venue.partner.job_t += dt;
        if far || g.venue.partner.job_t > 40.0 {
            g.players[1].seated = false;
            g.venue.partner.job = Job::None;
            return None;
        }
        g.players[1].facing = (g.players[0].pos - g.players[1].pos).normalize_or_zero();
        return Some(Vec2::ZERO);
    }
    if pos.distance(goal) > 0.35 && !(job == Job::Cook && g.players[1].seated) {
        let p = &mut g.venue.partner;
        p.walk_t += dt;
        if p.walk_t > 20.0 {
            p.walk_t = 0.0;
            let spot = rescue(&g.world, goal);
            g.players[1].place_at(spot);
        }
        g.players[1].seated = false;
        return Some(steer(&g.world, pos, goal));
    }
    match job {
        Job::Sit => {
            g.players[1].seated = true;
            g.venue.partner.job_t = 0.0;
            Some(Vec2::ZERO)
        }
        Job::Fish => {
            g.players[1].facing = vec2(1.0, 0.0);
            g.players[1].sniff = 0.2;
            g.venue.partner.job_t += dt;
            if g.venue.partner.job_t > 7.0 {
                let species = fish::pick(g.world.rand(), g.world.rand(), 0.0);
                let kind = fish::kind(species);
                g.dex_counts[species] += 1;
                g.inv.fish[kind] += 1;
                g.inv.acorns += fish::acorns(species);
                quest_event(&mut g.quests, QuestKind::Fish, 1, &mut g.inv, &mut g.toasts);
                g.audio.play_ch(3, &g.splash, 0.5);
                let c = g.players[1].character;
                g.audio.play_ch(1, &g.voices[c][4], 0.6);
                g.players[1].voice_timer = 0.5;
                g.toasts.push((format!("the {noun} caught a {}! +{} acorns", fish::SPECIES[species], fish::acorns(species)), 3.5));
                g.venue.partner.job = Job::None;
                g.brain.rest = 6.0;
                g.save_clock = 99.0;
            }
            Some(Vec2::ZERO)
        }
        Job::Cook => {
            g.players[1].place_at(DOG_SEAT);
            g.players[1].seated = true;
            g.players[1].facing = vec2(-1.0, 0.0);
            let before = g.venue.partner.job_t;
            g.venue.partner.job_t += dt;
            if (before * 2.0).floor() != (g.venue.partner.job_t * 2.0).floor() {
                let c = fire_center() + vec3(0.0, 0.6, 0.08);
                for k in 0..5 {
                    let a = k as f32 * 1.3 + g.venue.partner.job_t;
                    spawn(&mut g.particles, c, vec3(a.cos() * 0.6, 1.1, a.sin() * 0.3), 0.6, Color::from_rgba(255, 190, 110, 255), 0.03, 0.5);
                }
            }
            if g.venue.partner.job_t > 6.0 {
                let (dish, new, earned) = steps::notes::cook_dish(g, None);
                g.toasts.push((if new { format!("the {noun} cooked a new dish: {}! +{} acorns", DISHES[dish], earned) } else { format!("the {noun} cooked {} - yum, +{} acorns", DISHES[dish], earned) }, 3.5));
                let c = g.players[1].character;
                g.audio.play_ch(1, &g.voices[c][0], 0.6);
                g.players[1].voice_timer = 0.5;
                g.players[1].seated = false;
                g.venue.partner.job = Job::None;
                g.brain.rest = 6.0;
                g.save_clock = 99.0;
            }
            Some(Vec2::ZERO)
        }
        Job::None => None,
    }
}

pub(crate) fn draw_card(g: &Game, fr: &Frame) {
    let p = &g.venue.partner;
    if (p.shown <= 0.0 && p.waiting <= 0.0) || g.picking || g.paused || p.log.is_empty() && p.waiting <= 0.0 {
        return;
    }
    let noun = friend(g).1;
    let w = 240f32.min(low_w() - 12.0);
    let mut lines: Vec<(Color, String, f32)> = Vec::new();
    for (k, (otter, text)) in p.log.iter().rev().take(2).rev().enumerate() {
        let (who, col) = if *otter { (noun, ui::skin::GOLD) } else { ("you", ui::skin::CREAM) };
        for (i, l) in ui_wrap(&format!("{who}: {text}"), 8.0, w - 14.0).into_iter().enumerate() {
            lines.push((col, l, if i == 0 && k > 0 { 3.0 } else { 0.0 }));
        }
    }
    if p.waiting > 0.0 {
        let dots = ".".repeat(1 + ((fr.time * 3.0) as usize % 3));
        lines.push((ui::skin::GOLD, format!("{noun}: {dots}"), 3.0));
    }
    if lines.len() > 10 {
        lines.drain(..lines.len() - 10);
        lines[0].2 = 0.0;
    }
    let h = 8.0 + lines.iter().enumerate().map(|(i, l)| 9.5 + if i > 0 { l.2 } else { 0.0 }).sum::<f32>();
    let x = 6.0;
    let bottom = if fr.show_grid { fr.grid_y - 31.0 } else { fr.bar_y - 26.0 };
    let y = (bottom - h).max(44.0);
    let fade = p.shown.min(1.0).max(if p.waiting > 0.0 { 1.0 } else { 0.0 });
    let panel = Color::new(ui::skin::PANEL.r, ui::skin::PANEL.g, ui::skin::PANEL.b, 0.9 * fade);
    let edge = Color::new(ui::skin::EDGE.r, ui::skin::EDGE.g, ui::skin::EDGE.b, fade);
    ui_round_frame(x, y, w, h, 6.0, panel, edge);
    let mut ly = y + 11.0;
    for (i, (c, l, lead)) in lines.iter().enumerate() {
        if i > 0 {
            ly += *lead;
        }
        ui_text(l, x + 7.0, ly, 8.0, Color::new(c.r, c.g, c.b, fade));
        ly += 9.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_partner_answers_to_its_own_animal() {
        let cast = roster();
        let otter = cast.iter().find(|c| c.noun == "otter").unwrap();
        let o = |t: &str| called(t, otter.name, otter.noun);
        assert!(o("hey otter, wanna fish?"));
        assert!(o("OTTER!"));
        assert!(o("hi sea otter"));
        assert!(o("you otters are cute"));
        assert!(!o("hotter today"));
        assert!(!o("let's cook"));
        let lab = cast.iter().find(|c| c.name.contains("labrador")).unwrap();
        assert!(called("good dog", lab.name, lab.noun));
        assert!(called("labrador, come here", lab.name, lab.noun));
        assert!(!called("hey otter", lab.name, lab.noun));
        assert!(called_any("hey owl", &cast) && called_any("fox, come here", &cast) && called_any("hi buddy", &cast));
        assert!(called_any("ginger tabby!", &cast) && called_any("hey shiba", &cast));
        assert!(!called_any("what a nice day", &cast));
    }

    #[test]
    fn messages_go_out_as_valid_json() {
        let s = esc("say \"hi\" \\ back\nnow");
        assert_eq!(s, "\"say \\\"hi\\\" \\\\ back now\"");
    }
}
