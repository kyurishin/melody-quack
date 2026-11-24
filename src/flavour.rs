#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fx {
    pub(crate) pitch: f32,
    pub(crate) lpf: Option<f32>,
    pub(crate) hpf: Option<f32>,
    pub(crate) crush: Option<(u32, usize)>,
    pub(crate) vib: Option<(f32, f32)>,
    pub(crate) drive: f32,
    pub(crate) echo: Option<(f32, f32)>,
    pub(crate) room: f32,
}

const PLAIN: Fx = Fx { pitch: 1.0, lpf: None, hpf: None, crush: None, vib: None, drive: 0.0, echo: None, room: 0.0 };

pub(crate) const DRINKS: usize = 5;
pub(crate) const EAT_EMO: usize = 13;

pub(crate) fn drink(i: usize) -> (&'static str, Fx) {
    match i {
        0 => ("fizzy and crunchy", Fx { crush: Some((5, 3)), hpf: Some(200.0), ..PLAIN }),
        1 => ("fast and buzzy", Fx { pitch: 1.06, vib: Some((7.0, 0.45)), ..PLAIN }),
        2 => ("bright and airy", Fx { hpf: Some(900.0), room: 0.3, ..PLAIN }),
        3 => ("soft and muffled", Fx { lpf: Some(700.0), ..PLAIN }),
        _ => ("warm, with an echo", Fx { lpf: Some(3000.0), echo: Some((0.22, 0.45)), ..PLAIN }),
    }
}

pub(crate) fn dish(d: usize) -> (&'static str, Fx) {
    match d {
        0 => ("warm and cosy", Fx { lpf: Some(2200.0), room: 0.25, ..PLAIN }),
        1 => ("a deeper voice", Fx { pitch: 0.85, ..PLAIN }),
        2 => ("deep and soupy", Fx { pitch: 0.9, lpf: Some(1500.0), ..PLAIN }),
        3 => ("high and shimmering", Fx { pitch: 1.12, room: 0.5, ..PLAIN }),
        4 => ("a little wobbly", Fx { vib: Some((4.0, 0.5)), ..PLAIN }),
        5 => ("a dreamy wobble", Fx { vib: Some((2.5, 1.0)), lpf: Some(1800.0), ..PLAIN }),
        6 => ("hot and crunchy", Fx { drive: 4.0, ..PLAIN }),
        7 => ("sparkly echoes", Fx { pitch: 1.25, echo: Some((0.18, 0.4)), ..PLAIN }),
        8 => ("smoky", Fx { lpf: Some(900.0), drive: 2.0, ..PLAIN }),
        9 => ("a cathedral voice", Fx { room: 0.85, ..PLAIN }),
        10 => ("echoing over the lake", Fx { echo: Some((0.33, 0.55)), ..PLAIN }),
        11 => ("a royal boom", Fx { pitch: 0.75, room: 0.5, ..PLAIN }),
        12 => ("smooth and high", Fx { pitch: 1.18, ..PLAIN }),
        13 => ("crispy", Fx { crush: Some((5, 2)), ..PLAIN }),
        14 => ("fluttery, like a bird", Fx { hpf: Some(600.0), vib: Some((6.0, 0.25)), ..PLAIN }),
        15 => ("an old game console", Fx { crush: Some((4, 6)), ..PLAIN }),
        16 => ("fiery", Fx { drive: 6.0, pitch: 1.1, ..PLAIN }),
        17 => ("a soft double voice", Fx { echo: Some((0.12, 0.3)), lpf: Some(2500.0), ..PLAIN }),
        18 => ("chipmunk-high", Fx { pitch: 1.5, ..PLAIN }),
        19 => ("far away in the woods", Fx { lpf: Some(500.0), room: 0.4, ..PLAIN }),
        _ => ("a ghostly moon voice", Fx { pitch: 0.8, echo: Some((0.4, 0.6)), room: 0.6, ..PLAIN }),
    }
}

pub(crate) fn kind_fx(kind: usize) -> (&'static str, Fx) {
    if kind < DRINKS {
        drink(kind)
    } else {
        dish(kind - DRINKS)
    }
}

