#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// Reads both players' directions and the door keys.
pub(crate) fn read_movement(g: &mut Game, fr: &mut Frame) {
    fr.p1_dir = if g.two_player {
        direction(&[KeyCode::W], &[KeyCode::S], &[KeyCode::A], &[KeyCode::D])
    } else {
        direction(
            &[KeyCode::W, KeyCode::Up],
            &[KeyCode::S, KeyCode::Down],
            &[KeyCode::A, KeyCode::Left],
            &[KeyCode::D, KeyCode::Right],
        )
    };
    fr.p2_dir = direction(
        &[KeyCode::I, KeyCode::Up],
        &[KeyCode::K, KeyCode::Down],
        &[KeyCode::J, KeyCode::Left],
        &[KeyCode::L, KeyCode::Right],
    );
    (fr.p1_dir, fr.p2_dir) = if g.picking { (Vec2::ZERO, Vec2::ZERO) } else { (fr.p1_dir, fr.p2_dir) };
    if g.fishing.active && [fr.p1_dir, fr.p2_dir][g.fishing.caster] != Vec2::ZERO {
        g.fishing.active = false;
        g.toasts.push(("reeled in".to_string(), 1.5));
    }
    fr.go_in = false;
    fr.go_out = false;
    if g.inside {
        let h = &g.house;
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            let dir = [fr.p1_dir, fr.p2_dir][who];
            let walk_dir = if (g.shop_open && g.shop_owner == who) || g.dj[who] || g.ddr.on[who] || (fr.bag_busy && g.bag_owner == who) || (g.sampler.open && g.sampler.owner == who) { Vec2::ZERO } else { dir };
            g.players[who].step_with(walk_dir, fr.dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
            if home::local(g.players[who].pos).y > g.house.d() as f32 + 0.15 {
                fr.go_out = true;
            }
        }
    } else {
        g.players[0].step(fr.p1_dir, &g.world, fr.dt);
        if g.two_player {
            g.players[1].step(fr.p2_dir, &g.world, fr.dt);
        }
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            let dir = [fr.p1_dir, fr.p2_dir][who];
            if dir.y < -0.5 && g.players[who].pos.distance(DOOR_OUT) < 0.55 && !g.picking {
                fr.go_in = true;
            }
        }
    }
    if !g.players[0].seated && !(g.two_player && g.players[1].seated) {
        g.cooking = false;
    }
    fr.fish_press = [false; 2];
    fr.build_request = None;
}

