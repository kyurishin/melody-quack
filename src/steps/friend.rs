#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// Player two or the helper dog, stick deliveries and bedtime.
pub(crate) fn friend_and_deliveries(g: &mut Game, fr: &mut Frame) {
    fr.delivered = None;
    fr.delivered_by = 1usize;
    if !g.inside && g.players[0].carrying.is_some() && g.players[0].pos.distance(STICK_DROP) < 0.9 {
        fr.delivered = g.players[0].carrying.take();
        fr.delivered_by = 0;
    }
    if g.two_player && !g.inside {
        if g.players[1].carrying.is_some() && g.players[1].pos.distance(STICK_DROP) < 0.9 {
            fr.delivered = g.players[1].carrying.take();
            fr.delivered_by = 1;
        }
    } else if !g.inside {
        let cat_pos = g.players[0].pos;
        let dog = &mut g.players[1];
        let want_fire = g.camp.fuel < FUEL_WANT;
        let mut dir = Vec2::ZERO;
        let mut next: Option<Task> = None;
        g.brain.rest = (g.brain.rest - fr.dt).max(0.0);
        if g.first_pick {
            g.brain.rest = g.brain.rest.max(0.5);
        }
        match &mut g.brain.task {
            Task::Idle => {
                dog.seated = false;
                let errand = if g.first_pick || (g.jam && g.camp.fuel > 40.0) {
                    None
                } else if want_fire {
                    Some(Carry::Stick)
                } else if g.brain.rest > 0.0 || g.fishing.active {
                    None
                } else if g.inv.wood < 8 && g.inv.wood <= g.inv.stone {
                    Some(Carry::Stick)
                } else if g.inv.stone < 8 {
                    Some(Carry::Stone)
                } else if g.inv.wood < 8 {
                    Some(Carry::Stick)
                } else {
                    Some(Carry::Acorn)
                };
                if let (Some(carry), None) = (errand, dog.carrying) {
                    let kind = if carry == Carry::Stone { Kind::Stone } else { Kind::Tree };
                    let marked = if carry == Carry::Stick { nearest_stick_tree(&g.world, &g.stick_trees, dog.pos) } else { None };
                    let found = match marked {
                        Some(m) => Some(m),
                        None => find_source(&mut g.world, dog.pos, kind),
                    };
                    if let Some((tile, spot)) = found {
                        next = Some(Task::Seek { spot, tile, carry });
                        g.brain.trip = 0.0;
                    }
                } else if dog.carrying.is_some() {
                    next = Some(Task::Return);
                } else if g.jam {
                    next = Some(Task::Sit);
                } else {
                    let goal = if g.fishing.active { DOCK_END - vec2(1.0, 0.0) } else { cat_pos };
                    if dog.pos.distance(goal) > if g.fishing.active { 0.3 } else { 2.0 } {
                        dir = steer(&g.world, dog.pos, goal);
                    } else if g.fishing.active {
                        dog.facing = vec2(1.0, 0.0);
                    }
                }
            }
            Task::Seek { spot, tile, carry } => {
                dir = steer(&g.world, dog.pos, *spot);
                if dog.pos.distance(*spot) < 0.3 {
                    dir = Vec2::ZERO;
                    let t = *tile;
                    dog.facing = (vec2(t.0 as f32 + 0.5, t.1 as f32 + 0.5) - dog.pos).normalize_or_zero();
                    next = Some(Task::Sniff { tile: t, timer: 1.2, carry: *carry });
                }
            }
            Task::Sniff { tile, timer, carry } => {
                *timer -= fr.dt;
                dog.sniff = 0.2;
                if *timer <= 0.0 {
                    if *carry == Carry::Stick {
                        let t = *tile;
                        g.stick_trees.retain(|s| *s != t);
                    }
                    dog.carrying = Some(*carry);
                    let col = match *carry {
                        Carry::Stick => Color::from_rgba(110, 170, 80, 255),
                        Carry::Stone => Color::from_rgba(170, 170, 180, 255),
                        Carry::Acorn => ACORN_COLOR,
                    };
                    puff(&mut g.particles, tile.0, tile.1, col, 8);
                    next = Some(Task::Return);
                    g.brain.trip = 0.0;
                }
            }
            Task::Return => {
                dir = steer(&g.world, dog.pos, STICK_DROP);
                if dog.pos.distance(STICK_DROP) < 0.45 {
                    fr.delivered = dog.carrying.take();
                    g.brain.rest = if want_fire { 0.0 } else if fr.delivered == Some(Carry::Acorn) { 14.0 } else { 5.0 };
                    next = Some(Task::Idle);
                }
            }
            Task::Sit => {
                if !g.jam || g.camp.fuel < 40.0 {
                    dog.seated = false;
                    next = Some(Task::Idle);
                } else if !dog.seated {
                    dir = steer(&g.world, dog.pos, DOG_SEAT);
                    if dog.pos.distance(DOG_SEAT) < 0.45 {
                        dir = Vec2::ZERO;
                        dog.place_at(DOG_SEAT);
                        dog.seated = true;
                        dog.facing = vec2(-1.0, 0.0);
                    }
                }
            }
        }
        let target = match &g.brain.task {
            Task::Seek { spot, .. } => Some(*spot),
            Task::Return => Some(STICK_DROP),
            _ => None,
        };
        let moved = dog.step(dir, &g.world, fr.dt);
        if dir != Vec2::ZERO && moved < 0.3 * fr.dt {
            g.brain.stuck += fr.dt;
        } else {
            g.brain.stuck = 0.0;
        }
        g.brain.trip += fr.dt;
        if let Some(goal) = target {
            if g.brain.stuck > 1.2 {
                let hop = dog.pos + (goal - dog.pos).normalize_or_zero() * 1.5;
                let spot = rescue(&g.world, hop);
                dog.place_at(spot);
                g.brain.stuck = 0.0;
            }
            if g.brain.trip > 25.0 {
                let spot = rescue(&g.world, goal);
                dog.place_at(spot);
                g.brain.trip = 0.0;
            }
        } else if g.brain.stuck > 2.0 {
            let spot = rescue(&g.world, dog.pos + vec2(0.0, 1.0));
            dog.place_at(spot);
            g.brain.stuck = 0.0;
        }
        if let Some(t) = next {
            g.brain.task = t;
        }
    }

    if g.inside && !g.two_player && g.players[1].sleeping && !g.players[0].sleeping {
        g.players[1].sleeping = false;
        g.players[1].seated = false;
        g.players[1].perch = 0.0;
        let h = &g.house;
        let spot = h.free_spot_near(home::local(g.players[1].pos));
        g.players[1].place_at(home::world(spot));
    }
    if g.inside && !g.two_player && !g.players[1].sleeping {
        let h = &g.house;
        g.wander_clock -= fr.dt;
            let goal = if g.seat[0].is_some() || g.dj[0] { g.players[0].pos + vec2(0.9, 0.4) } else { g.wander_to };
            let to_goal = goal - g.players[1].pos;
            let dir = if to_goal.length() > 0.35 { to_goal.normalize() } else { Vec2::ZERO };
            let moved = g.players[1].step_with(dir, fr.dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
            if g.wander_clock <= 0.0 || (dir != Vec2::ZERO && moved < 0.2 * fr.dt) {
                g.wander_clock = 3.0 + hash((fr.time * 11.0) as i32, 6, 1) * 4.0;
                g.wander_to = home::world(h.free_spot(fr.time * 0.731 + 0.17));
            }
            if dir == Vec2::ZERO && (g.seat[0].is_some() || g.dj[0]) {
                g.players[1].facing = (g.players[0].pos - g.players[1].pos).normalize_or_zero();
            }
    }
    for p in g.players.iter_mut() {
        p.lift = if g.inside { g.house.height_at(home::local(p.pos)) } else { 0.0 };
    }
}

/// What happens when someone brings something to the fire.
pub(crate) fn receive_delivery(g: &mut Game, fr: &mut Frame) {
    fr.friend = g.cast_list[g.players[1].character].noun;
    fr.bringer = if fr.delivered_by == 0 { "you".to_string() } else if g.two_player { "player 2".to_string() } else { format!("the {}", fr.friend) };
    fr.bringer_voice = g.players[fr.delivered_by].character;
    fr.bringer_ch = fr.delivered_by;
    match fr.delivered {
        Some(Carry::Stick) if g.camp.fuel < FUEL_WANT => {
            g.camp.fuel = (g.camp.fuel + STICK_FUEL).min(240.0);
            g.camp.pile = (g.camp.fuel / STICK_FUEL).ceil() as u32;
            g.audio.play_ch(fr.bringer_ch, &g.voices[fr.bringer_voice][2], 0.7);
            g.players[fr.delivered_by].voice_timer = 0.4;
            if !g.camp.lit {
                g.camp.lit = true;
                g.toasts.push((format!("{} brought a stick - the fire is lit!", fr.bringer), 3.5));
                for k in 0..30 {
                    let a = k as f32 / 30.0 * 2.0 * PI;
                    spawn(&mut g.particles, fire_center() + vec3(0.0, 0.3, 0.0), vec3(a.cos() * 1.5, 2.0 + hash(k, 1, 3) * 1.5, a.sin() * 1.5), 0.9, Color::from_rgba(255, 190, 80, 255), 0.04, 0.6);
                }
            } else {
                g.toasts.push((format!("{} brought a stick for the fire", fr.bringer), 2.5));
            }
        }
        Some(Carry::Stick) => {
            g.inv.wood += 1;
            g.audio.play_ch(fr.bringer_ch, &g.voices[fr.bringer_voice][3], 0.6);
            g.players[fr.delivered_by].voice_timer = 0.4;
            g.toasts.push((format!("{} brought wood for building", fr.bringer), 2.5));
        }
        Some(Carry::Acorn) => {
            g.inv.acorns += 2;
            g.audio.play_ch(1, &g.voices[g.players[1].character][1], 0.6);
            g.players[1].voice_timer = 0.4;
            g.toasts.push((format!("the {} brought you 2 acorns", fr.friend), 2.5));
        }
        Some(Carry::Stone) => {
            g.inv.stone += 1;
            g.audio.play_ch(fr.bringer_ch, &g.voices[fr.bringer_voice][4], 0.6);
            g.players[fr.delivered_by].voice_timer = 0.4;
            g.toasts.push((format!("{} brought a stone for building", fr.bringer), 2.5));
        }
        None => {}
    }
}
