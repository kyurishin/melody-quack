//! The generated techno that plays in the chapel when no DJ is live.
//!
//! Everything is derived from the rave's seed and the shared server clock, so
//! every player in a rave hears the same track at the same bar.

use mq_shared::{Rng, Sound};
use std::f32::consts::{PI, TAU};

// ---------------------------------------------------------------------------
// Arrangement (shared with the visuals)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub name: &'static str,
    pub kick: bool,
    pub bass: bool,
    pub acid: bool,
    pub hats: bool,
    pub open_hat: bool,
    pub clap: bool,
    pub pad: bool,
    pub arp: bool,
    pub riser: f32,
    pub energy: f32,
    pub drop: bool,
}

pub const CYCLE_BARS: f64 = 64.0;

pub fn section(bar: f64) -> Section {
    let b = bar.rem_euclid(CYCLE_BARS);
    let base = Section {
        name: "INTRO",
        kick: true,
        bass: false,
        acid: false,
        hats: true,
        open_hat: false,
        clap: false,
        pad: false,
        arp: false,
        riser: 0.0,
        energy: 0.35,
        drop: false,
    };
    match b as u32 {
        0..=7 => base,
        8..=15 => Section { name: "GROOVE", bass: true, clap: true, energy: 0.5, ..base },
        16..=23 => Section {
            name: "ACID",
            bass: true,
            clap: true,
            acid: true,
            open_hat: true,
            energy: 0.7,
            ..base
        },
        24..=31 => {
            let rise = ((b - 26.0) / 6.0).clamp(0.0, 1.0) as f32;
            Section {
                name: "BREAKDOWN",
                kick: false,
                hats: b >= 28.0,
                pad: true,
                acid: true,
                riser: rise,
                energy: 0.2 + 0.5 * rise,
                ..base
            }
        }
        32..=47 => Section {
            name: "DROP",
            bass: true,
            clap: true,
            acid: true,
            open_hat: true,
            pad: true,
            arp: true,
            energy: 1.0,
            drop: true,
            ..base
        },
        48..=55 => Section {
            name: "PEAK",
            bass: true,
            clap: true,
            acid: true,
            open_hat: true,
            arp: true,
            energy: 0.9,
            ..base
        },
        _ => Section { name: "COOL DOWN", bass: true, pad: true, energy: 0.55, ..base },
    }
}

// ---------------------------------------------------------------------------
// Song: the seeded patterns
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct AcidStep {
    note: i32,
    on: bool,
    accent: bool,
    slide: bool,
}

struct Song {
    sound: Sound,
    root: i32,
    scale: [i32; 7],
    acid: [AcidStep; 16],
    hats: [f32; 16],
    chords: [[i32; 4]; 4],
    arp: [usize; 16],
    perc: [f32; 16],
    swing: f32,
}

