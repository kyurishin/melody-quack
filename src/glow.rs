#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use std::cell::RefCell;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const BLUR: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform vec2 texel;
uniform float night;

vec3 bright(vec2 p) {
    vec3 c = texture2D(Texture, p).rgb;
    float l = max(max(c.r, c.g), c.b);
    float s = l - min(min(c.r, c.g), c.b);
    return c * smoothstep(0.9 - 0.04 * night, 1.0, l) * smoothstep(0.3, 0.75, s + 0.2 * night);
}

void main() {
    vec3 glow = bright(uv) * 0.6;
    for (int i = 0; i < 8; i++) {
        float a = float(i) * 0.785398 + 0.39;
        vec2 d = vec2(cos(a), sin(a)) * texel;
        glow += bright(uv + d * 1.5) * 0.5;
        glow += bright(uv + d * 3.5) * 0.32;
        glow += bright(uv + d * 6.5) * 0.18;
    }
    gl_FragColor = vec4(glow / 8.6, 1.0);
}
"#;

const MIX: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform sampler2D Glow;
uniform float night;

void main() {
    vec3 base = texture2D(Texture, uv).rgb;
    vec3 glow = texture2D(Glow, uv).rgb;
    vec3 c = base + glow * (0.55 + 0.45 * night);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    c = mix(vec3(l), c, 1.12);
    c = c + vec3(0.16, 0.1, 0.3) * (1.0 - smoothstep(0.0, 0.45, l)) * 0.22;
    vec2 v = uv * 2.0 - 1.0;
    c *= 1.0 - dot(v, v) * (0.1 + 0.08 * night);
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), 1.0);
}
"#;

struct Glow {
    blur: Material,
    mix: Material,
    target: RenderTarget,
}

thread_local! {
    static GLOW: RefCell<Option<Glow>> = const { RefCell::new(None) };
    static NIGHT: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

pub(crate) fn set_night(n: f32) {
    NIGHT.with(|v| v.set(n.clamp(0.0, 1.0)));
}

fn ready(w: u32, h: u32) -> bool {
    GLOW.with(|cell| {
        let mut slot = cell.borrow_mut();
        if let Some(g) = slot.as_mut() {
            if g.target.texture.width() as u32 != w || g.target.texture.height() as u32 != h {
                g.target = make_target(w, h);
            }
            return true;
        }
        let blur = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: BLUR },
            MaterialParams { uniforms: vec![UniformDesc::new("texel", UniformType::Float2), UniformDesc::new("night", UniformType::Float1)], ..Default::default() },
        );
        let mix = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: MIX },
            MaterialParams { uniforms: vec![UniformDesc::new("night", UniformType::Float1)], textures: vec!["Glow".to_string()], ..Default::default() },
        );
        match (blur, mix) {
            (Ok(blur), Ok(mix)) => {
                *slot = Some(Glow { blur, mix, target: make_target(w, h) });
                true
            }
            _ => false,
        }
    })
}

fn make_target(w: u32, h: u32) -> RenderTarget {
    let t = render_target_ex(w, h, RenderTargetParams { sample_count: 1, depth: false });
    t.texture.set_filter(FilterMode::Linear);
    t
}

pub(crate) fn present(view: &RenderTarget) {
    let (w, h) = (low_w().round().max(1.0), low_h().round().max(1.0));
    let night = NIGHT.with(|n| n.get());
    let full = vec2(screen_width(), screen_height());
    let done = ready(w as u32, h as u32)
        && GLOW.with(|cell| {
            let slot = cell.borrow();
            let Some(g) = slot.as_ref() else { return false };
            set_camera(&Camera2D { render_target: Some(g.target.clone()), zoom: vec2(2.0 / w, -2.0 / h), target: vec2(w / 2.0, h / 2.0), ..Default::default() });
            clear_background(BLACK);
            g.blur.set_uniform("texel", vec2(1.0 / w, 1.0 / h));
            g.blur.set_uniform("night", night);
            gl_use_material(&g.blur);
            draw_texture_ex(&view.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_y: true, ..Default::default() });
            gl_use_default_material();
            set_default_camera();
            clear_background(BLACK);
            g.mix.set_uniform("night", night);
            g.mix.set_texture("Glow", g.target.texture.clone());
            gl_use_material(&g.mix);
            draw_texture_ex(&view.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(full), flip_y: true, ..Default::default() });
            gl_use_default_material();
            true
        });
    if !done {
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(&view.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(full), flip_y: true, ..Default::default() });
    }
}
