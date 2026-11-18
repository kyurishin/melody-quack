#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) struct Peer {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) to: Vec2,
    pub(crate) here: bool,
}

pub(crate) struct Away {
    pub(crate) host_name: String,
    pub(crate) own: home::Home,
    pub(crate) companion: bool,
}

#[derive(Default)]
pub(crate) struct Visits {
    pub(crate) code: String,
    pub(crate) sent_code: bool,
    pub(crate) draft: String,
    pub(crate) status: String,
    pub(crate) knocks: Vec<(u32, usize, String)>,
    pub(crate) peers: Vec<Peer>,
    pub(crate) bodies: Vec<Player>,
    pub(crate) away: Option<Away>,
    pub(crate) clock: f32,
    pub(crate) gone_clock: f32,
    pub(crate) sharing: bool,
    pub(crate) heard: f64,
}

pub(crate) fn home_code(token: u64) -> String {
    let mut r = ((token ^ (token >> 29) ^ 0x5bd1_e995) as u32) | 1;
    venues::make_code(&mut r)
}

pub(crate) fn my_code(g: &mut Game) -> String {
    if venues::tidy_code(&g.venue.visits.code).is_none() {
        g.venue.visits.code = home_code(g.venue.token ^ (get_time() * 1000.0) as u64);
        g.save_clock = 99.0;
    }
    g.venue.visits.code.clone()
}

pub(crate) fn visiting(g: &Game) -> bool {
    g.venue.visits.away.is_some()
}

pub(crate) fn own_house(g: &Game) -> &home::Home {
    g.venue.visits.away.as_ref().map(|a| &a.own).unwrap_or(&g.house)
}

fn wire_name(name: &str) -> String {
    let n: String = name.trim().replace(' ', "_").chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').take(16).collect();
    if n.is_empty() { "friend".to_string() } else { n }
}

pub(crate) fn show_name(name: &str) -> String {
    name.replace('_', " ")
}

pub(crate) fn knock(g: &mut Game) {
    let Some(code) = venues::tidy_code(&g.venue.visits.draft) else {
        g.venue.visits.status = "a home code is 9 letters and numbers, like ACD-EFG-HJK".to_string();
        return;
    };
    if code == my_code(g) {
        g.venue.visits.status = "that's your own home code! share it with a friend".to_string();
        return;
    }
    if net::state() != 2 {
        g.venue.visits.status = "you need to be online to visit".to_string();
        return;
    }
    if visiting(g) {
        g.venue.visits.status = "go home first, then knock on another door".to_string();
        return;
    }
    net::send(&format!("knock {} {} {}", code, g.players[0].character, wire_name(&g.venue.name)));
    g.venue.visits.status = "knocking...".to_string();
}

pub(crate) fn answer(g: &mut Game, id: u32, yes: bool) {
    g.venue.visits.knocks.retain(|k| k.0 != id);
    if yes {
        let house = own_house(g).to_text();
        net::send(&format!("letin {} {} {} {}", id, g.players[0].character, wire_name(&g.venue.name), house));
    } else {
        net::send(&format!("turnaway {id}"));
    }
}