fn midi_hz(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

impl Song {
    fn new(seed: u64, sound: Sound) -> Song {
        let mut r = Rng::new(seed);
        let minor = [0, 2, 3, 5, 7, 8, 10];
        let phrygian = [0, 1, 3, 5, 7, 8, 10];
        let harmonic = [0, 2, 3, 5, 7, 8, 11];
        let scale = match sound {
            Sound::Gothic => *r.pick(&[phrygian, harmonic]),
            Sound::Acid => *r.pick(&[minor, phrygian]),
            Sound::Trance => minor,
        };
        let root = 33 + *r.pick(&[0, -2, 1, 3, -4, 5]);
        let mut acid = [AcidStep::default(); 16];
        for (i, s) in acid.iter_mut().enumerate() {
            s.on = r.chance(if i % 4 == 0 { 0.85 } else { 0.6 });
            let deg = *r.pick(&[0, 0, 0, 2, 4, 6, 7, 3]);
            s.note = scale[deg % 7] + if deg >= 7 { 12 } else { 0 } + if r.chance(0.2) { 12 } else { 0 };
            s.accent = r.chance(0.3);
            s.slide = r.chance(0.22);
        }
        let mut hats = [0.0; 16];
        for (i, h) in hats.iter_mut().enumerate() {
            *h = match i % 4 {
                2 => 1.0,
                0 => 0.35,
                _ => r.range(0.25, 0.7),
            };
        }
        // Progressions as scale degrees.
        let progs: &[[usize; 4]] = match sound {
            Sound::Gothic => &[[0, 5, 3, 4], [0, 1, 0, 6], [0, 5, 6, 4]],
            Sound::Trance => &[[0, 5, 2, 6], [0, 3, 5, 4], [5, 3, 0, 4]],
            Sound::Acid => &[[0, 0, 5, 6], [0, 3, 0, 4]],
        };
        let prog = *r.pick(progs);
        let mut chords = [[0; 4]; 4];
        for (c, deg) in chords.iter_mut().zip(prog) {
            let tone = |d: usize| scale[d % 7] + 12 * (d / 7) as i32;
            *c = [tone(deg), tone(deg + 2), tone(deg + 4), tone(deg + 7)];
        }
        let shapes: [[usize; 16]; 3] = [
            [0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0, 2, 1, 3],
            [0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 3, 2, 1, 0],
            [0, 0, 1, 2, 3, 3, 2, 1, 0, 1, 2, 3, 2, 3, 1, 2],
        ];
        let arp = *r.pick(&shapes);
        let mut perc = [0.0; 16];
        for p in perc.iter_mut() {
            if r.chance(0.18) {
                *p = r.range(0.4, 1.0);
            }
        }
        Song {
            sound,
            root,
            scale,
            acid,
            hats,
            chords,
            arp,
            perc,
            swing: if sound == Sound::Trance { 0.0 } else { r.range(0.0, 0.08) },
        }
    }
}

// ---------------------------------------------------------------------------
// DSP building blocks
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct Svf {
    ic1: f32,
    ic2: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
}

impl Svf {
    fn set(&mut self, fc: f32, q: f32, sr: f32) {
        let g = (PI * fc.clamp(20.0, sr * 0.45) / sr).tan();
        self.k = 1.0 / q.max(0.1);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }
    /// Returns (low, band, high).
    fn run(&mut self, v0: f32) -> (f32, f32, f32) {
        let v3 = v0 - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v2, v1, v0 - self.k * v1 - v2)
    }
}

struct Comb {
    buf: Vec<f32>,
    i: usize,
    fb: f32,
    damp: f32,
    store: f32,
}

impl Comb {
    fn new(len: usize, fb: f32) -> Comb {
        Comb { buf: vec![0.0; len.max(1)], i: 0, fb, damp: 0.3, store: 0.0 }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.buf[self.i];
        self.store = y * (1.0 - self.damp) + self.store * self.damp;
        self.buf[self.i] = x + self.store * self.fb;
        self.i = (self.i + 1) % self.buf.len();
        y
    }
}

struct Allpass {
    buf: Vec<f32>,
    i: usize,
}

impl Allpass {
    fn new(len: usize) -> Allpass {
        Allpass { buf: vec![0.0; len.max(1)], i: 0 }
    }
    fn run(&mut self, x: f32) -> f32 {
        let b = self.buf[self.i];
        let y = b - x;
        self.buf[self.i] = x + b * 0.5;
        self.i = (self.i + 1) % self.buf.len();
        y
    }
}

/// A small Freeverb: big and stony, for the chapel.
struct Reverb {
    combs: [Vec<Comb>; 2],
    aps: [Vec<Allpass>; 2],
}

