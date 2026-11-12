//! Synthesised animal voices (woof, meow, quack, hoot...).

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn render_woof(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.34;
    let mut phase = 0.0_f32;
    let mut seed = 0x51ed_270b;
    (0..(length * sr) as usize)
        .map(|i| {
            let age = i as f32 / sr;
            let u = age / length;
            let pitch = 1.18 - 0.33 * smoothstep(u / 0.6);
            let bright = 1.0 - 0.6 * u;
            let amp = smoothstep(age / 0.012) * (-age * 8.0).exp();
            let p = phase * 2.0 * PI;
            let tone = p.sin()
                + bright
                    * (0.6 * (2.0 * p).sin() + 0.45 * (3.0 * p).sin() + 0.3 * (4.0 * p).sin() + 0.18 * (5.0 * p).sin());
            let breath = noise(&mut seed) * (-age * 40.0).exp() * 0.5;
            phase = (phase + freq * pitch / sr) % 1.0;
            (tone / 2.3 + breath) * amp * 0.6
        })
        .collect()
}

pub(crate) fn normalise(samples: Vec<f32>, level: f32) -> Vec<f32> {
    let peak = samples.iter().fold(0.0_f32, |m, v| m.max(v.abs())).max(1e-6);
    samples.into_iter().map(|v| v / peak * level).collect()
}

pub(crate) fn render_meow_style(freq: f32, length: f32, bright: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut phase = 0.0_f32;
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let age = i as f32 / sr;
            let u = age / length;
            let pitch = if u < 0.3 { 0.92 + 0.13 * smoothstep(u / 0.3) } else { 1.05 - 0.2 * smoothstep((u - 0.3) / 0.7) };
            let vibrato = 1.0 + 0.01 * (age * 2.0 * PI * 5.5).sin();
            let shape = if u < 0.25 { u / 0.25 } else { 1.0 - 0.8 * ((u - 0.25) / 0.75) };
            let b = shape * bright;
            let amp = smoothstep(age / 0.05) * smoothstep((length - age) / 0.16);
            let p = phase * 2.0 * PI;
            let tone = p.sin() + b * (0.45 * (2.0 * p).sin() + 0.25 * (3.0 * p).sin() + 0.12 * (4.0 * p).sin() + 0.06 * bright * (5.0 * p).sin());
            phase = (phase + freq * pitch * vibrato / sr) % 1.0;
            tone * amp
        })
        .collect();
    normalise(out, 0.5)
}

pub(crate) fn render_yap(freq: f32) -> Vec<f32> {
    let mut w = render_woof(freq);
    w.truncate((0.2 * SAMPLE_RATE as f32) as usize);
    let n = w.len() as f32;
    let out = w.into_iter().enumerate().map(|(i, v)| v * smoothstep((n - i as f32) / (0.05 * SAMPLE_RATE as f32))).collect();
    normalise(out, 0.5)
}

pub(crate) fn render_howl(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.62;
    let mut phase = 0.0_f32;
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let u = t / length;
            let glide = 0.84 + 0.16 * smoothstep(u / 0.25) - 0.06 * smoothstep((u - 0.7) / 0.3);
            let vib = 1.0 + 0.015 * (t * 2.0 * PI * 6.0).sin() * smoothstep((u - 0.2) / 0.2);
            let open = (u / 0.3).min(1.0) * (1.0 - 0.5 * smoothstep((u - 0.6) / 0.4));
            let p = phase * 2.0 * PI;
            phase = (phase + freq * glide * vib / sr) % 1.0;
            let tone = p.sin() + open * (0.5 * (2.0 * p).sin() + 0.2 * (3.0 * p).sin());
            tone * smoothstep(t / 0.06) * smoothstep((length - t) / 0.2)
        })
        .collect();
    normalise(out, 0.5)
}

pub(crate) fn formant_gain(x: f32, formants: &[(f32, f32, f32)]) -> f32 {
    formants.iter().map(|&(f, a, bw)| a / (1.0 + ((x - f) / (f * bw)).powi(2))).sum()
}

