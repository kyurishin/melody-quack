//! Rendering: the world, the people in it, lights, and every screen of UI.

use crate::computer;
use crate::game::{Game, Overlay, Screen};
use crate::gfx::{draw_glow, hex, mix, pal, with_alpha};
use crate::music::section;
use crate::poster::{FLYER_H, FLYER_W};
use crate::sprites::{CHAR_H, CHAR_W, CRITTERS, TILE};
use crate::ui::Tone;
use crate::world::{Ground, PropKind, SpotKind};
use macroquad::prelude::*;
use mq_shared::Scene;
use std::f32::consts::TAU;

/// What the lights are doing this frame.
#[derive(Clone, Copy, Default)]
pub struct Pulse {
    pub beat: f64,
    /// 1 on the beat, decaying to 0.
    pub hit: f32,
    pub energy: f32,
    pub drop: bool,
    pub section: &'static str,
}

pub fn world_camera(center: Vec2, s: f32) -> Camera2D {
    let vw = screen_width() / s;
    let vh = screen_height() / s;
    let c = (center * s).round() / s;
    Camera2D { target: c, zoom: vec2(2.0 / vw, 2.0 / vh), ..Default::default() }
}

impl Game {
    pub fn pulse(&self) -> Pulse {
        let live = self.rave.as_ref().map(|r| r.live).unwrap_or(false) && self.audio.live_flowing();
        if live || self.capture.is_some() {
            let lvl = if self.capture.is_some() { self.audio.input_level() } else { self.audio.level() };
            let hit = ((lvl - 0.25) / 0.6).clamp(0.0, 1.0);
            return Pulse { beat: self.time as f64 * 2.2, hit, energy: 0.6 + hit * 0.4, drop: hit > 0.8, section: "LIVE" };
        }
        let Some(cue) = self.cue() else { return Pulse::default() };
        let beat = cue.beat_at(self.server_ms());
        let sec = section(beat / 4.0);
        let frac = beat.rem_euclid(1.0) as f32;
        let hit = if sec.kick { (1.0 - frac).powf(3.0) } else { 0.15 * (1.0 - frac) };
        Pulse { beat, hit, energy: sec.energy, drop: sec.drop, section: sec.name }
    }

    /// Colour multiplier for anything standing in the current scene.
    fn lit(&self) -> Color {
        if self.scene == Scene::Rave {
            let p = self.pulse();
            mix(hex(0x8f78c8), WHITE, 0.15 + 0.45 * p.hit * p.energy)
        } else {
            WHITE
        }
    }