impl Reverb {
    fn new(sr: f32, size: f32) -> Reverb {
        let s = sr / 44100.0;
        let lens = [1116, 1188, 1277, 1356, 1422, 1491];
        let mk = |spread: usize| -> Vec<Comb> {
            lens.iter()
                .map(|l| Comb::new(((l + spread) as f32 * s) as usize, size))
                .collect()
        };
        let ap = |spread: usize| -> Vec<Allpass> {
            [556, 441, 341].iter().map(|l| Allpass::new(((l + spread) as f32 * s) as usize)).collect()
        };
        Reverb { combs: [mk(0), mk(23)], aps: [ap(0), ap(23)] }
    }
    fn run(&mut self, x: f32) -> (f32, f32) {
        let mut out = [0.0f32; 2];
        for ch in 0..2 {
            let mut acc = 0.0;
            for c in self.combs[ch].iter_mut() {
                acc += c.run(x);
            }
            acc *= 0.15;
            for a in self.aps[ch].iter_mut() {
                acc = a.run(acc);
            }
            out[ch] = acc;
        }
        (out[0], out[1])
    }
}

struct Delay {
    buf: Vec<f32>,
    i: usize,
}

impl Delay {
    fn new(max: usize) -> Delay {
        Delay { buf: vec![0.0; max.max(1)], i: 0 }
    }
    fn run(&mut self, x: f32, len: usize, fb: f32) -> f32 {
        let n = self.buf.len();
        let len = len.clamp(1, n - 1);
        let y = self.buf[(self.i + n - len) % n];
        self.buf[self.i] = x + y * fb;
        self.i = (self.i + 1) % n;
        y
    }
}

// ---------------------------------------------------------------------------
// Synth
// ---------------------------------------------------------------------------

pub struct Synth {
    sr: f32,
    song: Song,
    seed: u64,
    beat: f64,
    last_step: i64,
    noise: u32,
    // Voice clocks: seconds since the last trigger.
    t_kick: f32,
    t_clap: f32,
    t_hat: f32,
    hat_vel: f32,
    t_open: f32,
    t_perc: f32,
    perc_vel: f32,
    t_bass: f32,
    bass_hz: f32,
    // Acid
    t_acid: f32,
    acid_phase: f32,
    acid_hz: f32,
    acid_target: f32,
    acid_accent: bool,
    acid_gate: bool,
    acid_f: Svf,
    // Pad / arp
    pad_phase: [f32; 8],
    pad_level: f32,
    pad_f: Svf,
    arp_phase: f32,
    arp_hz: f32,
    t_arp: f32,
    bass_phase: f32,
    // FX
    hat_f: Svf,
    clap_f: Svf,
    riser_f: Svf,
    reverb: Reverb,
    delay: Delay,
    duck: f32,
    ctr: u32,
    sec: Section,
}

impl Synth {
    pub fn new(sr: f32, seed: u64, sound: Sound) -> Synth {
        Synth {
            sr,
            song: Song::new(seed, sound),
            seed,
            beat: 0.0,
            last_step: i64::MIN,
            noise: 0x1234_5678,
            t_kick: 9.0,
            t_clap: 9.0,
            t_hat: 9.0,
            hat_vel: 0.0,
            t_open: 9.0,
            t_perc: 9.0,
            perc_vel: 0.0,
            t_bass: 9.0,
            bass_hz: 55.0,
            t_acid: 9.0,
            acid_phase: 0.0,
            acid_hz: 110.0,
            acid_target: 110.0,
            acid_accent: false,
            acid_gate: false,
            acid_f: Svf::default(),
            pad_phase: [0.0; 8],
            pad_level: 0.0,
            pad_f: Svf::default(),
            arp_phase: 0.0,
            arp_hz: 440.0,
            t_arp: 9.0,
            bass_phase: 0.0,
            hat_f: Svf::default(),
            clap_f: Svf::default(),
            riser_f: Svf::default(),
            reverb: Reverb::new(sr, if sound == Sound::Gothic { 0.86 } else { 0.8 }),
            delay: Delay::new((sr * 1.5) as usize),
            duck: 0.0,
            ctr: 0,
            sec: section(0.0),
        }
    }

    pub fn matches(&self, seed: u64, sound: Sound) -> bool {
        self.seed == seed && self.song.sound == sound
    }

