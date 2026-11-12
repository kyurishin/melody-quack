#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn jam_and_fire(g: &mut Game, fr: &mut Frame) {
    fr.was_jam = g.jam;
    g.jam = !g.inside && (g.players[0].seated || (g.two_player && g.players[1].seated));
    if g.jam && !fr.was_jam {
        g.music_mode = true;
        if g.first_steps == 2 {
            g.first_steps = 3;
            g.toasts.push(("play together to earn your first star".to_string(), 3.0));
        }
    }

    if g.camp.lit {
        g.camp.fuel -= fr.dt;
        g.camp.pile = (g.camp.fuel / STICK_FUEL).ceil().max(0.0) as u32;
        if g.camp.fuel <= 0.0 {
            g.camp.fuel = 0.0;
            g.camp.lit = false;
            g.cooking = false;
            g.toasts.push(("the fire went out".to_string(), 3.0));
            for p in g.players.iter_mut() {
                p.seated = false;
            }
        }
        g.crackle_clock -= fr.dt;
        if g.crackle_clock <= 0.0 {
            g.crackle_clock = 0.6 + hash((fr.time * 10.0) as i32, 3, 9) * 1.2;
            let dist = g.players[0].pos.distance(FIRE_SPOT);
            let vol = (1.0 - dist / 7.0).max(0.0) * 0.35;
            if vol > 0.01 {
                g.audio.play_ch(3, &g.crackle, vol);
            }
        }
        if hash((fr.time * 60.0) as i32, 5, 1) > 0.7 {
            let c = fire_center();
            spawn(&mut g.particles, c + vec3(hash((fr.time * 97.0) as i32, 1, 2) * 0.4 - 0.2, 0.4, 0.1), vec3(hash((fr.time * 31.0) as i32, 2, 2) * 0.6 - 0.3, 1.6, 0.0), 0.9, Color::from_rgba(255, 180, 70, 255), 0.022, 0.05);
        }
    }
    if g.cooking && hash((fr.time * 40.0) as i32, 9, 2) > 0.8 {
        let c = fire_center();
        spawn(&mut g.particles, c + vec3(hash((fr.time * 53.0) as i32, 1, 3) * 0.3 - 0.15, 0.62, 0.06), vec3(0.0, 0.5, 0.0), 1.6, Color::from_rgba(240, 240, 245, 255), 0.05, -0.03);
    }
    if hash((fr.time * 30.0) as i32, 8, 1) > 0.85 {
        let (_, z0, x1, _) = COTTAGE;
        spawn(&mut g.particles, vec3(x1 as f32 - 0.6, 2.7, z0 as f32 + 0.6), vec3(0.15, 0.5, 0.0), 2.6, Color::from_rgba(220, 220, 228, 255), 0.06, -0.02);
    }
}
