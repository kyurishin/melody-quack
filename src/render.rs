//! Rendering the main view and the character picker.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn render_pick_preview(target: &RenderTarget, world: &World, cast: &[Character], chars: [usize; 2], chosen: usize, time: f32) {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / w, -2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        ..Default::default()
    });
    clear_background(Color::from_rgba(28, 24, 42, 255));
    for k in 0..12 {
        let t = k as f32 / 11.0;
        draw_rectangle(0.0, k as f32 * h / 12.0, w, h / 12.0 + 1.0, mix(Color::from_rgba(70, 58, 120, 255), Color::from_rgba(28, 24, 42, 255), t));
    }
    let mid = vec2(6.5, 16.5);
    let eye = vec3(mid.x, 1.25, mid.y + 2.9);
    set_camera(&Camera3D {
        position: eye,
        target: vec3(mid.x, 0.85, mid.y),
        up: Vec3::Y,
        fovy: 0.55,
        aspect: Some(w / h),
        render_target: Some(target.clone()),
        ..Default::default()
    });
    let light = Light { dusk: 0.15, fire: vec3(mid.x + 1.5, 2.0, mid.y + 1.5), fire_strength: 0.5, tint: vec3(1.0, 0.8, 0.6), lamps: Vec::new() };
    let mut b = Batch::new(eye, light);
    for (slot, &c) in chars.iter().enumerate() {
        let spot = vec2(mid.x + if slot == 0 { -0.6 } else { 0.6 }, mid.y);
        let ground = Player::new(c, cast[c].body, spot).ground(world);
        let ring = if slot == chosen { Color::from_rgba(255, 214, 110, 255) } else { Color::from_rgba(90, 80, 130, 255) };
        b.cube(vec3(spot.x, ground - 0.03, spot.y), vec3(0.8, 0.06, 0.8), ring);
        b.cube(vec3(spot.x, ground - 0.02, spot.y), vec3(0.7, 0.06, 0.7), Color::from_rgba(60, 52, 96, 255));
        let mut p = Player::new(c, cast[c].body, spot);
        p.yaw = if slot == chosen { time * 1.2 } else { 0.4 * (time * 0.8).sin() + if slot == 0 { 0.5 } else { -0.5 } };
        p.voice_timer = if slot == chosen && (time * 1.3).fract() < 0.25 { 0.2 } else { 0.0 };
        p.draw(&mut b, world, time, if slot == chosen { 0.6 } else { 0.0 });
    }
    b.flush();
}

pub(crate) struct View<'a> {
    pub(crate) world: &'a World,
    pub(crate) players: &'a [Player],
    pub(crate) particles: &'a [Particle],
    pub(crate) camp: &'a Camp,
    pub(crate) time: f32,
    pub(crate) dance: f32,
    pub(crate) dusk: f32,
    pub(crate) extras: &'a Extras,
}

pub(crate) const LANTERNS: [(f32, f32); 4] = [(2.5, 16.2), (6.4, 15.7), (10.4, 15.4), (13.3, 17.7)];

