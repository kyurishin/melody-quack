#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// Trees, fire, wind, particles, hearts and toasts.
pub(crate) fn world_tick(g: &mut Game, fr: &mut Frame) {
    g.stick_trees.retain(|&(x, z)| g.world.get(x, z).map(|t| t.kind == Kind::Tree && t.fire <= 0.0).unwrap_or(false));
    g.stick_clock -= fr.dt;
    if g.stick_clock <= 0.0 {
        g.stick_clock = if g.stick_trees.len() < 4 { 0.5 } else { 12.0 + g.world.rand() * 8.0 };
        if g.stick_trees.len() < 6 {
            grow_stick(&mut g.world, &mut g.stick_trees);
        }
    }
    g.tick_clock += fr.dt;
    while g.tick_clock >= TICK {
        g.tick_clock -= TICK;
        g.world.tick(&mut g.particles);
    }
    g.world.wind_time = (g.world.wind_time - fr.dt).max(0.0);

    for z in 0..MAP as i32 {
        for x in 0..MAP as i32 {
            let t = g.world.get(x, z).unwrap();
            if t.fire > 0.0 && hash(x, z, (fr.time * 20.0) as u32) > 0.8 {
                let drift = if g.world.wind_time > 0.0 { g.world.wind * 1.2 } else { Vec2::ZERO };
                spawn(
                    &mut g.particles,
                    vec3(x as f32 + 0.3 + hash(z, x, (fr.time * 30.0) as u32) * 0.4, 0.9, z as f32 + 0.5),
                    vec3(drift.x, 1.4, drift.y),
                    0.8,
                    Color::from_rgba(255, 190, 80, 255),
                    0.035,
                    0.0,
                );
            }
        }
    }
    for p in g.particles.iter_mut() {
        p.vel.y -= p.gravity * fr.dt * 9.8;
        p.pos += p.vel * fr.dt;
        p.life -= fr.dt;
    }
    g.particles.retain(|p| p.life > 0.0);
    for h in g.hearts.iter_mut() {
        h.0.y += fr.dt * 0.5;
        h.1 -= fr.dt;
    }
    g.hearts.retain(|h| h.1 > 0.0);
    for t in g.toasts.iter_mut() {
        t.1 -= fr.dt;
    }
    g.toasts.retain(|t| t.1 > 0.0);
    if g.toasts.len() > 3 {
        g.toasts.remove(0);
    }
}
