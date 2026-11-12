//! Audio: mixes the generated techno, a DJ's live stream and UI sounds into
//! the output device, and captures a DJ's set (e.g. Serato routed through a
//! loopback device) for streaming.

use crate::music::Synth;
use crate::net::{wall_ms, AudioUplink};
use audiopus::coder::{Decoder, Encoder};
use audiopus::{Application, Bitrate, Channels, SampleRate};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use mq_shared::{Sound, AUDIO_FRAME, AUDIO_SAMPLE_RATE};
use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sfx {
    Blip,
    Select,
    Back,
    Error,
    Door,
    Chime,
    Quack,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MusicCue {
    pub seed: u64,
    pub sound: Sound,
    pub bpm: f32,
    pub start_ms: u64,
}

impl MusicCue {
    pub fn beat_at(&self, server_ms: f64) -> f64 {
        (server_ms - self.start_ms as f64) / 60_000.0 * self.bpm as f64
    }
}

#[derive(Clone)]
pub struct Ctl {
    pub music: Option<MusicCue>,
    pub clock_offset: f64,
    /// 0..1, smoothed in the callback.
    pub music_gain: f32,
    /// 0 = clear, 1 = through a wall.
    pub muffle: f32,
    pub master: f32,
    /// Play the DJ stream instead of the generated set.
    pub live: bool,
    pub live_gain: f32,
}

impl Default for Ctl {
    fn default() -> Self {
        Ctl {
            music: None,
            clock_offset: 0.0,
            music_gain: 0.0,
            muffle: 0.0,
            master: 0.8,
            live: false,
            live_gain: 1.0,
        }
    }
}

struct Shared {
    ctl: Mutex<Ctl>,
    sfx: Mutex<Vec<Sfx>>,
    live: Mutex<VecDeque<f32>>,
    level: AtomicU32,
    live_flowing: AtomicBool,
    in_level: AtomicU32,
}

fn store(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::Relaxed);
}
fn load(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}

pub struct Audio {
    shared: Arc<Shared>,
    _stream: Option<cpal::Stream>,
    packet_tx: Sender<Vec<u8>>,
}

impl Audio {
    pub fn new() -> Audio {
        let shared = Arc::new(Shared {
            ctl: Mutex::new(Ctl::default()),
            sfx: Mutex::new(Vec::new()),
            live: Mutex::new(VecDeque::new()),
            level: AtomicU32::new(0),
            live_flowing: AtomicBool::new(false),
            in_level: AtomicU32::new(0),
        });
        let (packet_tx, packet_rx) = channel::<Vec<u8>>();
        {
            let shared = shared.clone();
            thread::Builder::new()
                .name("opus-decode".into())
                .spawn(move || decode_thread(packet_rx, shared))
                .expect("spawn decoder");
        }
        let stream = match open_output(shared.clone()) {
            Ok(v) => Some(v.0),
            Err(e) => {
                eprintln!("audio: no output ({e}); playing silently");
                None
            }
        };
        Audio { shared, _stream: stream, packet_tx }
    }

    /// Where the network should send incoming live-set packets.
    pub fn packet_sink(&self) -> Sender<Vec<u8>> {
        self.packet_tx.clone()
    }

    pub fn set(&self, f: impl FnOnce(&mut Ctl)) {
        if let Ok(mut c) = self.shared.ctl.lock() {
            f(&mut c);
        }
    }

    pub fn play(&self, s: Sfx) {
        if let Ok(mut q) = self.shared.sfx.lock() {
            if q.len() < 16 {
                q.push(s);
            }
        }
    }

    /// Output loudness, for lights.
    pub fn level(&self) -> f32 {
        load(&self.shared.level)
    }

    pub fn live_flowing(&self) -> bool {
        self.shared.live_flowing.load(Ordering::Relaxed)
    }

    /// Capture loudness while a DJ is streaming.
    pub fn input_level(&self) -> f32 {
        load(&self.shared.in_level)
    }

    pub fn clear_live(&self) {
        if let Ok(mut q) = self.shared.live.lock() {
            q.clear();
        }
    }

