//! Players, carrying things and keyboard input.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const CAT_RADIUS: f32 = 0.2;

pub(crate) fn ahead_local(p: &Player) -> (i32, i32) {
    let l = home::local(p.pos);
    let (dx, dz) = if p.facing.x.abs() > p.facing.y.abs() { (p.facing.x.signum() as i32, 0) } else { (0, p.facing.y.signum() as i32) };
    (l.x.floor() as i32 + dx, l.y.floor() as i32 + dz)
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Carry {
    Stick,
    Stone,
    Acorn,
}

pub(crate) struct Player {
    pub(crate) character: usize,
    pub(crate) body: Body,
    pub(crate) pos: Vec2,
    pub(crate) facing: Vec2,
    pub(crate) yaw: f32,
    pub(crate) voice_timer: f32,
    pub(crate) key_glow: [f32; 6],
    pub(crate) walking: bool,
    pub(crate) cam: Vec2,
    pub(crate) carrying: Option<Carry>,
    pub(crate) seated: bool,
    pub(crate) sleeping: bool,
    pub(crate) sleep_yaw: f32,
    pub(crate) emote: Option<(usize, f32)>,
    pub(crate) perch: f32,
    pub(crate) lift: f32,
    pub(crate) sniff: f32,
}

impl Player {
    pub(crate) fn new(character: usize, body: Body, pos: Vec2) -> Player {
        Player {
            character,
            body,
            pos,
            facing: vec2(0.0, 1.0),
            yaw: 0.0,
            voice_timer: 0.0,
            key_glow: [0.0; 6],
            walking: false,
            cam: pos,
            carrying: None,
            seated: false,
            sleeping: false,
            sleep_yaw: 0.0,
            emote: None,
            perch: 0.0,
            lift: 0.0,
            sniff: 0.0,
        }
    }

    pub(crate) fn place_at(&mut self, p: Vec2) {
        self.pos = p;
        self.cam = p;
    }

    pub(crate) fn step(&mut self, dir: Vec2, world: &World, dt: f32) -> f32 {
        self.step_with(dir, dt, |p| walkable(world, p), |p| rescue(world, p))
    }

    pub(crate) fn step_with(&mut self, dir: Vec2, dt: f32, walk: impl Fn(Vec2) -> bool, save: impl Fn(Vec2) -> Vec2) -> f32 {
        let before = self.pos;
        if dir != Vec2::ZERO && self.seated {
            self.seated = false;
        }
        if !self.seated {
            self.perch = 0.0;
            self.sleeping = false;
        }
        self.walking = dir != Vec2::ZERO && !self.seated;
        if self.walking {
            self.facing = dir.normalize();
            let step = self.facing * 3.2 * dt;
            if walk(self.pos + vec2(step.x, 0.0)) {
                self.pos.x += step.x;
            }
            if walk(self.pos + vec2(0.0, step.y)) {
                self.pos.y += step.y;
            }
        }
        let mut diff = self.facing.x.atan2(self.facing.y) - self.yaw;
        while diff > PI {
            diff -= 2.0 * PI;
        }
        while diff < -PI {
            diff += 2.0 * PI;
        }
        self.yaw += diff * (dt * 14.0).min(1.0);
        if !self.seated && !walk(self.pos) {
            let safe = save(self.pos);
            self.pos += (safe - self.pos) * (dt * 10.0).min(1.0);
        }
        self.voice_timer = (self.voice_timer - dt).max(0.0);
        self.sniff = (self.sniff - dt).max(0.0);
        for g in self.key_glow.iter_mut() {
            *g = (*g - dt * 3.0).max(0.0);
        }
        self.cam += (self.pos - self.cam) * (dt * 4.0).min(1.0);
        self.pos.distance(before)
    }

    pub(crate) fn ahead(&self) -> (i32, i32) {
        let (dx, dz) = if self.facing.x.abs() > self.facing.y.abs() {
            (self.facing.x.signum() as i32, 0)
        } else {
            (0, self.facing.y.signum() as i32)
        };
        (self.pos.x.floor() as i32 + dx, self.pos.y.floor() as i32 + dz)
    }

    pub(crate) fn ground(&self, world: &World) -> f32 {
        ground_top(world.get(self.pos.x.floor() as i32, self.pos.y.floor() as i32).map(|t| t.kind).unwrap_or(Kind::Grass))
    }

    pub(crate) fn draw(&self, b: &mut Batch, world: &World, time: f32, dance: f32) {
        let y = self.ground(world) + self.lift + if self.seated { 0.16 + self.perch } else { 0.0 };
        let speaking = self.voice_timer > 0.0;
        let excited = matches!(self.body, Body::Dog(_)) && (speaking || dance > 0.0 || self.carrying.is_some()) && !self.walking;
        let wiggle = if excited { (time * 16.0).sin() * 0.02 } else { 0.0 };
        let sniff = if self.sniff > 0.0 { ((time * 14.0).sin() * 0.5 + 0.5) * -0.04 } else { 0.0 };
        let at = vec3(self.pos.x + wiggle, y + sniff, self.pos.y);
        if self.sleeping {
            b.flush();
        }
        let first_vertex = b.vertices.len();
        match &self.body {
            Body::Cat(look) => {
                draw_cat(b, at, self.yaw, time, self.walking, speaking, dance, look);
                if self.carrying.is_some() {
                    draw_carry(b, at, self.yaw, vec3(0.0, 0.47, 0.36), self.carrying);
                }
            }
            Body::Dog(look) => draw_dog(b, at, self.yaw, time, self.walking, speaking, dance, self.carrying, look),
            Body::Duck => draw_duck(b, at, self.yaw, time, self.walking, speaking, dance, self.carrying),
            Body::Owl => draw_owl(b, at, self.yaw, time, self.walking, speaking, dance, self.carrying),
            Body::Fox => draw_fox(b, at, self.yaw, time, self.walking, speaking, dance, self.carrying),
            Body::Otter => draw_otter(b, at, self.yaw, time, self.walking, speaking, dance, self.carrying, self.voice_timer),
        }
        if self.sleeping {
            let base = vec3(self.pos.x, self.ground(world) + self.lift, self.pos.y);
            for vertex in b.vertices.iter_mut().skip(first_vertex) {
                let body = rotate_y(vertex.position - at, -self.yaw);
                let local = vec3(body.x * 0.9, 0.4 + (body.z + 0.3).max(0.0) * 0.32, 0.12 - body.y * 0.82);
                vertex.position = base + rotate_y(local, self.sleep_yaw);
            }
            b.flush();
            let quilt = Color::from_rgba(130, 170, 220, 255);
            let sheet = Color::from_rgba(250, 248, 240, 255);
            b.box_at(base, self.sleep_yaw, vec3(0.0, 0.5, 0.36), vec3(0.86, 0.17, 1.08), quilt);
            b.box_at(base, self.sleep_yaw, vec3(0.0, 0.59, -0.16), vec3(0.88, 0.035, 0.12), sheet);
            let breathe = 0.012 * (time * 2.2 + self.pos.x * 3.0).sin();
            b.box_at(base, self.sleep_yaw, vec3(0.0, 0.6 + breathe, 0.12), vec3(0.6, 0.05, 0.42), quilt);
        }
        b.cube(vec3(self.pos.x, self.ground(world) + 0.006, self.pos.y), vec3(0.4, 0.004, 0.4), Color::new(0.08, 0.1, 0.06, 0.25));
        if let Some((id, left)) = self.emote {
            let grow = ((3.0 - left) * 7.0).min(1.0) * (left * 3.0).min(1.0);
            let bob = (time * 3.0).sin() * 0.02;
            let px = 0.055 * grow;
            let top = vec3(self.pos.x, self.ground(world) + self.lift + if self.sleeping { 0.9 } else { 1.22 } + bob, self.pos.y + 0.05);
            let up = vec3(0.0, 0.74, -0.67);
            let toward = vec3(0.0, 0.67, 0.74);
            let at_px = |x: f32, y: f32, lift: f32| top + vec3(x * px, 0.0, 0.0) + up * (y * px) + toward * lift;
            b.unlit = true;
            for y in -5..5 {
                for x in -5..5 {
                    let corner = (x == -5 || x == 4) && (y == -5 || y == 4);
                    if !corner {
                        b.cube(at_px(x as f32 + 0.5, y as f32 + 0.5, -0.01), vec3(px, px * 0.78, px * 0.72), Color::new(1.0, 1.0, 1.0, 1.0));
                    }
                }
            }
            b.cube(at_px(-1.5, -5.5, -0.01), vec3(px * 2.0, px * 0.78, px * 0.72), Color::new(1.0, 1.0, 1.0, 1.0));
            b.cube(at_px(-2.0, -6.4, -0.01), vec3(px, px * 0.78, px * 0.72), Color::new(1.0, 1.0, 1.0, 1.0));
            for (x, y, c) in emoji::pixels(id) {
                b.cube(at_px(x as f32 - 3.5, 3.5 - y as f32, 0.012), vec3(px, px * 0.78, px * 0.72), c);
            }
            b.unlit = false;
        }
    }
}

pub(crate) const P2_NOTE_KEYS: [KeyCode; 6] = [
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
    KeyCode::Key0,
    KeyCode::Minus,
    KeyCode::Equal,
];

pub(crate) const P1_KEY_LABELS: [&str; 6] = ["1", "2", "3", "4", "5", "6"];

pub(crate) const P2_KEY_LABELS: [&str; 6] = ["7", "8", "9", "0", "-", "="];

pub(crate) fn held(keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| is_key_down(*k))
}

pub(crate) fn direction(up: &[KeyCode], down: &[KeyCode], left: &[KeyCode], right: &[KeyCode]) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if held(left) {
        dir.x -= 1.0;
    }
    if held(right) {
        dir.x += 1.0;
    }
    if held(up) {
        dir.y -= 1.0;
    }
    if held(down) {
        dir.y += 1.0;
    }
    dir
}
