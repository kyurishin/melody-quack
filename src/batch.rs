#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone)]
pub(crate) struct Light {
    pub(crate) dusk: f32,
    pub(crate) fire: Vec3,
    pub(crate) fire_strength: f32,
    pub(crate) tint: Vec3,
    pub(crate) lamps: Vec<(Vec3, f32, Vec3)>,
}

impl Light {
    pub(crate) fn fog(&self) -> Color {
        sky_haze(mix(FOG, DUSK_FOG, self.dusk), self.dusk)
    }

    pub(crate) fn apply(&self, p: Vec3, c: Color) -> Color {
        let amb = mix(WHITE, Color::new(0.5, 0.46, 0.7, 1.0), self.dusk);
        let amb = {
            let s = sky();
            let warm = s.glow * 0.35 * (1.0 - self.dusk * 0.5);
            let dim = [0.0, 0.08, 0.16, 0.02][s.weather];
            Color::new(amb.r * (1.0 - dim) + warm * 0.12, amb.g * (1.0 - dim) - warm * 0.04, amb.b * (1.0 - dim * 0.6) - warm * 0.12, 1.0)
        };
        let d = (p - self.fire).length();
        let glow = self.fire_strength * (1.0 - d / 5.0).max(0.0).powi(2);
        let mut add = self.tint * glow;
        let mut haze = vec2(glow * 0.08 * self.tint.x, glow * 0.04 * self.tint.y);
        for (at, strength, tint) in &self.lamps {
            let d = (p - *at).length();
            let g = strength * (1.0 - d / 4.5).max(0.0).powi(2);
            add += *tint * g;
            haze += vec2(g * 0.08 * tint.x, g * 0.04 * tint.y);
        }
        Color::new(
            (c.r * (amb.r + add.x) + haze.x).min(1.0),
            (c.g * (amb.g + add.y) + haze.y).min(1.0),
            (c.b * (amb.b + add.z)).min(1.0),
            c.a,
        )
    }
}

pub(crate) struct Batch {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u16>,
    pub(crate) eye: Vec3,
    pub(crate) light: Light,
    pub(crate) unlit: bool,
}

impl Batch {
    pub(crate) fn new(eye: Vec3, light: Light) -> Batch {
        Batch {
            vertices: Vec::new(),
            indices: Vec::new(),
            eye,
            light,
            unlit: false,
        }
    }

    pub(crate) fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        draw_mesh(&Mesh {
            vertices: std::mem::take(&mut self.vertices),
            indices: std::mem::take(&mut self.indices),
            texture: None,
        });
    }

    pub(crate) fn cube(&mut self, center: Vec3, size: Vec3, color: Color) {
        self.box_at(center, 0.0, Vec3::ZERO, size, color);
    }

    pub(crate) fn top(&mut self, center: Vec3, color: Color) {
        if self.vertices.len() > 3000 {
            self.flush();
        }
        let dist = (center - self.eye).length();
        let fog = smoothstep((dist - 9.0) / 16.0) * 0.75;
        let lit = self.light.apply(center, color);
        let col = Color::new((lit.r * 1.04).min(1.0), (lit.g * 1.02).min(1.0), lit.b * 0.96, color.a);
        let f = self.light.fog();
        let col = mix(col, Color::new(f.r, f.g, f.b, col.a), fog);
        let base = self.vertices.len() as u16;
        for (dx, dz) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            self.vertices.push(Vertex::new(center.x + dx, center.y, center.z + dz, 0.0, 0.0, col));
        }
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub(crate) fn box_at(&mut self, origin: Vec3, yaw: f32, local: Vec3, size: Vec3, color: Color) {
        if self.vertices.len() > 3000 {
            self.flush();
        }
        let light = vec3(-0.5, 0.0, 0.8).normalize();
        let (s, c) = yaw.sin_cos();
        let rot = |v: Vec3| vec3(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
        let h = size * 0.5;

        let faces: [(Vec3, [Vec3; 4]); 5] = [
            (vec3(0.0, 1.0, 0.0), [vec3(-1., 1., -1.), vec3(1., 1., -1.), vec3(1., 1., 1.), vec3(-1., 1., 1.)]),
            (vec3(0.0, 0.0, 1.0), [vec3(-1., -1., 1.), vec3(1., -1., 1.), vec3(1., 1., 1.), vec3(-1., 1., 1.)]),
            (vec3(0.0, 0.0, -1.0), [vec3(1., -1., -1.), vec3(-1., -1., -1.), vec3(-1., 1., -1.), vec3(1., 1., -1.)]),
            (vec3(1.0, 0.0, 0.0), [vec3(1., -1., 1.), vec3(1., -1., -1.), vec3(1., 1., -1.), vec3(1., 1., 1.)]),
            (vec3(-1.0, 0.0, 0.0), [vec3(-1., -1., -1.), vec3(-1., -1., 1.), vec3(-1., 1., 1.), vec3(-1., 1., -1.)]),
        ];

        let world_center = origin + rot(local);
        let dist = (world_center - self.eye).length();
        let fog = smoothstep((dist - 9.0) / 16.0) * 0.75;
        let fog_color = self.light.fog();
        let lit_center = if self.unlit { color } else { self.light.apply(world_center, color) };

        for (normal, corners) in faces {
            let n = rot(normal);
            let shade = if n.y > 0.5 || self.unlit {
                1.0
            } else {
                0.58 + 0.32 * n.dot(light).max(0.0)
            };
            let mut col = Color::new(lit_center.r * shade, lit_center.g * shade, lit_center.b * shade, color.a);
            if n.y > 0.5 {
                col = Color::new((col.r * 1.04).min(1.0), (col.g * 1.02).min(1.0), col.b * 0.96, col.a);
            }
            let col = mix(col, Color::new(fog_color.r, fog_color.g, fog_color.b, col.a), fog);
            let base = self.vertices.len() as u16;
            for k in corners {
                let p = origin + rot(local + k * h);
                self.vertices.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, col));
            }
            self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}