pub(crate) fn kinds() -> usize {
    DRINKS + DISHES.len()
}

fn biquad(x: &mut [f32], f: f32, rate: f32, high: bool) {
    let w = std::f32::consts::TAU * f.min(rate * 0.45) / rate;
    let alpha = w.sin() / 1.414;
    let c = w.cos();
    let a0 = 1.0 + alpha;
    let (b0, b1, b2) = if high { ((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0) } else { ((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0) };
    let (b0, b1, b2, a1, a2) = (b0 / a0, b1 / a0, b2 / a0, -2.0 * c / a0, (1.0 - alpha) / a0);
    let (mut x1, mut x2, mut y1, mut y2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for v in x.iter_mut() {
        let y = b0 * *v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = *v;
        y2 = y1;
        y1 = y;
        *v = y;
    }
}

pub(crate) fn render(dry: &[f32], fx: &Fx, rate: f32) -> Vec<f32> {
    let peak = dry.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-4);
    let mut x: Vec<f32> = if (fx.pitch - 1.0).abs() > 1e-3 {
        let n = ((dry.len() as f32) / fx.pitch) as usize;
        (0..n)
            .map(|i| {
                let p = i as f32 * fx.pitch;
                let k = p as usize;
                let f = p - k as f32;
                let a = dry.get(k).copied().unwrap_or(0.0);
                let b = dry.get(k + 1).copied().unwrap_or(0.0);
                a + (b - a) * f
            })
            .collect()
    } else {
        dry.to_vec()
    };
    if let Some((r, semis)) = fx.vib {
        let depth = (2f32.powf(semis / 12.0) - 1.0) * rate / (std::f32::consts::TAU * r) ;
        let src = x.clone();
        for (i, v) in x.iter_mut().enumerate() {
            let t = i as f32 / rate;
            let d = depth * (1.0 + (std::f32::consts::TAU * r * t).sin()) + 1.0;
            let p = i as f32 - d;
            if p < 0.0 {
                *v = 0.0;
                continue;
            }
            let k = p as usize;
            let f = p - k as f32;
            let a = src.get(k).copied().unwrap_or(0.0);
            let b = src.get(k + 1).copied().unwrap_or(0.0);
            *v = a + (b - a) * f;
        }
    }
    if fx.drive > 0.0 {
        let norm = (fx.drive).tanh();
        for v in x.iter_mut() {
            *v = (*v / peak * fx.drive).tanh() / norm * peak;
        }
    }
    if let Some((bits, hold)) = fx.crush {
        let steps = (1u32 << bits.clamp(2, 12)) as f32 / 2.0;
        let mut held = 0.0;
        for (i, v) in x.iter_mut().enumerate() {
            if i % hold.max(1) == 0 {
                held = (*v / peak * steps).round() / steps * peak;
            }
            *v = held;
        }
    }
    if let Some(f) = fx.hpf {
        biquad(&mut x, f, rate, true);
    }
    if let Some(f) = fx.lpf {
        biquad(&mut x, f, rate, false);
    }
    if let Some((secs, fb)) = fx.echo {
        let gap = (secs * rate) as usize;
        let repeats = ((0.02f32).ln() / fb.max(0.05).ln()).ceil().clamp(1.0, 8.0) as usize;
        let n = x.len() + gap * repeats;
        let mut out = vec![0.0f32; n];
        out[..x.len()].copy_from_slice(&x);
        for i in gap..n {
            out[i] += out[i - gap] * fb;
        }
        x = out;
    }
    if fx.room > 0.01 {
        x = strudel::render_fx(&x, None, fx.room, rate);
    }
    let after = x.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-4);
    let k = peak / after;
    for v in x.iter_mut() {
        *v *= k;
    }
    x
}

type Key = (u16, u8);

thread_local! {
    static CACHE: std::cell::RefCell<Vec<(Key, audio::Sound, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}

const KEEP: usize = 96;

pub(crate) fn variant(g: &Game, sound: &audio::Sound, kind: usize) -> Option<audio::Sound> {
    if sound.id() == u16::MAX || kind >= kinds() {
        return None;
    }
    let key = (sound.id(), kind as u8);
    let now = get_time();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(e) = c.iter_mut().find(|e| e.0 == key) {
            e.2 = now;
            return g.audio.is_loaded(&e.1).then(|| e.1.clone());
        }
        if c.len() >= KEEP {
            if let Some(oldest) = c.iter().enumerate().min_by(|a, b| a.1 .2.total_cmp(&b.1 .2)).map(|(i, _)| i) {
                let (_, s, _) = c.remove(oldest);
                g.audio.unload(&s);
            }
        }
        let wet = render(sound.samples(), &kind_fx(kind).1, SAMPLE_RATE as f32);
        let s = g.audio.load_loose(wet);
        c.push((key, s.clone(), now));
        g.audio.is_loaded(&s).then_some(s)
    })
}