/// Everything a player can do outside: walk, gather, fish, cook, sing.
pub(crate) fn outdoors(g: &mut Game, fr: &mut Frame) {
    if !g.inside && !g.picking && !fr.picker_closed {
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            let pressed = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
            if !pressed {
                continue;
            }
            if g.fish_card > 0.0 {
                g.fish_card = 0.0;
                g.fishing.cue_at = next_beat(g.looper.position) + FISH_CUE_GAP;
                continue;
            }
            if g.cook_card > 0.0 {
                g.cook_card = 0.0;
                continue;
            }
            if g.gather_card > 0.0 {
                g.gather_card = 0.0;
                continue;
            }
            let player = &mut g.players[who];
            if g.fishing.active && player.pos.distance(DOCK_END) < 2.2 {
                if g.fish_card > 0.0 {
                    g.fish_card = 0.0;
                    g.fishing.cue_at = next_beat(g.looper.position) + FISH_CUE_GAP;
                } else {
                    fr.fish_press[who] = true;
                }
            } else if player.seated {
                if g.camp.lit && near_fire(player) {
                    g.cooking = !g.cooking;
                    g.cook_progress = 0;
                    g.music_mode = true;
                    if g.cooking && !g.cook_lesson {
                        g.cook_card = 7.0;
                        g.cook_lesson = true;
                        g.save_clock = 99.0;
                    }
                    g.toasts.push((if g.cooking { "the pot goes on the fire".to_string() } else { "the pot comes off".to_string() }, 2.0));
                }
            } else if g.camp.lit && near_fire(player) {
                player.seated = true;
                player.place_at(if who == 0 { CAT_SEAT } else { DOG_SEAT });
                player.facing = if who == 0 { vec2(1.0, 0.0) } else { vec2(-1.0, 0.0) };
                g.music_mode = true;
                g.toasts.push((format!("player {} sits by the fire", who + 1), 2.5));
            } else if player.pos.distance(DOOR_OUT) < 0.8 {
                fr.go_in = true;
            } else if at_dock_end(player) && !g.fishing.active {
                player.place_at(DOCK_END);
                player.facing = vec2(1.0, 0.0);
                let lesson = !g.fish_lesson;
                if lesson {
                    g.fish_card = 6.0;
                }
                let wait = if lesson { (g.fish_card / g.looper.step_seconds() / 4.0).ceil() * 4.0 } else { 0.0 };
                g.fishing = Fishing {
                    active: true,
                    caster: who,
                    cue_at: next_beat(g.looper.position) + FISH_CUE_GAP + wait,
                    hits: 0,
                    tries: FISH_TRIES,
                    hit_by: [false; 2],
                    early: [false; 2],
                    lesson,
                    perfects: 0,
                    dip: 0.0,
                    jump: None,
                    tight: if lesson { 1.0 } else { 0.72 + 0.2 * hash((fr.time * 41.0) as i32, 6, 2) },
                };
                g.audio.play_ch(3, &g.splash, 0.35);
                if !lesson {
                    let key = if who == 0 { "E" } else { "O" };
                    g.toasts.push((format!("cast the line - {key} when the ring closes"), 2.5));
                }
            } else if let Some(k) = {
                let (ax, az) = player.ahead();
                let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                g.shrooms.iter().position(|(at, _)| at.distance(player.pos) < 0.9 || at.distance(probe) < 0.8)
            } {
                let (at, species) = g.shrooms.remove(k);
                let is_new = g.shroom_counts[species] == 0;
                g.shroom_counts[species] += 1;
                g.inv.acorns += mushroom::acorns(species);
                let note = if mushroom::edible(species) {
                    g.inv.mush[mushroom::kind(species)] += 1;
                    "into the basket"
                } else {
                    "not for eating - logged in the dex"
                };
                g.fish_pop = Some((species, 3.0, is_new, true));
                quest_event(&mut g.quests, QuestKind::Mushroom, 1, &mut g.inv, &mut g.toasts);
                player.sniff = 0.6;
                g.audio.play_ch(3, &g.plink, 0.45);
                g.audio.play_ch(who, &g.voices[player.character][3], 0.5);
                player.voice_timer = 0.3;
                let col = mushroom::look(species).cap;
                for n in 0..10 {
                    spawn(&mut g.particles, vec3(at.x, 0.35, at.y), vec3(hash(n, 3, 5) * 1.2 - 0.6, 1.0, hash(n, 4, 5) * 1.2 - 0.6), 0.7, col, 0.03, 0.6);
                }
                g.toasts.push((format!("player {} found a {} - {} (+{} acorns)", who + 1, mushroom::SPECIES[species], note, mushroom::acorns(species)), 3.0));
                g.save_clock = 99.0;
            } else if let Some(tile) = {
                let (ax, az) = player.ahead();
                let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                if player.carrying.is_none() {
                    g.stick_trees.iter().copied().find(|t| stick_spot(&g.world, *t).map(|p| p.distance(player.pos) < 0.95 || p.distance(probe) < 0.8).unwrap_or(false))
                } else {
                    None
                }
            } {
                g.stick_trees.retain(|t| *t != tile);
                if !g.gather_lesson {
                    g.gather_card = 7.0;
                    g.gather_lesson = true;
                    g.save_clock = 99.0;
                }
                let spot = stick_spot(&g.world, tile).unwrap_or(player.pos);
                player.carrying = Some(Carry::Stick);
                player.sniff = 0.6;
                puff(&mut g.particles, spot.x.floor() as i32, spot.y.floor() as i32, Color::from_rgba(110, 170, 80, 255), 8);
                g.audio.play_ch(3, &g.chop, 0.35);
                g.toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                quest_event(&mut g.quests, QuestKind::Sticks, 1, &mut g.inv, &mut g.toasts);
            } else {
                let (x, z) = player.ahead();
                match gather_at(&g.world, &mut g.stick_trees, &mut g.tree_rest, (x, z), fr.time, player.carrying.is_none()) {
                    Gathered::Stick => {
                        quest_event(&mut g.quests, QuestKind::Sticks, 1, &mut g.inv, &mut g.toasts);
                        if !g.gather_lesson {
                            g.gather_card = 7.0;
                            g.gather_lesson = true;
                            g.save_clock = 99.0;
                        }
                        player.carrying = Some(Carry::Stick);
                        player.sniff = 0.6;
                        puff(&mut g.particles, x, z, Color::from_rgba(110, 170, 80, 255), 8);
                        g.audio.play_ch(3, &g.chop, 0.35);
                        g.toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                    }
                    Gathered::Stone => {
                        if !g.gather_lesson {
                            g.gather_card = 7.0;
                            g.gather_lesson = true;
                            g.save_clock = 99.0;
                        }
                        player.carrying = Some(Carry::Stone);
                        player.sniff = 0.6;
                        puff(&mut g.particles, x, z, Color::from_rgba(170, 170, 180, 255), 8);
                        g.toasts.push((format!("player {} got a stone - bring it to the fire", who + 1), 2.5));
                    }
                    Gathered::Acorns(found) => {
                        quest_event(&mut g.quests, QuestKind::Shake, 1, &mut g.inv, &mut g.toasts);
                        if !g.gather_lesson {
                            g.gather_card = 7.0;
                            g.gather_lesson = true;
                            g.save_clock = 99.0;
                        }
                        g.inv.acorns += found;
                        g.audio.play_ch(3, &g.chop, 0.35);
                        g.audio.play_ch(3, &g.plink, 0.3);
                        player.sniff = 0.4;
                        puff(&mut g.particles, x, z, Color::from_rgba(110, 170, 80, 255), 10);
                        for k in 0..found as i32 * 2 {
                            spawn(&mut g.particles, vec3(x as f32 + 0.5, 1.6, z as f32 + 0.5), vec3(hash(k, x, 3) * 1.2 - 0.6, 0.6, hash(k, z, 4) * 1.2 - 0.6), 0.9, ACORN_COLOR, 0.06, 1.0);
                        }
                        g.toasts.push((format!("player {} shook the tree: +{} acorn{}", who + 1, found, if found == 1 { "" } else { "s" }), 2.5));
                    }
                    Gathered::Resting => g.toasts.push(("this tree needs a rest - try another".to_string(), 2.0)),
                    Gathered::Nothing => if !g.fishing.active { fr.build_request = Some((who, x, z)); },
                }
            }
        }
    }
}