    pub fn input_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut v: Vec<String> = host
            .input_devices()
            .map(|it| it.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default();
        v.dedup();
        v
    }

    /// Start capturing `device` (None = default input) and streaming it.
    pub fn start_capture(&self, device: Option<&str>, uplink: AudioUplink) -> Result<Capture, String> {
        let host = cpal::default_host();
        let dev = match device {
            Some(name) => host
                .input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| format!("couldn't find input \"{name}\""))?,
            None => host.default_input_device().ok_or("no audio input on this computer")?,
        };
        let name = dev.name().unwrap_or_else(|_| "input".into());
        let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
        let channels = cfg.channels() as usize;
        let sr = cfg.sample_rate().0;
        let (tx, rx) = channel::<Vec<f32>>();
        let err = |e| eprintln!("capture error: {e}");
        let stream = match cfg.sample_format() {
            cpal::SampleFormat::F32 => dev.build_input_stream(
                &cfg.into(),
                move |data: &[f32], _| {
                    let _ = tx.send(data.to_vec());
                },
                err,
                None,
            ),
            cpal::SampleFormat::I16 => dev.build_input_stream(
                &cfg.into(),
                move |data: &[i16], _| {
                    let _ = tx.send(data.iter().map(|s| *s as f32 / 32768.0).collect());
                },
                err,
                None,
            ),
            cpal::SampleFormat::I32 => dev.build_input_stream(
                &cfg.into(),
                move |data: &[i32], _| {
                    let _ = tx.send(data.iter().map(|s| *s as f32 / 2147483648.0).collect());
                },
                err,
                None,
            ),
            f => return Err(format!("unsupported input format {f:?}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        let shared = self.shared.clone();
        thread::Builder::new()
            .name("opus-encode".into())
            .spawn(move || encode_thread(rx, channels, sr, uplink, shared))
            .map_err(|e| e.to_string())?;
        Ok(Capture { _stream: stream, device: name })
    }
}

pub struct Capture {
    _stream: cpal::Stream,
    pub device: String,
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

struct Voice {
    kind: Sfx,
    t: f32,
}

struct Mixer {
    shared: Arc<Shared>,
    sr: f32,
    channels: usize,
    synth: Option<Synth>,
    ctl: Ctl,
    gain: f32,
    live_gain: f32,
    muffle_state: [f32; 2],
    muffle: f32,
    voices: Vec<Voice>,
    scratch: Vec<f32>,
    live_pos: f64,
    live_buffering: bool,
    level: f32,
}

impl Mixer {
    fn fill(&mut self, out: &mut [f32]) {
        if let Ok(c) = self.shared.ctl.try_lock() {
            self.ctl = c.clone();
        }
        if let Ok(mut q) = self.shared.sfx.try_lock() {
            for k in q.drain(..) {
                self.voices.push(Voice { kind: k, t: 0.0 });
            }
        }
        let frames = out.len() / self.channels;
        self.scratch.clear();
        self.scratch.resize(frames * 2, 0.0);

        // Generated set.
        let want_music = self.ctl.music.is_some() && !(self.ctl.live && self.live_flowing());
        let target = if want_music { self.ctl.music_gain } else { 0.0 };
        if let Some(cue) = self.ctl.music {
            let rebuild = match &self.synth {
                Some(s) => !s.matches(cue.seed, cue.sound),
                None => true,
            };
            if rebuild {
                self.synth = Some(Synth::new(self.sr, cue.seed, cue.sound));
            }
            if self.gain > 0.0005 || target > 0.0 {
                let server_ms = wall_ms() + self.ctl.clock_offset;
                let beat = cue.beat_at(server_ms);
                let bps = cue.bpm as f64 / 60.0 / self.sr as f64;
                let g0 = self.gain;
                // Render at unit gain, then apply the ramp.
                let synth = self.synth.as_mut().unwrap();
                synth.render(&mut self.scratch, beat, bps, 1.0);
                let n = frames.max(1) as f32;
                for (i, f) in self.scratch.chunks_exact_mut(2).enumerate() {
                    let g = g0 + (target - g0) * (i as f32 / n);
                    f[0] *= g;
                    f[1] *= g;
                }
            }
        }
        self.gain += (target - self.gain) * (frames as f32 / (self.sr * 0.4)).min(1.0);

        // Live set.
        let live_target = if self.ctl.live { self.ctl.live_gain } else { 0.0 };
        self.live_gain += (live_target - self.live_gain) * (frames as f32 / (self.sr * 0.2)).min(1.0);
        if let Ok(mut q) = self.shared.live.try_lock() {
            let have = q.len() / 2;
            let rate = AUDIO_SAMPLE_RATE as f64 / self.sr as f64;
            if self.live_buffering {
                if have > (AUDIO_SAMPLE_RATE as usize) / 4 {
                    self.live_buffering = false;
                    self.live_pos = 0.0;
                }
            } else if have < 2 {
                self.live_buffering = true;
            }
            if have > AUDIO_SAMPLE_RATE as usize {
                // Too far behind: skip ahead.
                let drop = have - (AUDIO_SAMPLE_RATE as usize) * 2 / 5;
                q.drain(..drop * 2);
            }
            self.shared.live_flowing.store(!self.live_buffering, Ordering::Relaxed);
            if !self.live_buffering {
                for f in self.scratch.chunks_exact_mut(2) {
                    let i = self.live_pos as usize;
                    if (i + 1) * 2 + 1 >= q.len() {
                        self.live_buffering = true;
                        break;
                    }
                    let fr = (self.live_pos - i as f64) as f32;
                    let l = q[i * 2] * (1.0 - fr) + q[i * 2 + 2] * fr;
                    let r = q[i * 2 + 1] * (1.0 - fr) + q[i * 2 + 3] * fr;
                    f[0] += l * self.live_gain;
                    f[1] += r * self.live_gain;
                    self.live_pos += rate;
                }
                let used = self.live_pos as usize;
                let n = (used * 2).min(q.len());
                q.drain(..n);
                self.live_pos -= used as f64;
            }
        }

        // Muffle (heard through the chapel walls).
        self.muffle += (self.ctl.muffle - self.muffle) * 0.05;
        if self.muffle > 0.01 {
            let fc = 4000.0 * (1.0 - self.muffle) + 160.0;
            let a = 1.0 - (-TAU * fc / self.sr).exp();
            let vol = 1.0 - 0.55 * self.muffle;
            for f in self.scratch.chunks_exact_mut(2) {
                for c in 0..2 {
                    self.muffle_state[c] += (f[c] - self.muffle_state[c]) * a;
                    f[c] = self.muffle_state[c] * vol;
                }
            }
        }

        // UI sounds.
        let dt = 1.0 / self.sr;
        for v in self.voices.iter_mut() {
            for f in self.scratch.chunks_exact_mut(2) {
                let s = sfx_sample(v.kind, v.t) * 0.35;
                f[0] += s;
                f[1] += s;
                v.t += dt;
            }
        }
        self.voices.retain(|v| v.t < sfx_len(v.kind));

        // Out.
        let master = self.ctl.master;
        let mut peak = 0.0f32;
        for (i, f) in out.chunks_exact_mut(self.channels).enumerate() {
            let l = (self.scratch[i * 2] * master).clamp(-1.0, 1.0);
            let r = (self.scratch[i * 2 + 1] * master).clamp(-1.0, 1.0);
            peak = peak.max(l.abs()).max(r.abs());
            match self.channels {
                1 => f[0] = (l + r) * 0.5,
                _ => {
                    f[0] = l;
                    f[1] = r;
                    for x in f.iter_mut().skip(2) {
                        *x = 0.0;
                    }
                }
            }
        }
        self.level = self.level.max(peak) * 0.85 + peak * 0.15;
        self.level *= 0.97;
        store(&self.shared.level, self.level.max(peak * 0.8));
    }

    fn live_flowing(&self) -> bool {
        self.shared.live_flowing.load(Ordering::Relaxed)
    }
}

fn open_output(shared: Arc<Shared>) -> Result<(cpal::Stream, String), String> {
    let host = cpal::default_host();
    let dev = host.default_output_device().ok_or("no output device")?;
    let name = dev.name().unwrap_or_else(|_| "speakers".into());
    let cfg = dev.default_output_config().map_err(|e| e.to_string())?;
    let sr = cfg.sample_rate().0 as f32;
    let channels = cfg.channels() as usize;
    let mut mixer = Mixer {
        shared,
        sr,
        channels,
        synth: None,
        ctl: Ctl::default(),
        gain: 0.0,
        live_gain: 0.0,
        muffle_state: [0.0; 2],
        muffle: 0.0,
        voices: Vec::new(),
        scratch: Vec::new(),
        live_pos: 0.0,
        live_buffering: true,
        level: 0.0,
    };
    let err = |e| eprintln!("audio error: {e}");
    let stream = match cfg.sample_format() {
        cpal::SampleFormat::F32 => dev.build_output_stream(
            &cfg.into(),
            move |data: &mut [f32], _| mixer.fill(data),
            err,
            None,
        ),
        cpal::SampleFormat::I16 => {
            let mut buf = Vec::new();
            dev.build_output_stream(
                &cfg.into(),
                move |data: &mut [i16], _| {
                    buf.resize(data.len(), 0.0);
                    mixer.fill(&mut buf);
                    for (o, s) in data.iter_mut().zip(&buf) {
                        *o = (s * 32767.0) as i16;
                    }
                },
                err,
                None,
            )
        }
        f => return Err(format!("unsupported output format {f:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok((stream, name))
}

// ---------------------------------------------------------------------------
// UI sounds
// ---------------------------------------------------------------------------

fn sfx_len(k: Sfx) -> f32 {
    match k {
        Sfx::Blip => 0.05,
        Sfx::Select => 0.14,
        Sfx::Back => 0.12,
        Sfx::Error => 0.3,
        Sfx::Door => 0.45,
        Sfx::Chime => 1.6,
        Sfx::Quack => 0.22,
    }
}

fn sq(phase: f32) -> f32 {
    if phase.fract() < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn sfx_sample(k: Sfx, t: f32) -> f32 {
    match k {
        Sfx::Blip => sq(t * 880.0) * (1.0 - t / 0.05) * 0.25,
        Sfx::Select => {
            let f = if t < 0.06 { 660.0 } else { 990.0 };
            sq(t * f) * (1.0 - t / 0.14) * 0.25
        }
        Sfx::Back => {
            let f = if t < 0.05 { 520.0 } else { 350.0 };
            sq(t * f) * (1.0 - t / 0.12) * 0.22
        }
        Sfx::Error => sq(t * 140.0) * (1.0 - t / 0.3) * 0.3,
        Sfx::Door => {
            let thud = (TAU * (90.0 - 50.0 * t) * t).sin() * (-t * 9.0).exp();
            thud * 0.6
        }
        Sfx::Chime => {
            // An original little startup arpeggio.
            let notes = [72.0, 79.0, 76.0, 84.0, 83.0];
            let mut s = 0.0;
            for (i, n) in notes.iter().enumerate() {
                let start = i as f32 * 0.12;
                if t >= start {
                    let tt = t - start;
                    let hz = 440.0 * 2f32.powf((n - 69.0) / 12.0);
                    s += ((TAU * hz * tt).sin() + 0.3 * (TAU * hz * 2.0 * tt).sin()) * (-tt * 2.5).exp() * 0.12;
                }
            }
            s
        }
        Sfx::Quack => {
            let f = 380.0 + 120.0 * (t * 40.0).sin() - 300.0 * t;
            let saw = 2.0 * (t * f).fract() - 1.0;
            saw * (1.0 - t / 0.22) * 0.25 * if t < 0.02 { t / 0.02 } else { 1.0 }
        }
    }
}

// ---------------------------------------------------------------------------
// Live set: decode (listeners) and encode (DJ)
// ---------------------------------------------------------------------------

fn decode_thread(rx: Receiver<Vec<u8>>, shared: Arc<Shared>) {
    let Ok(mut dec) = Decoder::new(SampleRate::Hz48000, Channels::Stereo) else {
        return;
    };
    let mut pcm = vec![0f32; AUDIO_FRAME * 2 * 6];
    while let Ok(packet) = rx.recv() {
        let Ok(p) = audiopus::packet::Packet::try_from(packet.as_slice()) else {
            continue;
        };
        let Ok(sig) = audiopus::MutSignals::try_from(&mut pcm[..]) else {
            continue;
        };
        if let Ok(n) = dec.decode_float(Some(p), sig, false) {
            if let Ok(mut q) = shared.live.lock() {
                q.extend(&pcm[..n * 2]);
                // Hard cap: ~3 s.
                let cap = AUDIO_SAMPLE_RATE as usize * 2 * 3;
                if q.len() > cap {
                    let extra = q.len() - cap;
                    q.drain(..extra);
                }
            }
        }
    }
}

fn encode_thread(rx: Receiver<Vec<f32>>, channels: usize, sr: u32, uplink: AudioUplink, shared: Arc<Shared>) {
    let Ok(mut enc) = Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio) else {
        return;
    };
    let _ = enc.set_bitrate(Bitrate::BitsPerSecond(160_000));
    let step = sr as f64 / AUDIO_SAMPLE_RATE as f64;
    let mut src: Vec<[f32; 2]> = Vec::new();
    let mut pos = 0.0f64;
    let mut frame: Vec<f32> = Vec::with_capacity(AUDIO_FRAME * 2);
    let mut out = vec![0u8; 4000];
    let mut level = 0.0f32;
    while let Ok(block) = rx.recv() {
        for f in block.chunks_exact(channels.max(1)) {
            let l = f[0];
            let r = if channels > 1 { f[1] } else { l };
            src.push([l, r]);
        }
        // Linear resample to 48 kHz.
        while pos + 1.0 < src.len() as f64 {
            let i = pos as usize;
            let fr = (pos - i as f64) as f32;
            let a = src[i];
            let b = src[i + 1];
            let l = a[0] + (b[0] - a[0]) * fr;
            let r = a[1] + (b[1] - a[1]) * fr;
            level = level.max(l.abs().max(r.abs()));
            frame.push(l);
            frame.push(r);
            pos += step;
            if frame.len() == AUDIO_FRAME * 2 {
                if let Ok(n) = enc.encode_float(&frame, &mut out) {
                    uplink.send(out[..n].to_vec());
                }
                frame.clear();
                store(&shared.in_level, level);
                level *= 0.6;
            }
        }
        let used = pos as usize;
        src.drain(..used.min(src.len()));
        pos -= used as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_round_trip() {
        let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();
        enc.set_bitrate(Bitrate::BitsPerSecond(160_000)).unwrap();
        let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap();
        let mut out = vec![0u8; 4000];
        let mut pcm = vec![0f32; AUDIO_FRAME * 2];
        let mut energy = 0.0;
        for f in 0..20 {
            let frame: Vec<f32> = (0..AUDIO_FRAME * 2)
                .map(|i| ((f * AUDIO_FRAME + i / 2) as f32 * TAU * 440.0 / 48_000.0).sin() * 0.5)
                .collect();
            let n = enc.encode_float(&frame, &mut out).unwrap();
            assert!(n > 0 && n < 1500, "packet {n} bytes");
            let p = audiopus::packet::Packet::try_from(&out[..n]).unwrap();
            let sig = audiopus::MutSignals::try_from(&mut pcm[..]).unwrap();
            let got = dec.decode_float(Some(p), sig, false).unwrap();
            assert_eq!(got, AUDIO_FRAME);
            if f > 5 {
                energy += pcm.iter().map(|x| x * x).sum::<f32>();
            }
        }
        // A 0.5 sine has mean square 0.125; allow codec loss.
        let ms = energy / (14.0 * AUDIO_FRAME as f32 * 2.0);
        assert!((ms - 0.125).abs() < 0.03, "mean square {ms}");
    }
}