    fn rand(&mut self) -> f32 {
        // xorshift32
        let mut x = self.noise;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.noise = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn trigger(&mut self, step: i64) {
        let s16 = step.rem_euclid(16) as usize;
        let bar = step.div_euclid(16) as f64;
        self.sec = section(bar);
        let sec = self.sec;
        let song = &self.song;
        let in_bar_of_cycle = bar.rem_euclid(CYCLE_BARS);
        // Fill: drop the kick for the last beat before a section change.
        let fill = (in_bar_of_cycle as u32 + 1) % 8 == 0 && s16 >= 12;

        if sec.kick && s16 % 4 == 0 && !fill {
            self.t_kick = 0.0;
        }
        if sec.clap && (s16 == 4 || s16 == 12) {
            self.t_clap = 0.0;
        }
        if fill && sec.energy > 0.4 && s16 % 2 == 0 {
            self.t_clap = 0.0; // snare roll
        }
        if sec.hats && song.hats[s16] > 0.0 {
            self.t_hat = 0.0;
            self.hat_vel = song.hats[s16] * if sec.drop { 1.0 } else { 0.8 };
        }
        if sec.open_hat && s16 % 4 == 2 {
            self.t_open = 0.0;
        }
        if sec.energy > 0.45 && song.perc[s16] > 0.0 {
            self.t_perc = 0.0;
            self.perc_vel = song.perc[s16];
        }
        let chord = song.chords[(bar.rem_euclid(8.0) / 2.0) as usize % 4];
        if sec.bass {
            let play = match song.sound {
                Sound::Trance => s16 % 4 == 2,
                _ => s16 % 4 != 0,
            };
            if play {
                self.t_bass = 0.0;
                self.bass_hz = midi_hz((song.root + chord[0]) as f32);
            }
        }
        if sec.acid {
            let a = song.acid[s16];
            if a.on {
                let hz = midi_hz((song.root + 12 + a.note) as f32);
                self.acid_target = hz;
                if !(a.slide && self.acid_gate) {
                    self.acid_hz = hz;
                    self.t_acid = 0.0;
                }
                self.acid_accent = a.accent;
                self.acid_gate = true;
            } else {
                self.acid_gate = false;
            }
        } else {
            self.acid_gate = false;
        }
        if sec.arp {
            let tone = chord[song.arp[s16]];
            self.arp_hz = midi_hz((song.root + 36 + tone) as f32);
            self.t_arp = 0.0;
        }
        let _ = song.scale;
    }

    /// Fill `out` (interleaved stereo) starting at `beat`, advancing
    /// `beats_per_sample` each frame. `beat` is where the shared clock says we
    /// should be; small drift is ignored, big jumps resync.
    pub fn render(&mut self, out: &mut [f32], beat: f64, beats_per_sample: f64, gain: f32) {
        if (self.beat - beat).abs() > 0.06 {
            self.beat = beat;
            self.last_step = (beat * 4.0).floor() as i64 - 1;
        }
        let sr = self.sr;
        let dt = 1.0 / sr;
        let bps = beats_per_sample;
        let sec_per_step = (bps * sr as f64).recip() as f32 / 4.0;
        for frame in out.chunks_exact_mut(2) {
            // Swing pushes odd 16ths late.
            let raw_step = self.beat * 4.0;
            let step_i = raw_step.floor() as i64;
            let frac = (raw_step - step_i as f64) as f32;
            let swung = if step_i.rem_euclid(2) == 1 { frac >= self.song.swing } else { true };
            if step_i != self.last_step && swung {
                self.last_step = step_i;
                self.trigger(step_i);
            }
            let sec = self.sec;

            // Coefficient updates every 32 samples.
            if self.ctr % 32 == 0 {
                let acid_env = (-self.t_acid * if self.acid_accent { 6.0 } else { 9.0 }).exp();
                let lfo = (self.beat as f32 * TAU / 64.0).sin() * 0.5 + 0.5;
                let cutoff = 180.0 + 2600.0 * acid_env * (0.4 + 0.6 * lfo) + if sec.drop { 600.0 } else { 0.0 };
                self.acid_f.set(cutoff, if self.acid_accent { 9.0 } else { 6.0 }, sr);
                self.hat_f.set(7500.0, 0.7, sr);
                self.clap_f.set(1400.0, 1.2, sr);
                let pad_cut = 500.0 + 2200.0 * sec.energy + 400.0 * lfo;
                self.pad_f.set(pad_cut, 0.8, sr);
                self.riser_f.set(300.0 + 9000.0 * sec.riser * sec.riser, 2.0, sr);
                let target = if sec.pad { 1.0 } else { 0.0 };
                self.pad_level += (target - self.pad_level) * 0.01;
            }
            self.ctr = self.ctr.wrapping_add(1);

            // Kick: pitch-swept sine with a click.
            let mut kick = 0.0;
            if self.t_kick < 0.5 {
                let t = self.t_kick;
                let ph = TAU * (48.0 * t + 160.0 / 28.0 * (1.0 - (-28.0 * t).exp()));
                let env = (-t * 7.5).exp();
                kick = (ph.sin() * env * 1.4).tanh() + (-t * 300.0).exp() * 0.25;
                self.duck = env;
            } else {
                self.duck *= 0.999;
            }
            let pump = 1.0 - 0.75 * self.duck;

            // Clap: three quick noise bursts, band-passed.
            let mut clap = 0.0;
            if self.t_clap < 0.4 {
                let t = self.t_clap;
                let bursts = if t < 0.03 { ((t * 100.0) % 1.0 < 0.5) as i32 as f32 } else { 1.0 };
                let n = self.rand();
                let (_, bp, _) = self.clap_f.run(n);
                clap = bp * bursts * (-(t - 0.02).max(0.0) * 18.0).exp() * 1.6;
            }

            // Hats.
            let mut hats = 0.0;
            if self.t_hat < 0.12 || self.t_open < 0.5 {
                let n = self.rand();
                let (_, _, hp) = self.hat_f.run(n);
                hats = hp
                    * (self.hat_vel * (-self.t_hat * 55.0).exp() * 0.35
                        + (-self.t_open * 9.0).exp() * 0.22 * if sec.open_hat { 1.0 } else { 0.0 });
            }

            // Perc: little tuned tom/rim.
            let mut perc = 0.0;
            if self.t_perc < 0.25 {
                let t = self.t_perc;
                perc = (TAU * (380.0 + 300.0 * (-t * 40.0).exp()) * t).sin()
                    * (-t * 22.0).exp()
                    * self.perc_vel
                    * 0.18;
            }

            // Bass.
            let mut bass = 0.0;
            if self.t_bass < 0.4 {
                self.bass_phase = (self.bass_phase + self.bass_hz * dt) % 1.0;
                let p = self.bass_phase;
                let s = (p * TAU).sin() + 0.3 * (2.0 * p - 1.0);
                let len = sec_per_step * 1.6;
                let env = if self.t_bass < len { 1.0 } else { (-(self.t_bass - len) * 40.0).exp() };
                bass = s * env * 0.42 * (1.0 - (-self.t_bass * 300.0).exp());
            }

            // Acid: saw into a screaming resonant low-pass.
            let mut acid = 0.0;
            if self.acid_gate || self.t_acid < 0.3 {
                self.acid_hz += (self.acid_target - self.acid_hz) * 0.004;
                self.acid_phase = (self.acid_phase + self.acid_hz * dt) % 1.0;
                let saw = 2.0 * self.acid_phase - 1.0;
                let (lp, _, _) = self.acid_f.run(saw);
                let gate = if self.acid_gate { 1.0 } else { (-self.t_acid * 30.0).exp() };
                let acc = if self.acid_accent { 1.3 } else { 0.9 };
                let level = if sec.name == "BREAKDOWN" { 0.12 } else { 0.22 };
                acid = (lp * 2.2).tanh() * gate * acc * level;
            }

            // Pad: detuned saws (organ-ish squares for gothic).
            let mut pad = 0.0;
            if self.pad_level > 0.001 {
                let bar = (self.beat / 4.0).rem_euclid(8.0);
                let chord = self.song.chords[(bar / 2.0) as usize % 4];
                let root = self.song.root + 24;
                for v in 0..8 {
                    let tone = chord[v / 2];
                    let det = if v % 2 == 0 { 0.997 } else { 1.004 };
                    let hz = midi_hz((root + tone) as f32) * det;
                    self.pad_phase[v] = (self.pad_phase[v] + hz * dt) % 1.0;
                    let ph = self.pad_phase[v];
                    pad += match self.song.sound {
                        Sound::Gothic => {
                            if ph < 0.5 {
                                0.6
                            } else {
                                -0.6
                            }
                        }
                        _ => 2.0 * ph - 1.0,
                    };
                }
                let (lp, _, _) = self.pad_f.run(pad * 0.08);
                pad = lp * self.pad_level * if sec.drop { pump } else { 1.0 };
            }

            // Arp pluck.
            let mut arp = 0.0;
            if self.t_arp < 0.3 {
                self.arp_phase = (self.arp_phase + self.arp_hz * dt) % 1.0;
                let sq = if self.arp_phase < 0.35 { 1.0 } else { -1.0 };
                arp = sq * (-self.t_arp * 18.0).exp() * 0.06;
            }

            // Riser: noise sweeping up through a band-pass.
            let mut riser = 0.0;
            if sec.riser > 0.0 {
                let n = self.rand();
                let (_, bp, _) = self.riser_f.run(n);
                riser = bp * sec.riser * 0.25;
            }

            // Mix.
            let dry = kick * 0.9 + clap * 0.45 + hats + perc + (bass + acid) * pump + pad + arp + riser;
            // Dotted-eighth echo: three 16ths.
            let dl = (3.0 / (4.0 * bps)) as usize;
            let delayed = self.delay.run(arp + acid * 0.3 + clap * 0.1, dl.max(1), 0.38);
            let (rl, rr) = self.reverb.run(clap * 0.5 + pad * 0.6 + arp * 0.8 + perc + delayed * 0.5 + riser);
            let l = dry + delayed * 0.35 + rl * 0.55;
            let r = dry + delayed * 0.3 + rr * 0.55;
            frame[0] += (l * 0.8).tanh() * gain;
            frame[1] += (r * 0.8).tanh() * gain;

            self.t_kick += dt;
            self.t_clap += dt;
            self.t_hat += dt;
            self.t_open += dt;
            self.t_perc += dt;
            self.t_bass += dt;
            self.t_acid += dt;
            self.t_arp += dt;
            self.beat += bps;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(sound: Sound, bar: f64) -> (f32, f32) {
        let sr = 48_000.0;
        let mut s = Synth::new(sr, 1234, sound);
        let bpm = sound.default_bpm() as f64;
        let bps = bpm / 60.0 / sr as f64;
        let mut buf = vec![0.0f32; 48_000 * 2 * 2]; // two seconds
        let mut beat = bar * 4.0;
        for chunk in buf.chunks_mut(1024) {
            s.render(chunk, beat, bps, 1.0);
            beat += (chunk.len() / 2) as f64 * bps;
        }
        assert!(buf.iter().all(|x| x.is_finite()), "NaN/inf in {sound:?} at bar {bar}");
        let peak = buf.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        let rms = (buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32).sqrt();
        (peak, rms)
    }

    #[test]
    fn every_section_is_audible_and_safe() {
        for sound in Sound::ALL {
            for bar in [0.0, 10.0, 18.0, 26.0, 34.0, 50.0, 58.0] {
                let (peak, rms) = stats(sound, bar);
                assert!(peak <= 1.0, "{sound:?} bar {bar} clips: {peak}");
                assert!(rms > 0.02, "{sound:?} bar {bar} too quiet: {rms}");
            }
        }
    }

    #[test]
    fn same_seed_same_track() {
        let render = || {
            let mut s = Synth::new(48_000.0, 7, Sound::Acid);
            let mut buf = vec![0.0f32; 9600];
            s.render(&mut buf, 40.0, 136.0 / 60.0 / 48_000.0, 1.0);
            buf
        };
        assert_eq!(render(), render());
    }
}
