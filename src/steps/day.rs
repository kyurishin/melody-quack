#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn day_and_mushrooms(g: &mut Game, fr: &mut Frame) {
    fr.real_night = update_sky().night * 0.95;
    {
        let sk = sky();
        if g.quests.day != sk.day {
            g.quests.day = sk.day;
            g.quests.list = daily_quests(sk.day, g.house.has(home::Item::Plant), g.house.has(home::Item::Arcade));
            let mut restored = false;
            if let Some((day, list)) = g.saved_quests.take() {
                if day == sk.day {
                    for (q, (kind, target, progress, done)) in g.quests.list.iter_mut().zip(list.into_iter()) {
                        if q.kind as u8 == kind && q.target == target {
                            q.progress = progress;
                            q.done = done;
                            restored = true;
                        }
                    }
                }
            }
            if !restored {
                g.toasts.push(("3 new daily quests - see them in your bag (M)".to_string(), 3.5));
            }
        }
        for piece in g.house.pieces.iter_mut().filter(|p| p.item == home::Item::Plant) {
            if piece.day == 0 {
                piece.day = sk.day as i32;
            } else if sk.day as i32 - piece.day >= 2 && piece.state != home::WITHERED {
                piece.state = home::WITHERED;
            }
        }
        let mut batch = 1;
        if sk.day != g.shroom_day {
            g.shroom_day = sk.day;
            g.shroom_today = mushroom::today(sk.day);
            g.shrooms.retain(|(_, sp)| g.shroom_today.contains(sp));
            g.shroom_clock = 0.0;
            batch = 6;
        }
        g.shroom_clock -= fr.dt;
        if g.shroom_clock <= 0.0 {
            g.shroom_clock = if sk.weather == 2 { 7.0 } else { 14.0 };
            for round in 0..batch {
            if g.shrooms.len() < 10 && !g.shroom_today.is_empty() {
                let seed = (fr.time * 97.0) as i32 + round * 131;
                for attempt in 0..30 {
                    let tx = (hash(seed, attempt, 31) * MAP as f32) as i32;
                    let tz = (hash(seed, attempt, 32) * MAP as f32) as i32;
                    if g.world.get(tx, tz).map(|t| t.kind == Kind::Tree).unwrap_or(false) {
                        let dx = (hash(seed, attempt, 33) * 3.0) as i32 - 1;
                        let dz = (hash(seed, attempt, 34) * 3.0) as i32 - 1;
                        let (gx, gz) = (tx + dx, tz + dz);
                        let ok = g.world.get(gx, gz).map(|t| matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Tree)).unwrap_or(false);
                        let at = vec2(gx as f32 + 0.2 + hash(seed, attempt, 35) * 0.6, gz as f32 + 0.2 + hash(seed, attempt, 36) * 0.6);
                        let trunk_clear = (dx != 0 || dz != 0) || at.distance(vec2(tx as f32 + 0.5, tz as f32 + 0.5)) > 0.3;
                        if ok && trunk_clear && g.shrooms.iter().all(|(o, _)| o.distance(at) > 1.0) {
                            let species = mushroom::pick_from(&g.shroom_today, hash(seed, attempt, 37));
                            g.shrooms.push((at, species));
                            break;
                        }
                    }
                }
            }
            }
        }
        SHROOMS.with(|c| *c.borrow_mut() = g.shrooms.clone());
    }
    fr.dusk_target = fr.real_night.max(if g.jam { 0.45 } else if g.camp.lit { 0.12 } else { 0.0 });
    g.dusk += (fr.dusk_target - g.dusk) * (fr.dt * 0.4).min(1.0);
    if g.jam && !g.two_player {
        let c = g.players[0].cam;
        g.players[0].cam = c + (vec2(8.5, 17.0) - c) * (fr.dt * 2.0).min(1.0);
    }
}