pub(crate) fn render_view(target: &RenderTarget, left: f32, width: f32, height: f32, focus_pos: Vec2, cam: Vec2, view: &View, split: bool) {
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / target.texture.width(), -2.0 / height),
        target: vec2(target.texture.width() / 2.0, height / 2.0),
        ..Default::default()
    });
    let d = view.dusk;
    let sk = sky();
    let fog = sky_haze(mix(FOG, DUSK_FOG, d), d);
    let top = sky_haze(mix(SKY_TOP, Color::new(0.12, 0.12, 0.3, 1.0), d), d);
    draw_rectangle(left, 0.0, width, height, fog);
    for k in 0..24 {
        let t = k as f32 / 23.0;
        let band = mix(mix(top, fog, t), Color::new(1.0, 0.52, 0.38, 1.0), sk.glow * t * 0.6 * (1.0 - d * 0.3));
        draw_rectangle(left, k as f32 * 5.0, width, 5.0, band);
    }
    {
        let day_part = ((sk.hour - 6.0) / 12.0).rem_euclid(1.0);
        let arc = (day_part * PI).sin();
        let bx = left + width * (0.1 + 0.8 * day_part);
        let by = 70.0 - arc * 55.0;
        let sun_up = sk.hour > 4.0 && sk.hour < 21.0 && d < 0.7;
        let veil = [1.0, 0.45, 0.15, 0.25][sk.weather];
        if sun_up {
            let sun = mix(Color::new(1.0, 0.95, 0.7, veil), Color::new(1.0, 0.55, 0.3, veil), sk.glow);
            draw_rectangle(bx.floor() - 3.0, by.floor() - 3.0, 7.0, 7.0, sun);
            draw_rectangle(bx.floor() - 4.0, by.floor() - 1.0, 9.0, 3.0, sun);
            draw_rectangle(bx.floor() - 1.0, by.floor() - 4.0, 3.0, 9.0, sun);
        } else if d > 0.5 {
            let mx = left + width * (0.15 + 0.7 * ((sk.hour + 6.0) / 24.0).rem_euclid(1.0));
            let moon = Color::new(0.95, 0.95, 0.85, veil);
            draw_rectangle(mx.floor() - 3.0, 18.0, 6.0, 6.0, moon);
            draw_rectangle(mx.floor() - 1.0, 17.0, 5.0, 5.0, Color::new(top.r, top.g, top.b, veil));
        }
        let clouds = [2, 7, 10, 7][sk.weather];
        let cloud_col = match sk.weather {
            2 => Color::new(0.5, 0.52, 0.58, 0.9),
            3 => Color::new(0.85, 0.87, 0.92, 0.9),
            _ => Color::new(0.97, 0.97, 1.0, 0.85),
        };
        let cloud_col = mix(cloud_col, Color::new(0.25, 0.24, 0.36, cloud_col.a), d * 0.8);
        let cloud_col = mix(cloud_col, Color::new(1.0, 0.7, 0.62, cloud_col.a), sk.glow * 0.5);
        for k in 0..clouds {
            let span = width + 60.0;
            let cx = left + (hash(k, 4, 21) * span + view.time * (2.0 + hash(k, 5, 21) * 3.0)).rem_euclid(span) - 30.0;
            let cy = 8.0 + hash(k, 6, 21) * 40.0;
            let cw = 18.0 + hash(k, 7, 21) * 22.0;
            draw_rectangle(cx.floor(), cy.floor(), cw.floor(), 5.0, cloud_col);
            draw_rectangle((cx + cw * 0.2).floor(), (cy - 3.0).floor(), (cw * 0.45).floor(), 4.0, cloud_col);
            draw_rectangle((cx + cw * 0.5).floor(), (cy - 5.0).floor(), (cw * 0.3).floor(), 6.0, cloud_col);
        }
    }
    if d > 0.4 && sk.weather == 0 {
        for k in 0..30 {
            let x = left + hash(k, 1, 7) * width;
            let y = hash(k, 2, 7) * 50.0;
            let tw = 0.5 + 0.5 * (view.time * (1.0 + hash(k, 3, 7) * 3.0) + k as f32).sin();
            draw_rectangle(x.floor(), y.floor(), 1.0, 1.0, Color::new(1.0, 1.0, 0.9, (d - 0.4) * tw));
        }
    }

    let (height_up, back, fov): (f32, f32, f32) = if split { (8.0, 7.0, 48.0) } else { (7.0, 6.5, 40.0) };
    let eye = vec3(cam.x, height_up, cam.y + back);
    let world_cam = Camera3D {
        position: eye,
        target: vec3(cam.x, 0.4, cam.y + 0.6),
        up: Vec3::Y,
        fovy: {
            let aspect = width / height;
            let base = (fov.to_radians() / 2.0).tan();
            if aspect < 4.0 / 3.0 && !split {
                2.0 * (base * (4.0 / 3.0) / aspect).min(36f32.to_radians().tan()).atan()
            } else {
                fov.to_radians()
            }
        },
        aspect: Some(width / height),
        render_target: Some(target.clone()),
        viewport: Some((left as i32, 0, width as i32, height as i32)),
        ..Default::default()
    };
    set_camera(&world_cam);
    {
        let clip = world_cam.matrix() * (BOBBER + vec3(0.0, 0.55, 0.0)).extend(1.0);
        if clip.w > 0.0 {
            let (nx, ny) = (clip.x / clip.w, clip.y / clip.w);
            let at = (left + (nx + 1.0) * 0.5 * width, (1.0 - ny) * 0.5 * height);
            BOBBER_AT.with(|cell| {
                let mut all = cell.get();
                all[if left > 0.5 { 1 } else { 0 }] = at;
                cell.set(all);
            });
        }
    }

    let flicker = 0.85 + 0.1 * (view.time * 11.0).sin() + 0.05 * (view.time * 23.0).sin();
    let strength = if view.camp.lit { (0.45 + 0.75 * d) * flicker * (0.6 + 0.4 * (view.camp.fuel / 90.0).min(1.0)) } else { 0.0 };
    let light = Light {
        dusk: d,
        fire: fire_center() + vec3(0.0, 0.4, 0.0),
        fire_strength: strength,
        tint: vec3(0.95, 0.58, 0.22),
        lamps: Vec::new(),
    };
    let mut b = Batch::new(eye, light);
    draw_world(&mut b, view.world, view.time, focus_pos, false);
    draw_shrooms(&mut b, view.time);
    draw_cottage(&mut b, d, false, focus_pos);
    draw_camp(&mut b, view.camp, view.time);
    for (lx, lz) in LANTERNS {
        draw_lantern(&mut b, vec3(lx, 0.5, lz), view.time);
    }
    for p in view.players {
        p.draw(&mut b, view.world, view.time, view.dance);
    }
    match sk.weather {
        2 => {
            let rain = Color::new(0.8, 0.86, 1.0, 0.9);
            for k in 0..240 {
                let x = cam.x + (hash(k, 1, 41) * 2.0 - 1.0) * 10.0;
                let z = cam.y + (hash(k, 2, 41) * 2.0 - 1.0) * 7.0 + 1.0;
                let y = (hash(k, 3, 41) * 6.0 - view.time * 9.0).rem_euclid(6.0);
                b.cube(vec3(x, y, z), vec3(0.03, 0.34, 0.03), rain);
            }
        }
        3 => {
            let flake = Color::new(1.0, 1.0, 1.0, 0.95);
            for k in 0..140 {
                let x = cam.x + (hash(k, 1, 43) * 2.0 - 1.0) * 10.0 + (view.time * 0.7 + k as f32).sin() * 0.3;
                let z = cam.y + (hash(k, 2, 43) * 2.0 - 1.0) * 7.0 + 1.0;
                let y = (hash(k, 3, 43) * 6.0 - view.time * 0.9).rem_euclid(6.0);
                b.cube(vec3(x, y, z), vec3(0.05, 0.05, 0.05), flake);
            }
        }
        _ => {}
    }
    let ex = view.extras;
    if ex.pot {
        let c = fire_center();
        let iron = Color::from_rgba(56, 56, 70, 255);
        let stick = Color::from_rgba(120, 82, 52, 255);
        for side in [-1.0_f32, 1.0] {
            b.cube(c + vec3(side * 0.3, 0.38, 0.0), vec3(0.04, 0.76, 0.04), stick);
        }
        b.cube(c + vec3(0.0, 0.76, 0.0), vec3(0.64, 0.04, 0.04), stick);
        b.cube(c + vec3(0.0, 0.42, 0.0), vec3(0.34, 0.24, 0.34), iron);
        b.cube(c + vec3(0.0, 0.55, 0.0), vec3(0.38, 0.03, 0.38), tinted(iron, 0.2));
        b.cube(c + vec3(0.0, 0.555, 0.0), vec3(0.28, 0.02, 0.28), Color::from_rgba(170, 110, 70, 255));
        b.cube(c + vec3(0.0, 0.66, 0.0), vec3(0.02, 0.2, 0.02), iron);
    }
    if ex.fishing {
        let cat = vec3(ex.cat.x, 0.48, ex.cat.y);
        let rod = Color::from_rgba(130, 90, 56, 255);
        let tip = cat + vec3(0.75, 0.95, 0.0);
        for k in 0..8 {
            let t = k as f32 / 7.0;
            b.cube(cat + vec3(0.15, 0.35, 0.1) + (tip - cat - vec3(0.15, 0.35, 0.1)) * t, vec3(0.04, 0.04, 0.04), rod);
        }
        let bob = BOBBER + vec3(0.0, (view.time * 3.0).sin() * 0.01 - ex.dip * 0.06, 0.0);
        for k in 0..14 {
            let t = k as f32 / 13.0;
            let sag = (t * PI).sin() * -0.15;
            b.cube(tip + (bob + vec3(0.0, 0.08, 0.0) - tip) * t + vec3(0.0, sag, 0.0), vec3(0.015, 0.015, 0.015), Color::new(0.95, 0.95, 0.95, 1.0));
        }
        b.cube(bob + vec3(0.0, 0.04, 0.0), vec3(0.08, 0.06, 0.08), Color::from_rgba(230, 60, 60, 255));
        b.cube(bob, vec3(0.08, 0.04, 0.08), Color::from_rgba(250, 250, 244, 255));
        let radius = 0.12 + ex.ring * 0.7;
        let col = if ex.in_window { Color::from_rgba(255, 220, 80, 255) } else { Color::from_rgba(245, 250, 255, 255) };
        for k in 0..16 {
            let a = k as f32 / 16.0 * 2.0 * PI;
            b.cube(bob + vec3(a.cos() * radius, -0.01, a.sin() * radius), vec3(0.05, 0.015, 0.05), col);
        }
    }
    for (k, &spot) in ex.sticks.iter().enumerate() {
        let at = vec3(spot.x, 0.54, spot.y);
        let yaw = 0.5 + k as f32 * 1.3;
        b.box_at(at, yaw, Vec3::ZERO, vec3(0.52, 0.07, 0.07), Color::from_rgba(150, 102, 60, 255));
        b.box_at(at, yaw, vec3(0.1, 0.0, 0.06), vec3(0.2, 0.05, 0.05), Color::from_rgba(130, 90, 56, 255));
        b.box_at(at, yaw, vec3(0.2, 0.05, 0.0), vec3(0.1, 0.07, 0.08), Color::from_rgba(104, 170, 78, 255));
        let bob = (view.time * 2.4 + k as f32 * 0.9).sin();
        let glow = 0.55 + 0.45 * (bob * 0.5 + 0.5);
        b.cube(at + vec3(0.0, 0.34 + bob * 0.05, 0.0), vec3(0.07, 0.07, 0.07), Color::new(1.0, 0.92, 0.5, glow));
        b.cube(at + vec3(0.0, 0.34 + bob * 0.05, 0.0), vec3(0.13, 0.025, 0.025), Color::new(1.0, 0.95, 0.65, glow * 0.7));
        b.cube(at + vec3(0.0, 0.34 + bob * 0.05, 0.0), vec3(0.025, 0.13, 0.025), Color::new(1.0, 0.95, 0.65, glow * 0.7));
    }
    if let Some(friend) = ex.friend_rod {
        let base = vec3(friend.x, 0.48, friend.y);
        let rod = Color::from_rgba(150, 104, 66, 255);
        let tip = base + vec3(0.7, 0.85, 0.45);
        for k in 0..8 {
            let t = k as f32 / 7.0;
            b.cube(base + vec3(0.15, 0.32, 0.1) + (tip - base - vec3(0.15, 0.32, 0.1)) * t, vec3(0.035, 0.035, 0.035), rod);
        }
        let bob = vec3(BOBBER.x - 0.3, BOBBER.y + (view.time * 2.6 + 1.0).sin() * 0.012, BOBBER.z + 0.75);
        for k in 0..10 {
            let t = k as f32 / 9.0;
            b.cube(tip + (bob + vec3(0.0, 0.06, 0.0) - tip) * t + vec3(0.0, (t * PI).sin() * -0.12, 0.0), vec3(0.014, 0.014, 0.014), Color::new(0.95, 0.95, 0.95, 1.0));
        }
        b.cube(bob + vec3(0.0, 0.03, 0.0), vec3(0.07, 0.05, 0.07), Color::from_rgba(80, 160, 230, 255));
        b.cube(bob, vec3(0.07, 0.03, 0.07), Color::from_rgba(250, 250, 244, 255));
    }
    if let Some((kind, t)) = ex.jump {
        let from = BOBBER;
        let to = vec3(ex.cat.x + 0.1, 0.9, ex.cat.y);
        let p = from + (to - from) * t + vec3(0.0, (t * PI).sin() * 0.9, 0.0);
        let body = [Color::from_rgba(140, 176, 160, 255), Color::from_rgba(180, 146, 90, 255), Color::from_rgba(250, 180, 50, 255)][kind];
        b.box_at(p, t * 6.0, Vec3::ZERO, vec3(0.28, 0.12, 0.08), body);
        b.box_at(p, t * 6.0, vec3(-0.18, 0.0, 0.0), vec3(0.08, 0.14, 0.04), tinted(body, -0.2));
    }
    for &(p, life) in &ex.hearts {
        let c = Color::new(1.0, 0.45, 0.6, life.min(1.0));
        b.cube(p + vec3(-0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.0, -0.02, 0.0), vec3(0.06, 0.06, 0.03), c);
    }
    if let Some((x, z, tool)) = ex.ghost {
        let pulse = 0.35 + 0.15 * (view.time * 6.0).sin();
        let t = view.world.get(x, z).map(|t| (t.kind, t.stack)).unwrap_or((Kind::Grass, 0));
        let base_h = if matches!(t.0, Kind::WoodWall | Kind::StoneWall) { 0.5 + WALL_HEIGHT * t.1 as f32 } else { 0.5 };
        let (col, h) = match tool {
            Tool::WoodWall => (Color::new(0.9, 0.7, 0.45, pulse), WALL_HEIGHT),
            Tool::StoneWall => (Color::new(0.8, 0.82, 0.9, pulse), WALL_HEIGHT),
            Tool::Floor => (Color::new(0.95, 0.8, 0.6, pulse), 0.06),
            Tool::Remove => (Color::new(1.0, 0.4, 0.45, pulse), 0.04),
        };
        let y = if tool == Tool::Remove { base_h + 0.03 } else { base_h + h / 2.0 };
        b.cube(vec3(x as f32 + 0.5, y, z as f32 + 0.5), vec3(1.0, h, 1.0), col);
    }
    for p in view.particles {
        let fade = (p.life / p.max_life).clamp(0.0, 1.0);
        let c = mix(fog, p.color, 0.4 + 0.6 * fade);
        let size = p.size * 1.4 * (0.5 + 0.5 * fade);
        let stretch = if p.vel.y < -3.0 { 3.0 } else { 1.0 };
        b.cube(p.pos, vec3(size, size * stretch, size), c);
    }
    b.flush();
    draw_world(&mut b, view.world, view.time, focus_pos, true);
    draw_cottage(&mut b, d, true, focus_pos);
    b.flush();
}