/// Walking in and out of the cottage.
pub(crate) fn doors(g: &mut Game, fr: &mut Frame) {
    if fr.go_in && !g.inside {
        g.inside = true;
        g.fishing.active = false;
        g.cooking = false;
        for p in g.players.iter_mut() {
            p.seated = false;
            p.sleeping = false;
        }
        g.seat = [None, None];
        g.dj = [false, false];
        g.shop_open = false;
        let door = g.house.door();
        g.players[0].place_at(home::world(door - vec2(0.0, 0.45)));
        g.players[0].facing = vec2(0.0, -1.0);
        g.players[1].place_at(home::world(door - vec2(1.0, 0.35)));
        g.players[1].facing = vec2(0.0, -1.0);
        g.wander_clock = 2.0;
        g.audio.play_ch(3, &g.chop, 0.3);
        g.toasts.push(("home sweet home".to_string(), 2.5));
    }
    if fr.go_out && g.inside {
        if g.holding.is_some() {
            g.players[g.holding_by].place_at(home::world(vec2(3.5, 4.0)));
            g.toasts.push(("put your furniture down first".to_string(), 2.0));
        } else {
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
            let spot = rescue(&g.world, DOOR_OUT + vec2(1.0, 0.4));
            g.players[1].place_at(spot);
            g.brain = DogBrain::new();
            g.audio.play_ch(3, &g.chop, 0.3);
        }
    }
}

/// Places or removes a wall where a player asked.
pub(crate) fn build_walls(g: &mut Game, fr: &mut Frame) {
    if let Some((who, x, z)) = fr.build_request {
        let blocked = g.players.iter().any(|p| touches_cat(x, z, p.pos));
        match build(&mut g.world, &mut g.inv, g.tools[who], x, z, blocked) {
            Ok(Some((kind, stack))) => {
                let i = ((x + z + stack as i32).rem_euclid(6)) as usize;
                match kind {
                    Kind::WoodWall => g.audio.play_ch(3, &g.marimba[i], 0.8),
                    Kind::StoneWall => g.audio.play_ch(3, &g.bells[i], 0.8),
                    _ => g.audio.play_ch(3, &g.chop, 0.7),
                }
                puff(&mut g.particles, x, z, Color::from_rgba(230, 210, 170, 255), 6);
                g.players[who].voice_timer = 0.2;
            }
            Ok(None) => {}
            Err(msg) => g.toasts.push((msg.to_string(), 2.5)),
        }
    }
}