    pub fn draw(&mut self) {
        clear_background(pal::INK);
        match self.screen {
            Screen::Title | Screen::Connecting => self.draw_title(),
            Screen::Playing => {
                self.draw_world();
                self.draw_hud();
                self.draw_overlay();
            }
        }
        if self.fade > 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), with_alpha(pal::INK, self.fade));
        }
    }

    // ---------------------------------------------------------------------
    // World
    // ---------------------------------------------------------------------

    fn view_center(&self) -> Vec2 {
        let s = self.ui.s;
        let vw = screen_width() / s;
        let vh = screen_height() / s;
        let size = self.map().px_size();
        let clamp = |c: f32, view: f32, size: f32| {
            if size <= view {
                size / 2.0
            } else {
                c.clamp(view / 2.0, size - view / 2.0)
            }
        };
        vec2(clamp(self.cam.x, vw, size.x), clamp(self.cam.y, vh, size.y))
    }

    pub fn draw_world_at(&self, center: Vec2) {
        let s = self.ui.s;
        set_camera(&world_camera(center, s));
        let vw = screen_width() / s;
        let vh = screen_height() / s;
        let view = Rect::new(center.x - vw / 2.0, center.y - vh / 2.0, vw, vh);
        let map = self.map();
        let pulse = self.pulse();

        // Ground.
        let x0 = (view.x / TILE).floor().max(0.0) as usize;
        let y0 = (view.y / TILE).floor().max(0.0) as usize;
        let x1 = ((view.right() / TILE).ceil() as usize).min(map.w);
        let y1 = ((view.bottom() / TILE).ceil() as usize).min(map.h);
        let wf = ((self.time * 2.0) as usize) % 2;
        for ty in y0..y1 {
            for tx in x0..x1 {
                let tex = match map.ground_at(tx, ty) {
                    Ground::Grass(v) => &self.sprites.grass[v as usize],
                    Ground::Path => &self.sprites.path,
                    Ground::Plaza => &self.sprites.plaza,
                    Ground::Water => &self.sprites.water[(wf + tx) % 2],
                    Ground::Carpet => &self.sprites.carpet,
                    Ground::Concrete => &self.sprites.chapel_floor,
                    Ground::Void => continue,
                };
                draw_texture(tex, tx as f32 * TILE, ty as f32 * TILE, WHITE);
            }
        }

        // Back walls for interiors.
        match self.scene {
            Scene::PcBang => self.draw_pcbang_wall(),
            Scene::Rave => self.draw_chapel_wall(&pulse),
            Scene::Town => {
                // Edge of the world: a dark treeline beyond the map.
                draw_rectangle(-400.0, -400.0, map.px_size().x + 800.0, 400.0, pal::GRASS_D);
            }
        }

        // The club is dark; people and props are lit by the rig instead.
        if self.scene == Scene::Rave {
            let dark = 0.66 - 0.18 * pulse.hit * pulse.energy;
            draw_rectangle(view.x, view.y, view.w, view.h, with_alpha(hex(0x07040f), dark));
        }

        // Campfire glow on the ground.
        if let Some(f) = map.campfire {
            draw_ellipse(f.x, f.y + 2.0, 16.0, 6.0, 0.0, hex(0x3a2a2a));
        }

        // Sortable things: props, players, ghosts.
        enum Item<'a> {
            Prop(&'a crate::world::Prop),
            Me,
            Remote(u32),
            Ghost(usize),
            Fire(Vec2),
        }
        let mut items: Vec<(f32, Item)> = Vec::new();
        for p in &map.props {
            if p.pos.x > view.right() + 40.0 || p.pos.x + 340.0 < view.x || p.pos.y > view.bottom() + 40.0 || p.sort_y + 40.0 < view.y {
                continue;
            }
            items.push((p.sort_y, Item::Prop(p)));
        }
        items.push((self.me.pos.y, Item::Me));
        for (id, r) in &self.remotes {
            items.push((r.pos.y, Item::Remote(*id)));
        }
        if self.scene == Scene::Rave {
            for (i, g) in self.ghosts.iter().enumerate() {
                items.push((g.pos.y, Item::Ghost(i)));
            }
        }
        if let Some(f) = map.campfire {
            items.push((f.y + 2.0, Item::Fire(f)));
        }
        items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, it) in &items {
            match it {
                Item::Prop(p) => self.draw_prop(p),
                Item::Me => {
                    let seated = self.me.seat.is_some();
                    let pos = self.me.seat.map(|i| self.pcbang.seats[i]).unwrap_or(self.me.pos);
                    self.draw_actor(
                        self.settings.avatar as usize,
                        pos,
                        self.me.facing,
                        self.me.moving,
                        self.me.anim,
                        self.me.dance,
                        seated,
                        &pulse,
                    );
                }
                Item::Remote(id) => {
                    let r = &self.remotes[id];
                    self.draw_actor(r.st.avatar as usize, r.pos, r.st.facing, r.st.moving, r.anim, r.st.dance, r.st.seated, &pulse);
                }
                Item::Ghost(i) => self.draw_ghost(*i, &pulse),
                Item::Fire(f) => self.draw_fire(*f),
            }
        }

        // Particles (sparks are drawn normally; fog is additive below).
        for p in &self.particles {
            if self.scene == Scene::Town {
                let a = 1.0 - p.life / p.max;
                draw_rectangle(p.pos.x.round(), p.pos.y.round(), 1.0, 1.0, with_alpha(p.color, a));
            }
        }

        // Lighting.
        match self.scene {
            Scene::Town => {
                draw_rectangle(view.x, view.y, view.w, view.h, with_alpha(hex(0x2a1660), 0.3));
                gl_use_material(&self.add);
                for (p, r, c) in &map.lights {
                    let flicker = if *c == pal::CORAL { 0.85 + 0.15 * (self.time * 13.0).sin() * (self.time * 7.3).cos() } else { 1.0 };
                    draw_glow(&self.glow, p.x, p.y, *r * flicker, with_alpha(*c, 0.32));
                }
                // The chapel windows breathe with the bass.
                if let Some(door) = map.spots.iter().find(|s| s.kind == SpotKind::ChapelDoor) {
                    let k = 0.15 + 0.35 * pulse.hit;
                    draw_glow(&self.glow, door.pos.x - 118.0, door.pos.y - 120.0, 70.0, with_alpha(pal::MAGENTA, k));
                    draw_glow(&self.glow, door.pos.x + 118.0, door.pos.y - 120.0, 70.0, with_alpha(pal::CYAN, k));
                }
                gl_use_default_material();
            }
            Scene::PcBang => {
                draw_rectangle(view.x, view.y, view.w, view.h, with_alpha(hex(0x0c0820), 0.35));
                gl_use_material(&self.add);
                for (p, r, c) in &map.lights {
                    draw_glow(&self.glow, p.x, p.y, *r, with_alpha(*c, 0.35));
                }
                let w = map.px_size().x;
                draw_glow(&self.glow, w / 2.0, 40.0, 180.0, with_alpha(pal::MAGENTA, 0.25));
                gl_use_default_material();
            }
            Scene::Rave => self.draw_rave_lights(&pulse, view),
        }

        // Name tags and chat bubbles above everything.
        let me_pos = self.me.seat.map(|i| self.pcbang.seats[i]).unwrap_or(self.me.pos);
        let me_dj = self.is_dj;
        self.name_tag(me_pos, &self.my_name, me_dj, true);
        for r in self.remotes.values() {
            self.name_tag(r.pos, &r.st.name, r.st.dj, false);
        }
        if let Some((t, _)) = &self.me.bubble {
            self.bubble(me_pos, t);
        }
        for r in self.remotes.values() {
            if let Some((t, _)) = &r.bubble {
                self.bubble(r.pos, t);
            }
        }

        set_default_camera();
    }

    fn draw_world(&mut self) {
        let c = self.view_center();
        self.draw_world_at(c);
        // Interaction prompt, in UI space.
        if matches!(self.overlay, Overlay::None) && self.me.seat.is_none() {
            if let Some(k) = self.nearest_spot() {
                let s = self.ui.s;
                let cam = world_camera(c, s);
                let anchor = self.me.pos + vec2(0.0, -CHAR_H - 14.0);
                let sp = cam.world_to_screen(anchor) / s;
                let label = self.spot_label(k);
                self.ui.prompt(&self.fonts, sp.x, sp.y, "E", &label);
            }
        }
    }

    fn draw_prop(&self, p: &crate::world::Prop) {
        let sp = &self.sprites;
        let tex = match p.kind {
            PropKind::Tree(i) => &sp.tree[i as usize % 3],
            PropKind::Pine => &sp.pine,
            PropKind::Cottage => &sp.cottage,
            PropKind::PcBang => &sp.pcbang,
            PropKind::Chapel => &sp.chapel,
            PropKind::Hoarding => &sp.hoarding,
            PropKind::Lamp => &sp.lamp,
            PropKind::Log => &sp.log,
            PropKind::Bush => &sp.bush,
            PropKind::Flowers => &sp.flowers,
            PropKind::Desk => &sp.desk,
            PropKind::Counter => &sp.counter,
            PropKind::Vending => &sp.vending,
            PropKind::Owner => &sp.owner,
            PropKind::Bouncer => &sp.bouncer,
            PropKind::Booth => &sp.booth,
            PropKind::Speaker => &sp.speaker,
            PropKind::Pillar => &sp.pillar,
        };
        // Shadows for things that stand up.
        if matches!(p.kind, PropKind::Tree(_) | PropKind::Pine | PropKind::Lamp | PropKind::Bouncer | PropKind::Owner) {
            let w = tex.width();
            draw_ellipse(p.pos.x + w / 2.0, p.sort_y - 3.0, w * 0.3, 4.0, 0.0, with_alpha(pal::INK, 0.35));
        }
        let mut y = p.pos.y;
        if p.kind == PropKind::Owner {
            y += ((self.time * 2.0).sin() * 1.0).round();
        }
        draw_texture_ex(tex, p.pos.x, y, self.lit(), DrawTextureParams { flip_x: p.flip, ..Default::default() });
        // Flyers pasted on walls.
        if matches!(p.kind, PropKind::Hoarding | PropKind::Chapel) {
            let r = Rect::new(p.pos.x, p.pos.y, tex.width(), tex.height());
            for (i, slot) in self.map().posters.iter().enumerate() {
                if r.contains(slot.pos) {
                    if let Some(Some((_, ft))) = self.wall.get(i) {
                        let w = FLYER_W * slot.scale;
                        let h = FLYER_H * slot.scale;
                        draw_rectangle(slot.pos.x + 1.0, slot.pos.y + 1.0, w, h, with_alpha(pal::INK, 0.5));
                        draw_texture_ex(ft, slot.pos.x, slot.pos.y, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
                    }
                }
            }
        }
        // Monitors glow and flicker.
        if p.kind == PropKind::Desk {
            let t = (self.time * 3.0 + p.pos.x * 0.1).sin() * 0.5 + 0.5;
            draw_rectangle(p.pos.x + 12.0, p.pos.y + 4.0, 18.0, 14.0, with_alpha(pal::WHITE, 0.05 + 0.05 * t));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_actor(&self, kind: usize, pos: Vec2, facing: i8, moving: bool, anim: f32, dance: u8, seated: bool, pulse: &Pulse) {
        let frames = &self.sprites.critters[kind % CRITTERS.len()];
        let mut frame = 0;
        let mut flip = facing < 0;
        let mut yoff = 0.0;
        let beat = pulse.beat;
        let half = (beat * 2.0).floor() as i64;
        if seated {
            frame = 6;
            flip = false;
        } else if moving {
            frame = 1 + ((anim * 8.0) as usize % 2);
        } else if self.scene == Scene::Rave {
            let on = beat.rem_euclid(1.0) < 0.5;
            match dance {
                1 => frame = if on { 3 } else { 0 },
                2 => frame = if beat.floor() as i64 % 2 == 0 { 4 } else { 5 },
                3 => {
                    let f = beat.rem_euclid(1.0) as f32;
                    yoff = -((f * std::f32::consts::PI).sin() * 8.0).round();
                    frame = 3;
                }
                4 => {
                    flip = half % 2 == 0;
                    frame = if on { 0 } else { 3 };
                }
                _ => {
                    // Nod along.
                    yoff = if on { 0.0 } else { 1.0 };
                }
            }
        }
        let x = (pos.x - CHAR_W / 2.0).round();
        let y = (pos.y - CHAR_H + 2.0 + yoff).round();
        if !seated {
            draw_ellipse(pos.x, pos.y, 9.0, 3.0, 0.0, with_alpha(pal::INK, 0.4));
        }
        draw_texture_ex(&frames[frame], x, y, self.lit(), DrawTextureParams { flip_x: flip, ..Default::default() });
        if seated {
            draw_texture(&self.sprites.chair_back, pos.x - 14.0, pos.y - 10.0, WHITE);
        }
    }

    fn draw_ghost(&self, i: usize, pulse: &Pulse) {
        let g = &self.ghosts[i];
        let frames = &self.sprites.critters[g.kind];
        let b = pulse.beat + g.phase as f64 * 0.1;
        let on = b.rem_euclid(1.0) < 0.5;
        let (frame, flip, yoff) = match g.style {
            0 => (if on && pulse.energy > 0.6 { 3 } else { 0 }, false, 0.0),
            1 => (if (b.floor() as i64) % 2 == 0 { 4 } else { 5 }, false, 0.0),
            2 => (0, (b * 2.0).floor() as i64 % 2 == 0, if on { 0.0 } else { 1.0 }),
            _ => (if pulse.drop && on { 3 } else { 0 }, g.phase > 0.5, -(pulse.hit * 3.0 * pulse.energy).round()),
        };
        // Silhouettes: the crowd you can't quite make out through the fog.
        let tint = mix(hex(0x0e0820), hex(0x3a2470), 0.15 + 0.35 * pulse.hit);
        draw_texture_ex(
            &frames[frame],
            (g.pos.x - CHAR_W / 2.0).round(),
            (g.pos.y - CHAR_H + 2.0 + yoff).round(),
            tint,
            DrawTextureParams { flip_x: flip, ..Default::default() },
        );
    }

    fn draw_fire(&self, f: Vec2) {
        // Logs.
        draw_rectangle(f.x - 10.0, f.y - 3.0, 20.0, 5.0, pal::WOOD_D);
        draw_rectangle(f.x - 7.0, f.y - 6.0, 14.0, 4.0, pal::WOOD);
        // Flames: stacked, flickering blobs.
        for i in 0..6 {
            let t = self.time * 9.0 + i as f32 * 1.7;
            let h = 8.0 + (t.sin() * 3.0) + i as f32 * 0.5;
            let x = f.x + ((i as f32 - 2.5) * 2.4) + (t * 0.7).cos();
            let c = if i % 2 == 0 { pal::CORAL } else { pal::YELLOW };
            draw_rectangle(x.round() - 1.5, (f.y - 4.0 - h).round(), 3.0, h.round(), c);
        }
        draw_rectangle(f.x - 2.0, f.y - 10.0, 4.0, 6.0, pal::CREAM);
    }

    fn name_tag(&self, pos: Vec2, name: &str, dj: bool, me: bool) {
        if name.is_empty() {
            return;
        }
        let size = 8.0;
        let font = &self.fonts.small;
        // Text in world units at a crisp screen size.
        let s = self.ui.s;
        let w = crate::gfx::text_w(font, name, size * s) / s;
        let dot = if dj { 7.0 } else { 0.0 };
        let total = w + dot + 6.0;
        let x = (pos.x - total / 2.0).round();
        let y = (pos.y - CHAR_H - 13.0).round();
        draw_rectangle(x, y, total, 10.0, with_alpha(pal::INK, 0.75));
        if dj {
            // The purple dot: this player is a DJ.
            draw_circle(x + 5.0, y + 5.0, 2.5, pal::DJ);
            draw_circle(x + 4.5, y + 4.5, 1.0, mix(pal::DJ, pal::WHITE, 0.6));
        }
        let col = if me { pal::YELLOW } else { pal::CREAM };
        draw_text_ex(
            name,
            x + 3.0 + dot,
            y + 8.0,
            TextParams { font: Some(font), font_size: (size * s) as u16, font_scale: 1.0 / s, color: col, ..Default::default() },
        );
    }

    fn bubble(&self, pos: Vec2, text: &str) {
        let s = self.ui.s;
        let font = &self.fonts.body;
        let lines = computer::wrap(text, 26);
        let size = 9.0;
        let w = lines.iter().map(|l| crate::gfx::text_w(font, l, size * s) / s).fold(0.0, f32::max) + 10.0;
        let h = lines.len() as f32 * 11.0 + 6.0;
        let x = (pos.x - w / 2.0).round();
        let y = (pos.y - CHAR_H - 18.0 - h).round();
        draw_rectangle(x + 2.0, y + 2.0, w, h, pal::INK);
        draw_rectangle(x, y, w, h, pal::INK);
        draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, pal::CREAM);
        draw_triangle(vec2(pos.x - 3.0, y + h - 1.0), vec2(pos.x + 3.0, y + h - 1.0), vec2(pos.x, y + h + 4.0), pal::CREAM);
        for (i, l) in lines.iter().enumerate() {
            draw_text_ex(
                l,
                x + 5.0,
                y + 11.0 + i as f32 * 11.0,
                TextParams { font: Some(font), font_size: (size * s) as u16, font_scale: 1.0 / s, color: pal::INK, ..Default::default() },
            );
        }
    }

    fn draw_pcbang_wall(&self) {
        let w = self.pcbang.px_size().x;
        draw_rectangle(0.0, 0.0, w, 2.0 * TILE + 8.0, hex(0x1d1a2e));
        draw_rectangle(0.0, 2.0 * TILE + 4.0, w, 4.0, hex(0x3b3552));
        // Neon strip and sign.
        draw_rectangle(0.0, 6.0, w, 2.0, pal::MAGENTA);
        let s = self.ui.s;
        let sign = "PC BANG / 24H / GAMES / RAMYEON / INTERNET";
        let font = &self.fonts.small;
        let tw = crate::gfx::text_w(font, sign, 8.0 * s) / s;
        draw_text_ex(sign, (w - tw) / 2.0, 26.0, TextParams { font: Some(font), font_size: (8.0 * s) as u16, font_scale: 1.0 / s, color: pal::CYAN, ..Default::default() });
        // The 방 isn't in that font: draw it as pixels instead of the fallback.
        // Clock.
        draw_rectangle(w - 70.0, 34.0, 40.0, 16.0, pal::BLACK);
        let hh = ((self.time / 60.0) as i32 + 23) % 24;
        let mm = (self.time as i32) % 60;
        draw_text_ex(
            &format!("{hh:02}:{mm:02}"),
            w - 66.0,
            46.0,
            TextParams { font: Some(&self.fonts.mono), font_size: (14.0 * s) as u16, font_scale: 1.0 / s, color: hex(0xff3d3d), ..Default::default() },
        );
        // Posters of games that definitely exist.
        for (i, c) in [pal::CORAL, pal::BLUE, pal::GREEN].iter().enumerate() {
            let x = 40.0 + i as f32 * 60.0;
            draw_rectangle(x, 32.0, 28.0, 30.0, *c);
            draw_rectangle(x + 4.0, 36.0, 20.0, 12.0, with_alpha(pal::INK, 0.4));
        }
    }

    fn draw_chapel_wall(&self, p: &Pulse) {
        let size = self.chapel.px_size();
        let wall_h = 6.0 * TILE + 8.0;
        draw_rectangle(0.0, -200.0, size.x, wall_h + 200.0, pal::CONCRETE_DD);
        // Board-formed concrete.
        let mut y = -200.0;
        while y < wall_h {
            draw_rectangle(0.0, y, size.x, 1.0, hex(0x37334a));
            y += 12.0;
        }
        // Pointed arches: three bays, tall lancet windows lit by the set.
        let bays = [size.x * 0.17, size.x * 0.5, size.x * 0.83];
        let cols = [pal::MAGENTA, pal::PURPLE, pal::CYAN];
        for (i, bx) in bays.iter().enumerate() {
            let w = if i == 1 { 120.0 } else { 80.0 };
            let top = if i == 1 { -60.0 } else { -20.0 };
            let bot = wall_h - 20.0;
            // Arch frame.
            draw_rectangle(bx - w / 2.0 - 8.0, top + w / 2.0, w + 16.0, bot - top - w / 2.0, pal::CONCRETE_D);
            draw_triangle(vec2(bx - w / 2.0 - 8.0, top + w / 2.0), vec2(*bx, top - 30.0), vec2(bx + w / 2.0 + 8.0, top + w / 2.0), pal::CONCRETE_D);
            // Glass.
            let glow = mix(hex(0x1a1030), cols[(i + (p.beat / 8.0) as usize) % 3], 0.35 + 0.55 * p.hit * p.energy);
            draw_rectangle(bx - w / 2.0, top + w / 2.0, w, bot - top - w / 2.0, glow);
            draw_triangle(vec2(bx - w / 2.0, top + w / 2.0), vec2(*bx, top - 22.0), vec2(bx + w / 2.0, top + w / 2.0), glow);
            // Tracery.
            draw_rectangle(bx - 1.5, top - 20.0, 3.0, bot - top + 20.0, pal::INK);
            let mut yy = top + w / 2.0 + 16.0;
            while yy < bot {
                draw_rectangle(bx - w / 2.0, yy, w, 2.0, pal::INK);
                yy += 24.0;
            }
            if i == 1 {
                draw_circle(*bx, top + 10.0, 26.0, pal::INK);
                draw_circle(*bx, top + 10.0, 22.0, mix(glow, pal::WHITE, 0.2 * p.hit));
                for k in 0..8 {
                    let a = k as f32 / 8.0 * TAU + p.beat as f32 * 0.2;
                    draw_line(*bx, top + 10.0, bx + a.cos() * 22.0, top + 10.0 + a.sin() * 22.0, 2.0, pal::INK);
                }
            }
        }
        // Heavy brutalist beam across the top.
        draw_rectangle(0.0, -40.0, size.x, 28.0, hex(0x4a4658));
        draw_rectangle(0.0, -14.0, size.x, 3.0, pal::INK);
        // Floor edge.
        draw_rectangle(0.0, wall_h - 6.0, size.x, 6.0, hex(0x1d1a28));
        // Event title projected onto the wall.
        if let Some(r) = &self.rave {
            let s = self.ui.s;
            let font = &self.fonts.title;
            let fs = 16.0;
            let tw = crate::gfx::text_w(font, &r.title, fs * s) / s;
            let x = (bays[0] + bays[1]) / 2.0 - tw / 2.0;
            draw_text_ex(&r.title, x, 120.0, TextParams { font: Some(font), font_size: (fs * s) as u16, font_scale: 1.0 / s, color: with_alpha(pal::CREAM, 0.25 + 0.3 * p.hit), ..Default::default() });
            let x2 = (bays[1] + bays[2]) / 2.0 - tw / 2.0;
            draw_text_ex(&r.title, x2, 120.0, TextParams { font: Some(font), font_size: (fs * s) as u16, font_scale: 1.0 / s, color: with_alpha(pal::CREAM, 0.25 + 0.3 * p.hit), ..Default::default() });
        }
    }

    fn draw_rave_lights(&self, p: &Pulse, view: Rect) {
        let size = self.chapel.px_size();
        // A light veil so the beams read.
        draw_rectangle(view.x, view.y, view.w, view.h, with_alpha(hex(0x07040f), 0.25));
        gl_use_material(&self.add);
        let t = self.time;
        let src = vec2(size.x / 2.0, 6.0 * TILE - 10.0);
        // Lasers fan out from the booth.
        let n = if p.drop { 10 } else { (3.0 + p.energy * 5.0) as usize };
        for i in 0..n {
            let k = i as f32 / (n.max(2) - 1) as f32;
            let sweep = (t * (0.6 + p.energy) + i as f32 * 0.35).sin() * 0.5;
            let a = std::f32::consts::FRAC_PI_2 + (k - 0.5) * 2.2 + sweep * 0.4;
            let end = src + vec2(a.cos(), a.sin()) * 900.0;
            let col = if i % 2 == 0 { pal::ACID } else { pal::MAGENTA };
            let alpha = 0.12 + 0.35 * p.hit;
            draw_line(src.x, src.y, end.x, end.y, 1.5, with_alpha(col, alpha));
            draw_line(src.x, src.y, end.x, end.y, 4.0, with_alpha(col, alpha * 0.25));
        }
        // Moving heads: soft beams on the crowd.
        for i in 0..4 {
            let x = size.x * (0.2 + 0.2 * i as f32) + (t * 0.8 + i as f32).sin() * 120.0;
            let y = 360.0 + (t * 0.6 + i as f32 * 2.0).cos() * 90.0;
            let c = [pal::MAGENTA, pal::CYAN, pal::PURPLE, pal::BLUE][i];
            draw_glow(&self.glow, x, y, 90.0, with_alpha(c, 0.18 + 0.25 * p.hit));
        }
        // Booth glow.
        draw_glow(&self.glow, src.x, src.y + 10.0, 120.0, with_alpha(pal::MAGENTA, 0.25 + 0.2 * p.hit));
        // Fog.
        for f in &self.particles {
            let a = (f.life / f.max * std::f32::consts::PI).sin() * 0.035;
            draw_glow(&self.glow, f.pos.x, f.pos.y, f.size, with_alpha(f.color, a));
        }
        gl_use_default_material();
        // Strobe on the drop.
        if p.drop && (p.beat * 4.0).rem_euclid(2.0) < 0.25 && p.section != "LIVE" {
            draw_rectangle(view.x, view.y, view.w, view.h, with_alpha(pal::WHITE, 0.07));
        }
    }

    // ---------------------------------------------------------------------
    // HUD
    // ---------------------------------------------------------------------

    fn draw_hud(&mut self) {
        let ui_w = self.ui.w;
        let ui_h = self.ui.h;
        let pulse = self.pulse();

        self.ui.toast_y = match self.scene {
            Scene::Rave => 80.0,
            _ => 40.0,
        };
        // Location pill.
        let (place, tone) = match self.scene {
            Scene::Town => ("QUACKVILLE", Tone::Cream),
            Scene::PcBang => ("PC방 · 24H", Tone::Blue),
            Scene::Rave => ("ST. BASSILICA", Tone::Pink),
        };
        let place = if self.scene == Scene::PcBang { "PC BANG · 24H" } else { place };
        let pw = self.ui.measure(&self.fonts.title, place, 12.0) + 20.0;
        self.ui.panel(Rect::new(10.0, 10.0, pw, 24.0), tone.color());
        self.ui.text(&self.fonts.title, place, 20.0, 27.0, 12.0, pal::INK);

        // Connection pill.
        let online = self.net.as_ref().map(|n| !n.is_offline()).unwrap_or(false);
        let count = self.remotes.len() + 1;
        let status = if online { format!("ONLINE · {count} HERE") } else { "OFFLINE".to_string() };
        let sw = self.ui.measure(&self.fonts.small, &status, 8.0) + 26.0;
        let sr = Rect::new(ui_w - sw - 14.0, 10.0, sw, 20.0);
        self.ui.panel(sr, pal::INK2);
        self.ui.rect(Rect::new(sr.x + 9.0, sr.y + 8.0, 5.0, 5.0), if online { pal::GREEN } else { pal::CONCRETE });
        self.ui.text(&self.fonts.small, &status, sr.x + 19.0, sr.y + 13.5, 8.0, pal::CREAM);

        // In the chapel: what's playing.
        if self.scene == Scene::Rave {
            if let Some(r) = self.rave.clone() {
                let live = r.live || self.capture.is_some();
                let now = if live {
                    let who = if r.host_is_dj { format!("LIVE · DJ {}", r.host_name.to_uppercase()) } else { "LIVE".into() };
                    who
                } else {
                    format!("GENERATED SET · {} · {} BPM", pulse.section, r.bpm.round())
                };
                let w = 300.0f32.min(ui_w - 40.0);
                let x = (ui_w - w) / 2.0;
                let rr = Rect::new(x, 8.0, w, 40.0);
                self.ui.panel(rr, pal::INK2);
                self.ui.text_center(&self.fonts.title, &r.title, ui_w / 2.0, 25.0, 13.0, pal::CREAM);
                let col = if live { pal::DJ } else { pal::ACID };
                if live {
                    self.ui.rect(Rect::new(x + 10.0, 33.0, 6.0, 6.0), with_alpha(pal::DJ, 0.5 + 0.5 * ((self.time * 4.0).sin() * 0.5 + 0.5)));
                }
                self.ui.text_center(&self.fonts.small, &now, ui_w / 2.0, 41.0, 7.0, col);
                // Beat meter.
                for i in 0..4 {
                    let on = (pulse.beat.rem_euclid(4.0) as usize) == i;
                    self.ui.rect(Rect::new(x + w - 40.0 + i as f32 * 7.0, 34.0, 5.0, 5.0), if on { pal::YELLOW } else { pal::INK });
                }
                // Code chip (click to copy).
                let code_label = format!("CODE {}", r.code);
                let cw = self.ui.measure(&self.fonts.small, &code_label, 8.0) + 16.0;
                let cr = Rect::new((ui_w - cw) / 2.0, 54.0, cw, 16.0);
                if self.ui.button(&self.fonts, cr, &code_label, Tone::Ink) {
                    miniquad::window::clipboard_set(&r.code);
                    self.ui.toast(format!("{} copied. bring your friends.", r.code), Tone::Green);
                }
            }
        } else if self.scene == Scene::Town {
            if let Some(r) = self.public_raves.iter().find(|r| r.live) {
                let t = format!("LIVE NOW AT ST. BASSILICA: DJ {}", r.host_name.to_uppercase());
                let w = self.ui.measure(&self.fonts.small, &t, 8.0) + 30.0;
                let rr = Rect::new((ui_w - w) / 2.0, 10.0, w, 20.0);
                self.ui.panel(rr, pal::DJ);
                self.ui.text(&self.fonts.small, &t, rr.x + 15.0, rr.y + 13.5, 8.0, pal::CREAM);
            }
        }

        // Chat log.
        let show_all = self.chat_open;
        let mut y = ui_h - 38.0;
        for c in self.chat.iter().rev().take(if show_all { 10 } else { 5 }) {
            let a = if show_all { 1.0 } else { (1.0 - (c.age - 8.0) / 2.0).clamp(0.0, 1.0) };
            if a <= 0.0 {
                continue;
            }
            let name = format!("{}:", c.name);
            let nw = self.ui.measure(&self.fonts.body, &name, 10.0);
            let tw = self.ui.measure(&self.fonts.body, &c.text, 10.0);
            let dot = if c.dj { 8.0 } else { 0.0 };
            self.ui.rect(Rect::new(10.0, y - 11.0, nw + tw + 14.0 + dot, 14.0), with_alpha(pal::INK, 0.6 * a));
            if c.dj {
                self.ui.rect(Rect::new(14.0, y - 6.0, 4.0, 4.0), with_alpha(pal::DJ, a));
            }
            self.ui.text(&self.fonts.body, &name, 14.0 + dot, y, 10.0, with_alpha(if c.dj { pal::PINK } else { pal::YELLOW }, a));
            self.ui.text(&self.fonts.body, &c.text, 18.0 + nw + dot, y, 10.0, with_alpha(pal::CREAM, a));
            y -= 15.0;
        }
        if self.chat_open {
            let r = Rect::new(10.0, ui_h - 30.0, (ui_w * 0.5).max(260.0), 20.0);
            let mut t = std::mem::take(&mut self.chat_text);
            self.ui.focus = Some("chat".into());
            let sent = self.ui.field(&self.fonts, "chat", r, &mut t, 120, "say something");
            if sent {
                let msg = t.trim().to_string();
                t.clear();
                self.chat_open = false;
                self.ui.focus = None;
                if !msg.is_empty() {
                    self.send(mq_shared::ClientMsg::Chat { text: msg });
                }
            }
            self.chat_text = t;
        } else if matches!(self.overlay, Overlay::None) {
            // Controls hint.
            let hint = match self.scene {
                Scene::Rave => "WASD MOVE · 1-4 DANCE · ENTER CHAT · ESC MENU",
                _ => "WASD MOVE · SHIFT RUN · E USE · ENTER CHAT · ESC MENU",
            };
            let w = self.ui.measure(&self.fonts.small, hint, 7.0);
            self.ui.text(&self.fonts.small, hint, ui_w - w - 12.0, ui_h - 10.0, 7.0, with_alpha(pal::CREAM, 0.6));
        }
    }

    // ---------------------------------------------------------------------
    // Overlays
    // ---------------------------------------------------------------------

    fn draw_overlay(&mut self) {
        let overlay = std::mem::replace(&mut self.overlay, Overlay::None);
        let mut keep = true;
        let (uw, uh) = (self.ui.w, self.ui.h);
        let overlay = match overlay {
            Overlay::None => Overlay::None,
            Overlay::Computer => {
                let clock = {
                    let mins = ((self.server_ms() / 60_000.0) as i64).rem_euclid(24 * 60);
                    format!("{:02}:{:02}", mins / 60, mins % 60)
                };
                let ctx = computer::Ctx {
                    flyers: &self.underground,
                    guestbook: &self.guestbook,
                    hosted: &self.hosted,
                    hosted_flyers: &self.hosted_tex,
                    is_dj: self.is_dj,
                    name: &self.my_name,
                    clock,
                    crt: self.settings.crt,
                };
                let acts = self.computer.frame(&mut self.ui, &self.fonts, &ctx);
                self.overlay = Overlay::Computer;
                self.computer_actions(acts);
                // An action may have changed the overlay (stand up, open flyer).
                return;
            }
            Overlay::Poster { flyer, tex } => {
                self.ui.dim(0.7);
                let h = (uh - 60.0).min(300.0);
                let w = h * FLYER_W / FLYER_H;
                let x = (uw - w) / 2.0 - 70.0;
                let y = (uh - h) / 2.0;
                self.ui.rect(Rect::new(x + 6.0, y + 6.0, w, h), pal::INK);
                let s = self.ui.s;
                draw_texture_ex(&tex, x * s, y * s, WHITE, DrawTextureParams { dest_size: Some(vec2(w * s, h * s)), ..Default::default() });
                // Side card.
                let cx = x + w + 20.0;
                let card = Rect::new(cx, y + 20.0, 170.0, 150.0);
                self.ui.panel(card, pal::CREAM);
                self.ui.text(&self.fonts.small, "THE CODE", card.x + 12.0, card.y + 20.0, 8.0, with_alpha(pal::INK, 0.6));
                self.ui.text(&self.fonts.mono, &flyer.code, card.x + 12.0, card.y + 44.0, 24.0, pal::INK);
                let note = if flyer.expired {
                    "this one's old and torn."
                } else if flyer.live {
                    "live set happening now."
                } else {
                    "take it to St. Bassilica."
                };
                self.ui.text(&self.fonts.body, note, card.x + 12.0, card.y + 64.0, 10.0, pal::INK);
                if self.ui.button(&self.fonts, Rect::new(card.x + 12.0, card.y + 78.0, 146.0, 24.0), "Copy code", Tone::Yellow) {
                    miniquad::window::clipboard_set(&flyer.code);
                    self.ui.toast(format!("{} copied", flyer.code), Tone::Green);
                    self.sfx(crate::audio::Sfx::Select);
                }
                if self.ui.button(&self.fonts, Rect::new(card.x + 12.0, card.y + 110.0, 146.0, 24.0), "Close", Tone::Cream) {
                    keep = false;
                    self.sfx(crate::audio::Sfx::Back);
                }
                Overlay::Poster { flyer, tex }
            }
            Overlay::Door { mut code, error, waiting } => {
                self.ui.dim(0.55);
                let w = 330.0;
                let h = 196.0;
                let r = Rect::new((uw - w) / 2.0, (uh - h) / 2.0, w, h);
                self.ui.panel(r, pal::INK2);
                // Bouncer portrait.
                let s = self.ui.s;
                draw_texture_ex(
                    &self.sprites.bouncer,
                    (r.x + 14.0) * s,
                    (r.y + 14.0) * s,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(40.0 * 2.0 * s, 56.0 * 2.0 * s)), ..Default::default() },
                );
                let tx = r.x + 112.0;
                self.ui.text(&self.fonts.small, "THE GOOSE", tx, r.y + 24.0, 8.0, pal::PINK);
                self.ui.text(&self.fonts.body, "evening. code?", tx, r.y + 42.0, 13.0, pal::CREAM);
                let fr = Rect::new(tx, r.y + 52.0, 200.0, 26.0);
                let submit_field = self.ui.field(&self.fonts, "door", fr, &mut code, 14, "QK7-MAX-9PE");
                let mut submit = submit_field;
                if self.ui.button_ex(&self.fonts, Rect::new(tx, r.y + 86.0, 96.0, 24.0), if waiting { "..." } else { "Enter" }, Tone::Yellow, !waiting) {
                    submit = true;
                }
                if self.ui.button(&self.fonts, Rect::new(tx + 104.0, r.y + 86.0, 96.0, 24.0), "Walk away", Tone::Cream) {
                    keep = false;
                    self.ui.focus = None;
                }
                let mut code_after = code;
                // Codes you've picked up.
                if !self.known_codes.is_empty() {
                    self.ui.text(&self.fonts.small, "CODES YOU'VE SEEN", r.x + 14.0, r.y + 140.0, 7.0, with_alpha(pal::CREAM, 0.6));
                    let mut cx = r.x + 14.0;
                    let mut cy = r.y + 146.0;
                    for c in self.known_codes.clone() {
                        let cw = self.ui.measure(&self.fonts.small, &c, 7.0) + 12.0;
                        if cx + cw > r.right() - 10.0 {
                            cx = r.x + 14.0;
                            cy += 18.0;
                        }
                        if cy > r.bottom() - 18.0 {
                            break;
                        }
                        if self.ui.button(&self.fonts, Rect::new(cx, cy, cw, 14.0), &c, Tone::Ink) {
                            code_after = c.clone();
                            submit = true;
                        }
                        cx += cw + 6.0;
                    }
                }
                if let Some(e) = &error {
                    for (i, line) in computer::wrap(e, 34).iter().enumerate() {
                        self.ui.text(&self.fonts.body, line, tx, r.y + 124.0 + i as f32 * 11.0, 9.5, pal::CORAL);
                    }
                }
                self.overlay = Overlay::Door { code: code_after.clone(), error, waiting };
                if submit && keep && !waiting {
                    self.try_code(&code_after);
                }
                if keep {
                    return;
                }
                Overlay::None
            }
            Overlay::Dialog { who, lines } => {
                let w = (uw - 80.0).min(420.0);
                let h = 40.0 + lines.len() as f32 * 14.0 + 30.0;
                let r = Rect::new((uw - w) / 2.0, uh - h - 40.0, w, h);
                self.ui.panel(r, pal::CREAM);
                let tag = Rect::new(r.x + 12.0, r.y - 10.0, self.ui.measure(&self.fonts.small, who, 8.0) + 16.0, 18.0);
                self.ui.panel(tag, pal::YELLOW);
                self.ui.text(&self.fonts.small, who, tag.x + 8.0, tag.y + 12.5, 8.0, pal::INK);
                for (i, l) in lines.iter().enumerate() {
                    self.ui.text(&self.fonts.body, l, r.x + 16.0, r.y + 28.0 + i as f32 * 14.0, 11.0, pal::INK);
                }
                if self.ui.button(&self.fonts, Rect::new(r.right() - 90.0, r.bottom() - 30.0, 78.0, 20.0), "OK", Tone::Yellow)
                    || is_key_pressed(KeyCode::E)
                    || is_key_pressed(KeyCode::Space)
                    || self.ui.enter
                {
                    keep = false;
                }
                Overlay::Dialog { who, lines }
            }
            Overlay::Decks => {
                self.ui.dim(0.5);
                let w = 360.0;
                let h = 230.0;
                let r = Rect::new((uw - w) / 2.0, (uh - h) / 2.0, w, h);
                self.ui.panel(r, pal::INK2);
                self.ui.rect(Rect::new(r.x + 14.0, r.y + 14.0, 8.0, 8.0), pal::DJ);
                self.ui.text(&self.fonts.title, "THE DECKS", r.x + 28.0, r.y + 23.0, 13.0, pal::CREAM);
                let live = self.capture.is_some();
                let mut y = r.y + 44.0;
                for l in [
                    "play your Serato set live to everyone in here.",
                    "route Serato's master output into a loopback",
                    "device (on a Mac: BlackHole 2ch, free) or plug",
                    "your mixer's record out into your line in,",
                    "then pick that input below.",
                ] {
                    self.ui.text(&self.fonts.body, l, r.x + 14.0, y, 10.0, with_alpha(pal::CREAM, 0.85));
                    y += 13.0;
                }
                y += 6.0;
                let name = if self.inputs.is_empty() {
                    "default input".to_string()
                } else {
                    self.inputs[self.input_sel.min(self.inputs.len() - 1)].clone()
                };
                let field = Rect::new(r.x + 14.0, y, w - 28.0, 22.0);
                self.ui.rect(field, pal::INK);
                self.ui.rect(Rect::new(field.x + 2.0, field.y + 2.0, field.w - 4.0, field.h - 4.0), pal::BLACK);
                let shown: String = name.chars().take(34).collect();
                self.ui.text(&self.fonts.body, &shown, field.x + 30.0, field.y + 15.0, 10.0, pal::ACID);
                if !live && self.inputs.len() > 1 {
                    if self.ui.button(&self.fonts, Rect::new(field.x + 3.0, field.y + 3.0, 20.0, 16.0), "<", Tone::Ink) {
                        self.input_sel = (self.input_sel + self.inputs.len() - 1) % self.inputs.len();
                    }
                    if self.ui.button(&self.fonts, Rect::new(field.right() - 23.0, field.y + 3.0, 20.0, 16.0), ">", Tone::Ink) {
                        self.input_sel = (self.input_sel + 1) % self.inputs.len();
                    }
                }
                y += 30.0;
                // Level meter.
                let lvl = if live { self.audio.input_level() } else { 0.0 };
                for i in 0..24 {
                    let on = (i as f32 / 24.0) < lvl;
                    let c = if i > 20 { pal::CORAL } else if i > 15 { pal::YELLOW } else { pal::GREEN };
                    self.ui.rect(Rect::new(r.x + 14.0 + i as f32 * 8.0, y, 6.0, 8.0), if on { c } else { pal::INK });
                }
                y += 18.0;
                if live {
                    if self.ui.button(&self.fonts, Rect::new(r.x + 14.0, y, 150.0, 26.0), "Stop live set", Tone::Coral) {
                        self.stop_live();
                    }
                } else if self.ui.button(&self.fonts, Rect::new(r.x + 14.0, y, 150.0, 26.0), "GO LIVE", Tone::Dj) {
                    self.go_live();
                }
                if self.ui.button(&self.fonts, Rect::new(r.right() - 110.0, y, 96.0, 26.0), "Close", Tone::Cream) {
                    keep = false;
                }
                Overlay::Decks
            }
            Overlay::Pause => {
                self.ui.dim(0.6);
                let w = 220.0;
                let h = 238.0;
                let r = Rect::new((uw - w) / 2.0, (uh - h) / 2.0, w, h);
                self.ui.panel(r, pal::CREAM);
                self.ui.text_center(&self.fonts.title, "PAUSED-ISH", r.x + w / 2.0, r.y + 26.0, 15.0, pal::INK);
                self.ui.text_center(&self.fonts.small, "(THE WORLD KEEPS RAVING)", r.x + w / 2.0, r.y + 40.0, 7.0, with_alpha(pal::INK, 0.6));
                let bx = r.x + 16.0;
                let bw = w - 32.0;
                let mut y = r.y + 52.0;
                if self.ui.button(&self.fonts, Rect::new(bx, y, bw, 24.0), "Back to the game", Tone::Yellow) {
                    keep = false;
                }
                y += 32.0;
                let vol = format!("Volume {}%", (self.settings.volume * 100.0).round());
                self.ui.text(&self.fonts.body, &vol, bx, y + 15.0, 11.0, pal::INK);
                if self.ui.button(&self.fonts, Rect::new(bx + bw - 54.0, y, 24.0, 22.0), "-", Tone::Cream) {
                    self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                    self.settings.save();
                }
                if self.ui.button(&self.fonts, Rect::new(bx + bw - 24.0, y, 24.0, 22.0), "+", Tone::Cream) {
                    self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                    self.settings.save();
                }
                y += 30.0;
                let crt = if self.settings.crt { "CRT scanlines: on" } else { "CRT scanlines: off" };
                if self.ui.button(&self.fonts, Rect::new(bx, y, bw, 22.0), crt, Tone::Cream) {
                    self.settings.crt = !self.settings.crt;
                    self.settings.save();
                }
                y += 28.0;
                let fs = if self.settings.fullscreen { "Windowed" } else { "Full screen" };
                if self.ui.button(&self.fonts, Rect::new(bx, y, bw, 22.0), fs, Tone::Cream) {
                    self.settings.fullscreen = !self.settings.fullscreen;
                    set_fullscreen(self.settings.fullscreen);
                    self.settings.save();
                }
                y += 28.0;
                if self.ui.button(&self.fonts, Rect::new(bx, y, bw, 22.0), "Leave to title", Tone::Pink) {
                    self.disconnect(None);
                    return;
                }
                y += 28.0;
                if self.ui.button(&self.fonts, Rect::new(bx, y, bw, 22.0), "Quit", Tone::Coral) {
                    self.settings.save();
                    std::process::exit(0);
                }
                Overlay::Pause
            }
        };
        if keep && matches!(self.overlay, Overlay::None) {
            self.overlay = overlay;
        }
    }

    // ---------------------------------------------------------------------
    // Title
    // ---------------------------------------------------------------------

    fn draw_title(&mut self) {
        // The town at dusk drifting behind the title.
        let size = self.town.px_size();
        let t = self.time * 12.0;
        let center = vec2(size.x * 0.5 + (t * 0.05).sin() * 300.0, size.y * 0.4 + (t * 0.03).cos() * 120.0);
        let saved = self.scene;
        self.scene = Scene::Town;
        self.draw_world_at(center);
        self.scene = saved;

        let (uw, uh) = (self.ui.w, self.ui.h);
        self.ui.rect(Rect::new(0.0, 0.0, uw, uh), with_alpha(pal::INK, 0.35));

        // Logo, cover-style.
        let tsize = 52.0;
        let title = "Melody Quack";
        let tw = self.ui.measure(&self.fonts.title, title, tsize);
        let tx = (uw - tw) / 2.0;
        let ty = 76.0;
        for (dx, dy) in [(4.0, 4.0), (4.0, 0.0), (0.0, 4.0), (-2.0, 0.0), (0.0, -2.0)] {
            self.ui.text(&self.fonts.title, title, tx + dx, ty + dy, tsize, pal::INK);
        }
        self.ui.text(&self.fonts.title, title, tx, ty, tsize, pal::YELLOW);
        self.ui.text_shadow(&self.fonts.body, "sing, jam and rave with friends", (uw - self.ui.measure(&self.fonts.body, "sing, jam and rave with friends", 12.0)) / 2.0, ty + 22.0, 12.0, pal::CREAM, 2.0);
        // The six note keys from the cover.
        let keys = [(pal::CORAL, "1", "D"), (pal::PINK, "2", "E"), (pal::BLUE, "3", "F#"), (pal::GREEN, "4", "A"), (pal::SKY, "5", "B"), (pal::YELLOW, "6", "C")];
        let kx0 = (uw - (6.0 * 26.0 - 4.0)) / 2.0;
        let pulse = self.pulse();
        for (i, (c, n, note)) in keys.iter().enumerate() {
            let on = (pulse.beat.rem_euclid(6.0) as usize) == i;
            let r = Rect::new(kx0 + i as f32 * 26.0, ty + 32.0 - if on { 2.0 } else { 0.0 }, 22.0, 22.0);
            self.ui.panel(r, *c);
            self.ui.text(&self.fonts.small, n, r.x + 4.0, r.y + 9.0, 6.0, pal::INK);
            self.ui.text(&self.fonts.small, note, r.x + 4.0, r.y + 17.0, 6.0, pal::INK);
        }

        if self.screen == Screen::Connecting {
            let r = Rect::new((uw - 220.0) / 2.0, 200.0, 220.0, 60.0);
            self.ui.panel(r, pal::CREAM);
            let dots = ".".repeat(((self.time * 3.0) as usize % 3) + 1);
            self.ui.text_center(&self.fonts.title, &format!("connecting{dots}"), uw / 2.0, r.y + 26.0, 13.0, pal::INK);
            self.ui.text_center(&self.fonts.small, &self.settings.server.to_uppercase(), uw / 2.0, r.y + 44.0, 7.0, with_alpha(pal::INK, 0.6));
            if self.ui.button(&self.fonts, Rect::new(r.x + 60.0, r.bottom() + 8.0, 100.0, 20.0), "Cancel", Tone::Cream) {
                self.disconnect(None);
            }
            return;
        }

        // Card.
        let cw = 380.0;
        let ch = 214.0;
        let card = Rect::new((uw - cw) / 2.0, 138.0, cw, ch);
        self.ui.panel(card, pal::CREAM);
        // Character picker.
        let pick = Rect::new(card.x + 14.0, card.y + 14.0, 110.0, 130.0);
        self.ui.rect(pick, pal::INK2);
        let kind = self.settings.avatar as usize % CRITTERS.len();
        let frame = if (self.time * 2.0) as i32 % 2 == 0 { 0 } else { 3 };
        let s = self.ui.s;
        draw_texture_ex(
            &self.sprites.critters[kind][frame],
            (pick.x + 23.0) * s,
            (pick.y + 14.0) * s,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(64.0 * s, 80.0 * s)), ..Default::default() },
        );
        self.ui.text_center(&self.fonts.small, &CRITTERS[kind].name.to_uppercase(), pick.x + pick.w / 2.0, pick.y + 110.0, 8.0, pal::CREAM);
        if self.ui.button(&self.fonts, Rect::new(pick.x + 4.0, pick.y + 112.0, 20.0, 16.0), "<", Tone::Cream) {
            self.settings.avatar = ((kind + CRITTERS.len() - 1) % CRITTERS.len()) as u8;
            self.sfx(crate::audio::Sfx::Blip);
        }
        if self.ui.button(&self.fonts, Rect::new(pick.right() - 24.0, pick.y + 112.0, 20.0, 16.0), ">", Tone::Cream) {
            self.settings.avatar = ((kind + 1) % CRITTERS.len()) as u8;
            self.sfx(crate::audio::Sfx::Blip);
        }
        // Fields.
        let fx = card.x + 136.0;
        let fw = card.w - 150.0;
        self.ui.text(&self.fonts.small, "YOUR NAME", fx, card.y + 22.0, 7.0, with_alpha(pal::INK, 0.65));
        let mut name = std::mem::take(&mut self.settings.name);
        self.ui.field(&self.fonts, "name", Rect::new(fx, card.y + 26.0, fw, 22.0), &mut name, 14, "duckling");
        self.settings.name = name;
        self.ui.text(&self.fonts.small, "SERVER", fx, card.y + 62.0, 7.0, with_alpha(pal::INK, 0.65));
        let mut server = std::mem::take(&mut self.settings.server);
        self.ui.field(&self.fonts, "server", Rect::new(fx, card.y + 66.0, fw, 22.0), &mut server, 60, "ws://127.0.0.1:7777");
        self.settings.server = server;
        // DJ key.
        let toggle = Rect::new(fx, card.y + 96.0, 10.0, 10.0);
        self.ui.rect(toggle, pal::INK);
        self.ui.rect(Rect::new(toggle.x + 2.0, toggle.y + 2.0, 6.0, 6.0), if self.show_dj_key { pal::DJ } else { pal::CREAM });
        self.ui.text(&self.fonts.body, "I'm a DJ", fx + 16.0, card.y + 105.0, 10.0, pal::INK);
        let hit = Rect::new(fx, card.y + 94.0, 80.0, 14.0);
        if self.ui.hover(hit) {
            self.ui.set_hot();
            if self.ui.clicked {
                self.ui.click_used = true;
                self.show_dj_key = !self.show_dj_key;
            }
        }
        if self.show_dj_key {
            let mut key = std::mem::take(&mut self.settings.dj_key);
            self.ui.field(&self.fonts, "djkey", Rect::new(fx + 74.0, card.y + 92.0, fw - 74.0, 18.0), &mut key, 40, "dj key");
            self.settings.dj_key = key;
        }
        // Buttons.
        let by = card.y + 152.0;
        if self.ui.button(&self.fonts, Rect::new(card.x + 14.0, by, 172.0, 30.0), "PLAY ONLINE", Tone::Yellow) || (self.ui.enter && self.ui.focus.is_none()) {
            self.connect(false);
        }
        if self.ui.button(&self.fonts, Rect::new(card.x + 194.0, by, 172.0, 30.0), "PLAY OFFLINE", Tone::Blue) {
            self.connect(true);
        }
        if let Some(e) = self.title_error.clone() {
            let lines = computer::wrap(&e, 60);
            for (i, l) in lines.iter().take(2).enumerate() {
                self.ui.text(&self.fonts.body, l, card.x + 14.0, card.y + 198.0 + i as f32 * 11.0, 9.0, hex(0xc0392b));
            }
        } else {
            self.ui.text(&self.fonts.body, "no server? play offline. you can still host raves for yourself.", card.x + 14.0, card.y + 202.0, 9.0, with_alpha(pal::INK, 0.6));
        }
        self.ui.text(&self.fonts.small, "V0.1 · PIXEL FONTS: PIXELIFY SANS, VT323, SILKSCREEN (OFL)", 8.0, uh - 8.0, 6.0, with_alpha(pal::CREAM, 0.45));
    }
}