pub(crate) fn on_message(g: &mut Game, parts: &[&str]) -> bool {
    match parts.first().copied().unwrap_or("") {
        "knock" if parts.len() >= 4 => {
            if let (Ok(id), Ok(ch)) = (parts[1].parse::<u32>(), parts[2].parse::<usize>()) {
                if !g.venue.visits.knocks.iter().any(|k| k.0 == id) {
                    g.venue.visits.knocks.push((id, ch % g.cast_list.len(), show_name(parts[3])));
                    g.audio.play_ch(3, &g.bells[2], 0.5);
                }
            }
        }
        "knocking" => g.venue.visits.status = "knock knock... waiting for them to answer".to_string(),
        "nobody" => g.venue.visits.status = "nobody's home with that code right now".to_string(),
        "turnedaway" => {
            g.venue.visits.status = "they can't have visitors right now".to_string();
            g.toasts.push(("they can't have visitors right now".to_string(), 3.0));
        }
        "welcome" if parts.len() >= 5 => {
            let (Ok(host), Some(house)) = (parts[1].parse::<u32>(), home::Home::from_text(parts[4])) else {
                return true;
            };
            let name = show_name(parts[3]);
            go_inside(g, host, name, house);
        }
        "hostgone" => {
            if visiting(g) {
                go_home(g, "your host went offline, so you headed home");
            }
        }
        "hpeer" if parts.len() >= 9 => {
            let (Ok(id), Ok(ch)) = (parts[1].parse::<u32>(), parts[2].parse::<usize>()) else { return true };
            let n: Vec<f32> = parts[4..9].iter().filter_map(|v| v.parse().ok()).collect();
            if n.len() < 5 {
                return true;
            }
            let ch = ch % g.cast_list.len();
            let to = home::world(vec2(n[0], n[1]));
            let here = n[4] < 8.5;
            let v = &mut g.venue.visits;
            if let Some(k) = v.peers.iter().position(|p| p.id == id) {
                v.peers[k].to = to;
                v.peers[k].here = here;
                if v.bodies[k].character != ch {
                    v.bodies[k] = Player::new(ch, g.cast_list[ch].body, to);
                }
                v.bodies[k].facing = vec2(n[2], n[3]);
            } else {
                let name = show_name(parts[3]);
                if !visiting(g) {
                    g.toasts.push((format!("{name} came in. welcome!"), 3.0));
                }
                let v = &mut g.venue.visits;
                v.peers.push(Peer { id, name, to, here });
                let mut body = Player::new(ch, g.cast_list[ch].body, to);
                body.facing = vec2(n[2], n[3]);
                v.bodies.push(body);
            }
        }
        "hsnd" if parts.len() >= 2 => {
            if !visiting(g) || !g.inside {
                return true;
            }
            for pair in parts[1].split(',').take(64) {
                if let Some((id, v)) = pair.split_once(':') {
                    if let (Ok(id), Ok(v)) = (id.parse::<u16>(), v.parse::<f32>()) {
                        if g.audio.shared_id_ok(id) && v.is_finite() {
                            g.audio.play_id(id, v.clamp(0.0, 1.5) * 0.9);
                        }
                    }
                }
            }
            let now = get_time();
            if now - g.venue.visits.heard > 20.0 {
                let host = g.venue.visits.away.as_ref().map(|a| a.host_name.clone()).unwrap_or_default();
                g.toasts.push((format!("{host} is on the decks"), 3.0));
            }
            g.venue.visits.heard = now;
        }
        "hgone" if parts.len() >= 2 => {
            if let Ok(id) = parts[1].parse::<u32>() {
                let v = &mut g.venue.visits;
                if let Some(k) = v.peers.iter().position(|p| p.id == id) {
                    let name = v.peers[k].name.clone();
                    v.peers.remove(k);
                    v.bodies.remove(k);
                    if !visiting(g) {
                        g.toasts.push((format!("{name} went home"), 3.0));
                    }
                }
            }
        }
        _ => return false,
    }
    true
}

fn go_inside(g: &mut Game, host: u32, host_name: String, house: home::Home) {
    if let Some(a) = g.venue.visits.away.take() {
        g.house = a.own;
        g.companion = a.companion;
    }
    let own = std::mem::replace(&mut g.house, house);
    let _ = host;
    g.venue.visits.away = Some(Away { host_name: host_name.clone(), own, companion: g.companion });
    g.back_to_pause = false;
    g.paused = false;
    g.venue.visits.peers.clear();
    g.venue.visits.bodies.clear();
    g.venue.visits.status.clear();
    g.venue.overlay = venues::Overlay::None;
    if g.venue.at != venues::Venue::None {
        g.venue.at = venues::Venue::None;
    }
    g.companion = false;
    g.inside = true;
    g.fishing.active = false;
    g.cooking = false;
    g.jam = false;
    for p in g.players.iter_mut() {
        p.seated = false;
        p.sleeping = false;
    }
    g.seat = [None, None];
    g.dj = [false, false];
    g.shop_open = false;
    g.show_inventory = false;
    g.holding = None;
    let door = g.house.door();
    g.players[0].place_at(home::world(door - vec2(0.0, 0.45)));
    g.players[0].facing = vec2(0.0, -1.0);
    g.players[1].place_at(home::world(door - vec2(1.0, 0.35)));
    g.audio.play_ch(3, &g.chop, 0.3);
    g.toasts.push((format!("welcome to {host_name}'s home!"), 3.5));
}

pub(crate) fn go_home(g: &mut Game, why: &str) {
    let Some(a) = g.venue.visits.away.take() else { return };
    g.back_to_pause = false;
    g.paused = false;
    g.house = a.own;
    g.companion = a.companion;
    g.venue.visits.peers.clear();
    g.venue.visits.bodies.clear();
    if net::state() == 2 {
        net::send("homeleave");
    }
    g.inside = false;
    g.seat = [None, None];
    for p in g.players.iter_mut() {
        p.seated = false;
        p.sleeping = false;
    }
    g.dj = [false, false];
    g.shop_open = false;
    g.players[0].place_at(DOOR_OUT + vec2(0.0, 0.25));
    g.players[0].facing = vec2(0.0, 1.0);
    g.players[1].place_at(DOOR_OUT + vec2(-0.8, 0.4));
    g.toasts.push((why.to_string(), 3.0));
}