pub(crate) fn warm(g: &Game, character: usize, kind: usize) {
    for n in 0..6 {
        variant(g, &g.voices[character][n], kind);
    }
}

pub(crate) fn mine(g: &Game) -> Option<usize> {
    g.venue.treat.filter(|t| t.1 > 0.0).map(|t| t.0)
}

pub(crate) fn voice(g: &Game, who: usize, note: usize) -> audio::Sound {
    let base = &g.voices[g.players[who].character][note];
    if who == 0 {
        if let Some(kind) = mine(g) {
            if let Some(s) = variant(g, base, kind) {
                return s;
            }
        }
    }
    base.clone()
}

pub(crate) fn peer_voice(g: &Game, character: usize, note: usize, treat: Option<(usize, f32)>) -> audio::Sound {
    let base = &g.voices[character][note];
    if let Some((kind, left)) = treat {
        if left > 0.0 {
            if let Some(s) = variant(g, base, kind) {
                return s;
            }
        }
    }
    base.clone()
}

pub(crate) fn tick(g: &mut Game, dt: f32) {
    if let Some((_, left)) = g.venue.treat.as_mut() {
        *left -= dt;
    }
    if g.venue.treat.map(|t| t.1 <= 0.0).unwrap_or(false) {
        g.venue.treat = None;
        g.toasts.push(("your voice is back to normal".to_string(), 2.0));
    }
}

pub(crate) fn start(g: &mut Game, kind: usize) {
    g.venue.treat = Some((kind, venues::TREAT_TIME));
    warm(g, g.players[0].character, kind);
    if g.venue.joined {
        let e = if kind < DRINKS { venues::TREAT_EMO + kind } else { EAT_EMO + kind - DRINKS };
        net::send(&format!("emo 0 {e}"));
    }
}

pub(crate) fn eat(g: &mut Game, d: usize) -> bool {
    if g.meals.get(d).copied().unwrap_or(0) == 0 {
        return false;
    }
    g.meals[d] -= 1;
    start(g, DRINKS + d);
    let (what, _) = dish(d);
    g.toasts.push((format!("you eat the {} - {} for 30 seconds", DISHES[d], what), 3.5));
    g.audio.play_ch(3, &g.bells[2], 0.35);
    let at = g.players[0].pos;
    for k in 0..4 {
        g.hearts.push((vec3(at.x + (k as f32 * 0.7).sin() * 0.2, 1.1 + k as f32 * 0.12, at.y), 1.4));
    }
    g.save_clock = 99.0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_dish_and_drink_has_its_own_sound() {
        let mut seen: Vec<Fx> = Vec::new();
        for k in 0..kinds() {
            let fx = kind_fx(k).1;
            assert!(!seen.contains(&fx), "kind {k} repeats an effect");
            seen.push(fx);
        }
        let tone: Vec<f32> = (0..22050).map(|i| (i as f32 * 0.05).sin() * 0.5 * (1.0 - i as f32 / 22050.0)).collect();
        for k in 0..kinds() {
            let out = render(&tone, &kind_fx(k).1, 44100.0);
            assert!(!out.is_empty());
            assert!(out.iter().all(|v| v.is_finite() && v.abs() <= 0.51), "kind {k}");
            assert!(out.iter().zip(tone.iter()).any(|(a, b)| (a - b).abs() > 0.01) || out.len() != tone.len(), "kind {k} sounds the same");
        }
    }
}