pub(crate) fn render_quack(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.3;
    let mut phase = 0.0_f32;
    let formants = [(900.0, 1.0, 0.14), (1700.0, 0.85, 0.1), (2800.0, 0.4, 0.08)];
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let u = t / length;
            let pitch = 1.18 - 0.28 * smoothstep(u / 0.6) - 0.1 * smoothstep((u - 0.6) / 0.4);
            let f = freq * pitch;
            let p = phase * 2.0 * PI;
            phase = (phase + f / sr) % 1.0;
            let mut tone = 0.0;
            let mut h = 1.0;
            while h * f < 9000.0 && h < 30.0 {
                tone += (h * p).sin() / h.powf(0.6) * (0.08 + formant_gain(h * f, &formants));
                h += 1.0;
            }
            let creak = 1.0 - 0.35 * (0.5 + 0.5 * (t * 2.0 * PI * 48.0).sin());
            tone * creak * smoothstep(t / 0.012) * (1.0 - smoothstep((u - 0.55) / 0.45))
        })
        .collect();
    normalise(out, 0.5)
}

pub(crate) fn render_hoot_machine(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.62;
    let mut phase = 0.0_f32;
    let formants = [(330.0, 1.0, 0.25), (820.0, 0.45, 0.15), (2300.0, 0.12, 0.1)];
    let band = |x: f32| 2f32.powf((x.log2() * 3.0).round() / 3.0);
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let syllable = |a: f32, b: f32| smoothstep((t - a) / 0.03) * smoothstep((b - t) / 0.06);
            let env = syllable(0.0, 0.24) + syllable(0.32, length) * 0.9;
            let p = phase * 2.0 * PI;
            phase += freq / sr;
            let mut tone = 0.0;
            for (ratio, level) in [(1.0_f32, 1.0_f32), (1.5, 0.45), (2.0, 0.3)] {
                let f = freq * ratio;
                let mut h = 1.0;
                while h * f < 8000.0 && h < 24.0 {
                    tone += (h * p * ratio).sin() / h * formant_gain(band(h * f), &formants) * level;
                    h += 1.0;
                }
            }
            tone * env
        })
        .collect();
    normalise(out, 0.5)
}

pub(crate) fn render_fox(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.48;
    let mut phase = 0.0_f32;
    let mut seed = 0x0f0c_5eed_u32;
    let mut lp = 0.0;
    let mut prev = 0.0;
    let formants = [(1150.0, 1.0, 0.16), (2450.0, 0.75, 0.1), (3500.0, 0.35, 0.08)];
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let u = t / length;
            let pitch = 0.9 + 0.22 * smoothstep(u / 0.2) - 0.32 * smoothstep((u - 0.25) / 0.75);
            let f = freq * pitch;
            let p = phase * 2.0 * PI;
            phase = (phase + f / sr) % 1.0;
            let mut tone = 0.0;
            let mut h = 1.0;
            while h * f < 9000.0 && h < 24.0 {
                tone += (h * p).sin() / h.powf(0.7) * (0.05 + formant_gain(h * f, &formants));
                h += 1.0;
            }
            let n = noise(&mut seed);
            lp += (n - lp) * 0.5;
            let hiss = lp - prev;
            prev = lp;
            let rasp = 1.0 - 0.45 * (0.5 + 0.5 * (t * 2.0 * PI * 71.0).sin());
            let env = smoothstep(t / 0.02) * (1.0 - smoothstep((u - 0.5) / 0.5));
            (tone * rasp + hiss * 2.2 * (0.5 + 0.5 * u)) * env
        })
        .collect();
    normalise(out, 0.5)
}

pub(crate) fn render_shell(freq: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let length = 0.32;
    let mut seed = 0x05e1_1a5c_u32;
    let partials = [(1.0, 1.0, 38.0), (2.31, 0.6, 52.0), (4.07, 0.35, 70.0), (6.73, 0.2, 95.0)];
    let mut lp = 0.0;
    let out: Vec<f32> = (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let n = noise(&mut seed);
            lp += (n - lp) * 0.7;
            let hit = |start: f32, gain: f32| {
                if t < start {
                    return 0.0;
                }
                let a = t - start;
                let ring: f32 = partials.iter().map(|(r, amp, d)| (2.0 * PI * freq * r * a).sin() * amp * (-a * d).exp()).sum();
                let click = lp * (-a * 400.0).exp() * 1.5;
                (ring + click) * gain
            };
            hit(0.0, 1.0) + hit(0.055, 0.35) + hit(0.09, 0.12)
        })
        .collect();
    normalise(out, 0.5)
}