fn share_set(g: &mut Game, online: bool) {
    let hosting = online && !visiting(g) && g.inside && g.dj[0] && g.venue.visits.peers.iter().any(|p| p.here);
    g.audio.share(hosting);
    if hosting && !g.venue.visits.sharing {
        g.toasts.push(("your guests can hear your set".to_string(), 3.0));
    }
    g.venue.visits.sharing = hosting;
    if hosting {
        let sounds = g.audio.take_shared();
        if !sounds.is_empty() {
            let list: Vec<String> = sounds.iter().map(|(id, v)| format!("{}:{:.2}", id, v.clamp(0.0, 1.5))).collect();
            net::send(&format!("hsnd {}", list.join(",")));
        }
    }
}

pub(crate) fn tick(g: &mut Game, dt: f32) {
    let online = net::state() == 2;
    share_set(g, online);
    if !online {
        g.venue.visits.sent_code = false;
        if visiting(g) {
            g.venue.visits.gone_clock += dt;
            if g.venue.visits.gone_clock > 6.0 {
                g.venue.visits.gone_clock = 0.0;
                go_home(g, "the connection dropped, so you headed home");
            }
        }
        return;
    }
    g.venue.visits.gone_clock = 0.0;
    if !g.venue.visits.sent_code {
        let code = my_code(g);
        net::send(&format!("homecode {code}"));
        g.venue.visits.sent_code = true;
    }
    for (p, b) in g.venue.visits.peers.iter().zip(g.venue.visits.bodies.iter_mut()) {
        let before = b.pos;
        b.pos += (p.to - b.pos) * (dt * 9.0).min(1.0);
        b.cam = b.pos;
        b.walking = b.pos.distance(before) > 0.004;
        let want = b.facing.x.atan2(b.facing.y);
        let mut diff = want - b.yaw;
        while diff > std::f32::consts::PI {
            diff -= std::f32::consts::TAU;
        }
        while diff < -std::f32::consts::PI {
            diff += std::f32::consts::TAU;
        }
        b.yaw += diff * (dt * 12.0).min(1.0);
    }
    let in_group = visiting(g) || !g.venue.visits.peers.is_empty();
    if !in_group {
        return;
    }
    g.venue.visits.clock -= dt;
    if g.venue.visits.clock > 0.0 {
        return;
    }
    let me = &g.players[0];
    if g.inside {
        g.venue.visits.clock = 0.12;
        let l = home::local(me.pos);
        let st = if me.seated { 2 } else if me.walking { 1 } else { 0 };
        net::send(&format!("hat {:.2} {:.2} {:.2} {:.2} {}", l.x, l.y, me.facing.x, me.facing.y, st));
    } else {
        g.venue.visits.clock = 1.0;
        net::send("hat 0 0 0 1 9");
    }
}

pub(crate) fn shown_bodies(g: &Game) -> Vec<&Player> {
    if !g.inside {
        return Vec::new();
    }
    g.venue.visits.peers.iter().zip(g.venue.visits.bodies.iter()).filter(|(p, _)| p.here).map(|(_, b)| b).collect()
}

pub(crate) fn hud_line(g: &Game) -> Option<String> {
    let v = &g.venue.visits;
    if let Some(a) = &v.away {
        return Some(format!("visiting {}'s home. walk out the door to go home", a.host_name));
    }
    let here: Vec<&str> = v.peers.iter().filter(|p| p.here && g.inside).map(|p| p.name.as_str()).collect();
    if here.is_empty() {
        None
    } else {
        Some(format!("visiting you: {}", here.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_home_travels_as_text_and_comes_back_the_same() {
        let mut h = home::Home::new();
        h.size = 2;
        h.wall = 1;
        h.floor = 3;
        h.pieces.push(home::Piece { item: home::Item::from_index(5), x: 2, z: 3, rot: 1, state: 1, day: 7 });
        let text = h.to_text();
        assert!(!text.contains(' '));
        let back = home::Home::from_text(&text).unwrap();
        assert_eq!((back.size, back.wall, back.floor, back.pieces.len()), (2, 1, 3, 1));
        assert_eq!((back.pieces[0].x, back.pieces[0].z, back.pieces[0].rot, back.pieces[0].state), (2, 3, 1, 1));
        assert!(home::Home::from_text("junk").is_none());
    }

    #[test]
    fn home_codes_are_stable_and_valid() {
        let a = home_code(12345);
        assert_eq!(a, home_code(12345));
        assert_ne!(a, home_code(54321));
        assert_eq!(venues::tidy_code(&a), Some(a.clone()));
    }
}
