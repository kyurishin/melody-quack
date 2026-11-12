//! Sound effects, the tape recorder and the echo effect.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const SAMPLE_RATE: u32 = 44100;

pub(crate) const TAPE_LIMIT: f32 = 300.0;

pub(crate) fn mix_tape(events: &[(f64, &[f32], f32)]) -> Vec<f32> {
    let sr = SAMPLE_RATE as f64;
    let end = events.iter().map(|(t, s, _)| (t.max(0.0) * sr) as usize + s.len()).max().unwrap_or(0);
    let mut out = vec![0.0_f32; end + SAMPLE_RATE as usize / 4];
    for (t, samples, volume) in events {
        let at = (t.max(0.0) * sr) as usize;
        for (i, x) in samples.iter().enumerate() {
            out[at + i] += x * volume;
        }
    }
    out.iter_mut().for_each(|x| *x = x.tanh());
    out
}

pub(crate) fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        wav.extend_from_slice(&((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    wav
}

pub(crate) const ECHO_TAPS: [f32; 6] = [0.62, 0.42, 0.29, 0.2, 0.13, 0.08];

pub(crate) fn ring_effect(dry: &[f32]) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let n = dry.len() + (sr * 0.35) as usize;
    let peak = dry.iter().fold(0.0_f32, |m, x| m.max(x.abs())).max(1e-4);
    let mut out = vec![0.0_f32; n];
    let (mut low, mut high_in, mut high) = (0.0_f32, 0.0_f32, 0.0_f32);
    let a_low = 1.0 - (-2.0 * PI * 2200.0 / sr).exp();
    let a_high = (-2.0 * PI * 140.0 / sr).exp();
    for i in 0..n {
        let x = if i < dry.len() { dry[i] } else { 0.0 };
        low += a_low * (x - low);
        high = a_high * (high + low - high_in);
        high_in = low;
        out[i] = high;
    }
    let alls = [347usize, 113, 37];
    for &len in &alls {
        let mut buf = vec![0.0_f32; len];
        let mut pos = 0;
        for v in out.iter_mut() {
            let b = buf[pos];
            let y = -*v * 0.6 + b;
            buf[pos] = *v + b * 0.6;
            pos = (pos + 1) % len;
            *v = y;
        }
    }
    let fade = (sr * 0.15) as usize;
    for k in 0..fade {
        out[n - 1 - k] *= k as f32 / fade as f32;
    }
    let wet_peak = out.iter().fold(0.0_f32, |m, x| m.max(x.abs())).max(1e-4);
    let gain = peak / wet_peak * 0.9;
    out.iter_mut().for_each(|x| *x *= gain);
    out
}

pub(crate) fn ring_tone(freq: f32, partials: &[(f32, f32, f32)], length: f32, attack: f32, gain: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let tone: f32 = partials.iter().map(|(r, a, d)| (2.0 * PI * freq * r * t).sin() * a * (-t * d).exp()).sum();
            tone * smoothstep(t / attack) * smoothstep((length - t) / 0.05) * gain
        })
        .collect()
}

pub(crate) fn noise_burst(length: f32, decay: f32, brightness: f32, seed: u32, gain: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut s = seed;
    let mut lp = 0.0;
    let mut prev = 0.0;
    (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let n = noise(&mut s);
            lp += (n - lp) * brightness;
            let hp = lp - prev;
            prev = lp;
            (hp * 1.5 + lp * 0.3) * (-t * decay).exp() * smoothstep(t / 0.002) * gain
        })
        .collect()
}

pub(crate) fn add_into(mut a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    if b.len() > a.len() {
        a.resize(b.len(), 0.0);
    }
    for (i, v) in b.into_iter().enumerate() {
        a[i] += v;
    }
    a
}

pub(crate) fn render_lid() -> Vec<f32> {
    add_into(
        ring_tone(540.0, &[(1.0, 0.5, 14.0), (2.76, 0.4, 18.0), (5.4, 0.3, 24.0), (8.93, 0.2, 30.0)], 0.5, 0.001, 0.4),
        noise_burst(0.2, 22.0, 0.7, 0x2468_ace1, 0.5),
    )
}

pub(crate) fn render_chop() -> Vec<f32> {
    add_into(ring_tone(1650.0, &[(1.0, 1.0, 70.0), (2.3, 0.4, 90.0)], 0.12, 0.0005, 0.35), noise_burst(0.05, 120.0, 0.9, 0x1111_2222, 0.4))
}

pub(crate) fn render_sizzle() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut s = 0x5a5a_1234;
    let mut prev = 0.0;
    (0..(0.3 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let n = noise(&mut s);
            let hp = n - prev;
            prev = n;
            let env = smoothstep(t / 0.04) * (-t * 9.0).exp();
            hp * env * 0.35
        })
        .collect()
}

pub(crate) fn render_plink() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut phase = 0.0_f32;
    (0..(0.35 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let f = 1400.0 * (1.0 - 0.35 * (1.0 - (-t * 30.0).exp()));
            phase += f / sr;
            (phase * 2.0 * PI).sin() * (-t * 12.0).exp() * smoothstep(t / 0.002) * 0.4
        })
        .collect()
}

pub(crate) fn render_splash() -> Vec<f32> {
    noise_burst(0.5, 7.0, 0.25, 0x7777_1357, 0.6)
}

pub(crate) fn render_marimba(freq: f32) -> Vec<f32> {
    ring_tone(freq, &[(1.0, 1.0, 7.0), (4.0, 0.35, 25.0), (10.0, 0.12, 60.0)], 0.7, 0.002, 0.45)
}

pub(crate) fn render_piano(freq: f32) -> Vec<f32> {
    let body = ring_tone(freq, &[(1.0, 1.0, 1.6), (2.0, 0.55, 2.4), (3.0, 0.3, 3.4), (4.0, 0.16, 5.0), (5.0, 0.08, 7.0)], 1.8, 0.003, 0.42);
    let hammer = noise_burst(0.03, 120.0, 0.6, (freq * 10.0) as u32, 0.06);
    add_into(body, hammer)
}

pub(crate) fn render_bell(freq: f32) -> Vec<f32> {
    ring_tone(freq, &[(1.0, 1.0, 2.5), (2.0, 0.5, 3.0), (2.76, 0.4, 4.0), (5.4, 0.25, 6.0), (8.93, 0.15, 9.0)], 1.6, 0.002, 0.28)
}

pub(crate) fn render_crackle() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut seed = 0x1357_9bdf_u32;
    let length = 0.7;
    let mut out = vec![0.0_f32; (length * sr) as usize];
    for k in 0..7 {
        let start = (noise(&mut seed) * 0.5 + 0.5) * (length - 0.05) * sr;
        let amp = 0.2 + (noise(&mut seed) * 0.5 + 0.5) * 0.5;
        let decay = 300.0 + k as f32 * 40.0;
        let mut lp = 0.0;
        for i in 0..(0.03 * sr) as usize {
            let t = i as f32 / sr;
            let n = noise(&mut seed);
            lp += (n - lp) * 0.6;
            let idx = start as usize + i;
            if idx < out.len() {
                out[idx] += (n - lp) * amp * (-t * decay).exp();
            }
        }
    }
    out
}
