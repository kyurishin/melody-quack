use macroquad::prelude::*;
use std::f32::consts::PI;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
mod tempo;
mod fish;
mod emoji;
mod home;
mod books;
mod mushroom;

#[cfg(not(target_arch = "wasm32"))]
mod audio {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};

    struct Voice {
        samples: Arc<Vec<f32>>,
        position: f32,
        volume: f32,
    }

    #[derive(Clone)]
    pub struct Sound(Arc<Vec<f32>>, Option<Arc<Vec<f32>>>, u16);

    pub struct Audio {
        voices: Arc<Mutex<Vec<Voice>>>,
        _stream: Option<cpal::Stream>,
        ring: std::cell::Cell<bool>,
        gap: std::cell::Cell<f32>,
        tape: std::cell::RefCell<Option<(f64, Vec<(f64, Arc<Vec<f32>>, f32)>)>>,
        mix: std::cell::Cell<[f32; 5]>,
        boost: std::cell::Cell<bool>,
        bank: std::cell::RefCell<Vec<Sound>>,
        jam: std::cell::RefCell<Option<(f64, Vec<(f32, u16, f32)>)>>,
    }

    impl Audio {
        pub fn new() -> Audio {
            let voices = Arc::new(Mutex::new(Vec::new()));
            let stream = start(voices.clone());
            Audio {
                voices,
                _stream: stream,
                ring: std::cell::Cell::new(false),
                gap: std::cell::Cell::new(0.3),
                tape: std::cell::RefCell::new(None),
                mix: std::cell::Cell::new([1.0; 5]),
                boost: std::cell::Cell::new(false),
                bank: std::cell::RefCell::new(Vec::new()),
                jam: std::cell::RefCell::new(None),
            }
        }

        pub fn start_jam(&self) {
            *self.jam.borrow_mut() = Some((macroquad::time::get_time(), Vec::new()));
        }

        pub fn stop_jam(&self) -> Vec<(f32, u16, f32)> {
            self.jam.borrow_mut().take().map(|(_, events)| events).unwrap_or_default()
        }

        pub fn play_id(&self, id: u16, volume: f32) {
            let sound = self.bank.borrow().get(id as usize).cloned();
            if let Some(sound) = sound {
                let mix = self.mix.get();
                self.play(&sound, volume * mix[4]);
            }
        }

        pub fn set_mix(&self, mix: [f32; 5]) {
            self.mix.set(mix);
        }

        pub fn set_boost(&self, on: bool) {
            self.boost.set(on);
        }

        fn echo_scale(&self) -> f32 {
            if self.boost.get() { 1.45 } else { 1.0 }
        }

        pub fn play_ch(&self, channel: usize, sound: &Sound, volume: f32) {
            let mix = self.mix.get();
            let level = mix[channel.min(3)] * mix[4];
            if channel <= 2 {
                if let Some((start, events)) = self.jam.borrow_mut().as_mut() {
                    if events.len() < 20000 {
                        events.push(((macroquad::time::get_time() - *start) as f32, Self::id_of(sound), volume * mix[channel.min(3)]));
                    }
                }
            }
            if level > 0.001 {
                self.play(sound, volume * level);
            }
        }

        pub fn start_tape(&self) {
            *self.tape.borrow_mut() = Some((macroquad::time::get_time(), Vec::new()));
        }

        pub fn tape_seconds(&self) -> Option<f32> {
            self.tape.borrow().as_ref().map(|(start, _)| (macroquad::time::get_time() - start) as f32)
        }

        pub fn stop_tape(&self) -> Option<Vec<f32>> {
            let (start, events) = self.tape.borrow_mut().take()?;
            let list: Vec<(f64, &[f32], f32)> = events.iter().map(|(t, s, v)| (t - start, s.as_slice(), *v)).collect();
            Some(crate::mix_tape(&list))
        }

        fn log(&self, at: f64, samples: &Arc<Vec<f32>>, volume: f32) {
            if let Some((_, events)) = self.tape.borrow_mut().as_mut() {
                events.push((at, samples.clone(), volume));
            }
        }

        pub fn load(&self, samples: Vec<f32>) -> Sound {
            self.keep(Sound(Arc::new(samples), None, self.next_id()))
        }

        pub fn load_ringing(&self, samples: Vec<f32>) -> Sound {
            let wet = crate::ring_effect(&samples);
            self.keep(Sound(Arc::new(samples), Some(Arc::new(wet)), self.next_id()))
        }

        fn next_id(&self) -> u16 {
            self.bank.borrow().len() as u16
        }

        fn keep(&self, sound: Sound) -> Sound {
            self.bank.borrow_mut().push(sound.clone());
            sound
        }

        fn id_of(sound: &Sound) -> u16 {
            sound.2
        }

        pub fn set_ring(&self, on: bool) {
            self.ring.set(on);
        }

        pub fn set_echo_gap(&self, seconds: f32) {
            self.gap.set(seconds);
        }

        pub fn tick(&self) {}

        pub fn is_loaded(&self, _sound: &Sound) -> bool {
            true
        }

        pub fn play_private(&self, sound: &Sound, volume: f32) {
            let mix = self.mix.get();
            if let Ok(mut list) = self.voices.lock() {
                list.push(Voice { samples: sound.0.clone(), position: 0.0, volume: volume * mix[2] * mix[4] });
            }
        }

        pub fn play(&self, sound: &Sound, volume: f32) {
            let now = macroquad::time::get_time();
            self.log(now, &sound.0, volume);
            if let (Some(wet), true) = (&sound.1, self.ring.get()) {
                for (k, gain) in crate::ECHO_TAPS.iter().enumerate() {
                    self.log(now + (k + 1) as f64 * self.gap.get() as f64, wet, volume * (gain * self.echo_scale()).min(1.0));
                }
            }
            if let Ok(mut list) = self.voices.lock() {
                list.push(Voice {
                    samples: sound.0.clone(),
                    position: 0.0,
                    volume,
                });
                if let (Some(wet), true) = (&sound.1, self.ring.get()) {
                    for (k, gain) in crate::ECHO_TAPS.iter().enumerate() {
                        list.push(Voice {
                            samples: wet.clone(),
                            position: -((k + 1) as f32 * self.gap.get() * crate::SAMPLE_RATE as f32),
                            volume: volume * (gain * self.echo_scale()).min(1.0),
                        });
                    }
                }
            }
        }
    }

    fn start(voices: Arc<Mutex<Vec<Voice>>>) -> Option<cpal::Stream> {
        let device = cpal::default_host().default_output_device()?;
        let config = device.default_output_config().ok()?;
        if config.sample_format() != cpal::SampleFormat::F32 {
            return None;
        }
        let channels = config.channels() as usize;
        let step = crate::SAMPLE_RATE as f32 / config.sample_rate() as f32;
        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _| {
                    for s in data.iter_mut() {
                        *s = 0.0;
                    }
                    if let Ok(mut list) = voices.lock() {
                        for v in list.iter_mut() {
                            for frame in data.chunks_mut(channels) {
                                if v.position < 0.0 {
                                    v.position += step;
                                    continue;
                                }
                                let i = v.position as usize;
                                if i + 1 >= v.samples.len() {
                                    break;
                                }
                                let f = v.position - i as f32;
                                let x = (v.samples[i] * (1.0 - f) + v.samples[i + 1] * f) * v.volume;
                                for out in frame.iter_mut() {
                                    *out += x;
                                }
                                v.position += step;
                            }
                        }
                        list.retain(|v| v.position < 0.0 || (v.position as usize) + 1 < v.samples.len());
                    }
                    for s in data.iter_mut() {
                        *s = s.tanh();
                    }
                },
                |e| eprintln!("audio error: {e}"),
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(stream)
    }
}

#[cfg(target_arch = "wasm32")]
mod audio {
    use crate::SAMPLE_RATE;

    #[link(wasm_import_module = "env")]
    extern "C" {
        fn audio_init();
        fn audio_add_buffer(content: *const u8, content_len: u32) -> u32;
        fn audio_play_buffer(buffer: u32, volume: f32, repeat: bool) -> u32;
        fn audio_source_is_loaded(buffer: u32) -> bool;
    }

    #[no_mangle]
    pub extern "C" fn macroquad_audio_crate_version() -> u32 {
        1
    }

    use std::rc::Rc;

    #[derive(Clone)]
    pub struct Sound(u32, Option<u32>, Rc<Vec<f32>>, Option<Rc<Vec<f32>>>, u16);

    pub struct Audio {
        ring: std::cell::Cell<bool>,
        gap: std::cell::Cell<f32>,
        pending: std::cell::RefCell<Vec<(f64, u32, f32)>>,
        tape: std::cell::RefCell<Option<(f64, Vec<(f64, Rc<Vec<f32>>, f32)>)>>,
        mix: std::cell::Cell<[f32; 5]>,
        boost: std::cell::Cell<bool>,
        bank: std::cell::RefCell<Vec<Sound>>,
        jam: std::cell::RefCell<Option<(f64, Vec<(f32, u16, f32)>)>>,
    }

    fn add(samples: &[f32]) -> u32 {
        let wav = to_wav(samples);
        unsafe { audio_add_buffer(wav.as_ptr(), wav.len() as u32) }
    }

    impl Audio {
        pub fn new() -> Audio {
            unsafe { audio_init() };
            Audio {
                ring: std::cell::Cell::new(false),
                gap: std::cell::Cell::new(0.3),
                pending: std::cell::RefCell::new(Vec::new()),
                tape: std::cell::RefCell::new(None),
                mix: std::cell::Cell::new([1.0; 5]),
                boost: std::cell::Cell::new(false),
                bank: std::cell::RefCell::new(Vec::new()),
                jam: std::cell::RefCell::new(None),
            }
        }

        pub fn start_jam(&self) {
            *self.jam.borrow_mut() = Some((macroquad::time::get_time(), Vec::new()));
        }

        pub fn stop_jam(&self) -> Vec<(f32, u16, f32)> {
            self.jam.borrow_mut().take().map(|(_, events)| events).unwrap_or_default()
        }

        pub fn play_id(&self, id: u16, volume: f32) {
            let sound = self.bank.borrow().get(id as usize).cloned();
            if let Some(sound) = sound {
                let mix = self.mix.get();
                self.play(&sound, volume * mix[4]);
            }
        }

        pub fn set_mix(&self, mix: [f32; 5]) {
            self.mix.set(mix);
        }

        pub fn set_boost(&self, on: bool) {
            self.boost.set(on);
        }

        fn echo_scale(&self) -> f32 {
            if self.boost.get() { 1.45 } else { 1.0 }
        }

        pub fn play_ch(&self, channel: usize, sound: &Sound, volume: f32) {
            let mix = self.mix.get();
            let level = mix[channel.min(3)] * mix[4];
            if channel <= 2 {
                if let Some((start, events)) = self.jam.borrow_mut().as_mut() {
                    if events.len() < 20000 {
                        events.push(((macroquad::time::get_time() - *start) as f32, Self::id_of(sound), volume * mix[channel.min(3)]));
                    }
                }
            }
            if level > 0.001 {
                self.play(sound, volume * level);
            }
        }

        pub fn start_tape(&self) {
            *self.tape.borrow_mut() = Some((macroquad::time::get_time(), Vec::new()));
        }

        pub fn tape_seconds(&self) -> Option<f32> {
            self.tape.borrow().as_ref().map(|(start, _)| (macroquad::time::get_time() - start) as f32)
        }

        pub fn stop_tape(&self) -> Option<Vec<f32>> {
            let (start, events) = self.tape.borrow_mut().take()?;
            let list: Vec<(f64, &[f32], f32)> = events.iter().map(|(t, s, v)| (t - start, s.as_slice(), *v)).collect();
            Some(crate::mix_tape(&list))
        }

        fn log(&self, at: f64, samples: &Rc<Vec<f32>>, volume: f32) {
            if let Some((_, events)) = self.tape.borrow_mut().as_mut() {
                events.push((at, samples.clone(), volume));
            }
        }

        pub fn load(&self, samples: Vec<f32>) -> Sound {
            let id = self.next_id();
            self.keep(Sound(add(&samples), None, Rc::new(samples), None, id))
        }

        pub fn load_ringing(&self, samples: Vec<f32>) -> Sound {
            let wet = crate::ring_effect(&samples);
            let id = self.next_id();
            self.keep(Sound(add(&samples), Some(add(&wet)), Rc::new(samples), Some(Rc::new(wet)), id))
        }

        fn next_id(&self) -> u16 {
            self.bank.borrow().len() as u16
        }

        fn keep(&self, sound: Sound) -> Sound {
            self.bank.borrow_mut().push(sound.clone());
            sound
        }

        fn id_of(sound: &Sound) -> u16 {
            sound.4
        }

        pub fn set_ring(&self, on: bool) {
            self.ring.set(on);
        }

        pub fn set_echo_gap(&self, seconds: f32) {
            self.gap.set(seconds);
        }

        pub fn tick(&self) {
            let now = macroquad::time::get_time();
            let mut list = self.pending.borrow_mut();
            for &(due, id, volume) in list.iter() {
                if due <= now {
                    unsafe { audio_play_buffer(id, volume, false) };
                }
            }
            list.retain(|&(due, _, _)| due > now);
        }

        pub fn is_loaded(&self, sound: &Sound) -> bool {
            let dry = unsafe { audio_source_is_loaded(sound.0) };
            let wet = match sound.1 {
                Some(w) => unsafe { audio_source_is_loaded(w) },
                None => true,
            };
            dry && wet
        }

        pub fn play_private(&self, sound: &Sound, volume: f32) {
            let mix = self.mix.get();
            unsafe { audio_play_buffer(sound.0, volume * mix[2] * mix[4], false) };
        }

        pub fn play(&self, sound: &Sound, volume: f32) {
            unsafe { audio_play_buffer(sound.0, volume, false) };
            let now = macroquad::time::get_time();
            self.log(now, &sound.2, volume);
            if let (Some(wet), Some(wet_samples), true) = (sound.1, &sound.3, self.ring.get()) {
                let mut list = self.pending.borrow_mut();
                for (k, gain) in crate::ECHO_TAPS.iter().enumerate() {
                    let at = now + (k + 1) as f64 * self.gap.get() as f64;
                    list.push((at, wet, volume * (gain * self.echo_scale()).min(1.0)));
                    self.log(at, wet_samples, volume * (gain * self.echo_scale()).min(1.0));
                }
            }
        }
    }

    fn to_wav(samples: &[f32]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut wav = Vec::with_capacity(44 + data_len as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
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
            let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            wav.extend_from_slice(&v.to_le_bytes());
        }
        wav
    }
}


#[cfg(not(target_arch = "wasm32"))]
mod mic {
    use crate::tempo::OnsetDetector;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};

    pub struct Mic {
        pub detector: Arc<Mutex<OnsetDetector>>,
        _stream: cpal::Stream,
    }

    fn on_error(e: cpal::Error) {
        eprintln!("mic error: {e}");
    }

    fn feed(shared: &Arc<Mutex<OnsetDetector>>, samples: impl Iterator<Item = f32>) {
        if let Ok(mut d) = shared.lock() {
            for s in samples {
                d.push(s);
            }
        }
    }

    pub fn start() -> Option<Mic> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let config = device.default_input_config().ok()?;
        let channels = config.channels() as usize;
        let detector = Arc::new(Mutex::new(OnsetDetector::new(config.sample_rate() as f32)));
        let shared = detector.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                config.into(),
                move |data: &[f32], _| {
                    feed(&shared, data.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32))
                },
                on_error,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                config.into(),
                move |data: &[i16], _| {
                    feed(
                        &shared,
                        data.chunks(channels)
                            .map(|c| c.iter().map(|s| *s as f32 / 32768.0).sum::<f32>() / channels as f32),
                    )
                },
                on_error,
                None,
            ),
            _ => return None,
        }
        .ok()?;
        stream.play().ok()?;
        Some(Mic {
            detector,
            _stream: stream,
        })
    }

    impl Mic {
        pub fn pump(&mut self) {}

        pub fn blocked(&self) -> bool {
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod mic {
    use crate::tempo::OnsetDetector;
    use std::sync::{Arc, Mutex};

    pub struct Mic {
        pub detector: Arc<Mutex<OnsetDetector>>,
        buf: Vec<f32>,
    }

    #[link(wasm_import_module = "env")]
    extern "C" {
        fn mic_start() -> f32;
        fn mic_state() -> i32;
        fn mic_pull(out: *mut f32, max: u32) -> u32;
    }

    pub fn start() -> Option<Mic> {
        let rate = unsafe { mic_start() };
        if rate <= 0.0 {
            return None;
        }
        Some(Mic {
            detector: Arc::new(Mutex::new(OnsetDetector::new(rate))),
            buf: vec![0.0; 4096],
        })
    }

    impl Mic {
        pub fn pump(&mut self) {
            loop {
                let n = unsafe { mic_pull(self.buf.as_mut_ptr(), self.buf.len() as u32) } as usize;
                if n == 0 {
                    break;
                }
                if let Ok(mut d) = self.detector.lock() {
                    for s in &self.buf[..n] {
                        d.push(*s);
                    }
                }
                if n < self.buf.len() {
                    break;
                }
            }
        }

        pub fn blocked(&self) -> bool {
            unsafe { mic_state() < 0 }
        }
    }
}

const MAP: usize = 24;
const TICK: f32 = 0.2;
const SAMPLE_RATE: u32 = 44100;
const TAPE_LIMIT: f32 = 300.0;

fn mix_tape(events: &[(f64, &[f32], f32)]) -> Vec<f32> {
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

fn wav_bytes(samples: &[f32]) -> Vec<u8> {
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

#[cfg(target_arch = "wasm32")]
mod radio {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn radio_text_len() -> i32;
        fn radio_text_get(out: *mut u8);
        fn radio_decks_len() -> i32;
        fn radio_decks_get(out: *mut u8);
        fn radio_next_len() -> i32;
        fn live_flag_get(i: i32) -> i32;
        fn live_flag_set(i: i32, on: i32);
        fn live_overlay(y: f32);
        fn radio_next_get(out: *mut u8);
        fn radio_bpm() -> f32;
        fn touch_mode() -> i32;
        fn ui_report(text: *const u8, len: u32);
        fn ask_text_open(prompt: *const u8, prompt_len: u32, current: *const u8, current_len: u32) -> i32;
        fn ask_text_get(out: *mut u8);
    }

    pub fn ask_text(prompt: &str, current: &str) -> Option<String> {
        let len = unsafe { ask_text_open(prompt.as_ptr(), prompt.len() as u32, current.as_ptr(), current.len() as u32) };
        if len < 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        if len > 0 {
            unsafe { ask_text_get(buf.as_mut_ptr()) };
        }
        String::from_utf8(buf).ok()
    }

    pub fn report(text: &str) {
        unsafe { ui_report(text.as_ptr(), text.len() as u32) };
    }

    pub fn touch_ui() -> bool {
        unsafe { touch_mode() != 0 }
    }

    pub fn now_playing() -> String {
        let len = unsafe { radio_text_len() };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len as usize];
        unsafe { radio_text_get(buf.as_mut_ptr()) };
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn decks() -> Vec<(String, String)> {
        let len = unsafe { radio_decks_len() };
        if len <= 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; len as usize];
        unsafe { radio_decks_get(buf.as_mut_ptr()) };
        String::from_utf8(buf)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.split_once('\t').map(|(d, t)| (d.to_string(), t.to_string())))
            .collect()
    }

    pub fn bpm() -> f32 {
        unsafe { radio_bpm() }
    }

    pub fn flag(i: i32) -> bool {
        unsafe { live_flag_get(i) != 0 }
    }

    pub fn set_flag(i: i32, on: bool) {
        unsafe { live_flag_set(i, on as i32) };
    }

    pub fn overlay(y: f32) {
        unsafe { live_overlay(y) };
    }

    pub fn next() -> Vec<String> {
        let len = unsafe { radio_next_len() };
        if len <= 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; len as usize];
        unsafe { radio_next_get(buf.as_mut_ptr()) };
        String::from_utf8(buf).unwrap_or_default().lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod radio {
    pub fn now_playing() -> String {
        String::new()
    }

    pub fn bpm() -> f32 {
        0.0
    }

    pub fn decks() -> Vec<(String, String)> {
        Vec::new()
    }

    pub fn next() -> Vec<String> {
        Vec::new()
    }

    pub fn overlay(_y: f32) {}

    static FLAGS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);

    pub fn flag(i: i32) -> bool {
        FLAGS.load(std::sync::atomic::Ordering::Relaxed) & (1 << i) != 0
    }

    pub fn set_flag(i: i32, on: bool) {
        let bit = 1u8 << i;
        if on {
            FLAGS.fetch_or(bit, std::sync::atomic::Ordering::Relaxed);
        } else {
            FLAGS.fetch_and(!bit, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn touch_ui() -> bool {
        false
    }

    pub fn report(_text: &str) {}

    pub fn ask_text(_prompt: &str, _current: &str) -> Option<String> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;

    fn home() -> PathBuf {
        std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    }

    fn save_path() -> PathBuf {
        home().join(".melody-quack").join("save.txt")
    }

    pub fn read() -> Option<String> {
        std::fs::read_to_string(save_path()).ok()
    }

    pub fn write(text: &str) {
        let path = save_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let temp = path.with_extension("tmp");
        if std::fs::write(&temp, text).is_ok() {
            let _ = std::fs::rename(&temp, &path);
        }
    }

    pub fn clear() {
        let _ = std::fs::remove_file(save_path());
    }

    pub fn restart() {
        if let Ok(exe) = std::env::current_exe() {
            if std::process::Command::new(exe).args(std::env::args().skip(1)).spawn().is_ok() {
                std::process::exit(0);
            }
        }
    }

    pub fn save_jam(bytes: &[u8]) -> Option<String> {
        let music = home().join("Music");
        let (dir, shown) = if music.is_dir() {
            (music.join("melody-quack"), "Music/melody-quack")
        } else {
            (home().join("melody-quack-jams"), "melody-quack-jams")
        };
        std::fs::create_dir_all(&dir).ok()?;
        let mut n = 1;
        while dir.join(format!("jam-{n:02}.wav")).exists() {
            n += 1;
        }
        let name = format!("jam-{n:02}.wav");
        std::fs::write(dir.join(&name), bytes).ok()?;
        Some(format!("{shown}/{name}"))
    }
}

#[cfg(target_arch = "wasm32")]
mod store {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn store_put(key: *const u8, key_len: u32, value: *const u8, value_len: u32);
        fn store_len(key: *const u8, key_len: u32) -> i32;
        fn store_get(key: *const u8, key_len: u32, out: *mut u8);
        fn store_remove(key: *const u8, key_len: u32);
        fn download_bytes(name: *const u8, name_len: u32, data: *const u8, data_len: u32);
        fn app_restart();
    }

    #[no_mangle]
    pub extern "C" fn melody_store_crate_version() -> u32 {
        1
    }

    fn get(key: &str) -> Option<String> {
        let len = unsafe { store_len(key.as_ptr(), key.len() as u32) };
        if len < 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        unsafe { store_get(key.as_ptr(), key.len() as u32, buf.as_mut_ptr()) };
        String::from_utf8(buf).ok()
    }

    fn put(key: &str, value: &str) {
        unsafe { store_put(key.as_ptr(), key.len() as u32, value.as_ptr(), value.len() as u32) };
    }

    pub fn read() -> Option<String> {
        get("melody-quack-save")
    }

    pub fn write(text: &str) {
        put("melody-quack-save", text);
    }

    pub fn clear() {
        let key = "melody-quack-save";
        unsafe { store_remove(key.as_ptr(), key.len() as u32) };
    }

    pub fn restart() {
        unsafe { app_restart() };
    }

    pub fn save_jam(bytes: &[u8]) -> Option<String> {
        let n = get("melody-quack-jams").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0) + 1;
        put("melody-quack-jams", &n.to_string());
        let name = format!("melody-quack-jam-{n:02}.wav");
        unsafe { download_bytes(name.as_ptr(), name.len() as u32, bytes.as_ptr(), bytes.len() as u32) };
        Some(format!("downloads/{name}"))
    }
}

const ECHO_TAPS: [f32; 6] = [0.62, 0.42, 0.29, 0.2, 0.13, 0.08];

fn ring_effect(dry: &[f32]) -> Vec<f32> {
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
const DEFAULT_BPM: f32 = 121.0;
const LOOP_STEPS: usize = 32;
const KICK_PATTERN: &str = "9...9...9...9...";
const CLAP_PATTERN: &str = "....8.......8...";
const HAT_PATTERN: &str = "2.7.2.7.2.7.2.75";
const CAT_RADIUS: f32 = 0.2;

const NOTES: [u8; 6] = [74, 76, 78, 81, 83, 84];
const NOTE_NAMES: [&str; 6] = ["D", "E", "F#", "A", "B", "C"];
const SPELL_NAMES: [&str; 6] = ["ember", "bloom", "rain", "gust", "frost", "sun"];
const NOTE_KEYS: [KeyCode; 6] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
];

fn screen_aspect() -> f32 {
    screen_width() / screen_height().max(1.0)
}

fn low_w() -> f32 {
    let aspect = screen_aspect();
    if aspect >= 1.0 { (LOW_H as f32 * aspect).round().clamp(240.0, 720.0) } else { LOW_H as f32 }
}

fn low_h() -> f32 {
    let aspect = screen_aspect();
    if aspect >= 1.0 { LOW_H as f32 } else { (LOW_H as f32 / aspect).round().clamp(240.0, 560.0) }
}
const LOW_H: u32 = 270;
const VIEW_W: u32 = 360;
const VIEW_H: u32 = 270;
const SKY_TOP: Color = Color::new(0.55, 0.74, 0.86, 1.0);
const FOG: Color = Color::new(0.80, 0.89, 0.84, 1.0);
const DUSK_FOG: Color = Color::new(0.36, 0.3, 0.48, 1.0);

fn help_box() -> (f32, f32, f32, f32) {
    let w = 300f32.min(low_w() - 12.0);
    let h = 210f32.min(low_h() - 24.0);
    (((low_w() - w) / 2.0).round(), ((low_h() - h) / 2.0).round().max(20.0), w, h)
}

fn ui_scale() -> (f32, f32) {
    (screen_width() / low_w(), screen_height() / low_h())
}

fn ui_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let (kx, ky) = ui_scale();
    draw_rectangle((x * kx).round(), (y * ky).round(), (w * kx).round().max(1.0), (h * ky).round().max(1.0), color);
}

fn ui_measure(text: &str, size: f32) -> f32 {
    let (kx, ky) = ui_scale();
    measure_text(text, None, (size * ky).round() as u16, 1.0).width / kx
}

fn ui_wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let joined = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if ui_measure(&joined, size) <= width {
            line = joined;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for ch in word.chars() {
            line.push(ch);
            if ui_measure(&line, size) > width && line.chars().count() > 1 {
                let last = line.pop().unwrap();
                lines.push(std::mem::take(&mut line));
                line.push(last);
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn ui_ticker(text: &str, size: f32, width: f32, time: f32) -> String {
    if ui_measure(text, size) <= width {
        return text.to_string();
    }
    let looped: Vec<char> = format!("{text}     ").chars().collect();
    let start = (time * 5.0) as usize % looped.len();
    let mut out = String::new();
    for k in 0..looped.len() * 2 {
        let ch = looped[(start + k) % looped.len()];
        out.push(ch);
        if ui_measure(&out, size) > width {
            out.pop();
            break;
        }
    }
    out
}

fn ui_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let (kx, ky) = ui_scale();
    let font_size = (size * ky).round() as u16;
    let shadow = (ky * 0.4).max(1.0).round();
    let (sx, sy) = ((x * kx).round(), (y * ky).round());
    draw_text_ex(
        text,
        sx + shadow,
        sy + shadow,
        TextParams {
            font_size,
            color: Color::new(0.05, 0.04, 0.1, color.a * 0.75),
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        sx,
        sy,
        TextParams {
            font_size,
            color,
            ..Default::default()
        },
    );
}

fn note_color(i: usize) -> Color {
    match i {
        0 => Color::from_rgba(255, 128, 98, 255),
        1 => Color::from_rgba(255, 168, 204, 255),
        2 => Color::from_rgba(116, 172, 255, 255),
        3 => Color::from_rgba(140, 222, 168, 255),
        4 => Color::from_rgba(186, 230, 255, 255),
        _ => Color::from_rgba(255, 214, 110, 255),
    }
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn hash(x: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (z as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}


fn render_woof(freq: f32) -> Vec<f32> {
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

fn normalise(samples: Vec<f32>, level: f32) -> Vec<f32> {
    let peak = samples.iter().fold(0.0_f32, |m, v| m.max(v.abs())).max(1e-6);
    samples.into_iter().map(|v| v / peak * level).collect()
}

fn render_meow_style(freq: f32, length: f32, bright: f32) -> Vec<f32> {
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

fn render_yap(freq: f32) -> Vec<f32> {
    let mut w = render_woof(freq);
    w.truncate((0.2 * SAMPLE_RATE as f32) as usize);
    let n = w.len() as f32;
    let out = w.into_iter().enumerate().map(|(i, v)| v * smoothstep((n - i as f32) / (0.05 * SAMPLE_RATE as f32))).collect();
    normalise(out, 0.5)
}

fn render_howl(freq: f32) -> Vec<f32> {
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

fn formant_gain(x: f32, formants: &[(f32, f32, f32)]) -> f32 {
    formants.iter().map(|&(f, a, bw)| a / (1.0 + ((x - f) / (f * bw)).powi(2))).sum()
}

fn render_quack(freq: f32) -> Vec<f32> {
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

fn render_hoot_machine(freq: f32) -> Vec<f32> {
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

fn render_fox(freq: f32) -> Vec<f32> {
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

fn render_shell(freq: f32) -> Vec<f32> {
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

fn noise(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn render_kick() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    (0..(0.45 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let phase = 2.0 * PI * (48.0 * t + 70.0 * (1.0 - (-t * 25.0).exp()) / 25.0);
            (phase.sin() * (-t * 6.5).exp() * 1.4).tanh() * 0.9
        })
        .collect()
}

fn render_clap() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut seed = 0x1357_9bdf;
    let mut lp = 0.0;
    let mut prev = 0.0;
    (0..(0.4 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let mut env = (-t * 10.0).exp() * 0.5;
            for b in [0.0, 0.012, 0.024] {
                if t >= b {
                    env += (-(t - b) * 160.0).exp();
                }
            }
            lp += (noise(&mut seed) - lp) * 0.4;
            let band = lp - prev * 0.6;
            prev = lp;
            band * env * 0.9
        })
        .collect()
}

fn render_hat() -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let mut seed = 0x2468_ace1;
    let mut prev = 0.0;
    (0..(0.12 * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let n = noise(&mut seed);
            let hp = n - prev;
            prev = n;
            hp * smoothstep(t / 0.003) * (-t * 35.0).exp() * 0.5
        })
        .collect()
}

struct BeatStyle {
    name: &'static str,
    kick: &'static str,
    clap: &'static str,
    hat: &'static str,
    open: &'static str,
}

const BEAT_STYLES: [BeatStyle; 5] = [
    BeatStyle { name: "techno", kick: "9...9...9...9...", clap: "....5.......5...", hat: "4343434343434343", open: "..8...8...8...8." },
    BeatStyle { name: "house", kick: "9...9...9...9...", clap: "....9.......9...", hat: "3.3.3.3.3.3.3.3.", open: "..7...7...7...7." },
    BeatStyle { name: "breaks", kick: "9.....9...9.....", clap: "....9.......9..6", hat: "6.6.6.6.6.6.6.6.", open: "..............7." },
    BeatStyle { name: "classic", kick: KICK_PATTERN, clap: CLAP_PATTERN, hat: HAT_PATTERN, open: "................" },
    BeatStyle { name: "off", kick: "................", clap: "................", hat: "................", open: "................" },
];

const DJ_PADS: [[&str; 6]; 2] = [["kick", "clap", "hi hat", "open", "low tom", "rim"], ["bass", "chord", "vox", "shaker", "bell", "snare"]];

fn render_open_hat() -> Vec<f32> {
    noise_burst(0.5, 6.5, 0.95, 0x51ed, 0.2)
}

fn render_tom() -> Vec<f32> {
    add_into(ring_tone(96.0, &[(1.0, 1.0, 7.0), (1.6, 0.35, 13.0), (2.3, 0.12, 20.0)], 0.55, 0.002, 0.75), noise_burst(0.02, 120.0, 0.5, 0x7011, 0.08))
}

fn render_rim() -> Vec<f32> {
    add_into(noise_burst(0.03, 120.0, 0.9, 0x77aa, 0.32), ring_tone(1650.0, &[(1.0, 0.6, 70.0), (2.7, 0.2, 90.0)], 0.06, 0.001, 0.4))
}

fn render_bass() -> Vec<f32> {
    ring_tone(55.0, &[(1.0, 1.0, 3.5), (2.0, 0.45, 5.5), (3.0, 0.18, 8.0)], 0.55, 0.004, 0.85)
}

fn velocity(pattern: &str, step: usize) -> Option<f32> {
    (pattern.as_bytes()[step % 16] as char).to_digit(10).map(|d| d as f32 / 9.0)
}

const LIVE_WINDOW: f32 = 0.03;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Quantise {
    Off,
    Soft,
    Tight,
}

impl Quantise {
    fn next(self) -> Quantise {
        match self {
            Quantise::Off => Quantise::Soft,
            Quantise::Soft => Quantise::Tight,
            Quantise::Tight => Quantise::Off,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Quantise::Off => "off",
            Quantise::Soft => "soft",
            Quantise::Tight => "tight",
        }
    }
}

fn note_owner(id: usize) -> usize {
    match id {
        6..=11 | 22..=27 | 28..=31 => 1,
        _ => 0,
    }
}

fn note_slot(id: usize) -> usize {
    match id {
        0..=11 => id % 6,
        12..=15 => id - 12,
        28..=31 => id - 28,
        _ => (id - 16) % 6,
    }
}

fn wrap_steps(p: f32) -> f32 {
    p.rem_euclid(LOOP_STEPS as f32)
}

struct Looper {
    notes: Vec<(f32, usize)>,
    skip: Vec<(f32, usize)>,
    pending: Vec<(f32, usize)>,
    recording: bool,
    beat: bool,
    bpm: f32,
    position: f32,
    last_step: Option<usize>,
    quantise: Quantise,
}

impl Looper {
    fn new() -> Looper {
        Looper {
            notes: Vec::new(),
            skip: Vec::new(),
            pending: Vec::new(),
            recording: false,
            beat: false,
            bpm: DEFAULT_BPM,
            position: 0.0,
            last_step: None,
            quantise: Quantise::Soft,
        }
    }

    fn step_seconds(&self) -> f32 {
        60.0 / self.bpm / 4.0
    }

    fn current_step(&self) -> usize {
        self.position as usize % LOOP_STEPS
    }

    fn sync_to_beat(&mut self, since_beat: f32) {
        let target = (since_beat * self.bpm / 60.0 * 4.0) % 4.0;
        let mut diff = target - self.position % 4.0;
        if diff > 2.0 {
            diff -= 4.0;
        }
        if diff < -2.0 {
            diff += 4.0;
        }
        self.position = (self.position + diff * 0.5).max(0.0);
    }

    fn press(&mut self, note: usize) -> bool {
        let late_by = self.position.fract() * self.step_seconds();
        let wait = self.quantise == Quantise::Tight && late_by > LIVE_WINDOW;
        let grid = if wait { self.position.floor() + 1.0 } else { self.position.floor() };
        if wait {
            self.pending.push((grid, note));
        }
        if self.recording {
            let raw = self.position;
            let at = match self.quantise {
                Quantise::Off => raw,
                Quantise::Soft => raw + (raw.round() - raw) * 0.5,
                Quantise::Tight => grid,
            };
            self.store(at, note);
        }
        !wait
    }

    fn store(&mut self, at_abs: f32, note: usize) {
        let at = wrap_steps(at_abs);
        let close = |p: f32| {
            let d = (p - at).rem_euclid(LOOP_STEPS as f32);
            d.min(LOOP_STEPS as f32 - d) < 0.25
        };
        if self.notes.iter().any(|&(p, n)| n == note && close(p)) {
            return;
        }
        self.notes.push((at, note));
        if at_abs > self.position {
            self.skip.push((at, note));
        }
    }

    fn tick(&mut self, dt: f32) -> Vec<usize> {
        let before = self.position;
        self.position += dt * self.bpm / 60.0 * 4.0;
        if self.position >= LOOP_STEPS as f32 * 64.0 {
            self.position -= LOOP_STEPS as f32 * 64.0;
            for p in self.pending.iter_mut() {
                p.0 -= LOOP_STEPS as f32 * 64.0;
            }
        }
        let after = self.position;

        let mut due = Vec::new();
        self.pending.retain(|&(at, note)| {
            if after >= at {
                due.push(note);
                false
            } else {
                true
            }
        });

        let span = after - before;
        if span > 0.0 && span < 4.0 {
            let from = wrap_steps(before);
            for k in 0..self.notes.len() {
                let (at, note) = self.notes[k];
                let d = (at - from).rem_euclid(LOOP_STEPS as f32);
                if d > 0.0 && d <= span {
                    if let Some(i) = self.skip.iter().position(|e| *e == (at, note)) {
                        self.skip.remove(i);
                    } else {
                        due.push(note);
                    }
                }
            }
        }
        due
    }

    fn beat_pulse(&self) -> f32 {
        (1.0 - (self.position / 4.0).fract()).powi(3)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Grass,
    Flowers,
    Sprout,
    Tree,
    Water,
    Ice,
    Ash,
    Stone,
    House,
    Campfire,
    Path,
    WoodWall,
    StoneWall,
    Floor,
    Dock,
}

#[derive(Clone, Copy)]
struct Tile {
    kind: Kind,
    fire: f32,
    wet: f32,
    tint: f32,
    stack: u8,
}

impl Tile {
    fn flammable(&self) -> bool {
        matches!(self.kind, Kind::Grass | Kind::Flowers | Kind::Sprout | Kind::Tree)
            && self.wet < 0.3
            && self.fire <= 0.0
    }

    fn solid(&self) -> bool {
        matches!(self.kind, Kind::Tree | Kind::Water | Kind::Stone | Kind::House | Kind::Campfire | Kind::WoodWall | Kind::StoneWall)
    }

    fn burn_time(&self) -> f32 {
        match self.kind {
            Kind::Tree => 4.0,
            Kind::Sprout => 1.2,
            _ => 1.6,
        }
    }
}

struct World {
    tiles: Vec<Tile>,
    wind: Vec2,
    wind_time: f32,
    seed: u32,
}

impl World {
    fn new() -> World {
        let path: [(i32, i32); 16] = [
            (4, 15), (4, 16), (5, 16), (6, 16), (7, 16), (8, 16), (9, 16), (9, 15),
            (9, 14), (9, 13), (10, 12), (10, 11), (11, 10), (12, 9), (12, 8), (13, 7),
        ];
        let mut tiles = Vec::with_capacity(MAP * MAP);
        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let edge = x == 0 || z == 0 || x == MAP as i32 - 1 || z == MAP as i32 - 1;
                let lake = vec2(x as f32 - 16.5, z as f32 - 18.5).length() < 3.1;
                let (cx0, cz0, cx1, cz1) = COTTAGE;
                let house = x >= cx0 && x < cx1 && z >= cz0 && z < cz1;
                let home = !in_forest(x, z) && !lake;
                let clear_home = (x - 8).abs() <= 2 && (z - 17).abs() <= 1 || (x >= 2 && x <= 7 && z >= 15 && z <= 16);
                let kind = if edge {
                    Kind::Tree
                } else if house {
                    Kind::House
                } else if (x, z) == CAMPFIRE {
                    Kind::Campfire
                } else if path.contains(&(x, z)) {
                    Kind::Path
                } else if DOCK.contains(&(x, z)) {
                    Kind::Dock
                } else if lake {
                    Kind::Water
                } else if clear_home {
                    Kind::Grass
                } else if home {
                    if hash(x, z, 4) > 0.72 {
                        Kind::Flowers
                    } else if hash(x, z, 3) > 0.96 {
                        Kind::Stone
                    } else if hash(x, z, 2) > 0.95 && (x < 3 || z > 19) {
                        Kind::Tree
                    } else {
                        Kind::Grass
                    }
                } else if hash(x, z, 1) > 0.56 {
                    Kind::Tree
                } else if hash(x, z, 3) > 0.93 {
                    Kind::Stone
                } else if hash(x, z, 4) > 0.8 {
                    Kind::Flowers
                } else {
                    Kind::Grass
                };
                tiles.push(Tile {
                    kind,
                    fire: 0.0,
                    wet: 0.0,
                    tint: hash(x, z, 5) * 0.14 - 0.07,
                    stack: 0,
                });
            }
        }
        World {
            tiles,
            wind: Vec2::ZERO,
            wind_time: 0.0,
            seed: 0x9e37_79b9,
        }
    }

    fn index(x: i32, z: i32) -> Option<usize> {
        if x < 0 || z < 0 || x >= MAP as i32 || z >= MAP as i32 {
            None
        } else {
            Some(z as usize * MAP + x as usize)
        }
    }

    fn get(&self, x: i32, z: i32) -> Option<&Tile> {
        World::index(x, z).map(|i| &self.tiles[i])
    }

    fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut Tile> {
        World::index(x, z).map(move |i| &mut self.tiles[i])
    }

    fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed & 0xffff) as f32 / 65535.0
    }

    fn tick(&mut self, particles: &mut Vec<Particle>) {
        let old = self.tiles.clone();
        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let i = World::index(x, z).unwrap();
                let t = old[i];

                if t.fire > 0.0 {
                    if t.wet > 0.3 {
                        self.tiles[i].fire = 0.0;
                        puff(particles, x, z, WHITE, 6);
                    } else {
                        self.tiles[i].fire -= TICK;
                        if self.tiles[i].fire <= 0.0 {
                            self.tiles[i].fire = 0.0;
                            self.tiles[i].kind = Kind::Ash;
                        }
                        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            let Some(ni) = World::index(x + dx, z + dz) else { continue };
                            let n = old[ni];
                            if n.kind == Kind::Ice {
                                self.tiles[ni].kind = Kind::Water;
                                puff(particles, x + dx, z + dz, WHITE, 4);
                            }
                            if n.flammable() && self.tiles[ni].fire <= 0.0 {
                                let mut chance = if n.kind == Kind::Tree { 0.07 } else { 0.12 };
                                if self.wind_time > 0.0 {
                                    let along = vec2(dx as f32, dz as f32).dot(self.wind);
                                    chance *= (1.0 + along * 2.0).max(0.15);
                                }
                                if self.rand() < chance {
                                    self.tiles[ni].fire = n.burn_time();
                                }
                            }
                        }
                    }
                }

                if t.kind != Kind::Water {
                    self.tiles[i].wet = (self.tiles[i].wet - 0.012).max(0.0);
                }

                if t.kind == Kind::Ash && t.fire <= 0.0 && self.rand() < 0.004 {
                    self.tiles[i].kind = Kind::Grass;
                }
            }
        }
    }
}

struct Particle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    max_life: f32,
    color: Color,
    size: f32,
    gravity: f32,
}

fn spawn(particles: &mut Vec<Particle>, pos: Vec3, vel: Vec3, life: f32, color: Color, size: f32, gravity: f32) {
    particles.push(Particle {
        pos,
        vel,
        life,
        max_life: life,
        color,
        size,
        gravity,
    });
}

fn puff(particles: &mut Vec<Particle>, x: i32, z: i32, color: Color, count: usize) {
    for k in 0..count {
        let a = hash(x, z, k as u32 + (get_time() * 60.0) as u32) * 2.0 * PI;
        spawn(
            particles,
            vec3(x as f32 + 0.5, 0.7, z as f32 + 0.5),
            vec3(a.cos() * 0.4, 0.9 + hash(z, x, k as u32) * 0.6, a.sin() * 0.4),
            0.9,
            color,
            0.06,
            -0.2,
        );
    }
}

fn touches_cat(x: i32, z: i32, cat: Vec2) -> bool {
    let nearest_x = cat.x.clamp(x as f32, x as f32 + 1.0);
    let nearest_z = cat.y.clamp(z as f32, z as f32 + 1.0);
    vec2(nearest_x, nearest_z).distance(cat) < CAT_RADIUS + 0.05
}

fn cast(spell: usize, world: &mut World, cat: Vec2, facing: Vec2, particles: &mut Vec<Particle>) {
    let cx = cat.x.floor() as i32;
    let cz = cat.y.floor() as i32;
    let ahead = cat + facing * 1.0;
    let tx = ahead.x.floor() as i32;
    let tz = ahead.y.floor() as i32;

    match spell {
        0 => {
            if let Some(t) = world.get_mut(tx, tz) {
                if t.flammable() {
                    t.fire = t.burn_time();
                } else if t.kind == Kind::Ice {
                    t.kind = Kind::Water;
                }
            }
            puff(particles, tx, tz, Color::from_rgba(255, 150, 60, 255), 10);
        }
        1 => {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (x, z) = (tx + dx, tz + dz);
                    let center = dx == 0 && dz == 0;
                    let blocked = touches_cat(x, z, cat);
                    if let Some(t) = world.get_mut(x, z) {
                        if t.fire > 0.0 {
                            continue;
                        }
                        t.kind = match t.kind {
                            Kind::Ash => Kind::Grass,
                            Kind::Grass => Kind::Flowers,
                            Kind::Flowers if center => Kind::Sprout,
                            Kind::Sprout if center && !blocked => Kind::Tree,
                            k => k,
                        };
                    }
                    puff(particles, x, z, Color::from_rgba(255, 170, 210, 255), 2);
                }
            }
        }
        2 => {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if let Some(t) = world.get_mut(cx + dx, cz + dz) {
                        t.wet = 1.0;
                        if t.fire > 0.0 {
                            t.fire = 0.0;
                        }
                    }
                }
            }
            for k in 0..70 {
                let ox = hash(k, 1, (get_time() * 10.0) as u32) * 5.0 - 2.5;
                let oz = hash(k, 2, (get_time() * 10.0) as u32) * 5.0 - 2.5;
                let h = 2.5 + hash(k, 3, 7) * 2.5;
                spawn(
                    particles,
                    vec3(cat.x + ox, h, cat.y + oz),
                    vec3(0.0, -7.0, 0.0),
                    h / 7.0,
                    Color::from_rgba(170, 210, 255, 255),
                    0.03,
                    0.0,
                );
            }
        }
        3 => {
            world.wind = facing;
            world.wind_time = 5.0;
            for k in 0..28 {
                let side = vec2(-facing.y, facing.x) * (hash(k, 4, (get_time() * 10.0) as u32) * 4.0 - 2.0);
                let start = cat + side - facing * 1.5;
                spawn(
                    particles,
                    vec3(start.x, 0.6 + hash(k, 5, 3) * 0.8, start.y),
                    vec3(facing.x * 5.0, 0.3, facing.y * 5.0),
                    1.2,
                    Color::from_rgba(160, 225, 150, 255),
                    0.045,
                    0.0,
                );
            }
        }
        4 => {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    if let Some(t) = world.get_mut(tx + dx, tz + dz) {
                        if t.kind == Kind::Water {
                            t.kind = Kind::Ice;
                        }
                        t.fire = 0.0;
                    }
                    puff(particles, tx + dx, tz + dz, Color::from_rgba(230, 245, 255, 255), 2);
                }
            }
        }
        _ => {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if let Some(t) = world.get_mut(cx + dx, cz + dz) {
                        if t.kind != Kind::Water {
                            t.wet = 0.0;
                        }
                        if t.kind == Kind::Ice {
                            t.kind = Kind::Water;
                        }
                    }
                }
            }
            for k in 0..24 {
                let a = k as f32 / 24.0 * 2.0 * PI;
                spawn(
                    particles,
                    vec3(cat.x + a.cos() * 0.5, 0.8, cat.y + a.sin() * 0.5),
                    vec3(a.cos() * 2.0, 1.2, a.sin() * 2.0),
                    0.8,
                    Color::from_rgba(255, 225, 120, 255),
                    0.05,
                    0.0,
                );
            }
        }
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}


fn tinted(c: Color, k: f32) -> Color {
    Color::new(
        (c.r * (1.0 + k)).clamp(0.0, 1.0),
        (c.g * (1.0 + k)).clamp(0.0, 1.0),
        (c.b * (1.0 + k)).clamp(0.0, 1.0),
        c.a,
    )
}

#[derive(Clone)]
struct Light {
    dusk: f32,
    fire: Vec3,
    fire_strength: f32,
    tint: Vec3,
    lamps: Vec<(Vec3, f32, Vec3)>,
}

impl Light {
    fn fog(&self) -> Color {
        sky_haze(mix(FOG, DUSK_FOG, self.dusk), self.dusk)
    }

    fn apply(&self, p: Vec3, c: Color) -> Color {
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

struct Batch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    eye: Vec3,
    light: Light,
    unlit: bool,
}

impl Batch {
    fn new(eye: Vec3, light: Light) -> Batch {
        Batch {
            vertices: Vec::new(),
            indices: Vec::new(),
            eye,
            light,
            unlit: false,
        }
    }

    fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        draw_mesh(&Mesh {
            vertices: std::mem::take(&mut self.vertices),
            indices: std::mem::take(&mut self.indices),
            texture: None,
        });
    }

    fn cube(&mut self, center: Vec3, size: Vec3, color: Color) {
        self.box_at(center, 0.0, Vec3::ZERO, size, color);
    }

    fn top(&mut self, center: Vec3, color: Color) {
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

    fn box_at(&mut self, origin: Vec3, yaw: f32, local: Vec3, size: Vec3, color: Color) {
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

const DOCK: [(i32, i32); 2] = [(14, 18), (15, 18)];
const WALL_HEIGHT: f32 = 0.45;

fn ground_top(kind: Kind) -> f32 {
    match kind {
        Kind::Water => 0.32,
        Kind::Ice => 0.46,
        Kind::Dock => 0.48,
        Kind::Floor => 0.55,
        _ => 0.5,
    }
}

fn surface_top(kind: Kind) -> f32 {
    match kind {
        Kind::Dock => 0.32,
        Kind::Floor => 0.5,
        k => ground_top(k),
    }
}

fn hides_point(x: f32, z: f32, focus: Vec2) -> bool {
    let dx = (x - focus.x).abs();
    let dz = z - focus.y;
    dz > 0.3 && dz < 2.6 && dx < 0.9 + dz * 0.1
}

fn draw_lantern(b: &mut Batch, base: Vec3, time: f32) {
    let post = Color::from_rgba(96, 66, 44, 255);
    let flicker = 0.9 + 0.1 * (time * 7.0 + base.x * 3.0).sin();
    b.cube(base + vec3(0.0, 0.32, 0.0), vec3(0.07, 0.64, 0.07), post);
    b.cube(base + vec3(0.0, 0.69, 0.0), vec3(0.2, 0.04, 0.2), post);
    b.cube(base + vec3(0.0, 0.6, 0.0), vec3(0.14, 0.13, 0.14), Color::new(1.0, 0.86 * flicker, 0.52 * flicker, 1.0));
    b.cube(base + vec3(0.0, 0.53, 0.0), vec3(0.17, 0.03, 0.17), post);
}

fn draw_world(b: &mut Batch, world: &World, time: f32, cat: Vec2, see_through: bool) {
    let sk = sky();
    let (mut grass, leaves, mut leaves_light) = match sk.season {
        0 => (rgba(118, 186, 96), rgba(72, 146, 84), rgba(240, 176, 200)),
        2 => (rgba(150, 168, 88), rgba(190, 104, 50), rgba(230, 162, 64)),
        3 => (rgba(140, 158, 134), rgba(52, 104, 76), rgba(78, 126, 92)),
        _ => (Color::from_rgba(112, 178, 92, 255), Color::from_rgba(58, 128, 78, 255), Color::from_rgba(92, 160, 86, 255)),
    };
    if sk.weather == 3 {
        grass = rgba(230, 236, 242);
        leaves_light = rgba(240, 244, 250);
    }
    let wet_grass = Color::from_rgba(70, 138, 112, 255);
    let trunk = Color::from_rgba(122, 86, 60, 255);
    let wood = Color::from_rgba(178, 126, 80, 255);
    let stone = Color::from_rgba(160, 162, 176, 255);
    let plank = Color::from_rgba(204, 160, 110, 255);
    let path = Color::from_rgba(196, 162, 116, 255);

    for z in 0..MAP as i32 {
        for x in 0..MAP as i32 {
            let t = world.get(x, z).unwrap();
            let fx = x as f32 + 0.5;
            let fz = z as f32 + 0.5;
            let is_wall = matches!(t.kind, Kind::WoodWall | Kind::StoneWall);
            let in_the_way = (t.kind == Kind::Tree || (is_wall && t.stack >= 2)) && hides_point(fx, fz, cat);
            if see_through && !in_the_way {
                continue;
            }
            let top = surface_top(t.kind);
            let alpha = if see_through { 0.35 } else { 1.0 };
            let ground = match t.kind {
                Kind::Water | Kind::Dock => {
                    let shimmer = (time * 2.0 + x as f32 * 0.7 + z as f32 * 1.1).sin() * 0.05;
                    tinted(Color::from_rgba(78, 146, 204, 255), shimmer)
                }
                Kind::Ice => Color::from_rgba(206, 232, 246, 255),
                Kind::Ash => tinted(Color::from_rgba(96, 90, 88, 255), t.tint),
                Kind::Path | Kind::Campfire => tinted(path, t.tint * 0.5),
                _ => tinted(mix(grass, wet_grass, t.wet * 0.7), t.tint),
            };
            if !see_through {
                let lower = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dz)| world.get(x + dx, z + dz).map(|n| surface_top(n.kind) < top - 0.01).unwrap_or(true));
                if lower {
                    b.cube(vec3(fx, (top - 0.6) / 2.0, fz), vec3(1.0, top + 0.6, 1.0), ground);
                } else {
                    b.top(vec3(fx, top, fz), ground);
                }
            }

            match t.kind {
                Kind::Flowers if !see_through => {
                    for k in 0..3 {
                        let ox = hash(x, z, 10 + k) * 0.7 - 0.35;
                        let oz = hash(x, z, 20 + k) * 0.7 - 0.35;
                        let petal = match k {
                            0 => Color::from_rgba(255, 170, 200, 255),
                            1 => Color::from_rgba(255, 236, 140, 255),
                            _ => Color::from_rgba(250, 250, 255, 255),
                        };
                        b.cube(vec3(fx + ox, top + 0.06, fz + oz), vec3(0.1, 0.1, 0.1), petal);
                    }
                }
                Kind::Sprout if !see_through => {
                    b.cube(vec3(fx, top + 0.1, fz), vec3(0.08, 0.2, 0.08), leaves_light);
                    b.cube(vec3(fx, top + 0.24, fz), vec3(0.22, 0.12, 0.22), leaves_light);
                }
                Kind::Tree if in_the_way == see_through => {
                    let sway = (time * 1.3 + x as f32 + z as f32 * 0.5).sin() * 0.03;
                    let fade = |c: Color| Color::new(c.r, c.g, c.b, alpha);
                    b.cube(vec3(fx, top + 0.35, fz), vec3(0.22, 0.7, 0.22), fade(trunk));
                    b.cube(vec3(fx + sway, top + 0.95, fz), vec3(0.9, 0.55, 0.9), fade(tinted(leaves, t.tint)));
                    b.cube(vec3(fx + sway * 1.5, top + 1.38, fz), vec3(0.6, 0.38, 0.6), fade(tinted(leaves_light, t.tint)));
                }
                Kind::Stone if !see_through => {
                    b.cube(vec3(fx, top + 0.25, fz), vec3(0.75, 0.5, 0.7), Color::from_rgba(150, 152, 164, 255));
                    b.cube(vec3(fx + 0.15, top + 0.55, fz - 0.05), vec3(0.4, 0.2, 0.4), Color::from_rgba(170, 172, 182, 255));
                }
                Kind::WoodWall | Kind::StoneWall if in_the_way == see_through => {
                    for level in 0..t.stack {
                        let base = if t.kind == Kind::WoodWall {
                            tinted(wood, if level % 2 == 0 { 0.0 } else { -0.08 })
                        } else {
                            tinted(stone, hash(x, z, level as u32) * 0.1 - 0.05)
                        };
                        let y = 0.5 + WALL_HEIGHT * (level as f32 + 0.5);
                        b.cube(vec3(fx, y, fz), vec3(1.0, WALL_HEIGHT, 1.0), Color::new(base.r, base.g, base.b, alpha));
                    }
                }
                Kind::Floor if !see_through => {
                    let p = if (x + z) % 2 == 0 { plank } else { tinted(plank, -0.06) };
                    b.cube(vec3(fx, 0.525, fz), vec3(0.98, 0.05, 0.98), p);
                }
                Kind::Dock if !see_through => {
                    for k in 0..4 {
                        let px = x as f32 + 0.125 + k as f32 * 0.25;
                        let c = if (k + x) % 2 == 0 { plank } else { tinted(plank, -0.07) };
                        b.cube(vec3(px, 0.455, fz), vec3(0.22, 0.05, 0.7), c);
                    }
                    for (ox, oz) in [(0.1, -0.33), (0.1, 0.33), (0.9, -0.33), (0.9, 0.33)] {
                        b.cube(vec3(x as f32 + ox, 0.38, z as f32 + 0.5 + oz), vec3(0.08, 0.26, 0.08), tinted(plank, -0.3));
                    }
                }
                _ => {}
            }

            if t.fire > 0.0 && !see_through {
                let flame_base = if t.kind == Kind::Tree { top + 0.6 } else { top };
                for k in 0..3 {
                    let flick = hash(x * 3 + k, z, (time * 12.0) as u32);
                    let h = 0.25 + flick * 0.45;
                    let ox = (k as f32 - 1.0) * 0.22;
                    let col = match k {
                        0 => Color::from_rgba(240, 90, 40, 255),
                        1 => Color::from_rgba(255, 170, 60, 255),
                        _ => Color::from_rgba(255, 225, 110, 255),
                    };
                    b.cube(vec3(fx + ox, flame_base + h / 2.0, fz + ox * 0.5), vec3(0.2, h, 0.2), col);
                }
            }
        }
    }
}

const COTTAGE: (i32, i32, i32, i32) = (3, 13, 6, 15);

fn draw_cottage(b: &mut Batch, dusk: f32, see_through: bool, focus: Vec2) {
    let (x0, z0, x1, z1) = COTTAGE;
    let in_the_way = focus.y < z0 as f32 && focus.y > z0 as f32 - 2.6 && focus.x > x0 as f32 - 0.8 && focus.x < x1 as f32 + 0.8;
    if in_the_way != see_through {
        return;
    }
    let a = if see_through { 0.35 } else { 1.0 };
    let log = Color::from_rgba(170, 118, 74, 255);
    let log_dark = Color::from_rgba(130, 88, 54, 255);
    let roof = Color::from_rgba(180, 78, 62, 255);
    let roof_dark = Color::from_rgba(140, 56, 46, 255);
    let fade = |c: Color| Color::new(c.r, c.g, c.b, a);
    for tz in z0..z1 {
        for tx in x0..x1 {
            for level in 0..4 {
                let c = if level % 2 == 0 { log } else { log_dark };
                b.cube(vec3(tx as f32 + 0.5, 0.5 + 0.125 + level as f32 * 0.25, tz as f32 + 0.5), vec3(1.0, 0.25, 1.0), fade(c));
            }
        }
    }
    for (cx, cz) in [(x0, z0), (x1, z0), (x0, z1), (x1, z1)] {
        b.cube(vec3(cx as f32, 1.02, cz as f32), vec3(0.16, 1.04, 0.16), fade(log_dark));
    }
    let depth = (z1 - z0) as f32;
    let mid_z = (z0 + z1) as f32 / 2.0;
    let width = (x1 - x0) as f32 + 0.5;
    let mid_x = (x0 + x1) as f32 / 2.0;
    for layer in 0..5 {
        let half = depth / 2.0 + 0.25 - layer as f32 * 0.27;
        let y = 1.6 + layer as f32 * 0.2;
        let c = if layer % 2 == 0 { roof } else { roof_dark };
        b.cube(vec3(mid_x, y, mid_z), vec3(width, 0.2, half * 2.0), fade(c));
    }
    b.cube(vec3(x1 as f32 - 0.6, 2.1, z0 as f32 + 0.6), vec3(0.34, 1.0, 0.34), fade(Color::from_rgba(150, 152, 166, 255)));
    if see_through {
        return;
    }
    let front = z1 as f32 + 0.02;
    let door_x = x0 as f32 + 1.5;
    b.cube(vec3(door_x, 0.85, front), vec3(0.42, 0.7, 0.04), Color::from_rgba(110, 70, 42, 255));
    b.cube(vec3(door_x + 0.13, 0.85, front + 0.025), vec3(0.05, 0.05, 0.02), Color::from_rgba(255, 210, 90, 255));
    let glass = mix(Color::from_rgba(150, 190, 220, 255), Color::from_rgba(255, 214, 130, 255), (dusk * 2.0).min(1.0));
    for wx in [x0 as f32 + 0.5, x1 as f32 - 0.5] {
        b.cube(vec3(wx, 1.0, front), vec3(0.42, 0.34, 0.04), Color::from_rgba(150, 100, 60, 255));
        b.cube(vec3(wx, 1.0, front + 0.01), vec3(0.32, 0.25, 0.04), glass);
        b.cube(vec3(wx, 0.8, front + 0.06), vec3(0.44, 0.07, 0.12), Color::from_rgba(150, 100, 60, 255));
        for (k, col) in [(-1.0, Color::from_rgba(255, 140, 170, 255)), (0.0, Color::from_rgba(255, 230, 110, 255)), (1.0, Color::from_rgba(250, 250, 250, 255))] {
            b.cube(vec3(wx + k * 0.13, 0.86, front + 0.06), vec3(0.08, 0.06, 0.08), col);
        }
    }
    b.cube(vec3(door_x, 0.53, front + 0.15), vec3(0.6, 0.06, 0.3), Color::from_rgba(204, 160, 110, 255));
}

fn draw_pixel_grid(pixels: &[(usize, usize, Color)], x: f32, y: f32, px: f32, tint: impl Fn(Color) -> Color) {
    let (kx, ky) = ui_scale();
    let cell = (px * kx.min(ky)).round().max(1.0);
    let (ox, oy) = ((x * kx).round(), (y * ky).round());
    for (ax, ay, c) in pixels {
        draw_rectangle(ox + *ax as f32 * cell, oy + *ay as f32 * cell, cell, cell, tint(*c));
    }
}

fn draw_fish_art(art: &[(usize, usize, Color)], x: f32, y: f32, px: f32, known: bool) {
    draw_pixel_grid(art, x, y, px, |c| if known { c } else { Color::new(0.27, 0.3, 0.44, 1.0) });
}

fn draw_emoji_ui(id: usize, x: f32, y: f32, px: f32, alpha: f32) {
    draw_pixel_grid(&emoji::pixels(id), x, y, px, |c| Color::new(c.r, c.g, c.b, alpha));
}

fn emoji_strip(who: usize, two_player: bool) -> (f32, f32, f32) {
    let cell = 13.0;
    let w = cell * emoji::COUNT as f32;
    let half = low_w() / 2.0;
    let x = if two_player { who as f32 * half + (half - w) / 2.0 } else { (low_w() - w) / 2.0 };
    let y = if low_h() > low_w() { 96.0 } else { 58.0 };
    (x, y, cell)
}

fn rotate_y(v: Vec3, yaw: f32) -> Vec3 {
    let (s, c) = yaw.sin_cos();
    vec3(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

fn draw_piece(b: &mut Batch, piece: &home::Piece, time: f32, alpha: f32, record_on: bool, pulse: f32, lift: f32) {
    use home::Item;
    let k = |c: Color| Color::new(c.r, c.g, c.b, alpha);
    let c = piece.center();
    let o = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + lift, home::ORIGIN.y + c.y);
    let yaw = piece.yaw();
    let wood = k(Color::from_rgba(150, 102, 64, 255));
    let wood_dark = k(Color::from_rgba(112, 74, 46, 255));
    let bx = |b: &mut Batch, at: Vec3, size: Vec3, col: Color| b.box_at(o, yaw, at, size, col);
    match piece.item {
        Item::Sampler => {
            bx(b, vec3(0.0, 0.35, 0.0), vec3(0.7, 0.7, 0.6), wood_dark);
            bx(b, vec3(0.0, 0.74, 0.0), vec3(0.66, 0.08, 0.5), k(Color::from_rgba(44, 42, 56, 255)));
            let step = vis().seq;
            b.unlit = true;
            for r in 0..4 {
                for c in 0..4 {
                    let lit = c == step % 4;
                    let col = note_color(r);
                    let g = if lit { 1.0 } else { 0.35 };
                    bx(b, vec3(-0.22 + c as f32 * 0.147, 0.79, -0.18 + r as f32 * 0.12), vec3(0.11, 0.02, 0.09), k(Color::new(col.r * g, col.g * g, col.b * g, 1.0)));
                }
            }
            b.unlit = false;
        }
        Item::MirrorBall => {
            let on = piece.state == 0;
            let spin = if on { time * 1.2 } else { 0.0 };
            bx(b, vec3(0.0, 1.95, 0.0), vec3(0.02, 0.5, 0.02), k(Color::from_rgba(150, 150, 160, 255)));
            let center = o + vec3(0.0, 1.55, 0.0);
            for (n, (sz, dy)) in [(0.2, -0.13), (0.3, -0.06), (0.32, 0.02), (0.28, 0.09), (0.18, 0.15)].iter().enumerate() {
                b.box_at(center, yaw + spin + n as f32 * 0.4, vec3(0.0, *dy, 0.0), vec3(*sz, 0.075, *sz), k(Color::from_rgba(200, 204, 214, 255)));
            }
            b.unlit = true;
            for f in 0..10 {
                let a = spin * 1.5 + f as f32 * 0.63;
                let tw = ((time * 6.0 + f as f32 * 1.7).sin() * 0.5 + 0.5) * if on { 1.0 } else { 0.3 };
                let at = vec3(a.cos() * 0.16, -0.1 + (f % 4) as f32 * 0.07, a.sin() * 0.16);
                b.box_at(center, 0.0, at, vec3(0.035, 0.035, 0.035), k(Color::new(0.8 + 0.2 * tw, 0.85 + 0.15 * tw, 1.0, 1.0)));
            }
            if on {
                for d in 0..18 {
                    let a = time * 0.7 + d as f32 * 0.349;
                    let r = 0.8 + (d % 3) as f32 * 0.7;
                    let c = hsv(d as f32 / 18.0 + time * 0.05, 0.45, 1.0);
                    b.box_at(o, 0.0, vec3(a.cos() * r, 0.02, a.sin() * r), vec3(0.09, 0.01, 0.09), k(Color::new(c.r, c.g, c.b, 0.9)));
                }
            }
            b.unlit = false;
        }
        Item::Arcade => {
            let body = k(Color::from_rgba(40, 34, 70, 255));
            let trim = k(Color::from_rgba(255, 90, 160, 255));
            bx(b, vec3(0.0, 0.85, -0.12), vec3(0.8, 1.7, 0.6), body);
            bx(b, vec3(0.0, 1.62, 0.2), vec3(0.82, 0.2, 0.08), trim);
            bx(b, vec3(0.0, 0.55, 0.19), vec3(0.6, 0.04, 0.04), trim);
            let pads = vis().pads;
            b.unlit = true;
            bx(b, vec3(0.0, 1.62, 0.245), vec3(0.6, 0.12, 0.01), k(hsv(time * 0.2, 0.5, 1.0)));
            for y in 0..5 {
                for x in 0..6 {
                    let lit = ((time * 3.0 + y as f32 * 0.7 + x as f32 * 1.3).sin() > 0.6) as u8 as f32;
                    let c = note_color(x % 4);
                    bx(b, vec3(-0.25 + x as f32 * 0.1, 1.0 + y as f32 * 0.09, 0.185), vec3(0.08, 0.07, 0.01), k(Color::new(c.r * (0.25 + 0.75 * lit), c.g * (0.25 + 0.75 * lit), c.b * (0.25 + 0.75 * lit), 1.0)));
                }
            }
            b.unlit = false;
            bx(b, vec3(0.0, 0.04, 0.95), vec3(0.96, 0.08, 0.96), k(Color::from_rgba(170, 172, 184, 255)));
            for (lane, (px, pz)) in [(-0.3, 0.95), (0.0, 1.25), (0.0, 0.65), (0.3, 0.95)].iter().enumerate() {
                let c = note_color(lane);
                let glow = pads[lane];
                b.unlit = glow > 0.05;
                bx(b, vec3(*px, 0.085, *pz), vec3(0.24, 0.02, 0.24), k(Color::new(c.r * (0.45 + 0.55 * glow), c.g * (0.45 + 0.55 * glow), c.b * (0.45 + 0.55 * glow), 1.0)));
                b.unlit = false;
            }
            bx(b, vec3(0.0, 0.7, 0.5), vec3(0.9, 0.04, 0.04), k(Color::from_rgba(190, 190, 200, 255)));
            for side in [-0.45f32, 0.45] {
                bx(b, vec3(side, 0.36, 0.5), vec3(0.04, 0.68, 0.04), k(Color::from_rgba(190, 190, 200, 255)));
            }
        }
        Item::Tv => {
            let frame = k(Color::from_rgba(24, 24, 30, 255));
            bx(b, vec3(0.0, 0.98, -0.46), vec3(1.5, 0.82, 0.06), frame);
            let (w, h) = (24usize, 14usize);
            let v = vis();
            b.unlit = true;
            for y in 0..h {
                for x in 0..w {
                    let col = if piece.state == 0 {
                        if x == w / 2 && y == h / 2 { Some(Color::new(0.3, 0.3, 0.35, 1.0)) } else { None }
                    } else {
                        tv_pixel(piece.state, x, y, w, h, time, &v)
                    };
                    let col = col.unwrap_or(Color::new(0.04, 0.04, 0.06, 1.0));
                    bx(b, vec3(-0.69 + (x as f32 + 0.5) * (1.38 / w as f32), 1.34 - (y as f32 + 0.5) * (0.72 / h as f32), -0.425), vec3(1.38 / w as f32 * 0.92, 0.72 / h as f32 * 0.92, 0.01), k(col));
                }
            }
            b.unlit = false;
        }
        Item::Rug => {
            bx(b, vec3(0.0, 0.01, 0.0), vec3(1.8, 0.02, 1.8), k(Color::from_rgba(196, 92, 98, 255)));
            bx(b, vec3(0.0, 0.022, 0.0), vec3(1.4, 0.02, 1.4), k(Color::from_rgba(240, 200, 120, 255)));
            bx(b, vec3(0.0, 0.034, 0.0), vec3(0.9, 0.02, 0.9), k(Color::from_rgba(104, 150, 196, 255)));
            bx(b, vec3(0.0, 0.046, 0.0), vec3(0.4, 0.02, 0.4), k(Color::from_rgba(250, 240, 220, 255)));
        }
        Item::Bed => {
            bx(b, vec3(0.0, 0.13, 0.0), vec3(1.86, 0.26, 1.86), wood);
            bx(b, vec3(0.0, 0.33, 0.08), vec3(1.72, 0.12, 1.62), k(Color::from_rgba(245, 242, 235, 255)));
            bx(b, vec3(0.0, 0.42, 0.24), vec3(1.68, 0.07, 1.18), k(Color::from_rgba(130, 170, 220, 255)));
            bx(b, vec3(-0.46, 0.44, -0.62), vec3(0.54, 0.1, 0.34), k(Color::from_rgba(255, 252, 245, 255)));
            bx(b, vec3(0.46, 0.44, -0.62), vec3(0.54, 0.1, 0.34), k(Color::from_rgba(255, 252, 245, 255)));
            bx(b, vec3(0.0, 0.45, -0.92), vec3(1.9, 0.66, 0.1), wood_dark);
            bx(b, vec3(0.0, 0.18, 0.92), vec3(1.82, 0.18, 0.08), wood_dark);
        }
        Item::Lamp => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.3, 0.06, 0.3), wood_dark);
            bx(b, vec3(0.0, 0.5, 0.0), vec3(0.05, 0.9, 0.05), k(Color::from_rgba(200, 180, 140, 255)));
            let glow = 0.9 + 0.1 * (time * 3.0).sin();
            let shade = if piece.state == 0 { Color::new(1.0, 0.86 * glow, 0.55 * glow, 1.0) } else { Color::from_rgba(150, 136, 112, 255) };
            b.unlit = piece.state == 0;
            bx(b, vec3(0.0, 1.0, 0.0), vec3(0.42, 0.26, 0.42), k(shade));
            b.unlit = false;
        }
        Item::Plant => {
            bx(b, vec3(0.0, 0.14, 0.0), vec3(0.34, 0.28, 0.34), k(Color::from_rgba(196, 110, 74, 255)));
            let withered = piece.state == home::WITHERED;
            let grow = if withered { 0.85 } else { 1.0 + piece.state.min(3) as f32 * 0.22 };
            for (lx, ly, lz) in [(0.0, 0.42, 0.0), (0.12, 0.55, 0.05), (-0.1, 0.6, -0.06), (0.04, 0.72, -0.08), (-0.06, 0.5, 0.12)] {
                let sway = if withered { 0.0 } else { (time * 1.4 + lx * 9.0).sin() * 0.02 };
                let droop = if withered { (ly - 0.28) * 0.45 } else { 0.0 };
                let spread = if withered { 1.6 } else { 1.0 };
                let leaf = if withered { Color::from_rgba(150, 120, 70, 255) } else { Color::from_rgba(90, 160, 84, 255) };
                bx(b, vec3((lx + sway) * grow * spread, 0.28 + (ly - 0.28) * grow - droop, lz * grow * spread), vec3(0.2, 0.16, 0.2) * grow.sqrt(), k(leaf));
            }
            if piece.state == 3 {
                for (fx, fz, col) in [(0.08, 0.06, Color::from_rgba(255, 140, 170, 255)), (-0.12, -0.02, Color::from_rgba(255, 230, 120, 255)), (0.02, -0.12, Color::from_rgba(255, 255, 255, 255))] {
                    bx(b, vec3(fx * grow, 0.28 + 0.5 * grow, fz * grow), vec3(0.09, 0.09, 0.09), k(col));
                }
            }
        }
        Item::Bookshelf => {
            bx(b, vec3(0.0, 0.62, -0.2), vec3(0.9, 1.24, 0.5), wood);
            let books = [Color::from_rgba(200, 80, 80, 255), Color::from_rgba(80, 130, 200, 255), Color::from_rgba(230, 190, 90, 255), Color::from_rgba(110, 170, 110, 255), Color::from_rgba(180, 120, 200, 255)];
            for shelf in 0..3 {
                for n in 0..5 {
                    let h = 0.22 + ((n * 7 + shelf * 3) % 4) as f32 * 0.025;
                    bx(b, vec3(-0.32 + n as f32 * 0.16, 0.12 + shelf as f32 * 0.38 + h / 2.0, 0.06), vec3(0.12, h, 0.04), k(books[(n + shelf) % 5]));
                }
            }
        }
        Item::Sofa => {
            let fabric = k(Color::from_rgba(110, 150, 120, 255));
            let fabric_light = k(Color::from_rgba(140, 180, 146, 255));
            bx(b, vec3(0.0, 0.14, 0.05), vec3(1.86, 0.28, 0.8), fabric);
            bx(b, vec3(-0.45, 0.33, 0.1), vec3(0.86, 0.1, 0.62), fabric_light);
            bx(b, vec3(0.45, 0.33, 0.1), vec3(0.86, 0.1, 0.62), fabric_light);
            bx(b, vec3(0.0, 0.5, -0.32), vec3(1.86, 0.5, 0.16), fabric);
            bx(b, vec3(-0.9, 0.36, 0.05), vec3(0.14, 0.3, 0.8), fabric);
            bx(b, vec3(0.9, 0.36, 0.05), vec3(0.14, 0.3, 0.8), fabric);
        }
        Item::Table => {
            bx(b, vec3(0.0, 0.48, 0.0), vec3(1.78, 0.06, 0.82), wood);
            for (lx, lz) in [(-0.76, -0.3), (0.76, -0.3), (-0.76, 0.3), (0.76, 0.3), (-0.1, -0.3), (0.1, 0.3)] {
                bx(b, vec3(lx, 0.22, lz), vec3(0.06, 0.44, 0.06), wood_dark);
            }
            for lx in [-0.56, -0.18, 0.18, 0.56] {
                bx(b, vec3(lx, 0.52, 0.0), vec3(0.12, 0.03, 0.12), k(Color::from_rgba(245, 245, 240, 255)));
            }
            if piece.state == 0 {
                bx(b, vec3(-0.52, 0.535, 0.0), vec3(0.14, 0.08, 0.14), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, vec3(0.52, 0.535, 0.0), vec3(0.14, 0.08, 0.14), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, vec3(0.0, 0.51, 0.0), vec3(0.28, 0.03, 0.18), k(Color::from_rgba(240, 160, 170, 255)));
            } else {
                let food = [Color::from_rgba(170, 110, 60, 255), Color::from_rgba(240, 150, 80, 255), Color::from_rgba(200, 90, 70, 255), Color::from_rgba(250, 200, 70, 255), Color::from_rgba(150, 100, 70, 255), Color::from_rgba(180, 130, 90, 255), Color::from_rgba(210, 140, 100, 255), Color::from_rgba(240, 210, 120, 255)][(piece.state as usize - 1) % 8];
                bx(b, vec3(0.0, 0.505, 0.0), vec3(0.48, 0.02, 0.34), k(Color::from_rgba(245, 245, 240, 255)));
                bx(b, vec3(0.0, 0.54, 0.0), vec3(0.28, 0.06, 0.2), k(food));
                let steam = (time * 1.5).fract();
                bx(b, vec3((time * 3.0).sin() * 0.03, 0.6 + steam * 0.25, 0.0), vec3(0.05, 0.04, 0.05), k(Color::new(1.0, 1.0, 1.0, 0.6 * (1.0 - steam))));
            }
        }
        Item::RecordPlayer => {
            bx(b, vec3(0.0, 0.25, 0.0), vec3(0.8, 0.5, 0.6), wood_dark);
            bx(b, vec3(0.0, 0.52, 0.0), vec3(0.76, 0.04, 0.56), wood);
            let spin = if record_on { time * 3.5 } else { 0.0 };
            b.box_at(o + vec3(0.0, 0.0, 0.0), yaw + spin, vec3(0.0, 0.56, 0.0), vec3(0.42, 0.02, 0.42), k(Color::from_rgba(24, 22, 28, 255)));
            b.box_at(o, yaw + spin, vec3(0.0, 0.575, 0.0), vec3(0.12, 0.012, 0.12), k(Color::from_rgba(220, 70, 80, 255)));
            b.box_at(o, yaw + spin, vec3(0.13, 0.575, 0.0), vec3(0.04, 0.014, 0.04), k(Color::from_rgba(250, 230, 200, 255)));
            bx(b, vec3(0.28, 0.6, 0.12), vec3(0.04, 0.04, 0.3), k(Color::from_rgba(200, 200, 210, 255)));
        }
        Item::Piano => {
            let lacquer = k(Color::from_rgba(40, 34, 44, 255));
            bx(b, vec3(0.0, 0.55, -0.2), vec3(1.8, 1.1, 0.5), lacquer);
            bx(b, vec3(0.0, 0.42, 0.12), vec3(1.7, 0.08, 0.3), lacquer);
            bx(b, vec3(0.0, 0.47, 0.12), vec3(1.6, 0.03, 0.26), k(Color::from_rgba(250, 248, 240, 255)));
            for n in 0..11 {
                if n % 7 == 2 || n % 7 == 6 {
                    continue;
                }
                bx(b, vec3(-0.72 + n as f32 * 0.145, 0.495, 0.06), vec3(0.07, 0.03, 0.14), lacquer);
            }
            bx(b, vec3(0.0, 1.12, -0.2), vec3(1.84, 0.05, 0.54), k(Color::from_rgba(60, 52, 66, 255)));
            bx(b, vec3(0.5, 1.2, -0.25), vec3(0.08, 0.12, 0.08), k(Color::from_rgba(255, 220, 120, 255)));
        }
        Item::Drums => {
            bx(b, vec3(0.0, 0.22, -0.12), vec3(0.5, 0.44, 0.36), k(Color::from_rgba(200, 60, 70, 255)));
            bx(b, vec3(0.0, 0.22, 0.07), vec3(0.42, 0.36, 0.02), k(Color::from_rgba(245, 240, 230, 255)));
            bx(b, vec3(-0.3, 0.42, 0.2), vec3(0.26, 0.12, 0.26), k(Color::from_rgba(230, 230, 236, 255)));
            bx(b, vec3(-0.3, 0.2, 0.2), vec3(0.03, 0.36, 0.03), k(Color::from_rgba(170, 170, 180, 255)));
            let wobble = (time * 6.0).sin() * 0.02;
            bx(b, vec3(0.32, 0.8 + wobble, 0.05), vec3(0.36, 0.015, 0.36), k(Color::from_rgba(230, 196, 90, 255)));
            bx(b, vec3(0.32, 0.4, 0.05), vec3(0.03, 0.8, 0.03), k(Color::from_rgba(170, 170, 180, 255)));
        }
        Item::WindChime => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.26, 0.06, 0.26), wood_dark);
            bx(b, vec3(0.0, 0.7, 0.0), vec3(0.04, 1.36, 0.04), wood_dark);
            bx(b, vec3(0.12, 1.36, 0.0), vec3(0.28, 0.03, 0.04), wood_dark);
            for n in 0..4 {
                let sway = (time * 2.2 + n as f32 * 1.3).sin() * 0.03;
                let len = 0.22 + n as f32 * 0.05;
                bx(b, vec3(0.04 + n as f32 * 0.07 + sway, 1.3 - len / 2.0, 0.0), vec3(0.03, len, 0.03), k(Color::from_rgba(190, 210, 230, 255)));
            }
        }
        Item::Speaker => {
            bx(b, vec3(0.0, 0.5, 0.0), vec3(0.5, 1.0, 0.45), k(Color::from_rgba(44, 40, 52, 255)));
            bx(b, vec3(0.0, 0.34, 0.23), vec3(0.32, 0.32, 0.02), k(Color::from_rgba(80, 76, 92, 255)));
            bx(b, vec3(0.0, 0.34, 0.245), vec3(0.12, 0.12, 0.02), k(Color::from_rgba(26, 24, 30, 255)));
            bx(b, vec3(0.0, 0.74, 0.23), vec3(0.16, 0.16, 0.02), k(Color::from_rgba(80, 76, 92, 255)));
            let led = if piece.state == 0 { 0.6 + 0.4 * (time * 4.0).sin().abs() } else { 0.0 };
            bx(b, vec3(0.16, 0.92, 0.23), vec3(0.04, 0.04, 0.02), k(if piece.state == 0 { Color::new(0.4, 1.0 * led, 0.6, 1.0) } else { Color::from_rgba(60, 40, 40, 255) }));
        }
        Item::DjBooth => {
            let concrete = k(Color::from_rgba(120, 120, 124, 255));
            let plinth = k(Color::from_rgba(34, 34, 38, 255));
            bx(b, vec3(0.0, 0.4, 0.1), vec3(1.9, 0.8, 0.52), concrete);
            bx(b, vec3(0.0, 0.81, 0.06), vec3(1.84, 0.02, 0.46), k(Color::from_rgba(48, 48, 54, 255)));
            let spin = if record_on { time * 3.4 } else { 0.0 };
            for (n, side) in [-0.58_f32, 0.58].iter().enumerate() {
                let c = vec3(*side, 0.0, 0.06);
                bx(b, c + vec3(0.0, 0.85, 0.0), vec3(0.6, 0.06, 0.44), plinth);
                for kk in 0..8 {
                    let turn = kk as f32 * std::f32::consts::FRAC_PI_8 / 2.0;
                    b.box_at(o + rotate_y(c, yaw), yaw + turn, vec3(0.0, 0.89, 0.0), vec3(0.34, 0.025, 0.34), k(Color::from_rgba(70, 70, 76, 255)));
                    b.box_at(o + rotate_y(c, yaw), yaw + turn + spin + n as f32, vec3(0.0, 0.905, 0.0), vec3(0.31, 0.012, 0.31), k(Color::from_rgba(16, 16, 20, 255)));
                    b.box_at(o + rotate_y(c, yaw), yaw + turn + spin, vec3(0.0, 0.913, 0.0), vec3(0.1, 0.008, 0.1), k([Color::from_rgba(220, 60, 70, 255), Color::from_rgba(240, 200, 80, 255)][n]));
                }
                b.box_at(o + rotate_y(c, yaw), yaw + spin + n as f32 * 2.0, vec3(0.03, 0.918, 0.0), vec3(0.025, 0.006, 0.025), k(Color::from_rgba(250, 250, 250, 255)));
                bx(b, c + vec3(0.21, 0.92, -0.12), vec3(0.03, 0.03, 0.03), k(Color::from_rgba(190, 190, 200, 255)));
                bx(b, c + vec3(0.16, 0.925, -0.02), vec3(0.02, 0.012, 0.22), k(Color::from_rgba(190, 190, 200, 255)));
                bx(b, c + vec3(-0.22, 0.89, 0.16), vec3(0.06, 0.02, 0.03), k(Color::from_rgba(60, 200, 120, 255)));
            }
            bx(b, vec3(0.0, 0.87, 0.06), vec3(0.42, 0.06, 0.42), k(Color::from_rgba(28, 28, 32, 255)));
            for row in 0..3 {
                for col in 0..3 {
                    bx(b, vec3(-0.11 + col as f32 * 0.11, 0.91, -0.1 + row as f32 * 0.07), vec3(0.04, 0.025, 0.04), k(Color::from_rgba(150, 150, 160, 255)));
                }
            }
            for col in 0..2 {
                let fade = (time * (1.1 + col as f32 * 0.7)).sin() * 0.04;
                bx(b, vec3(-0.06 + col as f32 * 0.12, 0.905, 0.14 + fade), vec3(0.03, 0.02, 0.05), k(Color::from_rgba(230, 230, 236, 255)));
            }
            bx(b, vec3(0.0, 0.905, 0.21), vec3(0.16, 0.015, 0.02), k(Color::from_rgba(230, 230, 236, 255)));
            for n in 0..6 {
                let level = (pulse.max(0.0) * 6.0) as i32;
                let lit = if n < level { 1.0 } else { 0.18 };
                let col = if n >= 4 { Color::new(1.0 * lit, 0.2 * lit, 0.2 * lit, 1.0) } else { Color::new(0.3 * lit, 1.0 * lit, 0.45 * lit, 1.0) };
                bx(b, vec3(0.17, 0.9, -0.12 + n as f32 * 0.035), vec3(0.025, 0.012, 0.025), k(col));
            }
            bx(b, vec3(0.0, 0.78, 0.365), vec3(1.7, 0.025, 0.012), k(Color::new(0.9, 0.1, 0.12, 1.0)));
        }
        Item::Subwoofer => {
            let cab = k(Color::from_rgba(26, 26, 30, 255));
            let cone = k(Color::from_rgba(52, 52, 58, 255));
            let pump = if piece.state == 0 { 0.025 * pulse.max(0.0) } else { 0.0 };
            for level in 0..2 {
                let y = 0.32 + level as f32 * 0.64;
                bx(b, vec3(0.0, y, 0.0), vec3(0.8, 0.62, 0.7), cab);
                bx(b, vec3(0.0, y, 0.36 + pump), vec3(0.5, 0.5, 0.02), cone);
                bx(b, vec3(0.0, y, 0.375 + pump * 1.5), vec3(0.18, 0.18, 0.02), k(Color::from_rgba(18, 18, 20, 255)));
            }
        }
        Item::RedLight => {
            bx(b, vec3(0.0, 0.03, 0.0), vec3(0.24, 0.06, 0.24), k(Color::from_rgba(40, 40, 44, 255)));
            bx(b, vec3(0.0, 0.75, 0.0), vec3(0.04, 1.44, 0.04), k(Color::from_rgba(40, 40, 44, 255)));
            let glow = if piece.state == 0 { 0.8 + 0.2 * pulse.max(0.0) } else { 0.3 };
            b.unlit = piece.state == 0;
            bx(b, vec3(0.0, 1.46, 0.0), vec3(0.8, 0.06, 0.06), k(Color::new(glow, 0.12 * glow, 0.14 * glow, 1.0)));
            b.unlit = false;
        }
        Item::ConcreteBench => {
            let concrete = k(Color::from_rgba(132, 132, 136, 255));
            bx(b, vec3(0.0, 0.2, 0.0), vec3(1.9, 0.4, 0.6), concrete);
            bx(b, vec3(0.0, 0.41, 0.0), vec3(1.9, 0.02, 0.6), k(Color::from_rgba(146, 146, 150, 255)));
            for n in 0..3 {
                bx(b, vec3(-0.6 + n as f32 * 0.6, 0.2, 0.301), vec3(0.03, 0.03, 0.01), k(Color::from_rgba(96, 96, 100, 255)));
            }
        }
        Item::SteelPillar => {
            let steel = k(Color::from_rgba(92, 94, 102, 255));
            bx(b, vec3(0.0, 0.85, 0.0), vec3(0.34, 1.7, 0.34), steel);
            bx(b, vec3(0.0, 0.04, 0.0), vec3(0.5, 0.08, 0.5), k(Color::from_rgba(70, 72, 78, 255)));
            bx(b, vec3(0.0, 1.66, 0.0), vec3(0.5, 0.08, 0.5), k(Color::from_rgba(70, 72, 78, 255)));
            for n in 0..4 {
                bx(b, vec3(0.0, 0.3 + n as f32 * 0.38, 0.175), vec3(0.04, 0.04, 0.01), k(Color::from_rgba(130, 132, 140, 255)));
            }
        }
    }
}

fn draw_home(b: &mut Batch, home: &home::Home, time: f32, ghost: Option<(home::Piece, bool)>, pulse: f32, beat_on: bool) {
    let ox = home::ORIGIN.x;
    let oz = home::ORIGIN.y;
    let w = home.w() as f32;
    let d = home.d() as f32;
    let loft = home.loft();
    let lh = home::LOFT_H;
    let door_x = home.door_x() as f32;
    let (_, plank_a, plank_b) = home::FLOORS[(home.floor as usize).min(home::FLOORS.len() - 1)];
    let checker = home.floor as usize == 2;
    let concrete_floor = home.floor == home::CONCRETE_FLOOR;
    for z in 0..home.d() {
        for x in 0..home.w() {
            let alt = if checker { (x + z) % 2 == 0 } else if concrete_floor { hash(x, z, 77) > 0.5 } else { z % 2 == 0 };
            let up = if home.level(x, z) == home::Level::Loft { lh } else { 0.0 };
            b.top(vec3(ox + x as f32 + 0.5, home::FLOOR_Y + up, oz + z as f32 + 0.5), if alt { plank_a } else { plank_b });
        }
    }
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y - 0.15, oz + d + 0.05), vec3(w + 0.2, 0.3, 0.1), tinted(plank_b, -0.25));
    let wall = home::wall_color(home);
    let trim = tinted(wall, -0.22);
    let wall_h = if loft { 1.7 + lh + 0.4 } else { 1.7 };
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + wall_h / 2.0, oz - 0.06), vec3(w + 0.24, wall_h, 0.12), wall);
    b.cube(vec3(ox - 0.06, home::FLOOR_Y + wall_h / 2.0, oz + d / 2.0), vec3(0.12, wall_h, d), wall);
    b.cube(vec3(ox + w + 0.06, home::FLOOR_Y + wall_h / 2.0, oz + d / 2.0), vec3(0.12, wall_h, d), wall);
    b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + 0.12, oz + 0.01), vec3(w, 0.24, 0.04), trim);
    b.cube(vec3(ox + 0.03, home::FLOOR_Y + 0.12, oz + d / 2.0), vec3(0.04, 0.24, d), trim);
    b.cube(vec3(ox + w - 0.03, home::FLOOR_Y + 0.12, oz + d / 2.0), vec3(0.04, 0.24, d), trim);
    for (x0, x1) in [(0.0_f32, door_x), (door_x + 1.0, w)] {
        b.cube(vec3(ox + (x0 + x1) / 2.0, home::FLOOR_Y + 0.1, oz + d - 0.04), vec3(x1 - x0, 0.2, 0.08), trim);
    }
    if home.wall == home::CONCRETE_WALL {
        let seam = tinted(wall, -0.12);
        let hole = tinted(wall, -0.35);
        let rows = ((wall_h - 0.3) / 0.18) as i32;
        for k in 1..rows {
            let y = home::FLOOR_Y + 0.24 + k as f32 * 0.18;
            b.cube(vec3(ox + w / 2.0, y, oz + 0.005), vec3(w, 0.012, 0.02), seam);
            b.cube(vec3(ox + 0.005, y, oz + d / 2.0), vec3(0.02, 0.012, d), seam);
            b.cube(vec3(ox + w - 0.005, y, oz + d / 2.0), vec3(0.02, 0.012, d), seam);
        }
        for row in 0..((wall_h - 0.5) / 0.45) as i32 + 1 {
            let y = home::FLOOR_Y + 0.5 + row as f32 * 0.45;
            for col in 0..home.w() {
                b.cube(vec3(ox + 0.5 + col as f32, y, oz + 0.01), vec3(0.05, 0.05, 0.02), hole);
            }
            for col in 0..home.d() {
                b.cube(vec3(ox + 0.012, y, oz + 0.5 + col as f32), vec3(0.02, 0.05, 0.05), hole);
                b.cube(vec3(ox + w - 0.012, y, oz + 0.5 + col as f32), vec3(0.02, 0.05, 0.05), hole);
            }
        }
    }
    if home.floor == home::CONCRETE_FLOOR {
        let seam = tinted(plank_b, -0.2);
        let mut x = 2.0_f32;
        while x < w {
            b.cube(vec3(ox + x, home::FLOOR_Y + 0.003, oz + d / 2.0), vec3(0.02, 0.004, d), seam);
            x += 2.0;
        }
        let mut z = 2.0_f32;
        while z < d {
            b.cube(vec3(ox + w / 2.0, home::FLOOR_Y + 0.003, oz + z), vec3(w, 0.004, 0.02), seam);
            z += 2.0;
        }
    }
    b.cube(vec3(ox + door_x + 0.5, home::FLOOR_Y + 0.01, oz + d + 0.15), vec3(0.8, 0.02, 0.3), Color::from_rgba(204, 160, 110, 255));
    let deco_up = if loft { lh } else { 0.0 };
    let window_x = ox + 4.6;
    let brutal = home.wall == home::CONCRETE_WALL;
    if loft {
        let x0 = home::LOFT_X0 as f32;
        let ld = home.loft_d() as f32;
        let face = tinted(plank_b, -0.18);
        b.cube(vec3(ox + (x0 + w) / 2.0, home::FLOOR_Y + lh / 2.0 - 0.005, oz + ld / 2.0), vec3(w - x0, lh - 0.01, ld), face);
        b.cube(vec3(ox + (x0 + w) / 2.0, home::FLOOR_Y + lh - 0.04, oz + ld + 0.005), vec3(w - x0, 0.08, 0.04), tinted(face, -0.2));
        let metal = Color::from_rgba(40, 40, 46, 255);
        let glass = Color::new(0.75, 0.88, 1.0, 0.28);
        let sx = home.stair_x() as f32;
        let rail_y = home::FLOOR_Y + lh;
        b.cube(vec3(ox + (x0 + sx) / 2.0, rail_y + 0.5, oz + ld - 0.03), vec3(sx - x0, 0.04, 0.04), metal);
        b.cube(vec3(ox + (x0 + sx) / 2.0, rail_y + 0.25, oz + ld - 0.03), vec3(sx - x0, 0.46, 0.02), glass);
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.5, oz + ld / 2.0), vec3(0.04, 0.04, ld), metal);
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.25, oz + ld / 2.0), vec3(0.02, 0.46, ld), glass);
        let mut px = x0;
        while px <= sx + 0.01 {
            b.cube(vec3(ox + px, rail_y + 0.26, oz + ld - 0.03), vec3(0.05, 0.52, 0.05), metal);
            px += 1.0;
        }
        b.cube(vec3(ox + x0 + 0.03, rail_y + 0.26, oz + 0.05), vec3(0.05, 0.52, 0.05), metal);
        let steps = 6;
        let len = home::STAIR_LEN as f32;
        let tread = tinted(plank_a, 0.04);
        let riser = tinted(plank_b, -0.15);
        for i in 0..steps {
            let h = lh * (i + 1) as f32 / steps as f32;
            let zc = ld + len - (i as f32 + 0.5) * len / steps as f32;
            b.cube(vec3(ox + sx + 0.5, home::FLOOR_Y + h / 2.0, oz + zc), vec3(0.9, h, len / steps as f32), riser);
            b.cube(vec3(ox + sx + 0.5, home::FLOOR_Y + h + 0.01, oz + zc), vec3(0.9, 0.02, len / steps as f32 - 0.02), tread);
        }
        for i in 0..=steps {
            let h = lh * i as f32 / steps as f32;
            let zc = ld + len - i as f32 * len / steps as f32;
            b.cube(vec3(ox + sx + 0.06, home::FLOOR_Y + h + 0.5, oz + zc), vec3(0.04, 0.04, len / steps as f32 + 0.02), metal);
        }
        let sk = sky();
        let win_bottom = home::FLOOR_Y + lh + 0.22;
        let win_h = 1.78;
        let sky_y = win_bottom + win_h / 2.0;
        let sky_w = w - x0 - 0.4;
        let sky_cx = ox + x0 + 0.2 + sky_w / 2.0;
        b.cube(vec3(sky_cx, sky_y, oz + 0.015), vec3(sky_w + 0.12, win_h + 0.1, 0.04), metal);
        let night = sk.night;
        let glow = sk.glow;
        let day_top = Color::from_rgba(96, 156, 220, 255);
        let day_low = Color::from_rgba(186, 216, 236, 255);
        let night_top = Color::from_rgba(14, 16, 40, 255);
        let night_low = Color::from_rgba(40, 36, 84, 255);
        let dusk_low = Color::from_rgba(255, 140, 96, 255);
        let grey = [0.0, 0.35, 0.55, 0.3][sk.weather];
        b.unlit = true;
        let bands = 10;
        for k in 0..bands {
            let t = k as f32 / (bands - 1) as f32;
            let base = mix(mix(day_top, day_low, t), mix(night_top, night_low, t), night);
            let warm = mix(base, mix(Color::from_rgba(250, 170, 140, 255), dusk_low, t), glow * (0.3 + 0.7 * t));
            let col = mix(warm, mix(Color::from_rgba(150, 156, 168, 255), Color::from_rgba(40, 42, 56, 255), night), grey);
            let bh = win_h / bands as f32;
            b.cube(vec3(sky_cx, win_bottom + win_h - (k as f32 + 0.5) * bh, oz + 0.03), vec3(sky_w, bh + 0.002, 0.02), col);
        }
        let day_part = ((sk.hour - 6.0) / 12.0).rem_euclid(1.0);
        if night < 0.6 && sk.weather != 2 {
            let sx = sky_cx - sky_w / 2.0 + sky_w * (0.1 + 0.8 * day_part);
            let sy = win_bottom + 0.75 + (day_part * PI).sin() * 0.8;
            let sun = mix(Color::from_rgba(255, 244, 190, 255), Color::from_rgba(255, 150, 90, 255), glow);
            b.cube(vec3(sx, sy, oz + 0.04), vec3(0.2, 0.2, 0.01), Color::new(sun.r, sun.g, sun.b, if sk.weather == 1 { 0.5 } else { 1.0 }));
        } else if night >= 0.6 {
            let mx = sky_cx - sky_w / 2.0 + sky_w * (0.2 + 0.6 * ((sk.hour + 6.0) / 24.0).rem_euclid(1.0));
            b.cube(vec3(mx, win_bottom + 1.45, oz + 0.04), vec3(0.15, 0.15, 0.01), Color::from_rgba(240, 240, 220, 255));
            if sk.weather == 0 {
                for k in 0..((sky_w * 4.0) as i32) {
                    let tw = 0.5 + 0.5 * (time * (1.0 + hash(k, 1, 9) * 2.0) + k as f32).sin();
                    b.cube(vec3(sky_cx - sky_w / 2.0 + hash(k, 2, 9) * sky_w, win_bottom + 1.0 + hash(k, 3, 9) * 0.7, oz + 0.04), vec3(0.03, 0.03, 0.01), Color::new(1.0, 1.0, 0.9, tw * night));
                }
            }
        }
        let clouds = [1, 4, 6, 4][sk.weather];
        for k in 0..clouds {
            let cx = sky_cx - sky_w / 2.0 + ((hash(k, 4, 12) * sky_w + time * 0.05 * (1.0 + hash(k, 5, 12))) % sky_w);
            let cy = win_bottom + 1.2 + hash(k, 6, 12) * 0.45;
            let cc = mix(mix(Color::from_rgba(250, 250, 255, 255), Color::from_rgba(120, 124, 140, 255), grey), Color::from_rgba(50, 50, 72, 255), night * 0.8);
            let cc = mix(cc, Color::from_rgba(255, 180, 160, 255), glow * 0.5);
            b.cube(vec3(cx, cy, oz + 0.042), vec3(0.6, 0.12, 0.01), cc);
            b.cube(vec3(cx + 0.08, cy + 0.08, oz + 0.042), vec3(0.32, 0.1, 0.01), cc);
        }
        let water_h = 0.22;
        let water = mix(mix(Color::from_rgba(70, 120, 170, 255), Color::from_rgba(16, 22, 46, 255), night), Color::from_rgba(220, 120, 90, 255), glow * 0.4);
        b.cube(vec3(sky_cx, win_bottom + water_h / 2.0, oz + 0.05), vec3(sky_w, water_h, 0.02), water);
        let facade_day = Color::from_rgba(120, 128, 150, 255);
        let facade_night = Color::from_rgba(20, 22, 44, 255);
        let n = (sky_w * 3.2) as i32;
        for k in 0..n {
            let bx = sky_cx - sky_w / 2.0 + 0.14 + k as f32 * (sky_w - 0.28) / (n - 1).max(1) as f32;
            let tall = 0.3 + hash(k, 3, 41) * 0.75;
            let base_y = win_bottom + water_h;
            let tint = 0.85 + hash(k, 8, 41) * 0.3;
            let facade = mix(Color::new(facade_day.r * tint, facade_day.g * tint, facade_day.b * tint, 1.0), facade_night, night);
            let facade = mix(facade, Color::from_rgba(200, 120, 110, 255), glow * 0.35);
            b.cube(vec3(bx, base_y + tall / 2.0, oz + 0.055), vec3(0.27, tall, 0.02), facade);
            let rows = (tall / 0.12) as i32;
            for r in 0..rows {
                for c in 0..2 {
                    let lit_chance = 0.15 + night * 0.6;
                    if hash(k * 7 + c, r, 61) < lit_chance {
                        let flick = 0.8 + 0.2 * (time * 0.3 + (k * 13 + r) as f32).sin();
                        let lamp = Color::new(1.0, 0.86 * flick, 0.5 * flick, 0.35 + 0.65 * night);
                        b.cube(vec3(bx - 0.06 + c as f32 * 0.12, base_y + 0.08 + r as f32 * 0.12, oz + 0.068), vec3(0.05, 0.05, 0.01), lamp);
                    }
                }
            }
        }
        let crane = mix(Color::from_rgba(200, 80, 60, 255), Color::from_rgba(30, 28, 50, 255), night);
        for (k, fx) in [0.18f32, 0.72].iter().enumerate() {
            let cx = sky_cx - sky_w / 2.0 + sky_w * fx;
            let base_y = win_bottom + water_h;
            b.cube(vec3(cx, base_y + 0.5, oz + 0.06), vec3(0.04, 1.0, 0.01), crane);
            b.cube(vec3(cx + if k == 0 { 0.25 } else { -0.25 }, base_y + 0.98, oz + 0.06), vec3(0.7, 0.035, 0.01), crane);
            if night > 0.5 && (time * 1.5 + k as f32).sin() > 0.0 {
                b.cube(vec3(cx, base_y + 1.03, oz + 0.065), vec3(0.04, 0.04, 0.01), Color::from_rgba(255, 60, 60, 255));
            }
        }
        match sk.weather {
            2 => {
                for k in 0..((sky_w * 14.0) as i32) {
                    let x = sky_cx - sky_w / 2.0 + hash(k, 1, 71) * sky_w;
                    let y = win_bottom + (hash(k, 2, 71) * win_h - time * 2.2).rem_euclid(win_h);
                    b.cube(vec3(x, y, oz + 0.075), vec3(0.012, 0.1, 0.01), Color::new(0.75, 0.82, 0.95, 0.7));
                }
            }
            3 => {
                for k in 0..((sky_w * 10.0) as i32) {
                    let x = sky_cx - sky_w / 2.0 + hash(k, 1, 73) * sky_w + (time + k as f32).sin() * 0.04;
                    let y = win_bottom + (hash(k, 2, 73) * win_h - time * 0.25).rem_euclid(win_h);
                    b.cube(vec3(x, y, oz + 0.075), vec3(0.03, 0.03, 0.01), Color::new(1.0, 1.0, 1.0, 0.95));
                }
            }
            _ => {}
        }
        b.unlit = false;
        for k in 1..((sky_w) as i32 + 1) {
            b.cube(vec3(sky_cx - sky_w / 2.0 + k as f32, sky_y, oz + 0.08), vec3(0.04, win_h, 0.02), metal);
        }
        b.cube(vec3(sky_cx, win_bottom + win_h * 0.62, oz + 0.08), vec3(sky_w, 0.03, 0.02), metal);
        b.cube(vec3(ox + 1.0, home::FLOOR_Y + 1.15, oz + 0.02), vec3(0.6, 0.45, 0.04), Color::from_rgba(120, 84, 50, 255));
        b.cube(vec3(ox + 1.0, home::FLOOR_Y + 1.15, oz + 0.04), vec3(0.5, 0.35, 0.04), Color::from_rgba(240, 190, 120, 255));
    }
    if brutal && !loft {
        b.cube(vec3(window_x, home::FLOOR_Y + 1.25 + deco_up, oz + 0.02), vec3(1.4, 0.18, 0.05), Color::from_rgba(40, 40, 46, 255));
        b.cube(vec3(window_x, home::FLOOR_Y + 1.25 + deco_up, oz + 0.04), vec3(1.3, 0.1, 0.05), Color::from_rgba(70, 80, 96, 255));
    }
    if !brutal && !loft {
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.01), vec3(1.0, 0.7, 0.05), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.03), vec3(0.86, 0.56, 0.05), Color::from_rgba(150, 196, 226, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 1.0, oz + 0.05), vec3(0.04, 0.56, 0.04), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(window_x, home::FLOOR_Y + 0.62, oz + 0.08), vec3(1.06, 0.06, 0.12), Color::from_rgba(150, 100, 60, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.1, oz + 0.02), vec3(0.5, 0.4, 0.04), Color::from_rgba(120, 84, 50, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.1, oz + 0.04), vec3(0.4, 0.3, 0.04), Color::from_rgba(240, 190, 120, 255));
    b.cube(vec3(ox + 2.2, home::FLOOR_Y + 1.06, oz + 0.06), vec3(0.16, 0.12, 0.04), Color::from_rgba(250, 120, 140, 255));
    }
    let (cx, cz) = home::CATALOGUE;
    let cat_pos = vec3(ox + cx as f32 + 0.5, home::FLOOR_Y, oz + cz as f32 + 0.5);
    b.cube(cat_pos + vec3(0.0, 0.3, 0.0), vec3(0.5, 0.6, 0.4), Color::from_rgba(150, 102, 64, 255));
    b.box_at(cat_pos, 0.0, vec3(0.0, 0.64, 0.05), vec3(0.56, 0.06, 0.4), Color::from_rgba(196, 72, 72, 255));
    b.cube(cat_pos + vec3(-0.12, 0.68, 0.05), vec3(0.24, 0.03, 0.34), Color::from_rgba(252, 248, 236, 255));
    b.cube(cat_pos + vec3(0.12, 0.68, 0.05), vec3(0.24, 0.03, 0.34), Color::from_rgba(246, 240, 226, 255));
    let bounce = (time * 3.0).sin() * 0.03;
    b.cube(cat_pos + vec3(0.0, 0.95 + bounce, 0.05), vec3(0.08, 0.08, 0.08), ACORN_COLOR);
    b.cube(cat_pos + vec3(0.0, 1.0 + bounce, 0.05), vec3(0.1, 0.03, 0.1), Color::from_rgba(120, 84, 50, 255));
    let mut order: Vec<&home::Piece> = home.pieces.iter().collect();
    order.sort_by_key(|p| if p.item == home::Item::Rug { 0 } else { 1 });
    for piece in order {
        let spinning = if piece.item == home::Item::DjBooth { beat_on } else { home.record_on };
        draw_piece(b, piece, time, 1.0, spinning, pulse, home.piece_lift(piece));
    }
    if let Some((piece, valid)) = ghost {
        let (fw, fd) = piece.footprint();
        let col = if valid { Color::new(0.4, 1.0, 0.55, 0.35) } else { Color::new(1.0, 0.35, 0.4, 0.4) };
        let lift = if home.level(piece.x, piece.z) == home::Level::Loft { home::LOFT_H } else { 0.0 };
        for z in piece.z..piece.z + fd {
            for x in piece.x..piece.x + fw {
                b.cube(vec3(ox + x as f32 + 0.5, home::FLOOR_Y + lift + 0.012, oz + z as f32 + 0.5), vec3(0.96, 0.02, 0.96), col);
            }
        }
        draw_piece(b, &piece, time, 0.6, false, 0.0, lift);
    }
}

fn booth_pov(home: &home::Home, side: f32) -> Option<(Vec3, Vec3)> {
    let booth = home.pieces.iter().find(|p| p.item == home::Item::DjBooth)?;
    let c = booth.center();
    let base = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + home.piece_lift(booth), home::ORIGIN.y + c.y);
    let yaw = booth.yaw();
    let eye = base + rotate_y(vec3(side * 0.42, 1.55, 1.05), yaw);
    let target = base + rotate_y(vec3(side * 0.22, 0.84, 0.02), yaw);
    Some((eye, target))
}

fn render_home(target: &RenderTarget, left: f32, width: f32, height: f32, view: &View, home: &home::Home, ghost: Option<(home::Piece, bool)>, pov: Option<(Vec3, Vec3)>, hide: Option<usize>, beat_on: bool) {
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / target.texture.width(), -2.0 / height),
        target: vec2(target.texture.width() / 2.0, height / 2.0),
        ..Default::default()
    });
    draw_rectangle(left, 0.0, width, height, Color::from_rgba(38, 30, 52, 255));
    let center = vec3(home::ORIGIN.x + home.w() as f32 / 2.0, 0.5, home::ORIGIN.y + home.d() as f32 / 2.0);
    let scale = (home.w() as f32 / 7.0).max(home.d() as f32 / 5.0);
    let raise = if home.loft() { 0.35 } else { 0.0 };
    let (eye, look_at, half_fov) = match pov {
        Some((e, t)) => (e, t, 30.0_f32),
        None => (center + vec3(0.0, 5.6 * scale + raise, 6.2 * scale), center + vec3(0.0, 0.1 + raise, 0.2), 21.0),
    };
    let aspect = width / height;
    let eye = if pov.is_some() && aspect < 1.0 { look_at + (eye - look_at) * (0.8 / aspect).clamp(1.0, 1.6) } else { eye };
    let base = (half_fov.to_radians()).tan();
    let fovy = if aspect < 4.0 / 3.0 { 2.0 * (base * (4.0 / 3.0) / aspect).min(48f32.to_radians().tan()).atan() } else { 2.0 * base.atan() };
    set_camera(&Camera3D {
        position: eye,
        target: look_at,
        up: Vec3::Y,
        fovy,
        aspect: Some(aspect),
        render_target: Some(target.clone()),
        viewport: Some((left as i32, 0, width as i32, height as i32)),
        ..Default::default()
    });
    let flicker = 0.95 + 0.05 * (view.time * 7.0).sin();
    let throb = 0.85 + 0.15 * view.dance.max(0.0);
    let mut lamps = Vec::new();
    let mut red = false;
    for piece in home.pieces.iter().filter(|p| p.state == 0) {
        let c = piece.center();
        let lift = home.piece_lift(piece);
        match piece.item {
            home::Item::Lamp => lamps.push((vec3(home::ORIGIN.x + c.x, 1.4 + lift, home::ORIGIN.y + c.y), 1.0 * flicker, vec3(0.95, 0.58, 0.22))),
            home::Item::MirrorBall => {
                for n in 0..3 {
                    let a = view.time * 0.7 + n as f32 * 2.09;
                    let col = hsv(n as f32 / 3.0 + view.time * 0.05, 0.6, 1.0);
                    lamps.push((vec3(home::ORIGIN.x + c.x + a.cos() * 1.6, 0.6 + lift, home::ORIGIN.y + c.y + a.sin() * 1.6), 0.6, vec3(col.r, col.g, col.b)));
                }
            }
            home::Item::RedLight => {
                red = true;
                lamps.push((vec3(home::ORIGIN.x + c.x, 1.3 + lift, home::ORIGIN.y + c.y), 1.5 * throb, vec3(1.1, 0.08, 0.1)));
            }
            _ => {}
        }
    }
    let night = sky().night;
    let (strength, tint, dusk) = if red {
        (0.0, vec3(1.1, 0.08, 0.1), 0.92)
    } else if !lamps.is_empty() {
        (0.0, vec3(0.95, 0.58, 0.22), 0.22 + 0.25 * night)
    } else if home.wall == home::CONCRETE_WALL {
        (0.45, vec3(0.6, 0.62, 0.7), 0.3 + 0.4 * night)
    } else {
        (0.55, vec3(0.95, 0.58, 0.22), 0.2 + 0.5 * night)
    };
    let light = Light {
        dusk,
        fire: center + vec3(0.0, 1.8, 0.0),
        fire_strength: strength * flicker,
        tint,
        lamps,
    };
    let mut b = Batch::new(if pov.is_some() { eye } else { center + vec3(0.0, 5.6, -1.8) }, light);
    draw_home(&mut b, home, view.time, ghost, view.dance, beat_on);
    for (k, p) in view.players.iter().enumerate() {
        if hide == Some(k) || pov.map(|(e, _)| vec2(e.x, e.z).distance(p.pos) < 1.0).unwrap_or(false) {
            continue;
        }
        p.draw(&mut b, view.world, view.time, view.dance);
    }
    for &(p, life) in &view.extras.hearts {
        let c = Color::new(1.0, 0.45, 0.6, life.min(1.0));
        b.cube(p + vec3(-0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.035, 0.03, 0.0), vec3(0.06, 0.06, 0.03), c);
        b.cube(p + vec3(0.0, -0.02, 0.0), vec3(0.06, 0.06, 0.03), c);
    }
    b.flush();
}

fn shop_row_info(row: home::Row, house: &home::Home) -> (String, String, String) {
    match row {
        home::Row::Upgrade => match house.next_upgrade() {
            Some((name, what, price)) => (format!("+ {}", name), format!("{} acorns", price), what.to_string()),
            None => ("home fully built".to_string(), "done".to_string(), "penthouse with loft".to_string()),
        },
        home::Row::Item(item) => (item.name().to_string(), format!("{} acorns", item.price()), item.note().to_string()),
        home::Row::Wall(i) => {
            let bit = home::wall_bit(i);
            let owned = house.owned_styles & bit != 0;
            let price = if house.wall as usize == i { "in use".to_string() } else if owned { "owned".to_string() } else { format!("{} acorns", home::STYLE_PRICE) };
            (home::WALLS[i].0.to_string(), price, "repaint the walls".to_string())
        }
        home::Row::Floor(i) => {
            let bit = home::floor_bit(i);
            let owned = house.owned_styles & bit != 0;
            let price = if house.floor as usize == i { "in use".to_string() } else if owned { "owned".to_string() } else { format!("{} acorns", home::STYLE_PRICE) };
            (home::FLOORS[i].0.to_string(), price, "change the floor pattern".to_string())
        }
        home::Row::Set => ("techno club set".to_string(), "free".to_string(), "concrete, red light, techno".to_string()),
    }
}

fn wrap_px(text: &str, size: u16, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
        if !line.is_empty() && measure_text(&candidate, None, size, 1.0).width > width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn paint_page(target: &RenderTarget, book: &books::Book, page: usize, left: bool) {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / w, 2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        ..Default::default()
    });
    clear_background(Color::from_rgba(247, 239, 219, 255));
    for i in 0..24 {
        let a = 0.016 * (24 - i) as f32;
        let x = if left { w - (i + 1) as f32 * 2.0 } else { i as f32 * 2.0 };
        draw_rectangle(x, 0.0, 2.0, h, Color::new(0.4, 0.28, 0.16, a));
    }
    let ink = Color::from_rgba(52, 38, 32, 255);
    let soft = Color::from_rgba(150, 128, 108, 255);
    let accent = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    let margin = 44.0;
    draw_text(book.title, margin, 54.0, 24.0, soft);
    draw_rectangle(margin, 68.0, 64.0, 5.0, accent);
    let (heading, body) = book.pages[page % book.pages.len()];
    let mut y = 128.0;
    for line in wrap_px(heading, 44, w - margin * 2.0) {
        draw_text(&line, margin, y, 44.0, ink);
        y += 44.0;
    }
    y += 18.0;
    for line in wrap_px(body, 30, w - margin * 2.0) {
        draw_text(&line, margin, y, 30.0, Color::from_rgba(78, 62, 50, 255));
        y += 37.0;
    }
    let dots = (page * 7 + 3) % 5 + 3;
    for d in 0..dots {
        let dx = margin + d as f32 * 22.0;
        draw_rectangle(dx, h - 92.0 - ((d * 13 + page * 5) % 4) as f32 * 8.0, 12.0, 12.0, Color::new(accent.r, accent.g, accent.b, 0.55));
    }
    let number = format!("{}", page + 1);
    let nx = if left { margin } else { w - margin - measure_text(&number, None, 24, 1.0).width };
    draw_text(&number, nx, h - 34.0, 24.0, soft);
}

fn paint_spine(target: &RenderTarget, book: &books::Book) {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / w, 2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        ..Default::default()
    });
    let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    clear_background(c);
    let band = Color::new(c.r * 0.7, c.g * 0.7, c.b * 0.7, 1.0);
    draw_rectangle(0.0, 0.0, 26.0, h, band);
    draw_rectangle(w - 26.0, 0.0, 26.0, h, band);
    let size = 54u16;
    let tw = measure_text(book.title, None, size, 1.0).width;
    draw_text(book.title, (w - tw) / 2.0, h / 2.0 + 18.0, size as f32, Color::from_rgba(255, 248, 230, 255));
}

fn reading_light() -> Light {
    Light {
        dusk: 0.15,
        fire: vec3(0.8, 1.2, -0.5),
        fire_strength: 0.55,
        tint: vec3(1.0, 0.75, 0.45),
        lamps: Vec::new(),
    }
}

fn draw_shelf_scene(sel: usize, time: f32, spines: &[RenderTarget]) {
    let eye = vec3(0.0, 0.62, 1.35);
    set_camera(&Camera3D {
        position: eye,
        target: vec3(0.0, 0.42, 0.0),
        up: Vec3::Y,
        fovy: 0.85,
        ..Default::default()
    });
    clear_background(Color::from_rgba(38, 28, 34, 255));
    let mut b = Batch::new(eye, reading_light());
    let wood = Color::from_rgba(140, 92, 58, 255);
    let dark = Color::from_rgba(96, 62, 40, 255);
    b.cube(vec3(0.0, 0.55, -0.32), vec3(1.7, 1.3, 0.04), dark);
    b.cube(vec3(0.0, 0.07, 0.0), vec3(1.7, 0.06, 0.66), wood);
    b.cube(vec3(0.0, 1.02, 0.0), vec3(1.7, 0.05, 0.66), wood);
    b.cube(vec3(-0.83, 0.55, 0.0), vec3(0.05, 1.0, 0.66), wood);
    b.cube(vec3(0.83, 0.55, 0.0), vec3(0.05, 1.0, 0.66), wood);
    b.cube(vec3(0.72, 0.15, -0.05), vec3(0.12, 0.1, 0.12), Color::from_rgba(200, 110, 80, 255));
    b.cube(vec3(0.72, 0.26, -0.05), vec3(0.07, 0.14, 0.07), Color::from_rgba(90, 160, 80, 255));
    b.cube(vec3(-0.72, 0.13, -0.02), vec3(0.14, 0.06, 0.2), Color::from_rgba(230, 220, 200, 255));
    let mut faces = Vec::new();
    for (i, book) in books::BOOKS.iter().enumerate() {
        let hgt = 0.6 + ((i * 7) % 3) as f32 * 0.04;
        let x = (i as f32 - (books::BOOKS.len() as f32 - 1.0) / 2.0) * 0.2;
        let out = if i == sel { 0.12 + 0.012 * (time * 3.0).sin() } else { 0.0 };
        let lift = if i == sel { 0.03 } else { 0.0 };
        let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
        let center = vec3(x, 0.1 + hgt / 2.0 + lift, out);
        b.cube(center, vec3(0.16, hgt, 0.5), c);
        b.cube(center + vec3(0.0, 0.0, -0.005), vec3(0.17, hgt * 0.96, 0.48), Color::from_rgba(240, 230, 205, 255));
        faces.push((vec3(x - 0.08, 0.1 + lift, out + 0.251), hgt));
    }
    b.flush();
    for (i, (corner, hgt)) in faces.iter().enumerate() {
        draw_affine_parallelogram(*corner, vec3(0.16, 0.0, 0.0), vec3(0.0, *hgt, 0.0), Some(&spines[i].texture), WHITE);
    }
}

fn draw_book_scene(book: &books::Book, pages: &[RenderTarget], turn: f32, dir: f32, time: f32) {
    let eye = vec3(0.0, 1.12, 0.3);
    set_camera(&Camera3D {
        position: eye,
        target: vec3(0.0, 0.0, -0.02),
        up: Vec3::Y,
        fovy: 0.74,
        ..Default::default()
    });
    clear_background(Color::from_rgba(38, 28, 34, 255));
    let mut b = Batch::new(eye, reading_light());
    let cover = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    b.cube(vec3(0.0, -0.09, 0.0), vec3(3.2, 0.06, 2.2), Color::from_rgba(122, 80, 50, 255));
    b.cube(vec3(0.0, -0.045, 0.0), vec3(1.0, 0.03, 0.7), cover);
    b.cube(vec3(-0.235, -0.012, 0.0), vec3(0.46, 0.04, 0.64), Color::from_rgba(226, 216, 190, 255));
    b.cube(vec3(0.235, -0.012, 0.0), vec3(0.46, 0.04, 0.64), Color::from_rgba(226, 216, 190, 255));
    b.cube(vec3(-0.78, -0.02, -0.28), vec3(0.14, 0.12, 0.14), Color::from_rgba(236, 236, 236, 255));
    b.cube(vec3(-0.78, 0.035, -0.28), vec3(0.11, 0.01, 0.11), Color::from_rgba(120, 76, 46, 255));
    b.cube(vec3(0.8, -0.04, -0.4), vec3(0.18, 0.04, 0.18), Color::from_rgba(60, 60, 70, 255));
    b.cube(vec3(0.8, 0.12, -0.42), vec3(0.03, 0.3, 0.03), Color::from_rgba(60, 60, 70, 255));
    b.unlit = true;
    b.cube(vec3(0.74, 0.27, -0.36), vec3(0.16, 0.08, 0.16), Color::from_rgba(255, 220, 140, 255));
    b.unlit = false;
    let fur = Color::from_rgba(244, 224, 196, 255);
    let pad = Color::from_rgba(240, 160, 170, 255);
    let breathe = 0.006 * (time * 1.6).sin();
    for side in [-1.0f32, 1.0] {
        let paw = vec3(side * 0.4, 0.05 + breathe, 0.3);
        b.cube(paw, vec3(0.13, 0.07, 0.12), fur);
        b.cube(paw + vec3(0.0, 0.0, 0.1), vec3(0.11, 0.06, 0.1), fur);
        for t in 0..3 {
            b.cube(paw + vec3(-0.035 + t as f32 * 0.035, 0.04, -0.035), vec3(0.025, 0.012, 0.025), pad);
        }
    }
    b.flush();
    draw_affine_parallelogram(vec3(-0.46, 0.042, -0.31), vec3(0.0, 0.0, 0.62), vec3(0.46, -0.03, 0.0), Some(&pages[0].texture), WHITE);
    draw_affine_parallelogram(vec3(0.0, 0.012, -0.31), vec3(0.0, 0.0, 0.62), vec3(0.46, 0.03, 0.0), Some(&pages[1].texture), WHITE);
    if turn > 0.0 {
        let p = 1.0 - turn;
        let angle = if dir > 0.0 { p * PI } else { PI - p * PI };
        let reach = vec3(angle.cos() * 0.46, 0.012 + angle.sin() * 0.3, 0.0);
        let shade = 0.82 + 0.18 * angle.sin();
        draw_affine_parallelogram(vec3(0.0, 0.016, -0.31), vec3(0.0, 0.0, 0.62), reach, None, Color::new(0.97 * shade, 0.94 * shade, 0.86 * shade, 1.0));
    }
}

fn render_pick_preview(target: &RenderTarget, world: &World, cast: &[Character], chars: [usize; 2], chosen: usize, time: f32) {
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

fn render_shop_preview(target: &RenderTarget, left: f32, top: f32, width: f32, height: f32, row: home::Row, house: &home::Home, time: f32, beat_on: bool) {
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / target.texture.width(), -2.0 / target.texture.height()),
        target: vec2(target.texture.width() / 2.0, target.texture.height() / 2.0),
        ..Default::default()
    });
    clear_background(Color::from_rgba(28, 24, 42, 255));
    let (wall_col, floor_a, floor_b) = match row {
        home::Row::Wall(i) => (home::WALLS[i].1, home::FLOORS[house.floor as usize].1, home::FLOORS[house.floor as usize].2),
        home::Row::Floor(i) => (home::WALLS[house.wall as usize].1, home::FLOORS[i].1, home::FLOORS[i].2),
        home::Row::Set => (home::WALLS[home::CONCRETE_WALL as usize].1, home::FLOORS[home::CONCRETE_FLOOR as usize].1, home::FLOORS[home::CONCRETE_FLOOR as usize].2),
        _ => (home::WALLS[house.wall as usize].1, home::FLOORS[house.floor as usize].1, home::FLOORS[house.floor as usize].2),
    };
    let base_center = vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y + 0.4, home::ORIGIN.y + 1.8);
    let eye = base_center + vec3(3.0, 2.6, 3.8);
    set_camera(&Camera3D {
        position: eye,
        target: base_center + vec3(0.0, 0.15, 0.0),
        up: Vec3::Y,
        fovy: 0.62,
        aspect: Some(width / height),
        render_target: Some(target.clone()),
        viewport: Some((left as i32, top as i32, width as i32, height as i32)),
        ..Default::default()
    });
    let light = Light {
        dusk: 0.4,
        fire: base_center + vec3(0.6, 1.8, -0.8),
        fire_strength: if beat_on { 0.9 } else { 0.7 },
        tint: vec3(0.95, 0.58, 0.22),
        lamps: Vec::new(),
    };
    let mut b = Batch::new(eye, light);
    let dark_wall = Color::new(wall_col.r * 0.84, wall_col.g * 0.84, wall_col.b * 0.84, 1.0);
    b.cube(vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y - 0.04, home::ORIGIN.y + 1.8), vec3(3.9, 0.06, 3.9), floor_a);
    b.cube(vec3(home::ORIGIN.x + 1.8, 1.1, home::ORIGIN.y - 0.15), vec3(3.9, 2.3, 0.08), wall_col);
    b.cube(vec3(home::ORIGIN.x - 0.15, 1.1, home::ORIGIN.y + 1.8), vec3(0.08, 2.3, 3.9), dark_wall);
    for n in 0..4 {
        let x = home::ORIGIN.x + 0.45 + n as f32 * 0.9;
        b.cube(vec3(x, home::FLOOR_Y - 0.015, home::ORIGIN.y + 1.8), vec3(0.02, 0.01, 3.7), floor_b);
    }
    for n in 0..4 {
        let z = home::ORIGIN.y + 0.45 + n as f32 * 0.9;
        b.cube(vec3(home::ORIGIN.x + 1.8, home::FLOOR_Y - 0.015, z), vec3(3.7, 0.01, 0.02), floor_b);
    }
    let bob = 0.08 + 0.03 * (time * 2.0).sin();
    match row {
        home::Row::Item(item) => {
            let state = match item {
                home::Item::Plant => 3,
                home::Item::Table => 1,
                home::Item::Lamp | home::Item::Speaker | home::Item::Subwoofer | home::Item::RedLight => 0,
                _ => 0,
            };
            let piece = home::Piece { item, x: 1, z: 1, rot: 0, state, day: 0 };
            let c = piece.center();
            let shadow = vec3(home::ORIGIN.x + c.x, home::FLOOR_Y + 0.01, home::ORIGIN.y + c.y);
            let (fw, fd) = piece.footprint();
            b.cube(shadow, vec3(fw as f32 * 0.9 + 0.3, 0.02, fd as f32 * 0.9 + 0.3), Color::new(0.0, 0.0, 0.0, 0.18));
            draw_piece(&mut b, &piece, time, 1.0, true, if beat_on { (time * 6.0).sin() * 0.5 + 0.5 } else { 0.0 }, bob);
        }
        home::Row::Upgrade => {
            let piece = home::Piece { item: home::Item::Bed, x: 1, z: 1, rot: 0, state: 0, day: 0 };
            draw_piece(&mut b, &piece, time, 0.9, false, 0.0, 0.0);
            b.cube(vec3(home::ORIGIN.x + 3.2, 0.8, home::ORIGIN.y + 2.8), vec3(1.1, 1.6, 1.1), Color::new(1.0, 0.9, 0.5, 0.75));
        }
        home::Row::Wall(_) | home::Row::Floor(_) => {}
        home::Row::Set => {
            for piece in [
                home::Piece { item: home::Item::DjBooth, x: 1, z: 2, rot: 0, state: 0, day: 0 },
                home::Piece { item: home::Item::RedLight, x: 0, z: 0, rot: 0, state: 0, day: 0 },
                home::Piece { item: home::Item::ConcreteBench, x: 0, z: 2, rot: 0, state: 0, day: 0 },
            ] {
                draw_piece(&mut b, &piece, time, 1.0, false, 0.7, 0.0);
            }
        }
    }
    b.flush();
}

const CAMPFIRE: (i32, i32) = (8, 17);
const DOOR_OUT: Vec2 = Vec2::new(4.5, 15.55);

#[derive(Clone)]
struct DiaryEntry {
    when: f64,
    seconds: u32,
    stars: u8,
    who: (usize, usize),
    events: Vec<(f32, u16, f32)>,
}

fn stars_for(seconds: f32) -> u8 {
    if seconds >= 180.0 {
        3
    } else if seconds >= 60.0 {
        2
    } else if seconds >= 30.0 {
        1
    } else {
        0
    }
}

fn date_label(epoch: f64) -> String {
    let days = (epoch / 86400.0).floor() as i64;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let names = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    format!("{} {}", names[(month - 1).clamp(0, 11) as usize], day)
}

fn ui_star(x: f32, y: f32, size: f32, color: Color) {
    let a = size / 5.0;
    ui_rect(x + a * 2.0, y, a, size, color);
    ui_rect(x, y + a * 2.0, size, a, color);
    ui_rect(x + a, y + a, a * 3.0, a * 3.0, color);
}

enum Gathered {
    Stick,
    Stone,
    Acorns(u32),
    Resting,
    Nothing,
}

fn gather_at(world: &World, stick_trees: &mut Vec<(i32, i32)>, tree_rest: &mut std::collections::HashMap<(i32, i32), f32>, tile: (i32, i32), time: f32, hands_free: bool) -> Gathered {
    let kind = match world.get(tile.0, tile.1) {
        Some(t) if t.fire <= 0.0 => t.kind,
        _ => return Gathered::Nothing,
    };
    match kind {
        Kind::Tree => {
            if hands_free && stick_trees.contains(&tile) {
                stick_trees.retain(|t| *t != tile);
                return Gathered::Stick;
            }
            let ready = tree_rest.get(&tile).map_or(true, |&at| time - at > TREE_REST);
            if ready {
                tree_rest.insert(tile, time);
                Gathered::Acorns(1 + (hash(tile.0, tile.1, time as u32) * 3.0) as u32)
            } else {
                Gathered::Resting
            }
        }
        Kind::Stone if hands_free => Gathered::Stone,
        _ => Gathered::Nothing,
    }
}

fn stick_spot(world: &World, tile: (i32, i32)) -> Option<Vec2> {
    for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
        if world.get(tile.0 + dx, tile.1 + dz).map(|t| !t.solid()).unwrap_or(false) {
            return Some(vec2(tile.0 as f32 + 0.5 + dx as f32 * 0.9, tile.1 as f32 + 0.5 + dz as f32 * 0.9));
        }
    }
    None
}

fn grow_stick(world: &mut World, stick_trees: &mut Vec<(i32, i32)>) {
    let mut options = Vec::new();
    for z in 1..MAP as i32 - 1 {
        for x in 1..MAP as i32 - 1 {
            let ok = world.get(x, z).map(|t| t.kind == Kind::Tree && t.fire <= 0.0).unwrap_or(false);
            if ok && in_forest(x, z) && !stick_trees.contains(&(x, z)) && stick_spot(world, (x, z)).is_some() {
                options.push((x, z));
            }
        }
    }
    if !options.is_empty() {
        let pick = (world.rand() * options.len() as f32) as usize % options.len();
        stick_trees.push(options[pick]);
    }
}

fn nearest_stick_tree(world: &World, stick_trees: &[(i32, i32)], from: Vec2) -> Option<((i32, i32), Vec2)> {
    let mut best: Option<((i32, i32), Vec2)> = None;
    let mut best_d = f32::MAX;
    for &tile in stick_trees {
        for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
            let spot = vec2((tile.0 + dx) as f32 + 0.5, (tile.1 + dz) as f32 + 0.5);
            if walkable(world, spot) {
                let d = spot.distance(from);
                if d < best_d {
                    best_d = d;
                    best = Some((tile, spot));
                }
                break;
            }
        }
    }
    best
}

fn ahead_local(p: &Player) -> (i32, i32) {
    let l = home::local(p.pos);
    let (dx, dz) = if p.facing.x.abs() > p.facing.y.abs() { (p.facing.x.signum() as i32, 0) } else { (0, p.facing.y.signum() as i32) };
    (l.x.floor() as i32 + dx, l.y.floor() as i32 + dz)
}
const CAT_SEAT: Vec2 = Vec2::new(7.3, 17.5);
const DOG_SEAT: Vec2 = Vec2::new(9.7, 17.5);
const STICK_DROP: Vec2 = Vec2::new(8.5, 16.3);

fn fire_center() -> Vec3 {
    vec3(CAMPFIRE.0 as f32 + 0.5, 0.5, CAMPFIRE.1 as f32 + 0.5)
}

struct Camp {
    fuel: f32,
    lit: bool,
    pile: u32,
}

fn draw_camp(b: &mut Batch, camp: &Camp, time: f32) {
    let c = fire_center();
    let stone = Color::from_rgba(140, 144, 158, 255);
    let log = Color::from_rgba(120, 82, 52, 255);
    let log_end = Color::from_rgba(200, 160, 110, 255);
    for k in 0..8 {
        let a = k as f32 / 8.0 * 2.0 * PI;
        b.cube(c + vec3(a.cos() * 0.36, 0.06, a.sin() * 0.36), vec3(0.16, 0.12, 0.16), tinted(stone, (k % 3) as f32 * 0.05));
    }
    b.cube(c + vec3(0.0, 0.07, 0.0), vec3(0.5, 0.08, 0.1), log);
    b.cube(c + vec3(0.0, 0.07, 0.0), vec3(0.1, 0.08, 0.5), log);
    for seat in [CAT_SEAT, DOG_SEAT] {
        b.cube(vec3(seat.x, 0.58, seat.y), vec3(0.24, 0.16, 0.7), log);
        b.cube(vec3(seat.x, 0.58, seat.y + 0.36), vec3(0.2, 0.13, 0.02), log_end);
    }
    for k in 0..camp.pile.min(5) {
        let kf = k as f32;
        b.cube(vec3(c.x - 0.75 + kf * 0.03, 0.54 + kf * 0.05, c.z - 0.5), vec3(0.4, 0.05, 0.06), tinted(log, kf * 0.04));
    }
    if camp.lit {
        let scale = 0.7 + (camp.fuel / 120.0).min(1.0) * 0.5;
        for k in 0..4 {
            let flick = hash(k, 3, (time * 12.0) as u32);
            let h = (0.22 + flick * 0.3) * scale;
            let a = k as f32 / 4.0 * 2.0 * PI + time;
            let off = vec3(a.cos() * 0.08, 0.0, a.sin() * 0.08);
            let col = match k {
                0 => Color::from_rgba(240, 90, 40, 255),
                1 => Color::from_rgba(255, 140, 50, 255),
                2 => Color::from_rgba(255, 190, 70, 255),
                _ => Color::from_rgba(255, 235, 140, 255),
            };
            let w = 0.16 - k as f32 * 0.02;
            b.cube(c + off + vec3(0.0, 0.1 + h / 2.0, 0.0), vec3(w, h, w), col);
        }
    }
}

#[derive(Clone, Copy)]
struct CatLook {
    fur: Color,
    stripes: Option<Color>,
    eye: Option<Color>,
    hat: Color,
    hat_dark: Color,
    band: Color,
}

#[derive(Clone, Copy)]
struct DogLook {
    fur: Color,
    ear: Color,
    muzzle: Color,
    tail: Option<bool>,
    collar: Option<Color>,
    hat: Option<(Color, Color, Color)>,
}

#[derive(Clone, Copy)]
enum Body {
    Cat(CatLook),
    Dog(DogLook),
    Duck,
    Owl,
    Fox,
    Otter,
}

#[cfg(target_arch = "wasm32")]
mod clock {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn sky_now() -> f64;
        fn sky_offset() -> f64;
        fn sky_weather() -> i32;
        fn sky_temp() -> f32;
        fn sky_sun(i: i32) -> f64;
        fn sky_lat() -> f32;
    }

    pub fn now() -> f64 {
        unsafe { sky_now() }
    }

    pub fn offset() -> f64 {
        unsafe { sky_offset() }
    }

    pub fn weather() -> i32 {
        unsafe { sky_weather() }
    }

    pub fn temp() -> Option<f32> {
        let t = unsafe { sky_temp() };
        if t < -500.0 { None } else { Some(t) }
    }

    pub fn sun(i: i32) -> f64 {
        unsafe { sky_sun(i) }
    }

    pub fn lat() -> f32 {
        unsafe { sky_lat() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod clock {
    use std::sync::OnceLock;

    pub fn now() -> f64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }

    pub fn offset() -> f64 {
        static OFFSET: OnceLock<f64> = OnceLock::new();
        *OFFSET.get_or_init(|| {
            let out = std::process::Command::new("date").arg("+%z").output().ok();
            let text = out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
            if text.len() == 5 {
                let sign = if text.starts_with('-') { -1.0 } else { 1.0 };
                let h: f64 = text[1..3].parse().unwrap_or(0.0);
                let m: f64 = text[3..5].parse().unwrap_or(0.0);
                sign * (h * 3600.0 + m * 60.0)
            } else {
                0.0
            }
        })
    }

    pub fn weather() -> i32 {
        -1
    }

    pub fn temp() -> Option<f32> {
        None
    }

    pub fn sun(_i: i32) -> f64 {
        0.0
    }

    pub fn lat() -> f32 {
        53.55
    }
}

#[derive(Clone, Copy)]
struct Sky {
    night: f32,
    glow: f32,
    hour: f32,
    day: i64,
    season: usize,
    weather: usize,
    temp: Option<f32>,
}

const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];
const WEATHERS: [&str; 4] = ["clear", "cloudy", "rain", "snow"];

thread_local! {
    static SKY: std::cell::Cell<Sky> = std::cell::Cell::new(Sky { night: 0.0, glow: 0.0, hour: 12.0, day: 0, season: 1, weather: 0, temp: None });
}

fn sky() -> Sky {
    SKY.with(|s| s.get())
}

fn civil_month(day: i64) -> (u32, u32) {
    let z = day + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (m, d)
}

fn update_sky() -> Sky {
    let now = clock::now();
    let offset = clock::offset();
    let local = now + offset;
    let day = (local / 86400.0).floor() as i64;
    let hour = ((local - day as f64 * 86400.0) / 3600.0) as f32;
    let (month, mday) = civil_month(day);
    let lat = clock::lat();
    let south = lat < 0.0;
    let m = if south { (month + 5) % 12 + 1 } else { month };
    let season = match m {
        3..=5 => 0,
        6..=8 => 1,
        9..=11 => 2,
        _ => 3,
    };
    let doy = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(month - 1) as usize] as f32 + mday as f32;
    let (sunrise, sunset) = {
        let (a, b) = (clock::sun(0), clock::sun(1));
        if a > 0.0 && b > 0.0 {
            (((a + offset).rem_euclid(86400.0) / 3600.0) as f32, ((b + offset).rem_euclid(86400.0) / 3600.0) as f32)
        } else {
            let decl = 23.44f32.to_radians() * (2.0 * PI * (284.0 + doy) / 365.0).sin();
            let cos_h = (-(lat.to_radians().tan()) * decl.tan()).clamp(-1.0, 1.0);
            let half = cos_h.acos().to_degrees() / 15.0;
            let noon = 12.3 + (offset as f32 / 3600.0 - 1.0).clamp(-1.0, 1.0);
            (noon - half, noon + half)
        }
    };
    let ramp = |x: f32| x.clamp(0.0, 1.0);
    let wrap = |x: f32| (x + 12.0).rem_euclid(24.0) - 12.0;
    let day_len = (sunset - sunrise).rem_euclid(24.0);
    let pos = (hour - sunrise).rem_euclid(24.0);
    let edge = if pos < day_len { pos.min(day_len - pos) } else { -(pos - day_len).min(24.0 - pos) };
    let night = ramp((0.4 - edge) / 1.1);
    let glow = (1.0 - wrap(hour - sunset).abs() / 0.9).max(0.0) + 0.7 * (1.0 - wrap(hour - sunrise).abs() / 0.8).max(0.0);
    let live_weather = clock::weather();
    let weather = if live_weather >= 0 {
        live_weather as usize
    } else {
        let roll = hash(day as i32, (hour / 3.0) as i32, 77);
        let table: [f32; 3] = match season {
            0 => [0.4, 0.7, 1.0],
            1 => [0.55, 0.8, 1.0],
            2 => [0.25, 0.6, 1.0],
            _ => [0.2, 0.5, 0.75],
        };
        if roll < table[0] { 0 } else if roll < table[1] { 1 } else if roll < table[2] { 2 } else { 3 }
    };
    let weather = if weather == 3 && season == 1 { 2 } else { weather };
    let out = Sky { night, glow: glow.min(1.0), hour, day, season, weather, temp: clock::temp() };
    SKY.with(|s| s.set(out));
    out
}

fn sky_haze(c: Color, dusk: f32) -> Color {
    let s = sky();
    let mut c = c;
    if s.glow > 0.0 {
        c = mix(c, Color::new(1.0, 0.6, 0.48, c.a), s.glow * 0.45 * (1.0 - dusk * 0.4));
    }
    match s.weather {
        1 => mix(c, Color::new(0.62, 0.64, 0.68, c.a), 0.25),
        2 => mix(c, Color::new(0.45, 0.5, 0.58, c.a), 0.38),
        3 => mix(c, Color::new(0.88, 0.9, 0.95, c.a), 0.3),
        _ => c,
    }
}

#[derive(Clone, Copy)]
enum VoiceKind {
    Meow { shift: i32, bright: f32, length: f32 },
    Woof,
    Yap,
    Howl,
    Quack,
    Hoot,
    Bellow,
    Shell,
}

fn voice_word(kind: VoiceKind) -> &'static str {
    match kind {
        VoiceKind::Meow { .. } => "meow",
        VoiceKind::Woof => "woof",
        VoiceKind::Yap => "yap",
        VoiceKind::Howl => "awoo",
        VoiceKind::Quack => "quack",
        VoiceKind::Hoot => "hoo",
        VoiceKind::Bellow => "yowl",
        VoiceKind::Shell => "clack",
    }
}

struct Character {
    name: &'static str,
    noun: &'static str,
    body: Body,
    voice: VoiceKind,
}

fn rgba(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn roster() -> Vec<Character> {
    vec![
        Character {
            name: "cream cat",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(250, 226, 192), stripes: None, eye: None, hat: rgba(78, 66, 176), hat_dark: rgba(50, 40, 124), band: rgba(255, 208, 92) }),
            voice: VoiceKind::Meow { shift: 0, bright: 1.0, length: 0.55 },
        },
        Character {
            name: "russian blue",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(98, 102, 118), stripes: None, eye: Some(rgba(186, 214, 72)), hat: rgba(116, 72, 168), hat_dark: rgba(84, 50, 128), band: rgba(236, 184, 72) }),
            voice: VoiceKind::Meow { shift: 0, bright: 0.55, length: 0.6 },
        },
        Character {
            name: "ginger tabby",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(240, 160, 84), stripes: Some(rgba(198, 112, 52)), eye: Some(rgba(120, 200, 96)), hat: rgba(54, 136, 90), hat_dark: rgba(34, 96, 62), band: rgba(250, 236, 200) }),
            voice: VoiceKind::Meow { shift: 0, bright: 1.45, length: 0.45 },
        },
        Character {
            name: "black cat",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(54, 52, 64), stripes: None, eye: Some(rgba(244, 204, 60)), hat: rgba(178, 48, 60), hat_dark: rgba(120, 30, 40), band: rgba(255, 208, 92) }),
            voice: VoiceKind::Meow { shift: -12, bright: 0.9, length: 0.62 },
        },
        Character {
            name: "labrador",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(232, 196, 128), ear: rgba(200, 156, 92), muzzle: rgba(232, 196, 128), tail: Some(false), collar: None, hat: Some((rgba(40, 140, 128), rgba(24, 94, 88), rgba(214, 224, 236))) }),
            voice: VoiceKind::Woof,
        },
        Character {
            name: "cream pup",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(248, 230, 194), ear: rgba(214, 164, 104), muzzle: rgba(254, 244, 222), tail: None, collar: Some(rgba(52, 178, 168)), hat: None }),
            voice: VoiceKind::Yap,
        },
        Character {
            name: "shiba",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(226, 140, 72), ear: rgba(214, 126, 60), muzzle: rgba(250, 240, 226), tail: Some(true), collar: Some(rgba(200, 50, 60)), hat: None }),
            voice: VoiceKind::Howl,
        },
        Character { name: "duck", noun: "duck", body: Body::Duck, voice: VoiceKind::Quack },
        Character { name: "owl", noun: "owl", body: Body::Owl, voice: VoiceKind::Hoot },
        Character { name: "fox", noun: "fox", body: Body::Fox, voice: VoiceKind::Bellow },
        Character { name: "sea otter", noun: "otter", body: Body::Otter, voice: VoiceKind::Shell },
    ]
}

fn render_voice(kind: VoiceKind, note: u8) -> Vec<f32> {
    let f = |shift: i32| midi_to_freq((note as i32 + shift) as u8);
    match kind {
        VoiceKind::Meow { shift, bright, length } => render_meow_style(f(shift), length, bright),
        VoiceKind::Woof => render_woof(f(-12)),
        VoiceKind::Yap => render_yap(f(-5)),
        VoiceKind::Howl => render_howl(f(-12)),
        VoiceKind::Quack => render_quack(f(-12)),
        VoiceKind::Hoot => render_hoot_machine(f(-12)),
        VoiceKind::Bellow => render_fox(f(0)),
        VoiceKind::Shell => render_shell(f(0)),
    }
}

fn draw_cat(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, meowing: bool, dance: f32, look: &CatLook) {
    let fur = look.fur;
    let dark = tinted(fur, -0.5);
    let pink = Color::from_rgba(255, 160, 175, 255);
    let (hat, hat_dark, gold) = (look.hat, look.hat_dark, look.band);

    let bob = if walking { (time * 14.0).sin().abs() * 0.05 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 14.0).sin() * 0.05 } else { 0.0 };

    for (lx, lz, phase) in [(-0.1, 0.12, 1.0), (0.1, 0.12, -1.0), (-0.1, -0.18, -1.0), (0.1, -0.18, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.08, lz + step * phase), vec3(0.08, 0.16, 0.08), fur);
    }
    b.box_at(o, yaw, vec3(0.0, 0.27, -0.03), vec3(0.32, 0.24, 0.44), fur);
    if let Some(st) = look.stripes {
        for k in 0..3 {
            b.box_at(o, yaw, vec3(0.0, 0.392, -0.16 + k as f32 * 0.12), vec3(0.3, 0.01, 0.05), st);
        }
    }

    for k in 0..3 {
        let kf = k as f32;
        let sway = (time * 3.0 - kf * 0.6).sin() * 0.05 * (kf + 1.0);
        let c = match look.stripes {
            Some(st) if k == 1 => st,
            _ => fur,
        };
        b.box_at(o, yaw, vec3(sway, 0.36 + kf * 0.1, -0.28 - kf * 0.04), vec3(0.07, 0.1, 0.07), c);
    }

    b.box_at(o, yaw, vec3(0.0, 0.54, 0.16), vec3(0.4, 0.32, 0.32), fur);
    if let Some(st) = look.stripes {
        for dx in [-0.08_f32, 0.0, 0.08] {
            b.box_at(o, yaw, vec3(dx, 0.68, 0.321), vec3(0.03, 0.06, 0.01), st);
        }
    }
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.15, 0.76, 0.16), vec3(0.1, 0.13, 0.08), fur);
        b.box_at(o, yaw, vec3(side * 0.15, 0.75, 0.205), vec3(0.05, 0.08, 0.01), pink);
    }

    let blinking = time % 3.4 < 0.12;
    for side in [-1.0_f32, 1.0] {
        if blinking || meowing {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.08, 0.025, 0.01), dark);
        } else if let Some(eye) = look.eye {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.07, 0.085, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.325), vec3(0.055, 0.07, 0.01), eye);
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.328), vec3(0.022, 0.065, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09 - 0.012, 0.58, 0.331), vec3(0.016, 0.016, 0.01), WHITE);
        } else {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.06, 0.08, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09 + 0.012, 0.575, 0.328), vec3(0.02, 0.02, 0.01), WHITE);
        }
        b.box_at(o, yaw, vec3(side * 0.14, 0.49, 0.323), vec3(0.06, 0.035, 0.01), Color::from_rgba(255, 175, 185, 255));
    }
    b.box_at(o, yaw, vec3(0.0, 0.5, 0.324), vec3(0.04, 0.03, 0.01), pink);
    if meowing {
        b.box_at(o, yaw, vec3(0.0, 0.45, 0.324), vec3(0.05, 0.04, 0.01), Color::from_rgba(150, 70, 80, 255));
    }

    b.box_at(o, yaw, vec3(0.0, 0.72, 0.16), vec3(0.44, 0.035, 0.4), hat_dark);
    b.box_at(o, yaw, vec3(0.0, 0.77, 0.15), vec3(0.3, 0.07, 0.27), hat);
    b.box_at(o, yaw, vec3(0.0, 0.81, 0.15), vec3(0.305, 0.025, 0.275), gold);
    b.box_at(o, yaw, vec3(0.0, 0.88, 0.13), vec3(0.22, 0.12, 0.2), hat);
    b.box_at(o, yaw, vec3(0.01, 0.98, 0.1), vec3(0.14, 0.1, 0.13), hat);
    b.box_at(o, yaw, vec3(0.03, 1.06, 0.06), vec3(0.08, 0.08, 0.08), hat);
    b.box_at(o, yaw, vec3(0.06, 1.11, 0.02), vec3(0.05, 0.05, 0.05), hat_dark);
    b.box_at(o, yaw, vec3(0.0, 0.88, 0.235), vec3(0.06, 0.06, 0.01), gold);
}

fn draw_carry(b: &mut Batch, o: Vec3, yaw: f32, at: Vec3, carry: Option<Carry>) {
    match carry {
        Some(Carry::Stick) => {
            b.box_at(o, yaw, at, vec3(0.42, 0.05, 0.05), Color::from_rgba(130, 90, 56, 255));
            b.box_at(o, yaw, at + vec3(0.1, 0.04, 0.0), vec3(0.08, 0.06, 0.06), Color::from_rgba(104, 170, 78, 255));
        }
        Some(Carry::Stone) => b.box_at(o, yaw, at + vec3(0.0, 0.0, 0.02), vec3(0.13, 0.1, 0.11), Color::from_rgba(160, 162, 176, 255)),
        Some(Carry::Acorn) => {
            b.box_at(o, yaw, at + vec3(0.0, -0.01, 0.02), vec3(0.09, 0.1, 0.09), ACORN_COLOR);
            b.box_at(o, yaw, at + vec3(0.0, 0.05, 0.02), vec3(0.11, 0.04, 0.11), Color::from_rgba(120, 84, 50, 255));
        }
        None => {}
    }
}

fn draw_dog(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, barking: bool, dance: f32, carry: Option<Carry>, look: &DogLook) {
    let fur = look.fur;
    let nose = Color::from_rgba(56, 38, 32, 255);
    let ink = Color::from_rgba(30, 22, 20, 255);

    let bob = if walking { (time * 12.0).sin().abs() * 0.05 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 12.0).sin() * 0.06 } else { 0.0 };

    for (lx, lz, phase) in [(-0.11, 0.16, 1.0), (0.11, 0.16, -1.0), (-0.11, -0.2, -1.0), (0.11, -0.2, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.1, lz + step * phase), vec3(0.09, 0.2, 0.09), fur);
    }
    b.box_at(o, yaw, vec3(0.0, 0.32, -0.02), vec3(0.3, 0.25, 0.56), fur);
    b.box_at(o, yaw, vec3(0.0, 0.25, 0.1), vec3(0.24, 0.12, 0.3), look.muzzle);

    let wag = (time * 14.0).sin() * 0.09;
    match look.tail {
        Some(false) => {
            b.box_at(o, yaw, vec3(wag * 0.5, 0.42, -0.33), vec3(0.06, 0.06, 0.1), fur);
            b.box_at(o, yaw, vec3(wag, 0.47, -0.4), vec3(0.06, 0.06, 0.1), fur);
        }
        Some(true) => {
            b.box_at(o, yaw, vec3(wag * 0.3, 0.48, -0.3), vec3(0.1, 0.1, 0.1), fur);
            b.box_at(o, yaw, vec3(wag * 0.5, 0.56, -0.22), vec3(0.09, 0.08, 0.1), fur);
            b.box_at(o, yaw, vec3(wag * 0.6, 0.52, -0.13), vec3(0.08, 0.07, 0.08), look.muzzle);
        }
        None => {}
    }

    b.box_at(o, yaw, vec3(0.0, 0.58, 0.25), vec3(0.32, 0.28, 0.3), fur);
    b.box_at(o, yaw, vec3(0.0, 0.52, 0.44), vec3(0.18, 0.13, 0.16), look.muzzle);
    b.box_at(o, yaw, vec3(0.0, 0.57, 0.525), vec3(0.08, 0.06, 0.03), nose);
    if let Some(c) = look.collar {
        b.box_at(o, yaw, vec3(0.0, 0.45, 0.2), vec3(0.32, 0.05, 0.2), c);
        b.box_at(o, yaw, vec3(0.0, 0.41, 0.31), vec3(0.05, 0.05, 0.02), Color::from_rgba(240, 190, 70, 255));
    }

    let flop = if walking { (time * 12.0).sin() * 0.02 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        if look.tail == Some(true) {
            b.box_at(o, yaw, vec3(side * 0.11, 0.76, 0.22), vec3(0.09, 0.12, 0.06), look.ear);
        } else if look.tail.is_none() {
            b.box_at(o, yaw, vec3(side * 0.2, 0.7 + flop * side, 0.22), vec3(0.14, 0.12, 0.05), look.ear);
            b.box_at(o, yaw, vec3(side * 0.25, 0.77 + flop * side, 0.22), vec3(0.08, 0.08, 0.05), tinted(look.ear, -0.15));
        } else {
            b.box_at(o, yaw, vec3(side * 0.185, 0.54 + flop * side, 0.24), vec3(0.06, 0.2, 0.14), look.ear);
        }
        let blinking = time % 4.1 < 0.12;
        if blinking || barking {
            b.box_at(o, yaw, vec3(side * 0.08, 0.63, 0.402), vec3(0.06, 0.02, 0.01), ink);
        } else {
            b.box_at(o, yaw, vec3(side * 0.08, 0.63, 0.402), vec3(0.05, 0.06, 0.01), ink);
            b.box_at(o, yaw, vec3(side * 0.08 + 0.012, 0.645, 0.408), vec3(0.018, 0.018, 0.01), WHITE);
        }
    }
    if barking {
        b.box_at(o, yaw, vec3(0.0, 0.445, 0.47), vec3(0.14, 0.03, 0.1), Color::from_rgba(120, 50, 50, 255));
        b.box_at(o, yaw, vec3(0.0, 0.43, 0.5), vec3(0.06, 0.02, 0.08), Color::from_rgba(255, 140, 160, 255));
    }
    draw_carry(b, o, yaw, vec3(0.0, 0.47, 0.5), carry);

    if let Some((hat, hat_dark, band)) = look.hat {
        b.box_at(o, yaw, vec3(0.0, 0.735, 0.24), vec3(0.4, 0.035, 0.38), hat_dark);
        b.box_at(o, yaw, vec3(0.0, 0.79, 0.23), vec3(0.28, 0.07, 0.26), hat);
        b.box_at(o, yaw, vec3(0.0, 0.83, 0.23), vec3(0.285, 0.025, 0.265), band);
        b.box_at(o, yaw, vec3(0.0, 0.9, 0.21), vec3(0.2, 0.12, 0.19), hat);
        b.box_at(o, yaw, vec3(-0.01, 1.0, 0.18), vec3(0.13, 0.1, 0.12), hat);
        b.box_at(o, yaw, vec3(-0.03, 1.08, 0.14), vec3(0.08, 0.08, 0.08), hat);
        b.box_at(o, yaw, vec3(-0.06, 1.13, 0.1), vec3(0.05, 0.05, 0.05), hat_dark);
        b.box_at(o, yaw, vec3(0.0, 0.9, 0.305), vec3(0.06, 0.06, 0.01), band);
    }
}

fn draw_duck(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, quacking: bool, dance: f32, carry: Option<Carry>) {
    let white = Color::from_rgba(250, 250, 244, 255);
    let wing = Color::from_rgba(222, 224, 228, 255);
    let orange = Color::from_rgba(250, 156, 50, 255);
    let ink = Color::from_rgba(26, 24, 30, 255);
    let hat = Color::from_rgba(220, 90, 150, 255);
    let hat_dark = Color::from_rgba(160, 54, 108, 255);
    let waddle = if walking { (time * 12.0).sin() * 0.12 } else { 0.0 };
    let bob = if walking { (time * 12.0).sin().abs() * 0.03 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let y2 = yaw + waddle * 0.6;
    for (side, phase) in [(-1.0_f32, 1.0_f32), (1.0, -1.0)] {
        let lift = if walking { ((time * 12.0).sin() * phase).max(0.0) * 0.04 } else { 0.0 };
        b.box_at(o, yaw, vec3(side * 0.07, 0.015 + lift, 0.06), vec3(0.1, 0.03, 0.14), orange);
        b.box_at(o, yaw, vec3(side * 0.07, 0.06 + lift, 0.02), vec3(0.04, 0.07, 0.04), orange);
    }
    b.box_at(o, y2, vec3(0.0, 0.22, -0.02), vec3(0.3, 0.22, 0.38), white);
    b.box_at(o, y2, vec3(0.0, 0.3, -0.24), vec3(0.14, 0.1, 0.1), white);
    let flap = if quacking || dance > 0.3 { (time * 30.0).sin().abs() * 0.08 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, y2, vec3(side * (0.165 + flap * 0.5), 0.24 + flap, -0.02), vec3(0.04, 0.13, 0.26), wing);
    }
    b.box_at(o, y2, vec3(0.0, 0.38, 0.1), vec3(0.14, 0.14, 0.12), white);
    b.box_at(o, y2, vec3(0.0, 0.5, 0.12), vec3(0.24, 0.22, 0.22), white);
    let open = if quacking { 0.035 } else { 0.0 };
    b.box_at(o, y2, vec3(0.0, 0.47 + open * 0.5, 0.27), vec3(0.15, 0.04, 0.12), orange);
    b.box_at(o, y2, vec3(0.0, 0.43 - open * 0.5, 0.26), vec3(0.13, 0.03, 0.1), tinted(orange, -0.15));
    let blinking = time % 3.9 < 0.12;
    for side in [-1.0_f32, 1.0] {
        let h = if blinking { 0.012 } else { 0.045 };
        b.box_at(o, y2, vec3(side * 0.07, 0.54, 0.232), vec3(0.04, h, 0.01), ink);
        b.box_at(o, y2, vec3(side * 0.12, 0.49, 0.232), vec3(0.04, 0.025, 0.01), Color::from_rgba(255, 180, 190, 255));
    }
    draw_carry(b, o, y2, vec3(0.0, 0.45, 0.32), carry);
    b.box_at(o, y2, vec3(0.0, 0.625, 0.11), vec3(0.32, 0.03, 0.3), hat_dark);
    b.box_at(o, y2, vec3(0.0, 0.68, 0.1), vec3(0.2, 0.08, 0.19), hat);
    b.box_at(o, y2, vec3(0.01, 0.76, 0.08), vec3(0.13, 0.08, 0.12), hat);
    b.box_at(o, y2, vec3(0.03, 0.83, 0.05), vec3(0.07, 0.07, 0.07), hat);
    b.box_at(o, y2, vec3(0.0, 0.69, 0.197), vec3(0.05, 0.05, 0.01), Color::from_rgba(255, 220, 90, 255));
}

fn draw_owl(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, singing: bool, dance: f32, carry: Option<Carry>) {
    let brown = Color::from_rgba(140, 100, 70, 255);
    let brown_dark = Color::from_rgba(104, 72, 50, 255);
    let belly = Color::from_rgba(214, 186, 146, 255);
    let face = Color::from_rgba(232, 214, 186, 255);
    let ink = Color::from_rgba(26, 22, 24, 255);
    let phones = Color::from_rgba(52, 52, 64, 255);
    let led = mix(Color::from_rgba(60, 200, 190, 255), Color::from_rgba(255, 240, 140, 255), if singing { 0.5 + 0.5 * (time * 20.0).sin() } else { 0.0 });
    let hop = if walking { (time * 10.0).sin().abs() * 0.08 } else { dance * 0.06 };
    let o = pos + vec3(0.0, hop, 0.0);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.07, 0.02, 0.08), vec3(0.08, 0.04, 0.08), Color::from_rgba(230, 190, 80, 255));
    }
    b.box_at(o, yaw, vec3(0.0, 0.26, 0.0), vec3(0.36, 0.42, 0.32), brown);
    b.box_at(o, yaw, vec3(0.0, 0.22, 0.161), vec3(0.24, 0.28, 0.01), belly);
    for (dx, dy) in [(-0.06, 0.28), (0.05, 0.24), (-0.03, 0.16), (0.07, 0.12)] {
        b.box_at(o, yaw, vec3(dx, dy, 0.167), vec3(0.04, 0.025, 0.01), brown_dark);
    }
    let flap = if singing || dance > 0.3 { (time * 18.0).sin() * 0.06 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * (0.2 + flap.abs()), 0.27 + flap, -0.01), vec3(0.05, 0.3, 0.26), brown_dark);
    }
    b.box_at(o, yaw, vec3(0.0, 0.58, 0.0), vec3(0.4, 0.26, 0.32), brown);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.14, 0.74, -0.01), vec3(0.07, 0.1, 0.06), brown_dark);
    }
    b.box_at(o, yaw, vec3(0.0, 0.57, 0.161), vec3(0.32, 0.2, 0.01), face);
    let blinking = time % 4.4 < 0.15;
    for side in [-1.0_f32, 1.0] {
        if blinking {
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.168), vec3(0.1, 0.02, 0.01), ink);
        } else {
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.166), vec3(0.11, 0.11, 0.01), Color::from_rgba(250, 196, 50, 255));
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.169), vec3(0.055, 0.055, 0.01), ink);
            b.box_at(o, yaw, vec3(side * 0.075 - 0.015, 0.595, 0.172), vec3(0.018, 0.018, 0.01), WHITE);
        }
    }
    b.box_at(o, yaw, vec3(0.0, 0.51, 0.175), vec3(0.04, if singing { 0.07 } else { 0.05 }, 0.03), Color::from_rgba(200, 150, 80, 255));
    b.box_at(o, yaw, vec3(0.0, 0.725, 0.0), vec3(0.46, 0.03, 0.06), phones);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.215, 0.6, 0.0), vec3(0.06, 0.13, 0.13), phones);
        b.box_at(o, yaw, vec3(side * 0.247, 0.6, 0.0), vec3(0.01, 0.05, 0.05), led);
    }
    b.box_at(o, yaw, vec3(0.16, 0.53, 0.1), vec3(0.025, 0.025, 0.16), Color::from_rgba(176, 180, 192, 255));
    b.box_at(o, yaw, vec3(0.11, 0.52, 0.18), vec3(0.06, 0.045, 0.035), Color::from_rgba(118, 196, 186, 255));
    draw_carry(b, o, yaw, vec3(0.0, 0.46, 0.22), carry);
}

fn draw_fox(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, calling: bool, dance: f32, carry: Option<Carry>) {
    let orange = Color::from_rgba(232, 112, 46, 255);
    let orange_dark = Color::from_rgba(196, 86, 34, 255);
    let cream = Color::from_rgba(250, 240, 226, 255);
    let sock = Color::from_rgba(60, 40, 36, 255);
    let ink = Color::from_rgba(26, 20, 20, 255);
    let scarf = Color::from_rgba(70, 150, 140, 255);
    let bob = if walking { (time * 13.0).sin().abs() * 0.045 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 13.0).sin() * 0.06 } else { 0.0 };
    for (lx, lz, phase) in [(-0.09, 0.15, 1.0), (0.09, 0.15, -1.0), (-0.09, -0.17, -1.0), (0.09, -0.17, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.1, lz + step * phase), vec3(0.07, 0.2, 0.07), orange);
        b.box_at(o, yaw, vec3(lx, 0.035, lz + step * phase), vec3(0.075, 0.07, 0.075), sock);
    }
    b.box_at(o, yaw, vec3(0.0, 0.3, -0.01), vec3(0.24, 0.2, 0.5), orange);
    b.box_at(o, yaw, vec3(0.0, 0.25, 0.12), vec3(0.18, 0.1, 0.26), cream);
    let swish = (time * 4.0).sin() * 0.08 + if calling { (time * 20.0).sin() * 0.05 } else { 0.0 };
    b.box_at(o, yaw, vec3(swish * 0.4, 0.34, -0.33), vec3(0.14, 0.14, 0.18), orange);
    b.box_at(o, yaw, vec3(swish * 0.8, 0.31, -0.48), vec3(0.17, 0.16, 0.18), orange_dark);
    b.box_at(o, yaw, vec3(swish, 0.28, -0.62), vec3(0.13, 0.13, 0.12), cream);
    let lift = if calling { 0.06 } else { 0.0 };
    b.box_at(o, yaw, vec3(0.0, 0.44, 0.2), vec3(0.28, 0.06, 0.16), scarf);
    b.box_at(o, yaw, vec3(0.1, 0.38, 0.27), vec3(0.06, 0.12, 0.04), scarf);
    b.box_at(o, yaw, vec3(0.0, 0.56 + lift, 0.24), vec3(0.28, 0.24, 0.26), orange);
    b.box_at(o, yaw, vec3(0.0, 0.5 + lift, 0.41), vec3(0.14, 0.1, 0.16), cream);
    b.box_at(o, yaw, vec3(0.0, 0.53 + lift, 0.49), vec3(0.05, 0.04, 0.03), ink);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.13, 0.48 + lift, 0.3), vec3(0.05, 0.08, 0.14), cream);
        b.box_at(o, yaw, vec3(side * 0.1, 0.74 + lift, 0.22), vec3(0.1, 0.14, 0.05), orange);
        b.box_at(o, yaw, vec3(side * 0.1, 0.83 + lift, 0.22), vec3(0.06, 0.06, 0.05), sock);
        b.box_at(o, yaw, vec3(side * 0.1, 0.73 + lift, 0.247), vec3(0.05, 0.08, 0.01), Color::from_rgba(250, 200, 190, 255));
        let blinking = time % 3.6 < 0.12;
        let h = if blinking || calling { 0.015 } else { 0.05 };
        b.box_at(o, yaw, vec3(side * 0.07, 0.6 + lift, 0.372), vec3(0.05, h, 0.01), ink);
    }
    if calling {
        b.box_at(o, yaw, vec3(0.0, 0.46 + lift, 0.45), vec3(0.1, 0.03, 0.08), Color::from_rgba(130, 50, 50, 255));
    }
    draw_carry(b, o, yaw, vec3(0.0, 0.48 + lift, 0.5), carry);
}

fn draw_otter(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, smashing: bool, dance: f32, carry: Option<Carry>, voice: f32) {
    let brown = Color::from_rgba(118, 82, 58, 255);
    let brown_dark = Color::from_rgba(90, 62, 44, 255);
    let face = Color::from_rgba(214, 196, 170, 255);
    let ink = Color::from_rgba(24, 20, 20, 255);
    let shell = Color::from_rgba(240, 196, 180, 255);
    let rock = Color::from_rgba(150, 152, 164, 255);
    let bob = if walking { (time * 11.0).sin().abs() * 0.04 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let sway = if walking { (time * 11.0).sin() * 0.08 } else { 0.0 };
    let y2 = yaw + sway;
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.08, 0.03, 0.06), vec3(0.09, 0.06, 0.12), brown_dark);
    }
    b.box_at(o, y2, vec3(0.0, 0.08, -0.22), vec3(0.1, 0.06, 0.24), brown_dark);
    b.box_at(o, y2, vec3(0.0, 0.26, 0.0), vec3(0.3, 0.36, 0.26), brown);
    b.box_at(o, y2, vec3(0.0, 0.26, 0.131), vec3(0.2, 0.26, 0.01), tinted(brown, 0.15));
    b.box_at(o, y2, vec3(0.0, 0.29, 0.15), vec3(0.16, 0.05, 0.08), shell);
    b.box_at(o, y2, vec3(0.0, 0.32, 0.16), vec3(0.1, 0.03, 0.06), tinted(shell, -0.12));
    let strike = if smashing { (1.0 - (voice / 0.4).min(1.0)).powi(2) } else { 1.0 };
    let rock_y = 0.36 + 0.14 * strike;
    b.box_at(o, y2, vec3(0.1, rock_y, 0.16), vec3(0.07, 0.07, 0.07), brown_dark);
    b.box_at(o, y2, vec3(0.04, rock_y + 0.01, 0.2), vec3(0.08, 0.07, 0.07), rock);
    b.box_at(o, y2, vec3(-0.1, 0.33, 0.15), vec3(0.07, 0.08, 0.07), brown_dark);
    b.box_at(o, y2, vec3(0.0, 0.55, 0.02), vec3(0.3, 0.24, 0.24), brown);
    b.box_at(o, y2, vec3(0.0, 0.51, 0.13), vec3(0.22, 0.13, 0.04), face);
    b.box_at(o, y2, vec3(0.0, 0.53, 0.155), vec3(0.06, 0.04, 0.02), ink);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, y2, vec3(side * 0.15, 0.66, 0.0), vec3(0.06, 0.05, 0.05), brown_dark);
        let blinking = time % 4.0 < 0.12;
        b.box_at(o, y2, vec3(side * 0.07, 0.6, 0.141), vec3(0.04, if blinking { 0.012 } else { 0.04 }, 0.01), ink);
        for dy in [0.0_f32, 0.02] {
            b.box_at(o, y2, vec3(side * 0.16, 0.5 + dy, 0.14), vec3(0.09, 0.006, 0.006), Color::from_rgba(240, 236, 226, 255));
        }
    }
    draw_carry(b, o, y2, vec3(0.0, 0.47, 0.2), carry);
}

fn walkable(world: &World, p: Vec2) -> bool {


    let r = CAT_RADIUS;
    for (ox, oz) in [(-r, -r), (r, -r), (-r, r), (r, r)] {
        let x = (p.x + ox).floor() as i32;
        let z = (p.y + oz).floor() as i32;
        match world.get(x, z) {
            Some(t) if t.kind == Kind::Tree => {
                let trunk = vec2(x as f32 + 0.5, z as f32 + 0.5);
                if (p.x - trunk.x).abs() < TRUNK_HALF + r && (p.y - trunk.y).abs() < TRUNK_HALF + r {
                    return false;
                }
            }
            Some(t) if !t.solid() => {}
            _ => return false,
        }
    }
    true
}

const TRUNK_HALF: f32 = 0.12;

fn rescue(world: &World, cat: Vec2) -> Vec2 {
    let cx = cat.x.floor() as i32;
    let cz = cat.y.floor() as i32;
    let mut best = cat;
    let mut best_dist = f32::MAX;
    for dz in -3..=3 {
        for dx in -3..=3 {
            let spot = vec2((cx + dx) as f32 + 0.5, (cz + dz) as f32 + 0.5);
            let d = spot.distance(cat);
            if d < best_dist && walkable(world, spot) {
                best = spot;
                best_dist = d;
            }
        }
    }
    best
}

fn icon_pixels<const N: usize>(png: &[u8]) -> [u8; N] {
    let img = Image::from_file_with_format(png, None).expect("could not read icon");
    img.bytes.try_into().expect("icon has the wrong size")
}

const KINDS: [Kind; 15] = [
    Kind::Grass, Kind::Flowers, Kind::Sprout, Kind::Tree, Kind::Water, Kind::Ice, Kind::Ash, Kind::Stone,
    Kind::House, Kind::Campfire, Kind::Path, Kind::WoodWall, Kind::StoneWall, Kind::Floor, Kind::Dock,
];

struct Progress {
    tiles: Vec<(u8, u8)>,
    wood: u32,
    stone: u32,
    fish: [u32; 3],
    fuel: f32,
    lit: bool,
    pile: u32,
    dishes: Vec<usize>,
    cook_progress: u32,
    characters: [usize; 2],
    spots: [Vec2; 2],
    echo: bool,
    beat: bool,
    bpm: f32,
    quantise: u8,
    notes: Vec<(f32, usize)>,
    songbook: Vec<Vec<(usize, usize)>>,
    partner_xp: u32,
    mix: [f32; 5],
    acorns: u32,
    pieces: Vec<home::Piece>,
    styles: (u8, u8, u16, bool),
    diary: Vec<DiaryEntry>,
    fish_lesson: bool,
    cook_lesson: bool,
    gather_lesson: bool,
    room: u8,
    dex: Vec<u32>,
    mush: [u32; 3],
    shroomdex: Vec<u32>,
    quests: Option<(i64, Vec<(u8, u32, u32, bool)>)>,
    stored: Vec<u8>,
    seq: [u16; 4],
}

fn kind_index(kind: Kind) -> u8 {
    KINDS.iter().position(|k| *k == kind).unwrap_or(0) as u8
}

fn encode_progress(p: &Progress) -> String {
    let list = |v: Vec<String>| v.join(",");
    let mut out = String::from("melody-quack-save 1\n");
    out += &format!("tiles {}\n", list(p.tiles.iter().map(|(k, s)| format!("{k}:{s}")).collect()));
    out += &format!("inv {} {} {} {} {}\n", p.wood, p.stone, p.fish[0], p.fish[1], p.fish[2]);
    out += &format!("camp {:.1} {} {}\n", p.fuel, p.lit as u8, p.pile);
    out += &format!("dishes {}\n", list(p.dishes.iter().map(|d| d.to_string()).collect()));
    out += &format!("cook {}\n", p.cook_progress);
    out += &format!("chars {} {}\n", p.characters[0], p.characters[1]);
    out += &format!("spots {:.2} {:.2} {:.2} {:.2}\n", p.spots[0].x, p.spots[0].y, p.spots[1].x, p.spots[1].y);
    out += &format!("sound {} {} {:.1} {}\n", p.echo as u8, p.beat as u8, p.bpm, p.quantise);
    out += &format!("notes {}\n", list(p.notes.iter().map(|(t, id)| format!("{t:.3}:{id}")).collect()));
    let tunes: Vec<String> = p.songbook.iter().map(|tune| list(tune.iter().map(|(at, n)| format!("{at}:{n}")).collect())).collect();
    out += &format!("songbook {}\n", tunes.join(";"));
    out += &format!("partner {}\n", p.partner_xp);
    out += &format!("mix {:.2} {:.2} {:.2} {:.2} {:.2}\n", p.mix[0], p.mix[1], p.mix[2], p.mix[3], p.mix[4]);
    out += &format!("acorns {}\n", p.acorns);
    out += &format!("lessons {} {} {}\n", p.fish_lesson as u8, p.cook_lesson as u8, p.gather_lesson as u8);
    out += &format!("room {}\n", p.room);
    out += &format!("dex {}\n", list(p.dex.iter().map(|n| n.to_string()).collect()));
    out += &format!("mush {} {} {}\n", p.mush[0], p.mush[1], p.mush[2]);
    out += &format!("shroomdex {}\n", list(p.shroomdex.iter().map(|n| n.to_string()).collect()));
    if p.seq.iter().any(|r| *r != 0) {
        out += &format!("seq {} {} {} {}\n", p.seq[0], p.seq[1], p.seq[2], p.seq[3]);
    }
    if !p.stored.is_empty() {
        out += &format!("stored {}\n", list(p.stored.iter().map(|n| n.to_string()).collect()));
    }
    if let Some((day, qs)) = &p.quests {
        out += &format!("quests {} {}\n", day, list(qs.iter().map(|(k, t, pr, d)| format!("{}:{}:{}:{}", k, t, pr, *d as u8)).collect()));
    }
    out += &format!("home {}\n", list(p.pieces.iter().map(|q| if q.day != 0 { format!("{}:{}:{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot, q.state, q.day) } else if q.state == 0 { format!("{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot) } else { format!("{}:{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot, q.state) }).collect()));
    out += &format!("style {} {} {} {}\n", p.styles.0, p.styles.1, p.styles.2, p.styles.3 as u8);
    let keep_from = p.diary.len().saturating_sub(10);
    for (k, e) in p.diary.iter().enumerate() {
        let events = if k >= keep_from { e.events.iter().map(|(t, id, v)| format!("{:.2}:{}:{:.2}", t, id, v)).collect::<Vec<_>>().join(";") } else { String::new() };
        out += &format!("diary {:.0}|{}|{}|{}|{}|{}\n", e.when, e.seconds, e.stars, e.who.0, e.who.1, events);
    }
    out
}

fn decode_progress(text: &str, characters: usize) -> Option<Progress> {
    let mut lines = text.lines();
    if lines.next()? != "melody-quack-save 1" {
        return None;
    }
    let mut p = Progress {
        tiles: Vec::new(),
        wood: 0,
        stone: 0,
        fish: [0; 3],
        fuel: 0.0,
        lit: false,
        pile: 0,
        dishes: Vec::new(),
        cook_progress: 0,
        characters: [0, 4],
        spots: [vec2(6.5, 16.5), vec2(7.5, 16.5)],
        echo: false,
        beat: false,
        bpm: DEFAULT_BPM,
        quantise: 1,
        notes: Vec::new(),
        songbook: Vec::new(),
        partner_xp: 0,
        mix: [1.0; 5],
        acorns: 0,
        pieces: Vec::new(),
        styles: (0, 0, 1 | (1 << 4), true),
        diary: Vec::new(),
        fish_lesson: false,
        cook_lesson: false,
        gather_lesson: false,
        room: 0,
        dex: vec![0; fish::COUNT],
        mush: [0; 3],
        shroomdex: vec![0; mushroom::COUNT],
        quests: None,
        stored: Vec::new(),
        seq: [0; 4],
    };
    for line in lines {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        let nums: Vec<f32> = rest.split_whitespace().filter_map(|v| v.parse().ok()).collect();
        let pairs = || rest.split(',').filter_map(|v| v.split_once(':')).collect::<Vec<_>>();
        match key {
            "tiles" => {
                p.tiles = pairs().iter().filter_map(|(k, s)| Some((k.parse().ok()?, s.parse().ok()?))).collect();
            }
            "inv" if nums.len() == 5 => {
                p.wood = nums[0] as u32;
                p.stone = nums[1] as u32;
                p.fish = [nums[2] as u32, nums[3] as u32, nums[4] as u32];
            }
            "camp" if nums.len() == 3 => {
                p.fuel = nums[0];
                p.lit = nums[1] > 0.0;
                p.pile = nums[2] as u32;
            }
            "dishes" => p.dishes = rest.split(',').filter_map(|v| v.parse().ok()).filter(|d| *d < DISHES.len()).collect(),
            "cook" if nums.len() == 1 => p.cook_progress = (nums[0] as u32).min(COOK_GOAL),
            "chars" if nums.len() == 2 => p.characters = [(nums[0] as usize).min(characters - 1), (nums[1] as usize).min(characters - 1)],
            "spots" if nums.len() == 4 => p.spots = [vec2(nums[0], nums[1]), vec2(nums[2], nums[3])],
            "sound" if nums.len() == 4 => {
                p.echo = nums[0] > 0.0;
                p.beat = nums[1] > 0.0;
                p.bpm = nums[2].clamp(60.0, 200.0);
                p.quantise = nums[3] as u8;
            }
            "notes" => {
                p.notes = pairs().iter().filter_map(|(t, id)| Some((t.parse::<f32>().ok()?, id.parse::<usize>().ok()?))).filter(|(t, id)| *id < 32 && t.is_finite()).collect();
            }
            "songbook" => {
                p.songbook = rest
                    .split(';')
                    .map(|tune| tune.split(',').filter_map(|v| v.split_once(':')).filter_map(|(a, n)| Some((a.parse::<usize>().ok()?, n.parse::<usize>().ok()?))).filter(|(a, n)| *a < LOOP_STEPS && *n < 6).collect::<Vec<_>>())
                    .filter(|tune| !tune.is_empty())
                    .take(8)
                    .collect();
            }
            "partner" if nums.len() == 1 => p.partner_xp = nums[0] as u32,
            "acorns" if nums.len() == 1 => p.acorns = nums[0] as u32,
            "lessons" if !nums.is_empty() => {
                p.fish_lesson = nums[0] > 0.0;
                if nums.len() > 1 {
                    p.cook_lesson = nums[1] > 0.0;
                }
                if nums.len() > 2 {
                    p.gather_lesson = nums[2] > 0.0;
                }
            },
            "room" if nums.len() == 1 => p.room = (nums[0].max(0.0) as u8).min(home::MAX_SIZE),
            "dex" => {
                for (k, v) in rest.split(',').enumerate().take(fish::COUNT) {
                    p.dex[k] = v.trim().parse().unwrap_or(0);
                }
            }
            "mush" if nums.len() == 3 => p.mush = [nums[0] as u32, nums[1] as u32, nums[2] as u32],
            "seq" if nums.len() == 4 => p.seq = [nums[0] as u16, nums[1] as u16, nums[2] as u16, nums[3] as u16],
            "stored" => p.stored = rest.split(',').filter_map(|v| v.trim().parse::<u8>().ok()).filter(|v| (*v as usize) < home::ITEMS.len()).collect(),
            "quests" => {
                let mut parts = rest.splitn(2, ' ');
                if let (Some(day), Some(items)) = (parts.next().and_then(|d| d.trim().parse::<i64>().ok()), parts.next()) {
                    let qs: Vec<(u8, u32, u32, bool)> = items
                        .split(',')
                        .filter_map(|q| {
                            let f: Vec<u32> = q.split(':').filter_map(|n| n.trim().parse().ok()).collect();
                            if f.len() == 4 { Some((f[0] as u8, f[1], f[2], f[3] > 0)) } else { None }
                        })
                        .collect();
                    p.quests = Some((day, qs));
                }
            }
            "shroomdex" => {
                for (k, v) in rest.split(',').enumerate().take(mushroom::COUNT) {
                    p.shroomdex[k] = v.trim().parse().unwrap_or(0);
                }
            }
            "diary" => {
                let parts: Vec<&str> = rest.splitn(6, '|').collect();
                if parts.len() == 6 {
                    let events = parts[5]
                        .split(';')
                        .filter_map(|ev| {
                            let f: Vec<&str> = ev.split(':').collect();
                            if f.len() == 3 {
                                Some((f[0].parse::<f32>().ok()?, f[1].parse::<u16>().ok()?, f[2].parse::<f32>().ok()?.clamp(0.0, 1.5)))
                            } else {
                                None
                            }
                        })
                        .collect();
                    if let (Ok(when), Ok(seconds), Ok(stars), Ok(a), Ok(b)) = (parts[0].parse::<f64>(), parts[1].parse::<u32>(), parts[2].parse::<u8>(), parts[3].parse::<usize>(), parts[4].parse::<usize>()) {
                        p.diary.push(DiaryEntry { when, seconds, stars: stars.min(3), who: (a.min(characters - 1), b.min(characters - 1)), events });
                    }
                }
            }
            "home" => {
                p.pieces = rest
                    .split(',')
                    .filter_map(|v| {
                        let parts: Vec<i32> = v.split(':').filter_map(|n| n.parse().ok()).collect();
                        if (4..=6).contains(&parts.len()) {
                            Some(home::Piece { item: home::Item::from_index(parts[0].max(0) as u8), x: parts[1], z: parts[2], rot: (parts[3].rem_euclid(4)) as u8, state: parts.get(4).copied().unwrap_or(0).clamp(0, 255) as u8, day: parts.get(5).copied().unwrap_or(0) })
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            "style" if nums.len() == 4 => {
                p.styles = ((nums[0] as u8).min(home::WALLS.len() as u8 - 1), (nums[1] as u8).min(home::FLOORS.len() as u8 - 1), nums[2] as u16, nums[3] > 0.0);
            }
            "mix" if nums.len() == 5 => {
                for (k, v) in nums.iter().enumerate() {
                    p.mix[k] = v.clamp(0.0, 1.0);
                }
            }
            _ => {}
        }
    }
    Some(p)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Melody Quack".to_owned(),
        window_width: (VIEW_W * 3) as i32,
        window_height: (VIEW_H * 3) as i32,
        icon: Some(miniquad::conf::Icon {
            small: icon_pixels(include_bytes!("../assets/icon16.png")),
            medium: icon_pixels(include_bytes!("../assets/icon32.png")),
            big: icon_pixels(include_bytes!("../assets/icon64.png")),
        }),
        ..Default::default()
    }
}


#[derive(Clone, Copy, PartialEq)]
enum Carry {
    Stick,
    Stone,
    Acorn,
}

struct Player {
    character: usize,
    body: Body,
    pos: Vec2,
    facing: Vec2,
    yaw: f32,
    voice_timer: f32,
    key_glow: [f32; 6],
    walking: bool,
    cam: Vec2,
    carrying: Option<Carry>,
    seated: bool,
    sleeping: bool,
    sleep_yaw: f32,
    emote: Option<(usize, f32)>,
    perch: f32,
    lift: f32,
    sniff: f32,
}

impl Player {
    fn new(character: usize, body: Body, pos: Vec2) -> Player {
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

    fn place_at(&mut self, p: Vec2) {
        self.pos = p;
        self.cam = p;
    }

    fn step(&mut self, dir: Vec2, world: &World, dt: f32) -> f32 {
        self.step_with(dir, dt, |p| walkable(world, p), |p| rescue(world, p))
    }

    fn step_with(&mut self, dir: Vec2, dt: f32, walk: impl Fn(Vec2) -> bool, save: impl Fn(Vec2) -> Vec2) -> f32 {
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

    fn ahead(&self) -> (i32, i32) {
        let (dx, dz) = if self.facing.x.abs() > self.facing.y.abs() {
            (self.facing.x.signum() as i32, 0)
        } else {
            (0, self.facing.y.signum() as i32)
        };
        (self.pos.x.floor() as i32 + dx, self.pos.y.floor() as i32 + dz)
    }

    fn ground(&self, world: &World) -> f32 {
        ground_top(world.get(self.pos.x.floor() as i32, self.pos.y.floor() as i32).map(|t| t.kind).unwrap_or(Kind::Grass))
    }

    fn draw(&self, b: &mut Batch, world: &World, time: f32, dance: f32) {
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

const P2_NOTE_KEYS: [KeyCode; 6] = [
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
    KeyCode::Key0,
    KeyCode::Minus,
    KeyCode::Equal,
];
const P1_KEY_LABELS: [&str; 6] = ["1", "2", "3", "4", "5", "6"];
const P2_KEY_LABELS: [&str; 6] = ["7", "8", "9", "0", "-", "="];

fn held(keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| is_key_down(*k))
}

fn direction(up: &[KeyCode], down: &[KeyCode], left: &[KeyCode], right: &[KeyCode]) -> Vec2 {
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

fn ring_tone(freq: f32, partials: &[(f32, f32, f32)], length: f32, attack: f32, gain: f32) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    (0..(length * sr) as usize)
        .map(|i| {
            let t = i as f32 / sr;
            let tone: f32 = partials.iter().map(|(r, a, d)| (2.0 * PI * freq * r * t).sin() * a * (-t * d).exp()).sum();
            tone * smoothstep(t / attack) * smoothstep((length - t) / 0.05) * gain
        })
        .collect()
}

fn noise_burst(length: f32, decay: f32, brightness: f32, seed: u32, gain: f32) -> Vec<f32> {
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

fn add_into(mut a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    if b.len() > a.len() {
        a.resize(b.len(), 0.0);
    }
    for (i, v) in b.into_iter().enumerate() {
        a[i] += v;
    }
    a
}

fn render_lid() -> Vec<f32> {
    add_into(
        ring_tone(540.0, &[(1.0, 0.5, 14.0), (2.76, 0.4, 18.0), (5.4, 0.3, 24.0), (8.93, 0.2, 30.0)], 0.5, 0.001, 0.4),
        noise_burst(0.2, 22.0, 0.7, 0x2468_ace1, 0.5),
    )
}

fn render_chop() -> Vec<f32> {
    add_into(ring_tone(1650.0, &[(1.0, 1.0, 70.0), (2.3, 0.4, 90.0)], 0.12, 0.0005, 0.35), noise_burst(0.05, 120.0, 0.9, 0x1111_2222, 0.4))
}

fn render_sizzle() -> Vec<f32> {
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

fn render_plink() -> Vec<f32> {
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

fn render_splash() -> Vec<f32> {
    noise_burst(0.5, 7.0, 0.25, 0x7777_1357, 0.6)
}

fn render_marimba(freq: f32) -> Vec<f32> {
    ring_tone(freq, &[(1.0, 1.0, 7.0), (4.0, 0.35, 25.0), (10.0, 0.12, 60.0)], 0.7, 0.002, 0.45)
}

fn render_piano(freq: f32) -> Vec<f32> {
    let body = ring_tone(freq, &[(1.0, 1.0, 1.6), (2.0, 0.55, 2.4), (3.0, 0.3, 3.4), (4.0, 0.16, 5.0), (5.0, 0.08, 7.0)], 1.8, 0.003, 0.42);
    let hammer = noise_burst(0.03, 120.0, 0.6, (freq * 10.0) as u32, 0.06);
    add_into(body, hammer)
}

fn render_bell(freq: f32) -> Vec<f32> {
    ring_tone(freq, &[(1.0, 1.0, 2.5), (2.0, 0.5, 3.0), (2.76, 0.4, 4.0), (5.4, 0.25, 6.0), (8.93, 0.15, 9.0)], 1.6, 0.002, 0.28)
}

fn render_crackle() -> Vec<f32> {
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

struct View<'a> {
    world: &'a World,
    players: &'a [Player],
    particles: &'a [Particle],
    camp: &'a Camp,
    time: f32,
    dance: f32,
    dusk: f32,
    extras: &'a Extras,
}

const LANTERNS: [(f32, f32); 4] = [(2.5, 16.2), (6.4, 15.7), (10.4, 15.4), (13.3, 17.7)];

fn render_view(target: &RenderTarget, left: f32, width: f32, height: f32, focus_pos: Vec2, cam: Vec2, view: &View, split: bool) {
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

enum Task {
    Idle,
    Seek { spot: Vec2, tile: (i32, i32), carry: Carry },
    Sniff { tile: (i32, i32), timer: f32, carry: Carry },
    Return,
    Sit,
}

struct DogBrain {
    task: Task,
    stuck: f32,
    trip: f32,
    rest: f32,
}

impl DogBrain {
    fn new() -> DogBrain {
        DogBrain { task: Task::Idle, stuck: 0.0, trip: 0.0, rest: 0.0 }
    }
}

fn in_forest(x: i32, z: i32) -> bool {
    z < 11 || x > 13
}

fn find_source(world: &mut World, from: Vec2, kind: Kind) -> Option<((i32, i32), Vec2)> {
    let mut options = Vec::new();
    for z in 1..MAP as i32 - 1 {
        for x in 1..MAP as i32 - 1 {
            if world.get(x, z).map(|t| t.kind != kind || t.fire > 0.0).unwrap_or(true) {
                continue;
            }
            if kind == Kind::Tree && !in_forest(x, z) {
                continue;
            }
            for (dx, dz) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                let spot = vec2((x + dx) as f32 + 0.5, (z + dz) as f32 + 0.5);
                let dist = spot.distance(from);
                if walkable(world, spot) && dist > 2.0 && dist < 14.0 {
                    options.push(((x, z), spot));
                    break;
                }
            }
        }
    }
    if options.is_empty() {
        return None;
    }
    let pick = (world.rand() * options.len() as f32) as usize % options.len();
    Some(options[pick])
}

fn steer(world: &World, from: Vec2, to: Vec2) -> Vec2 {
    let want = to - from;
    if want.length() < 0.05 {
        return Vec2::ZERO;
    }
    let dir = want.normalize();
    for angle in [0.0_f32, 0.6, -0.6, 1.2, -1.2, 1.6, -1.6] {
        let (s, c) = angle.sin_cos();
        let d = vec2(dir.x * c - dir.y * s, dir.x * s + dir.y * c);
        if walkable(world, from + d * 0.35) {
            return d;
        }
    }
    dir
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    WoodWall,
    StoneWall,
    Floor,
    Remove,
}

impl Tool {
    fn next(self) -> Tool {
        match self {
            Tool::WoodWall => Tool::StoneWall,
            Tool::StoneWall => Tool::Floor,
            Tool::Floor => Tool::Remove,
            Tool::Remove => Tool::WoodWall,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Tool::WoodWall => "wood wall",
            Tool::StoneWall => "stone wall",
            Tool::Floor => "floor",
            Tool::Remove => "remove",
        }
    }
}

struct Inventory {
    wood: u32,
    stone: u32,
    fish: [u32; 3],
    mush: [u32; 3],
    acorns: u32,
}

const ACORN_COLOR: Color = Color::new(0.78, 0.5, 0.26, 1.0);
const TREE_REST: f32 = 40.0;


const DISHES: [&str; 8] = ["campfire tea", "grilled fish", "fish stew", "golden koi soup", "grilled mushrooms", "mushroom stew", "fish & mushroom hotpot", "golden truffle soup"];
const DOCK_END: Vec2 = Vec2::new(15.5, 18.5);
const BOBBER: Vec3 = Vec3::new(16.75, 0.31, 18.5);

#[derive(Clone, Copy)]
struct Vis {
    bands: [f32; 4],
    pulse: f32,
    pads: [f32; 4],
    seq: usize,
}

thread_local! {
    static VIS: std::cell::Cell<Vis> = std::cell::Cell::new(Vis { bands: [0.0; 4], pulse: 0.0, pads: [0.0; 4], seq: 0 });
}

fn vis() -> Vis {
    VIS.with(|v| v.get())
}

fn hsv(h: f32, s: f32, v: f32) -> Color {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let (r, g, b) = match h as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Color::new(r + m, g + m, b + m, 1.0)
}

const TV_CHANNELS: [&str; 5] = ["spectrum", "tunnel", "plasma", "rain", "scope"];

fn tv_pixel(channel: u8, x: usize, y: usize, w: usize, h: usize, time: f32, v: &Vis) -> Option<Color> {
    let fx = x as f32 / w as f32;
    let fy = y as f32 / h as f32;
    let bass = v.bands[0];
    let mids = (v.bands[1] + v.bands[2]) * 0.5;
    let highs = v.bands[3];
    let energy = (bass + mids + highs) / 3.0;
    match channel {
        1 => {
            let band = (fx * 4.0) as usize % 4;
            let local = (fx * 4.0).fract();
            if local < 0.15 || local > 0.85 {
                return None;
            }
            let level = (v.bands[band] * 0.85 + v.pulse * 0.15).min(1.0);
            let from_mid = (fy - 0.5).abs() * 2.0;
            if from_mid < level {
                Some(hsv(0.55 - band as f32 * 0.14 + time * 0.03, 0.8, 0.5 + 0.5 * (1.0 - from_mid)))
            } else {
                None
            }
        }
        2 => {
            let dx = (fx - 0.5) * 1.8;
            let dy = fy - 0.5;
            let r = dx.abs().max(dy.abs());
            let wave = (r * 9.0 - time * (2.0 + bass * 6.0)).sin();
            if wave > 0.2 - energy * 0.8 {
                Some(hsv(r * 0.8 + time * 0.1 + bass * 0.3, 0.75, 0.35 + 0.65 * (wave * 0.5 + 0.5)))
            } else {
                Some(Color::new(0.03, 0.02, 0.06, 1.0))
            }
        }
        3 => {
            let a = (fx * 7.0 + time * 1.3).sin() + (fy * 6.0 - time * 1.1).sin() + ((fx + fy) * 5.0 + time * (0.8 + bass * 3.0)).sin();
            Some(hsv(a * 0.12 + time * 0.05 + mids * 0.4, 0.7, 0.35 + 0.4 * energy + 0.25 * v.pulse))
        }
        4 => {
            let speed = 0.6 + highs * 3.0 + v.pulse;
            let col_seed = ((x as u32).wrapping_mul(2654435761) >> 16) as f32 / 65535.0;
            let head = (time * speed * (0.6 + col_seed) + col_seed * 10.0).fract();
            let d = (fy - head).rem_euclid(1.0);
            if d < 0.25 {
                let glow = 1.0 - d / 0.25;
                Some(hsv(0.33 + mids * 0.3, 0.7, glow * (0.4 + 0.6 * energy.max(0.3))))
            } else {
                None
            }
        }
        5 => {
            let dx = (fx - 0.5) * 1.8;
            let dy = fy - 0.5;
            let ang = dy.atan2(dx);
            let ring = 0.22 + 0.12 * bass + 0.05 * (ang * 6.0 + time * 3.0).sin() * (0.3 + highs);
            let dist = (dx * dx + dy * dy).sqrt();
            if (dist - ring).abs() < 0.035 + mids * 0.04 {
                Some(hsv(ang / 6.283 + time * 0.2, 0.6, 1.0))
            } else if (dist - ring * 0.55).abs() < 0.02 {
                Some(hsv(0.9 + time * 0.1, 0.6, 0.5 + v.pulse * 0.5))
            } else {
                None
            }
        }
        _ => None,
    }
}

thread_local! {
    static SHROOMS: std::cell::RefCell<Vec<(Vec2, usize)>> = std::cell::RefCell::new(Vec::new());
}

fn draw_shrooms(b: &mut Batch, time: f32) {
    SHROOMS.with(|list| {
        for (at, species) in list.borrow().iter() {
            let look = mushroom::look(*species);
            let base = vec3(at.x, 0.5, at.y);
            let bob = 0.01 * (time * 2.0 + at.x).sin();
            match look.shape {
                1 => b.cube(base + vec3(0.0, 0.07 + bob, 0.0), vec3(0.16, 0.13, 0.16), look.cap),
                2 => {
                    b.cube(base + vec3(0.0, 0.06, 0.0), vec3(0.05, 0.12, 0.05), look.stem);
                    b.cube(base + vec3(0.0, 0.17 + bob, 0.0), vec3(0.09, 0.14, 0.09), look.cap);
                }
                3 => {
                    b.cube(base + vec3(0.0, 0.05 + bob, 0.0), vec3(0.2, 0.04, 0.14), look.cap);
                    b.cube(base + vec3(0.03, 0.1 + bob, 0.0), vec3(0.15, 0.04, 0.11), look.cap);
                }
                4 => {
                    for k in 0..3 {
                        let ox = (k as f32 - 1.0) * 0.05;
                        b.cube(base + vec3(ox, 0.08 + bob, (k as f32 * 1.7).sin() * 0.03), vec3(0.035, 0.16 - k as f32 * 0.02, 0.035), look.cap);
                    }
                }
                _ => {
                    b.cube(base + vec3(0.0, 0.05, 0.0), vec3(0.05, 0.1, 0.05), look.stem);
                    b.cube(base + vec3(0.0, 0.12 + bob, 0.0), vec3(0.18, 0.06, 0.18), look.cap);
                    if let Some(spot) = look.spot {
                        b.cube(base + vec3(0.04, 0.155 + bob, -0.03), vec3(0.04, 0.012, 0.04), spot);
                        b.cube(base + vec3(-0.05, 0.155 + bob, 0.04), vec3(0.03, 0.012, 0.03), spot);
                    }
                }
            }
            if ((time * 0.8 + at.x * 3.1 + at.y).sin()) > 0.97 {
                b.unlit = true;
                b.cube(base + vec3(0.06, 0.3, 0.0), vec3(0.03, 0.03, 0.03), Color::from_rgba(255, 250, 200, 255));
                b.unlit = false;
            }
        }
    });
}

thread_local! {
    static BOBBER_AT: std::cell::Cell<[(f32, f32); 2]> = std::cell::Cell::new([(0.0, 0.0); 2]);
}
const COOK_GOAL: u32 = 24;
const STICK_FUEL: f32 = 45.0;
const FUEL_WANT: f32 = 140.0;
const FIRE_SPOT: Vec2 = Vec2::new(8.5, 17.5);
const SEAT_TILES: [(i32, i32); 2] = [(7, 17), (9, 17)];
const PERC_NAMES: [&str; 4] = ["kick", "lid", "chop", "sizzle"];

fn buildable(world: &World, x: i32, z: i32) -> bool {
    if in_forest(x, z) || x <= 0 || z <= 0 || x >= MAP as i32 - 1 || z >= MAP as i32 - 1 || SEAT_TILES.contains(&(x, z)) {
        return false;
    }
    world
        .get(x, z)
        .map(|t| matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash | Kind::Floor | Kind::WoodWall | Kind::StoneWall) && t.fire <= 0.0)
        .unwrap_or(false)
}

fn build(world: &mut World, inv: &mut Inventory, tool: Tool, x: i32, z: i32, blocked: bool) -> Result<Option<(Kind, u8)>, &'static str> {
    if !buildable(world, x, z) {
        return Ok(None);
    }
    let t = world.get_mut(x, z).unwrap();
    match tool {
        Tool::WoodWall | Tool::StoneWall => {
            let (kind, stock) = if tool == Tool::WoodWall { (Kind::WoodWall, &mut inv.wood) } else { (Kind::StoneWall, &mut inv.stone) };
            if *stock == 0 {
                return Err(if tool == Tool::WoodWall { "need wood - your friend can fetch some" } else { "need stone - your friend can fetch some" });
            }
            if t.kind == kind && t.stack < 3 {
                t.stack += 1;
            } else if matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash | Kind::Floor) && !blocked {
                t.kind = kind;
                t.stack = 1;
            } else {
                return Ok(None);
            }
            *stock -= 1;
            Ok(Some((kind, t.stack)))
        }
        Tool::Floor => {
            if inv.wood == 0 {
                return Err("need wood - your friend can fetch some");
            }
            if matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash) {
                t.kind = Kind::Floor;
                inv.wood -= 1;
                Ok(Some((Kind::Floor, 0)))
            } else {
                Ok(None)
            }
        }
        Tool::Remove => match t.kind {
            Kind::WoodWall | Kind::StoneWall => {
                let kind = t.kind;
                if kind == Kind::WoodWall {
                    inv.wood += 1;
                } else {
                    inv.stone += 1;
                }
                t.stack -= 1;
                if t.stack == 0 {
                    t.kind = Kind::Grass;
                }
                Ok(Some((kind, t.stack)))
            }
            Kind::Floor => {
                t.kind = Kind::Grass;
                inv.wood += 1;
                Ok(Some((Kind::Floor, 0)))
            }
            _ => Ok(None),
        },
    }
}

struct Fishing {
    active: bool,
    caster: usize,
    cue_at: f32,
    hits: u32,
    tries: u32,
    hit_by: [bool; 2],
    early: [bool; 2],
    lesson: bool,
    perfects: u32,
    dip: f32,
    jump: Option<(usize, f32)>,
    tight: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum QuestKind {
    Fish,
    Mushroom,
    Cook,
    Water,
    Sticks,
    Shake,
    Sing,
    Dance,
    Star,
    Emoji,
}

const QUEST_KINDS: [QuestKind; 10] = [QuestKind::Fish, QuestKind::Mushroom, QuestKind::Cook, QuestKind::Water, QuestKind::Sticks, QuestKind::Shake, QuestKind::Sing, QuestKind::Dance, QuestKind::Star, QuestKind::Emoji];

#[derive(Clone, Copy)]
struct Quest {
    kind: QuestKind,
    target: u32,
    progress: u32,
    done: bool,
}

impl Quest {
    fn reward(&self) -> u32 {
        match self.kind {
            QuestKind::Fish => 12 * self.target,
            QuestKind::Mushroom => 5 * self.target,
            QuestKind::Cook => 15 * self.target,
            QuestKind::Water => 10,
            QuestKind::Sticks => 6 * self.target,
            QuestKind::Shake => 4 * self.target,
            QuestKind::Sing => 10,
            QuestKind::Dance => 20,
            QuestKind::Star => 20,
            QuestKind::Emoji => 3 * self.target,
        }
    }

    fn label(&self) -> String {
        let n = self.target;
        let s = if n == 1 { "" } else { "s" };
        match self.kind {
            QuestKind::Fish => format!("catch {n} fish"),
            QuestKind::Mushroom => format!("pick {n} mushroom{s}"),
            QuestKind::Cook => format!("cook {n} dish{}", if n == 1 { "" } else { "es" }),
            QuestKind::Water => "water a plant".to_string(),
            QuestKind::Sticks => format!("pick up {n} stick{s}"),
            QuestKind::Shake => format!("shake acorns from {n} tree{s}"),
            QuestKind::Sing => format!("sing {n} notes"),
            QuestKind::Dance => format!("land {n} steps on the dance arcade"),
            QuestKind::Star => "earn a star playing together".to_string(),
            QuestKind::Emoji => format!("send {n} emoji{s}"),
        }
    }
}

struct Quests {
    day: i64,
    list: Vec<Quest>,
}

fn daily_quests(day: i64, has_plant: bool, has_arcade: bool) -> Vec<Quest> {
    let mut out: Vec<Quest> = Vec::new();
    let mut salt = 0;
    while out.len() < 3 && salt < 60 {
        let kind = QUEST_KINDS[(hash(day as i32, salt, 501) * QUEST_KINDS.len() as f32) as usize % QUEST_KINDS.len()];
        salt += 1;
        if out.iter().any(|q| q.kind == kind) || (kind == QuestKind::Water && !has_plant) || (kind == QuestKind::Dance && !has_arcade) {
            continue;
        }
        let roll = hash(day as i32, salt, 502);
        let target = match kind {
            QuestKind::Fish => 1 + (roll * 3.0) as u32,
            QuestKind::Mushroom => 2 + (roll * 4.0) as u32,
            QuestKind::Cook => 1 + (roll * 2.0) as u32,
            QuestKind::Sticks => 1 + (roll * 3.0) as u32,
            QuestKind::Shake => 2 + (roll * 3.0) as u32,
            QuestKind::Sing => 20 + (roll * 4.0) as u32 * 10,
            QuestKind::Dance => 20 + (roll * 3.0) as u32 * 10,
            QuestKind::Emoji => 2 + (roll * 3.0) as u32,
            _ => 1,
        };
        out.push(Quest { kind, target, progress: 0, done: false });
    }
    out
}

fn quest_event(quests: &mut Quests, kind: QuestKind, amount: u32, inv: &mut Inventory, toasts: &mut Vec<(String, f32)>) {
    for q in quests.list.iter_mut() {
        if q.kind == kind && !q.done {
            q.progress = (q.progress + amount).min(q.target);
            if q.progress >= q.target {
                q.done = true;
                inv.acorns += q.reward();
                toasts.push((format!("daily quest done: {} - +{} acorns", q.label(), q.reward()), 3.5));
            }
        }
    }
}

struct Sampler {
    open: bool,
    owner: usize,
    grid: [[bool; 16]; 4],
    cursor: (usize, usize),
    pads: [Option<audio::Sound>; 4],
    recording: Option<(usize, f32)>,
    step: usize,
}

const SAMPLER_ROWS: [&str; 4] = ["kick", "clap", "hat", "voice"];

fn tidy_recording(raw: &[f32], rate: f32) -> Option<Vec<f32>> {
    let start = raw.iter().position(|s| s.abs() > 0.03)?;
    let ratio = rate / SAMPLE_RATE as f32;
    let len = ((raw.len() - start) as f32 / ratio).min(SAMPLE_RATE as f32 * 1.2) as usize;
    let mut out: Vec<f32> = (0..len)
        .map(|i| {
            let p = start as f32 + i as f32 * ratio;
            let k = p as usize;
            let f = p - k as f32;
            let a = raw.get(k).copied().unwrap_or(0.0);
            let b = raw.get(k + 1).copied().unwrap_or(a);
            a * (1.0 - f) + b * f
        })
        .collect();
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak < 0.03 {
        return None;
    }
    let fade = (SAMPLE_RATE as f32 * 0.04) as usize;
    let n = out.len();
    for (i, s) in out.iter_mut().enumerate() {
        *s *= 0.8 / peak;
        if i + fade > n {
            *s *= (n - i) as f32 / fade as f32;
        }
    }
    Some(out)
}

struct Arrow {
    lane: usize,
    at: f32,
    slot: i64,
    hit: [bool; 2],
    missed: [bool; 2],
}

struct Ddr {
    on: [bool; 2],
    arrows: Vec<Arrow>,
    score: [u32; 2],
    combo: [u32; 2],
    judge: [Option<(&'static str, Color, f32)>; 2],
    flash: [[f32; 4]; 2],
    last_mic: f32,
}

impl Ddr {
    fn add(&mut self, lane: usize, at: f32, slot_len: f32) {
        let slot = (at / slot_len).round() as i64;
        if self.arrows.iter().any(|a| a.slot == slot && a.lane == lane) || self.arrows.iter().filter(|a| a.slot == slot).count() >= 2 {
            return;
        }
        self.arrows.push(Arrow { lane, at, slot, hit: [false; 2], missed: [false; 2] });
    }
}

const DDR_LANE_OF_BAND: [usize; 4] = [0, 1, 3, 2];

fn ddr_arrow_pixels(lane: usize) -> Vec<(usize, usize, Color)> {
    let rows = ["....x....", "...xxx...", "..xxxxx..", ".xxxxxxx.", "xxxxxxxxx", "...xxx...", "...xxx...", "...xxx...", "...xxx..."];
    let mut out = Vec::new();
    let c = note_color(lane);
    let edge = Color::new(c.r * 0.55, c.g * 0.55, c.b * 0.55, 1.0);
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch != 'x' {
                continue;
            }
            let filled = |xx: i32, yy: i32| xx >= 0 && yy >= 0 && (yy as usize) < 9 && (xx as usize) < 9 && rows[yy as usize].as_bytes()[xx as usize] == b'x';
            let border = !(filled(x as i32 - 1, y as i32) && filled(x as i32 + 1, y as i32) && filled(x as i32, y as i32 - 1) && filled(x as i32, y as i32 + 1));
            let (rx, ry) = match lane {
                0 => (y, 8 - x),
                1 => (8 - x, 8 - y),
                3 => (8 - y, x),
                _ => (x, y),
            };
            out.push((rx, ry, if border { edge } else { c }));
        }
    }
    out
}

const FISH_TRIES: u32 = 3;
const FISH_CUE_GAP: f32 = 8.0;

fn next_beat(p: f32) -> f32 {
    (p / 4.0).floor() * 4.0 + 4.0
}

struct Extras {
    fishing: bool,
    ring: f32,
    in_window: bool,
    dip: f32,
    jump: Option<(usize, f32)>,
    cat: Vec2,
    pot: bool,
    hearts: Vec<(Vec3, f32)>,
    ghost: Option<(i32, i32, Tool)>,
    friend_rod: Option<Vec2>,
    sticks: Vec<Vec2>,
}

#[macroquad::main(window_conf)]
async fn main() {
    let audio = audio::Audio::new();
    let cast_list = roster();
    let voices: Vec<Vec<audio::Sound>> = cast_list.iter().map(|c| NOTES.iter().map(|n| audio.load_ringing(render_voice(c.voice, *n))).collect()).collect();
    let marimba: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_marimba(midi_to_freq(*n - 12)))).collect();
    let bells: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_bell(midi_to_freq(*n)))).collect();
    let piano: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_piano(midi_to_freq(*n)))).collect();
    let open_hat = audio.load_ringing(render_open_hat());
    let tom = audio.load_ringing(render_tom());
    let rim = audio.load_ringing(render_rim());
    let bass = audio.load_ringing(render_bass());
    let kick = audio.load_ringing(render_kick());
    let clap = audio.load_ringing(render_clap());
    let hat = audio.load(render_hat());
    let crackle = audio.load(render_crackle());
    let lid = audio.load_ringing(render_lid());
    let chop = audio.load_ringing(render_chop());
    let sizzle = audio.load_ringing(render_sizzle());
    let plink = audio.load(render_plink());
    let splash = audio.load(render_splash());
    let fixed = [&kick, &clap, &hat, &crackle, &lid, &chop, &sizzle, &plink, &splash];
    let dj_sounds = [&open_hat, &tom, &rim, &bass];
    while !voices.iter().flatten().chain(marimba.iter()).chain(bells.iter()).chain(piano.iter()).chain(fixed).chain(dj_sounds).all(|s| audio.is_loaded(s)) {
        next_frame().await;
    }
    let perc = [&kick, &lid, &chop, &sizzle];
    let play_pad = |who: usize, i: usize, volume: f32, friend_voice: usize| {
        if who == 0 {
            let s = [&kick, &clap, &hat, &open_hat, &tom, &rim][i.min(5)];
            audio.play_ch(2, s, volume);
        } else {
            match i {
                0 => audio.play_ch(2, &bass, volume),
                1 => {
                    for n in [0, 2, 4] {
                        audio.play_ch(1, &piano[n], volume * 0.55);
                    }
                }
                2 => audio.play_ch(1, &voices[friend_voice][4], volume),
                3 => audio.play_ch(2, &sizzle, volume),
                4 => audio.play_ch(2, &bells[5], volume * 0.8),
                _ => {
                    audio.play_ch(2, &clap, volume * 0.8);
                    audio.play_ch(2, &chop, volume * 0.6);
                }
            }
        }
    };

    let make_view = |w: u32, h: u32| {
        let target = render_target_ex(w, h, RenderTargetParams { sample_count: 1, depth: true });
        target.texture.set_filter(FilterMode::Nearest);
        target
    };
    let mut view_w = low_w() as u32;
    let mut view_h = low_h() as u32;
    let mut full_view = make_view(view_w, view_h);
    let catalogue_view = make_view(400, 150);
    let pick_view = make_view(300, 130);

    let mut world = World::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut players = vec![Player::new(0, cast_list[0].body, vec2(6.5, 16.5)), Player::new(4, cast_list[4].body, vec2(7.5, 16.5))];
    let mut picking = true;
    let mut first_pick = true;
    let mut first_steps = 0u8;
    let mut pick_row = 0usize;
    let mut brain = DogBrain::new();
    let mut camp = Camp { fuel: 0.0, lit: false, pile: 0 };
    let mut inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
    let mut tools = [Tool::WoodWall, Tool::StoneWall];
    let mut fishing = Fishing { active: false, caster: 0, cue_at: 0.0, hits: 0, tries: FISH_TRIES, hit_by: [false; 2], early: [false; 2], lesson: false, perfects: 0, dip: 0.0, jump: None, tight: 1.0 };
    let mut fish_lesson = false;
    let mut cook_lesson = false;
    let mut gather_lesson = false;
    let mut fish_card = 0.0_f32;
    let mut cook_card = 0.0_f32;
    let mut gather_card = 0.0_f32;
    let mut fish_say: Option<(String, Color, f32)> = None;
    let mut dex_counts: Vec<u32> = vec![0; fish::COUNT];
    let mut shroom_counts: Vec<u32> = vec![0; mushroom::COUNT];
    let shroom_art: Vec<Vec<(usize, usize, Color)>> = (0..mushroom::COUNT).map(mushroom::art).collect();
    let mut shrooms: Vec<(Vec2, usize)> = Vec::new();
    let mut shroom_clock = 3.0f32;
    let mut shroom_day = i64::MIN;
    let mut shroom_today: Vec<usize> = Vec::new();
    let mut dex_mode = 0usize;
    let mut dex_from_shop = false;
    let fish_art: Vec<Vec<(usize, usize, Color)>> = (0..fish::COUNT).map(fish::art).collect();
    let mut fish_pop: Option<(usize, f32, bool, bool)> = None;
    let mut dex_open = false;
    let mut dex_query = String::new();
    let mut dex_sel = 0usize;
    let mut dex_drag: Option<(f32, usize)> = None;
    let mut emoji_choose: [Option<(usize, f32)>; 2] = [None, None];
    let mut emoji_last = [0usize, 0usize];
    let mut emoji_reply: Option<(usize, f32)> = None;
    let mut cooking = false;
    let mut cook_progress = 0u32;
    let mut dishes: Vec<usize> = Vec::new();
    let mut hearts: Vec<(Vec3, f32)> = Vec::new();
    let mut jam = false;
    let mut dusk = update_sky().night * 0.95;
    let mut echoes: Vec<(f32, usize)> = Vec::new();
    let mut partner_phrase: Vec<(usize, usize)> = Vec::new();
    let mut partner_soloing = false;
    let mut partner_solo_count = 0u32;
    let mut songbook: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut partner_xp = 0u32;
    let mut cat_phrase_notes = 0u32;
    let mut recent_cat: Vec<usize> = Vec::new();
    let mut crackle_clock = 0.0_f32;
    let mut toasts: Vec<(String, f32)> = Vec::new();
    let mut two_player = false;
    let mut ringing = false;
    let mut ring_flash = 0.0_f32;
    let mut show_help = false;
    let mut help_page = 0usize;
    let mut show_inventory = false;
    let mut bag_owner = 0usize;
    let mut bag_sel = 0usize;
    let mut bag_clear_armed = 0.0f32;
    let mut stored: Vec<home::Item> = Vec::new();
    let mut tick_clock = 0.0_f32;
    let mut looper = Looper::new();
    let mut music_mode = true;
    let mut microphone: Option<mic::Mic> = None;
    let mut listening = false;
    let mut listen_clock = 0.0_f32;
    let mut bpm_history: Vec<f32> = Vec::new();
    let mut locked = false;
    let mut mic_message: Option<&str> = None;
    let mut save_clock = 0.0_f32;
    let mut sleep_fade = 0.0_f32;
    let mut shelf_open = false;
    let mut shelf_sel = 0usize;
    let mut open_book: Option<usize> = None;
    let mut book_spread = 0usize;
    let mut page_turn = 0.0f32;
    let mut turn_dir = 1.0f32;
    let mut painted: Option<(usize, usize)> = None;
    let mut spines_painted = false;
    let page_rt: Vec<RenderTarget> = (0..2).map(|_| {
        let rt = render_target(512, 680);
        rt.texture.set_filter(FilterMode::Linear);
        rt
    }).collect();
    let spine_rt: Vec<RenderTarget> = (0..books::BOOKS.len()).map(|_| {
        let rt = render_target(640, 96);
        rt.texture.set_filter(FilterMode::Linear);
        rt
    }).collect();
    let mut mic_message_clock = 0.0_f32;
    let mut tree_rest: std::collections::HashMap<(i32, i32), f32> = std::collections::HashMap::new();
    let mut stick_trees: Vec<(i32, i32)> = Vec::new();
    let mut diary: Vec<DiaryEntry> = Vec::new();
    let mut session: Option<f32> = None;
    let mut session_stars = 0u8;
    let mut last_note_at = -100.0_f32;
    let mut shop_tab = 0usize;
    let mut diary_row = 0usize;
    let mut playback: Option<(usize, f64, usize)> = None;
    let mut dj = [false, false];
    let mut ddr = Ddr { on: [false; 2], arrows: Vec::new(), score: [0; 2], combo: [0; 2], judge: [None; 2], flash: [[0.0; 4]; 2], last_mic: -99.0 };
    let mut ddr_request: Option<(usize, usize)> = None;
    let mut quests = Quests { day: i64::MIN, list: Vec::new() };
    let mut sampler = Sampler { open: false, owner: 0, grid: [[false; 16]; 4], cursor: (0, 0), pads: [None, None, None, None], recording: None, step: 0 };
    let mut saved_quests: Option<(i64, Vec<(u8, u32, u32, bool)>)> = None;
    let ddr_art: Vec<Vec<(usize, usize, Color)>> = (0..4).map(ddr_arrow_pixels).collect();
    let mut beat_style = 3usize;
    let mut stick_clock = 0.0_f32;
    let mut house = home::Home::new();
    let mut inside = false;
    let mut holding: Option<home::Piece> = None;
    let mut holding_by = 0usize;
    let mut shop_open = false;
    let mut shop_owner = 0usize;
    let mut shop_row = 0usize;
    let mut seat: [Option<home::Item>; 2] = [None, None];
    let mut chime_clock = 3.0_f32;
    let mut record_tune: Vec<(usize, usize)> = Vec::new();
    let mut record_pick = 0usize;
    let mut wander_clock = 0.0_f32;
    let mut wander_to = home::world(home::Home::new().door());
    let mut paused = false;
    let mut restart_now = false;
    let mut restart_confirm = 0.0_f32;
    let mut mix = [1.0_f32; 5];
    let mut mix_drag: Option<usize> = None;
    let mut first_click_done = false;
    let mut radio_clock = 0.0_f32;
    let mut last_report = String::new();
    let mut now_playing = String::new();
    let mut live_decks: Vec<(String, String)> = Vec::new();
    let mut live_next: Vec<String> = Vec::new();
    let mut radio_bpm = 0.0_f32;
    let mut fresh_confirm = 0.0_f32;
    if let Some(saved) = store::read().and_then(|t| decode_progress(&t, cast_list.len())) {
        if saved.tiles.len() == world.tiles.len() {
            for (tile, (k, st)) in world.tiles.iter_mut().zip(saved.tiles.iter()) {
                tile.kind = KINDS[(*k as usize).min(KINDS.len() - 1)];
                tile.stack = *st;
                tile.fire = 0.0;
            }
        }
        inv.wood = saved.wood;
        inv.stone = saved.stone;
        inv.fish = saved.fish;
        camp.fuel = saved.fuel;
        camp.lit = saved.lit;
        camp.pile = saved.pile;
        dishes = saved.dishes;
        cook_progress = saved.cook_progress;
        for who in 0..2 {
            players[who].character = saved.characters[who];
            players[who].body = cast_list[saved.characters[who]].body;
            let spot = rescue(&world, saved.spots[who]);
            players[who].place_at(spot);
        }
        ringing = saved.echo;
        audio.set_ring(ringing);
        looper.beat = saved.beat;
        looper.bpm = saved.bpm;
        looper.quantise = [Quantise::Off, Quantise::Soft, Quantise::Tight][(saved.quantise as usize).min(2)];
        looper.notes = saved.notes;
        mix = saved.mix;
        audio.set_mix(mix);
        inv.acorns = saved.acorns;
        house.pieces = saved.pieces;
        house.wall = saved.styles.0;
        house.floor = saved.styles.1;
        house.owned_styles = saved.styles.2 | 1 | (1 << 4);
        house.record_on = saved.styles.3;
        house.size = saved.room;
        dex_counts = saved.dex;
        dex_counts.resize(fish::COUNT, 0);
        inv.mush = saved.mush;
        shroom_counts = saved.shroomdex;
        saved_quests = saved.quests;
        stored = saved.stored.iter().map(|i| home::Item::from_index(*i)).collect();
        for row in 0..4 {
            for k in 0..16 {
                sampler.grid[row][k] = saved.seq[row] & (1 << k) != 0;
            }
        }
        shroom_counts.resize(mushroom::COUNT, 0);
        diary = saved.diary;
        fish_lesson = saved.fish_lesson;
        cook_lesson = saved.cook_lesson;
        gather_lesson = saved.gather_lesson;
        songbook = saved.songbook;
        partner_xp = saved.partner_xp;
        picking = false;
        first_pick = false;
        first_steps = 4;
        toasts.push(("welcome back".to_string(), 3.5));
    }
    prevent_quit();


    loop {
        let dt = get_frame_time().min(0.05);
        let time = get_time() as f32;

        let mut click = is_mouse_button_pressed(MouseButton::Left);
        if click && !first_click_done {
            first_click_done = true;
            if cfg!(target_arch = "wasm32") {
                click = false;
            }
        }
        let (ux, uy) = {
            let (kx, ky) = ui_scale();
            let (mx, my) = mouse_position();
            (mx / kx, my / ky)
        };
        let mut emoji_send: [Option<usize>; 2] = [None, None];
        for who in 0..2 {
            if who == 1 && !two_player {
                emoji_choose[1] = None;
                continue;
            }
            let pressed = if who == 0 { is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Apostrophe) } else { is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Slash) };
            if pressed && !dex_open {
                emoji_choose[who] = Some(match emoji_choose[who] {
                    None => (emoji_last[who], 0.0),
                    Some((id, _)) => ((id + 1) % emoji::COUNT, 0.0),
                });
            }
            if let Some((id, t)) = emoji_choose[who].as_mut() {
                *t += dt;
                let (sx, sy, cell) = emoji_strip(who, two_player);
                if click && uy > sy - 3.0 && uy < sy + cell + 3.0 && ux > sx && ux < sx + cell * emoji::COUNT as f32 {
                    *id = (((ux - sx) / cell) as usize).min(emoji::COUNT - 1);
                    *t = 99.0;
                    click = false;
                }
                if *t > 1.1 {
                    emoji_send[who] = Some(*id);
                }
            }
            if let Some(id) = emoji_send[who] {
                emoji_choose[who] = None;
                emoji_last[who] = id;
                players[who].emote = Some((id, 3.0));
                quest_event(&mut quests, QuestKind::Emoji, 1, &mut inv, &mut toasts);
                if who == 0 && !two_player {
                    emoji_reply = Some((emoji::reply(id, hash((time * 17.0) as i32, 8, 3)), 0.8));
                }
            }
        }
        if let Some((id, t)) = emoji_reply.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                players[1].emote = Some((*id, 3.0));
                players[1].voice_timer = 0.3;
                emoji_reply = None;
            }
        }
        for p in players.iter_mut() {
            if let Some((_, left)) = p.emote.as_mut() {
                *left -= dt;
            }
            if p.emote.map(|e| e.1 <= 0.0).unwrap_or(false) {
                p.emote = None;
            }
        }

        radio::overlay(if paused || dex_open || picking || shelf_open { 0.0 } else if inside && (dj[0] || dj[1]) { 0.075 } else { 0.52 });
        if shelf_open {
            let left_k = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::J);
            let right_k = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::L);
            let pick_k = is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::O) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
            let back_k = is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace);
            let close_hit = click && ux > low_w() - 30.0 && uy < 26.0;
            let tap = click && !close_hit;
            let (tap_left, tap_right) = (tap && ux < low_w() * 0.33, tap && ux > low_w() * 0.67);
            let tap_mid = tap && !tap_left && !tap_right;
            if open_book.is_none() || !spines_painted {
                for (i, book) in books::BOOKS.iter().enumerate() {
                    paint_spine(&spine_rt[i], book);
                }
                spines_painted = painted.is_some();
            }
            let count = books::BOOKS.len();
            match open_book {
                None => {
                    if left_k || tap_left {
                        shelf_sel = (shelf_sel + count - 1) % count;
                        audio.play_ch(3, &plink, 0.2);
                    }
                    if right_k || tap_right {
                        shelf_sel = (shelf_sel + 1) % count;
                        audio.play_ch(3, &plink, 0.2);
                    }
                    if pick_k || tap_mid {
                        open_book = Some(shelf_sel);
                        book_spread = 0;
                        page_turn = 0.0;
                        audio.play_ch(3, &bells[3], 0.25);
                    }
                    if back_k || close_hit {
                        shelf_open = false;
                        save_clock = 99.0;
                    }
                }
                Some(_) => {
                    if (left_k || tap_left) && book_spread > 0 && page_turn <= 0.0 {
                        book_spread -= 1;
                        page_turn = 1.0;
                        turn_dir = -1.0;
                        audio.play_ch(3, &plink, 0.25);
                    }
                    if (right_k || tap_right || pick_k || tap_mid) && book_spread + 1 < books::SPREADS && page_turn <= 0.0 {
                        book_spread += 1;
                        page_turn = 1.0;
                        turn_dir = 1.0;
                        audio.play_ch(3, &plink, 0.25);
                    }
                    if back_k || close_hit {
                        open_book = None;
                    }
                }
            }
            page_turn = (page_turn - get_frame_time() * 2.6).max(0.0);
            if let Some(b) = open_book {
                paint_page(&page_rt[0], &books::BOOKS[b], book_spread * 2, true);
                paint_page(&page_rt[1], &books::BOOKS[b], book_spread * 2 + 1, false);
                painted = Some((b, book_spread));
                draw_book_scene(&books::BOOKS[b], &page_rt, page_turn, turn_dir, time);
            } else {
                draw_shelf_scene(shelf_sel, time, &spine_rt);
            }
            set_default_camera();
            let touch = radio::touch_ui();
            ui_rect(low_w() - 26.0, 6.0, 20.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.16));
            ui_text("x", low_w() - 19.0, 19.0, 12.0, WHITE);
            match open_book {
                None => {
                    let book = &books::BOOKS[shelf_sel];
                    let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
                    ui_text("the bookshelf", 10.0, 18.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                    ui_text(book.title, (low_w() - ui_measure(book.title, 18.0)) / 2.0, low_h() - 30.0, 18.0, c);
                    let hint = if touch { "tap a side to choose, the middle to open" } else { "A/D choose   E open   X leave" };
                    ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 12.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
                }
                Some(b) => {
                    let book = &books::BOOKS[b];
                    ui_text(book.title, 10.0, 18.0, 12.0, Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255));
                    let at = format!("pages {}-{} of {}", book_spread * 2 + 1, book_spread * 2 + 2, book.pages.len());
                    ui_text(&at, (low_w() - ui_measure(&at, 10.0)) / 2.0, low_h() - 24.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.8));
                    let hint = if touch { "tap left or right to turn the page" } else { "A/D turn the page   X back to the shelf" };
                    ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
                }
            }
            next_frame().await;
            continue;
        }

        if dex_open {
            let shrooms_mode = dex_mode == 1;
            let counts: &Vec<u32> = if shrooms_mode { &shroom_counts } else { &dex_counts };
            let total = if shrooms_mode { mushroom::COUNT } else { fish::COUNT };
            let name_of = |i: usize| if shrooms_mode { mushroom::SPECIES[i] } else { fish::SPECIES[i] };
            let code_of = |i: usize| if shrooms_mode { mushroom::code(i) } else { fish::code(i) };
            let rarity_of = |i: usize| if shrooms_mode { mushroom::rarity(i) } else { fish::rarity(i) };
            let hint_of = |i: usize| if shrooms_mode { mushroom::hint(i) } else { fish::hint(i) };
            let acorns_of = |i: usize| if shrooms_mode { mushroom::acorns(i) } else { fish::acorns(i) };
            let art_of = |i: usize| if shrooms_mode { &shroom_art[i] } else { &fish_art[i] };
            let (art_w, art_h) = if shrooms_mode { (mushroom::ART, mushroom::ART) } else { (fish::ART_W, fish::ART_H) };
            let verb = if shrooms_mode { "found" } else { "caught" };
            let numeric = dex_query.trim().trim_start_matches('#').trim_start_matches('m').parse::<usize>().is_ok();
            let found: Vec<usize> = (if shrooms_mode { mushroom::search(&dex_query) } else { fish::search(&dex_query) }).into_iter().filter(|i| dex_query.trim().is_empty() || numeric || counts[*i] > 0).collect();
            let tab_names = ["catalogue", "diary", "fishdex", "mushrooms"];
            let mut tab_go: Option<usize> = None;
            if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Right) {
                tab_go = Some(if dex_mode == 0 { 3 } else if dex_from_shop { 0 } else { 2 });
            }
            if is_key_pressed(KeyCode::Left) {
                tab_go = Some(if dex_mode == 1 { 2 } else if dex_from_shop { 1 } else { 3 });
            }
            if is_key_pressed(KeyCode::Up) && dex_sel == 0 && dex_from_shop {
                tab_go = Some(if dex_mode == 1 { 2 } else { 1 });
            }
            while let Some(ch) = get_char_pressed() {
                if (ch.is_ascii_alphanumeric() || ch == ' ' || ch == '#') && dex_query.len() < 20 {
                    dex_query.push(ch.to_ascii_lowercase());
                    dex_sel = 0;
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                dex_query.pop();
                dex_sel = 0;
            }
            let n = found.len();
            if n > 0 {
                if is_key_pressed(KeyCode::Down) {
                    dex_sel = (dex_sel + 1).min(n - 1);
                }
                if is_key_pressed(KeyCode::Up) && tab_go.is_none() {
                    dex_sel = dex_sel.saturating_sub(1);
                }
                let wheel = mouse_wheel().1;
                if wheel < 0.0 {
                    dex_sel = (dex_sel + 1).min(n - 1);
                } else if wheel > 0.0 {
                    dex_sel = dex_sel.saturating_sub(1);
                }
            }
            dex_sel = dex_sel.min(n.saturating_sub(1));
            let caught = counts.iter().filter(|c| **c > 0).count();
            let w = (low_w() - 12.0).min(390.0);
            let h = (low_h() - 16.0).min(260.0);
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0).round().max(8.0);
            let wide = w >= 300.0;
            let detail_h = if wide { 0.0 } else { 74.0 };
            let list_w = if wide { (w * 0.52).round() } else { w - 16.0 };
            let search_y = y + 24.0 + detail_h;
            let list_y = search_y + 20.0;
            let row_h = 14.0;
            let shown = (((y + h - 16.0) - list_y) / row_h).floor().max(1.0) as usize;
            let first = dex_sel.saturating_sub(shown / 2).min(n.saturating_sub(shown));
            let on_search = ux > x + 8.0 && ux < x + 8.0 + list_w && uy > search_y && uy < search_y + 15.0;
            let on_list = ux > x + 8.0 && ux < x + 8.0 + list_w && uy > list_y && uy < list_y + shown as f32 * row_h;
            let inside = ux > x && ux < x + w && uy > y && uy < y + h;
            if click && on_search && radio::touch_ui() {
                if let Some(text) = radio::ask_text(if shrooms_mode { "search the mushrooms: a name or a number" } else { "search the fishdex: a name or a number" }, &dex_query) {
                    dex_query = text.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '#').take(20).collect();
                    dex_sel = 0;
                }
            } else if click && on_list {
                let r = first + ((uy - list_y) / row_h) as usize;
                if r < n {
                    dex_sel = r;
                }
                dex_drag = Some((uy, dex_sel));
            } else if click && !inside {
                dex_open = false;
            }
            {
                let mut tx = x + 8.0;
                for (k, name) in tab_names.iter().enumerate() {
                    let tw = ui_measure(name, 11.0);
                    let usable = k >= 2 || dex_from_shop;
                    if click && usable && ux > tx - 2.0 && ux < tx + tw + 2.0 && uy > y + 4.0 && uy < y + 20.0 {
                        tab_go = Some(k);
                    }
                    tx += tw + 10.0;
                }
            }
            if let Some(k) = tab_go {
                if k >= 2 {
                    if k - 2 != dex_mode {
                        dex_mode = k - 2;
                        dex_sel = 0;
                        dex_query.clear();
                    }
                } else {
                    dex_open = false;
                    shop_open = true;
                    shop_tab = k;
                }
            }
            if is_mouse_button_down(MouseButton::Left) {
                if let Some((start, from)) = dex_drag {
                    let moved = ((start - uy) / row_h) as i32;
                    dex_sel = (from as i32 + moved).clamp(0, n.saturating_sub(1) as i32) as usize;
                }
            } else {
                dex_drag = None;
            }
            if is_key_pressed(KeyCode::Escape) {
                dex_open = false;
            }
            set_default_camera();
            clear_background(BLACK);
            draw_texture_ex(
                &full_view.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.06, 0.14, 0.6));
            ui_rect(x, y, w, h, Color::new(0.08, 0.12, 0.22, 0.96));
            ui_rect(x, y, w, 3.0, Color::from_rgba(110, 190, 255, 255));
            {
                let mut tx = x + 8.0;
                for (k, name) in tab_names.iter().enumerate() {
                    let active = k == dex_mode + 2;
                    let usable = k >= 2 || dex_from_shop;
                    let col = if active { Color::from_rgba(150, 215, 255, 255) } else if usable { Color::new(1.0, 1.0, 1.0, 0.55) } else { Color::new(1.0, 1.0, 1.0, 0.2) };
                    ui_text(name, tx, y + 16.0, 11.0, col);
                    if active {
                        ui_rect(tx, y + 18.0, ui_measure(name, 11.0), 1.0, col);
                    }
                    tx += ui_measure(name, 11.0) + 10.0;
                }
            }
            let count = format!("{}/{} {}", caught, total, verb);
            ui_text(&count, x + w - 8.0 - ui_measure(&count, 11.0), y + 16.0, 11.0, WHITE);
            ui_rect(x + 8.0, search_y, list_w, 15.0, Color::new(1.0, 1.0, 1.0, if on_search { 0.16 } else { 0.1 }));
            let placeholder = if radio::touch_ui() { "tap to search a name or #number" } else { "type a name or #number" };
            let cursor = if (time * 2.0).fract() < 0.5 && !radio::touch_ui() { "_" } else { "" };
            if dex_query.is_empty() {
                ui_text(placeholder, x + 13.0, search_y + 11.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.45));
            } else {
                ui_text(&format!("{}{}", dex_query, cursor), x + 13.0, search_y + 11.0, 11.0, WHITE);
            }
            if n == 0 {
                ui_text(if shrooms_mode { "no mushrooms found" } else { "no fish found" }, x + 13.0, list_y + 11.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
            for r in first..(first + shown).min(n) {
                let i = found[r];
                let known = counts[i] > 0;
                let ry = list_y + (r - first) as f32 * row_h;
                if r == dex_sel {
                    ui_rect(x + 6.0, ry - 1.0, list_w + 4.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
                }
                ui_text(&code_of(i), x + 10.0, ry + 9.5, 9.0, Color::new(0.75, 0.85, 1.0, 0.8));
                draw_fish_art(art_of(i), x + 34.0 + if shrooms_mode { 2.0 } else { 0.0 }, ry + if shrooms_mode { 0.0 } else { 1.0 }, if shrooms_mode { 1.05 } else { 0.95 }, known);
                let name = if known { name_of(i) } else { "???" };
                ui_text(name, x + 54.0, ry + 9.5, 10.0, if known { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) });
                if known {
                    let c = format!("x{}", counts[i]);
                    ui_text(&c, x + 6.0 + list_w - ui_measure(&c, 9.0), ry + 9.5, 9.0, fish::RARITY_COLORS[rarity_of(i)]);
                }
            }
            if let Some(&i) = found.get(dex_sel) {
                let known = counts[i] > 0;
                let (dx, dy, dw) = if wide { (x + list_w + 18.0, y + 30.0, w - list_w - 26.0) } else { (x + 8.0, y + 24.0, w - 16.0) };
                let px = if wide { (dw / art_w as f32).min(5.5) } else { 3.6 };
                let art_w = px * art_w as f32;
                let art_x = if wide { dx + (dw - art_w) / 2.0 } else { dx };
                ui_rect(art_x - 4.0, dy - 2.0, art_w + 8.0, px * art_h as f32 + 6.0, Color::new(0.3, 0.55, 0.8, 0.25));
                draw_fish_art(art_of(i), art_x, dy + 1.0, px, known);
                let tx = if wide { dx } else { dx + art_w + 12.0 };
                let mut ty = if wide { dy + px * art_h as f32 + 18.0 } else { dy + 10.0 };
                let rarity = rarity_of(i);
                ui_text(&code_of(i), tx, ty, 11.0, Color::new(0.75, 0.85, 1.0, 0.9));
                ty += 14.0;
                ui_text(if known { name_of(i) } else { "???" }, tx, ty, 13.0, WHITE);
                ty += 13.0;
                ui_text(fish::RARITY_NAMES[rarity], tx, ty, 10.0, fish::RARITY_COLORS[rarity]);
                ty += 12.0;
                let line = if known { format!("{} {} - {} acorns", verb, counts[i], acorns_of(i)) } else { format!("not {} yet", verb) };
                ui_text(&line, tx, ty, 10.0, Color::new(1.0, 1.0, 1.0, 0.75));
                ty += 12.0;
                ui_text(hint_of(i), tx, ty, 9.0, Color::new(0.8, 0.9, 1.0, 0.6));
            }
            let help = if radio::touch_ui() { "tap a row - tap the tabs - tap outside to close" } else { "type to search - up/down - left/right tabs - esc to close" };
            ui_text(help, x + (w - ui_measure(help, 9.0)) / 2.0, y + h - 5.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
            next_frame().await;
            continue;
        }
        let mut just_paused = false;
        let mut help_closed = false;
        if show_help && !paused {
            let (hx, hy, hw, hh) = help_box();
            let tab_hit = (0..3).find(|k| click && uy > hy + 20.0 && uy < hy + 36.0 && ux > hx + 10.0 + *k as f32 * (hw - 20.0) / 3.0 && ux < hx + 10.0 + (*k + 1) as f32 * (hw - 20.0) / 3.0);
            if let Some(k) = tab_hit {
                help_page = k;
                click = false;
            } else if is_key_pressed(KeyCode::Escape) || (click && !(ux > hx && ux < hx + hw && uy > hy && uy < hy + hh)) {
                show_help = false;
                help_closed = true;
                click = false;
            } else if click {
                click = false;
            }
            if is_key_pressed(KeyCode::Right) {
                help_page = (help_page + 1) % 3;
            }
            if is_key_pressed(KeyCode::Left) {
                help_page = (help_page + 2) % 3;
            }
        }
        if sampler.open && !paused && is_key_pressed(KeyCode::Escape) {
            sampler.open = false;
            help_closed = true;
        }
        if (ddr.on[0] || ddr.on[1]) && !paused && is_key_pressed(KeyCode::Escape) {
            ddr.on = [false; 2];
            help_closed = true;
        }
        if !paused && !picking && !help_closed {
            let tap_ok = if radio::touch_ui() && !two_player {
                if low_h() > low_w() { uy < low_h() * 0.55 && uy > 20.0 } else { ux > low_w() * 0.235 && ux < low_w() * 0.765 && uy > 20.0 }
            } else {
                true
            };
            if (click && tap_ok) || is_key_pressed(KeyCode::Escape) {
                paused = true;
                just_paused = true;
                mix_drag = None;
            }
        }
        if paused {
            let labels: [String; 5] = if two_player {
                [
                    cast_list[players[0].character].name.to_string(),
                    cast_list[players[1].character].name.to_string(),
                    "beat & pot".to_string(),
                    "world".to_string(),
                    "master".to_string(),
                ]
            } else {
                [
                    "you".to_string(),
                    format!("the {}", cast_list[players[1].character].noun),
                    "beat & pot".to_string(),
                    "world".to_string(),
                    "master".to_string(),
                ]
            };
            let w = 230f32.min(low_w() - 10.0);
            let row_h = 24.0;
            let h = 40.0 + row_h * 5.0 + 74.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = if radio::touch_ui() && low_h() > low_w() { (low_h() * 0.16).round() } else { ((low_h() - h) / 2.0).round() };
            let label_w = 70.0;
            let bar_x = x + 10.0 + label_w;
            let bar_w = w - 20.0 - label_w - 30.0;
            let row_y = |r: usize| y + 40.0 + r as f32 * row_h + row_h / 2.0;
            if is_mouse_button_down(MouseButton::Left) {
                if mix_drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
                    for r in 0..5 {
                        if (uy - row_y(r)).abs() < row_h / 2.0 && ux > bar_x - 10.0 && ux < bar_x + bar_w + 10.0 {
                            mix_drag = Some(r);
                        }
                    }
                }
                if let Some(r) = mix_drag {
                    mix[r] = ((ux - bar_x) / bar_w).clamp(0.0, 1.0);
                    audio.set_mix(mix);
                }
            } else {
                mix_drag = None;
            }
            let inside = ux > x && ux < x + w && uy > y && uy < y + h;
            let (dex_x, btn_y, dex_w, btn_h) = (x + 10.0, y + 40.0 + row_h * 5.0 + 4.0, ((w - 26.0) * 0.42).round(), 20.0);
            let (btn_x, btn_w) = (dex_x + dex_w + 6.0, w - 26.0 - dex_w);
            let on_button = ux > btn_x && ux < btn_x + btn_w && uy > btn_y && uy < btn_y + btn_h;
            let on_dex = ux > dex_x && ux < dex_x + dex_w && uy > btn_y && uy < btn_y + btn_h;
            if !just_paused && ((click && on_dex) || is_key_pressed(KeyCode::F)) {
                paused = false;
                dex_open = true;
                dex_from_shop = false;
                dex_query.clear();
                dex_sel = 0;
                while get_char_pressed().is_some() {}
                click = false;
            }
            let live_y = btn_y + btn_h + 4.0;
            let live_w = ((w - 26.0) / 2.0).round();
            let live_x = [dex_x, dex_x + live_w + 6.0];
            let on_live = |i: usize| ux > live_x[i] && ux < live_x[i] + live_w && uy > live_y && uy < live_y + btn_h;
            for (i, key) in [KeyCode::J, KeyCode::K].into_iter().enumerate() {
                if !just_paused && ((click && on_live(i) && mix_drag.is_none()) || is_key_pressed(key)) {
                    radio::set_flag(i as i32, !radio::flag(i as i32));
                    click = false;
                }
            }
            restart_confirm = (restart_confirm - get_frame_time()).max(0.0);
            if !just_paused && ((click && on_button && mix_drag.is_none()) || is_key_pressed(KeyCode::R)) {
                if restart_confirm > 0.0 {
                    restart_now = true;
                    paused = false;
                } else {
                    restart_confirm = 3.0;
                }
            }
            if !just_paused && ((click && !inside && mix_drag.is_none()) || is_key_pressed(KeyCode::Escape)) {
                paused = false;
                restart_confirm = 0.0;
                save_clock = 99.0;
            }
            set_default_camera();
            clear_background(BLACK);
            draw_texture_ex(
                &full_view.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.06, 0.04, 0.14, 0.55));
            ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.92));
            let title = "paused";
            ui_text(title, x + (w - ui_measure(title, 18.0)) / 2.0, y + 22.0, 18.0, Color::from_rgba(255, 226, 150, 255));
            ui_text("mixer", x + 10.0, y + 34.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.55));
            for r in 0..5 {
                let cy = row_y(r);
                let mut name = labels[r].clone();
                while ui_measure(&name, 11.0) > label_w - 4.0 && name.len() > 3 {
                    name.pop();
                }
                ui_text(&name, x + 10.0, cy + 4.0, 11.0, WHITE);
                ui_rect(bar_x, cy - 2.0, bar_w, 4.0, Color::new(1.0, 1.0, 1.0, 0.18));
                let fill = if r == 4 { Color::from_rgba(255, 214, 110, 255) } else { note_color([0, 2, 4, 3][r]) };
                ui_rect(bar_x, cy - 2.0, bar_w * mix[r], 4.0, fill);
                ui_rect(bar_x + bar_w * mix[r] - 4.0, cy - 6.0, 8.0, 12.0, if mix_drag == Some(r) { WHITE } else { Color::new(0.95, 0.92, 0.85, 1.0) });
                ui_text(&format!("{:.0}", mix[r] * 100.0), bar_x + bar_w + 8.0, cy + 4.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
            }
            ui_rect(dex_x, btn_y, dex_w, btn_h, if on_dex { Color::from_rgba(90, 160, 230, 255) } else { Color::from_rgba(60, 110, 170, 255) });
            let dex_label = if radio::touch_ui() { "fishdex" } else { "fishdex (F)" };
            ui_text(dex_label, dex_x + (dex_w - ui_measure(dex_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
            let armed = restart_confirm > 0.0;
            ui_rect(btn_x, btn_y, btn_w, btn_h, if armed { Color::from_rgba(220, 70, 80, 255) } else if on_button { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) });
            let btn_label = if armed {
                if radio::touch_ui() { "tap again" } else { "again to restart" }
            } else if radio::touch_ui() {
                "restart"
            } else {
                "restart (R)"
            };
            let mut btn_label = btn_label.to_string();
            while ui_measure(&btn_label, 11.0) > btn_w - 6.0 && btn_label.len() > 4 {
                btn_label.pop();
            }
            ui_text(&btn_label, btn_x + (btn_w - ui_measure(&btn_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
            for i in 0..2 {
                let on = radio::flag(i as i32);
                let name = if i == 0 { "dj set" } else { "karaoke" };
                let key = if radio::touch_ui() { "" } else if i == 0 { " (J)" } else { " (K)" };
                let mut label = format!("{}: {}{}", name, if on { "on" } else { "off" }, key);
                while ui_measure(&label, 10.0) > live_w - 6.0 && label.len() > 4 {
                    label.pop();
                }
                let fill = if on { Color::from_rgba(255, 214, 110, 255) } else if on_live(i) { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) };
                ui_rect(live_x[i], live_y, live_w, btn_h, fill);
                let ink = if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE };
                ui_text(&label, live_x[i] + (live_w - ui_measure(&label, 10.0)) / 2.0, live_y + 14.0, 10.0, ink);
            }
            let hint = if radio::touch_ui() { "tap outside to play" } else { "click outside or esc to play" };
            ui_text(hint, x + (w - ui_measure(hint, 10.0)) / 2.0, y + h - 8.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
            next_frame().await;
            continue;
        }

        if is_key_pressed(KeyCode::P) {
            two_player = !two_player;
            brain = DogBrain::new();
            players[1].seated = false;
            players[1].sleeping = false;
            seat[1] = None;
            dj[1] = false;
            players[1].carrying = None;
            if !two_player {
                if shop_open && shop_owner == 1 {
                    shop_owner = 0;
                }
                if holding.is_some() && holding_by == 1 {
                    holding_by = 0;
                }
                if fishing.active && fishing.caster == 1 {
                    fishing.active = false;
                    fish_card = 0.0;
                    toasts.push(("the fishing line was reeled in".to_string(), 1.5));
                }
            }
            if two_player {
                let start = if inside {
                    home::world(house.free_spot_near(home::local(players[0].pos) + vec2(1.0, 0.0)))
                } else {
                    rescue(&world, players[0].pos + vec2(-1.0, 0.0))
                };
                players[1].place_at(start);
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            music_mode = !music_mode;
        }
        if is_key_pressed(KeyCode::R) {
            looper.recording = !looper.recording;
        }
        let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if is_key_pressed(KeyCode::C) && shift_held {
            songbook.clear();
            partner_xp = 0;
            partner_phrase.clear();
            partner_soloing = false;
            toasts.push((format!("the {} forgot every tune - starting from scratch", cast_list[players[1].character].noun), 3.0));
        } else if is_key_pressed(KeyCode::C) || (two_player && is_key_pressed(KeyCode::Y)) {
            let only = if !two_player { None } else if is_key_pressed(KeyCode::C) { Some(0) } else { Some(1) };
            match only {
                None => {
                    looper.notes.clear();
                    looper.skip.clear();
                }
                Some(who) => {
                    looper.notes.retain(|&(_, id)| note_owner(id) != who);
                    looper.skip.retain(|&(_, id)| note_owner(id) != who);
                    looper.pending.retain(|&(_, id)| note_owner(id) != who);
                    toasts.push((format!("player {} cleared their loop", who + 1), 1.5));
                }
            }
        }
        if is_key_pressed(KeyCode::B) {
            looper.beat = !looper.beat;
        }
        if is_key_pressed(KeyCode::H) {
            show_help = !show_help;
        }
        let bag_keys = [is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Enter), two_player && is_key_pressed(KeyCode::Backspace)];
        for who in 0..2 {
            if !bag_keys[who] || picking {
                continue;
            }
            if inside && holding.is_some() && holding_by == who {
                if let Some(piece) = holding.take() {
                    stored.push(piece.item);
                    audio.play_ch(3, &plink, 0.35);
                    toasts.push((format!("the {} goes into your bag", piece.item.name()), 2.0));
                    save_clock = 99.0;
                }
            } else if show_inventory && bag_owner != who {
                bag_owner = who;
            } else {
                show_inventory = !show_inventory;
                bag_owner = who;
                bag_sel = 0;
                bag_clear_armed = 0.0;
            }
        }
        bag_clear_armed = (bag_clear_armed - dt).max(0.0);
        let bag_busy = show_inventory && inside && !shop_open;
        if bag_busy && holding.is_none() {
            let owner = bag_owner;
            let (up, down, act, clear) = if owner == 0 {
                (is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up), is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down), is_key_pressed(KeyCode::E), is_key_pressed(KeyCode::X))
            } else {
                (is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::O), is_key_pressed(KeyCode::U))
            };
            if up {
                bag_sel = bag_sel.saturating_sub(1);
            }
            if down && bag_sel + 1 < stored.len() {
                bag_sel += 1;
            }
            if act && bag_sel < stored.len() {
                let item = stored.remove(bag_sel);
                let ahead = ahead_local(&players[owner]);
                holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                holding_by = owner;
                show_inventory = false;
                bag_sel = bag_sel.min(stored.len().saturating_sub(1));
                toasts.push((format!("took the {} out - {} to place it", item.name(), if owner == 0 { "E" } else { "O" }), 2.5));
            }
            if clear && !house.pieces.is_empty() {
                if bag_clear_armed > 0.0 {
                    let n = house.pieces.len();
                    for piece in house.pieces.drain(..) {
                        stored.push(piece.item);
                    }
                    for k in 0..2 {
                        seat[k] = None;
                        players[k].seated = false;
                        players[k].sleeping = false;
                        dj[k] = false;
                        ddr.on[k] = false;
                    }
                    bag_clear_armed = 0.0;
                    save_clock = 99.0;
                    toasts.push((format!("packed away all {} pieces into your bag", n), 2.5));
                } else {
                    bag_clear_armed = 3.0;
                }
            }
        }
        if is_key_pressed(KeyCode::T) {
            if audio.tape_seconds().is_some() {
                let msg = match audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
                    Some(place) => format!("jam saved: {place}"),
                    None => "could not save the jam".to_string(),
                };
                toasts.push((msg, 4.0));
            } else {
                audio.start_tape();
                toasts.push(("tape rolling - T again to stop and save".to_string(), 2.5));
            }
        }
        if audio.tape_seconds().map_or(false, |t| t > TAPE_LIMIT) {
            if let Some(place) = audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
                toasts.push((format!("tape full (5 min) - jam saved: {place}"), 4.0));
            }
        }
        fresh_confirm = (fresh_confirm - get_frame_time()).max(0.0);
        let mut start_fresh = false;
        if is_key_pressed(KeyCode::N) && !picking {
            if fresh_confirm > 0.0 {
                start_fresh = true;
            } else {
                fresh_confirm = 3.0;
                toasts.push(("press N again to start a fresh world".to_string(), 3.0));
            }
        }
        if start_fresh {
            store::clear();
            world = World::new();
            inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
            camp = Camp { fuel: 0.0, lit: false, pile: 0 };
            dishes.clear();
            cook_progress = 0;
            cooking = false;
            fishing.active = false;
            fish_lesson = false;
            cook_lesson = false;
            gather_lesson = false;
            fish_card = 0.0;
            cook_card = 0.0;
            gather_card = 0.0;
            looper.notes.clear();
            looper.skip.clear();
            songbook.clear();
            partner_xp = 0;
            house = home::Home::new();
            dj = [false, false];
            stick_trees.clear();
            diary.clear();
            playback = None;
            inside = false;
            holding = None;
            shop_open = false;
            seat = [None, None];
            brain = DogBrain::new();
            for (who, spot) in [vec2(6.5, 16.5), vec2(7.5, 16.5)].into_iter().enumerate() {
                players[who].place_at(spot);
                players[who].seated = false;
                players[who].carrying = None;
            }
            fresh_confirm = 0.0;
            picking = true;
            first_pick = true;
            first_steps = 0;
            pick_row = 0;
        }
        {
            let labels: Vec<&str> = (0..6)
                .map(|i| {
                    if inside && dj[0] {
                        DJ_PADS[0][i]
                    } else if seat[0] == Some(home::Item::Piano) {
                        "piano"
                    } else if seat[0] == Some(home::Item::Drums) {
                        ["kick", "clap", "hat", "lid", "chop", "sizzle"][i]
                    } else if cooking && players[0].seated && i < 4 {
                        PERC_NAMES[i]
                    } else if music_mode {
                        voice_word(cast_list[players[0].character].voice)
                    } else {
                        SPELL_NAMES[i]
                    }
                })
                .collect();
            audio.set_boost(house.switched_on(home::Item::Speaker));
            let tool_label = if inside {
                let ahead = ahead_local(&players[0]);
                if shop_open {
                    "close".to_string()
                } else if dj[0] {
                    "leave".to_string()
                } else if holding.is_some() {
                    "turn".to_string()
                } else if seat[0].is_some() {
                    "get up".to_string()
                } else if house.piece_at(ahead.0, ahead.1).is_some() {
                    "move".to_string()
                } else {
                    "-".to_string()
                }
            } else {
                tools[0].name().to_string()
            };
            let report = format!(
                "mode={}|rec={}|echo={}|tape={}|tool={}|notes={}",
                if cooking { "cooking" } else if music_mode { "music" } else { "spells" },
                looper.recording as u8,
                ringing as u8,
                audio.tape_seconds().map(|t| t as u32).map_or(-1, |t| t as i64),
                tool_label,
                labels.join(",")
            );
            if report != last_report {
                radio::report(&report);
                last_report = report;
            }
        }
        radio_clock += get_frame_time();
        if radio_clock > 0.5 {
            radio_clock = 0.0;
            live_decks = radio::decks();
            live_next = radio::next();
            let title = radio::now_playing();
            if title != now_playing {
                if !title.is_empty() {
                    toasts.push((format!("now playing: {title}"), 4.0));
                }
                now_playing = title;
            }
            let bpm = radio::bpm();
            if bpm > 0.0 && (bpm - radio_bpm).abs() > 0.05 {
                looper.bpm = bpm.clamp(60.0, 200.0);
                toasts.push((format!("the beat follows the mix: {:.0} bpm", looper.bpm), 2.5));
            }
            radio_bpm = bpm;
        }
        save_clock += get_frame_time();
        let quitting = is_quit_requested();
        if (save_clock > 5.0 || quitting || restart_now) && !first_pick {
            save_clock = 0.0;
            let progress = Progress {
                tiles: world.tiles.iter().map(|t| (kind_index(t.kind), t.stack)).collect(),
                wood: inv.wood,
                stone: inv.stone,
                fish: inv.fish,
                fuel: camp.fuel,
                lit: camp.lit,
                pile: camp.pile,
                dishes: dishes.clone(),
                cook_progress,
                characters: [players[0].character, players[1].character],
                spots: if inside { [DOOR_OUT + vec2(0.0, 0.25), DOOR_OUT + vec2(1.0, 0.4)] } else { [players[0].pos, players[1].pos] },
                echo: ringing,
                beat: looper.beat,
                bpm: looper.bpm,
                quantise: [Quantise::Off, Quantise::Soft, Quantise::Tight].iter().position(|q| *q == looper.quantise).unwrap_or(1) as u8,
                notes: looper.notes.clone(),
                songbook: songbook.clone(),
                partner_xp,
                mix,
                acorns: inv.acorns + holding.map_or(0, |h| h.item.price() / 2),
                pieces: house.pieces.clone(),
                styles: (house.wall, house.floor, house.owned_styles, house.record_on),
                diary: diary.clone(),
                fish_lesson,
                cook_lesson,
                gather_lesson,
                room: house.size,
                dex: dex_counts.clone(),
                mush: inv.mush,
                shroomdex: shroom_counts.clone(),
                stored: stored.iter().map(|i| i.index()).collect(),
                seq: sampler.grid.map(|row| row.iter().enumerate().fold(0u16, |m, (k, on)| if *on { m | (1 << k) } else { m })),
                quests: if quests.day == i64::MIN { None } else { Some((quests.day, quests.list.iter().map(|q| (q.kind as u8, q.target, q.progress, q.done)).collect())) },
            };
            store::write(&encode_progress(&progress));
        }
        if quitting || restart_now {
            if let Some(mix) = audio.stop_tape() {
                store::save_jam(&wav_bytes(&mix));
            }
            if restart_now {
                restart_now = false;
                store::restart();
            } else {
                break;
            }
        }
        if is_key_pressed(KeyCode::G) {
            ringing = !ringing;
            audio.set_ring(ringing);
            ring_flash = 1.5;
        }
        audio.set_echo_gap(looper.step_seconds() * 3.0);
        audio.tick();
        ring_flash = (ring_flash - get_frame_time()).max(0.0);
        if is_key_pressed(KeyCode::Q) {
            looper.quantise = looper.quantise.next();
        }
        let mut picker_closed = false;
        if is_key_pressed(KeyCode::V) {
            picking = !picking;
            picker_closed = !picking;
        }
        if picking {
            if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                pick_row = 0;
            }
            if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                pick_row = 1;
            }
            let step = if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                cast_list.len() - 1
            } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                1
            } else {
                0
            };
            if step > 0 {
                let idx = (players[pick_row].character + step) % cast_list.len();
                players[pick_row].character = idx;
                players[pick_row].body = cast_list[idx].body;
                players[pick_row].voice_timer = 0.4;
                audio.play_ch(pick_row, &voices[idx][2], 0.7);
            }
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                picking = false;
                picker_closed = true;
            }
        }
        if picker_closed && first_pick {
            first_pick = false;
            first_steps = 1;
            toasts.push((format!("the {} trots off to find sticks", cast_list[players[1].character].noun), 4.0));
        }
        if is_key_pressed(KeyCode::X) && !picking && !inside {
            tools[0] = tools[0].next();
        }
        if is_key_pressed(KeyCode::U) && two_player && !inside {
            tools[1] = tools[1].next();
        }
        if is_key_pressed(KeyCode::F) {
            if listening {
                listening = false;
                looper.bpm = (looper.bpm * 10.0).round() / 10.0;
            } else {
                if microphone.is_none() {
                    microphone = mic::start();
                }
                if microphone.is_some() {
                    listening = true;
                    locked = false;
                    bpm_history.clear();
                    mic_message = None;
                } else {
                    mic_message = Some("no microphone");
                }
            }
        }

        if listening || sampler.recording.is_some() {
            if let Some(m) = microphone.as_mut() {
                m.pump();
                if m.blocked() {
                    listening = false;
                    locked = false;
                    microphone = None;
                    mic_message = Some("microphone blocked");
                }
            }
        }
        if listening {
            listen_clock += dt;
            if listen_clock >= 0.5 {
                listen_clock = 0.0;
                if let Some(m) = &microphone {
                    let (onsets, fps, level) = match m.detector.lock() {
                        Ok(d) => (d.onsets.iter().copied().collect::<Vec<f32>>(), d.fps, d.level),
                        Err(_) => (Vec::new(), 100.0, 0.0),
                    };
                    match tempo::estimate(&onsets, fps) {
                        Some(t) if t.confidence > 2.2 && level > 0.002 => {
                            bpm_history.push(t.bpm);
                            if bpm_history.len() > 6 {
                                bpm_history.remove(0);
                            }
                            let mut sorted = bpm_history.clone();
                            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            looper.bpm = sorted[sorted.len() / 2];
                            looper.sync_to_beat(t.since_beat);
                            locked = true;
                        }
                        _ => locked = false,
                    }
                }
            }
        }

        let p1_dir = if two_player {
            direction(&[KeyCode::W], &[KeyCode::S], &[KeyCode::A], &[KeyCode::D])
        } else {
            direction(
                &[KeyCode::W, KeyCode::Up],
                &[KeyCode::S, KeyCode::Down],
                &[KeyCode::A, KeyCode::Left],
                &[KeyCode::D, KeyCode::Right],
            )
        };
        let p2_dir = direction(
            &[KeyCode::I, KeyCode::Up],
            &[KeyCode::K, KeyCode::Down],
            &[KeyCode::J, KeyCode::Left],
            &[KeyCode::L, KeyCode::Right],
        );
        let (p1_dir, p2_dir) = if picking { (Vec2::ZERO, Vec2::ZERO) } else { (p1_dir, p2_dir) };
        if fishing.active && [p1_dir, p2_dir][fishing.caster] != Vec2::ZERO {
            fishing.active = false;
            toasts.push(("reeled in".to_string(), 1.5));
        }
        let mut go_in = false;
        let mut go_out = false;
        if inside {
            let h = &house;
            for who in 0..(if two_player { 2 } else { 1 }) {
                let dir = [p1_dir, p2_dir][who];
                let walk_dir = if (shop_open && shop_owner == who) || dj[who] || ddr.on[who] || (bag_busy && bag_owner == who) || (sampler.open && sampler.owner == who) { Vec2::ZERO } else { dir };
                players[who].step_with(walk_dir, dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
                if home::local(players[who].pos).y > house.d() as f32 + 0.15 {
                    go_out = true;
                }
            }
        } else {
            players[0].step(p1_dir, &world, dt);
            if two_player {
                players[1].step(p2_dir, &world, dt);
            }
            for who in 0..(if two_player { 2 } else { 1 }) {
                let dir = [p1_dir, p2_dir][who];
                if dir.y < -0.5 && players[who].pos.distance(DOOR_OUT) < 0.55 && !picking {
                    go_in = true;
                }
            }
        }
        if !players[0].seated && !(two_player && players[1].seated) {
            cooking = false;
        }
        let near_fire = |p: &Player| p.pos.distance(FIRE_SPOT) < 2.2;
        let at_dock_end = |p: &Player| p.pos.distance(DOCK_END) < 0.7;
        let mut fish_press = [false; 2];
        let mut build_request: Option<(usize, i32, i32)> = None;

        if !inside && !picking && !picker_closed {
            for who in 0..(if two_player { 2 } else { 1 }) {
                let pressed = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
                if !pressed {
                    continue;
                }
                if fish_card > 0.0 {
                    fish_card = 0.0;
                    fishing.cue_at = next_beat(looper.position) + FISH_CUE_GAP;
                    continue;
                }
                if cook_card > 0.0 {
                    cook_card = 0.0;
                    continue;
                }
                if gather_card > 0.0 {
                    gather_card = 0.0;
                    continue;
                }
                let player = &mut players[who];
                if fishing.active && player.pos.distance(DOCK_END) < 2.2 {
                    if fish_card > 0.0 {
                        fish_card = 0.0;
                        fishing.cue_at = next_beat(looper.position) + FISH_CUE_GAP;
                    } else {
                        fish_press[who] = true;
                    }
                } else if player.seated {
                    if camp.lit && near_fire(player) {
                        cooking = !cooking;
                        cook_progress = 0;
                        music_mode = true;
                        if cooking && !cook_lesson {
                            cook_card = 7.0;
                            cook_lesson = true;
                            save_clock = 99.0;
                        }
                        toasts.push((if cooking { "the pot goes on the fire".to_string() } else { "the pot comes off".to_string() }, 2.0));
                    }
                } else if camp.lit && near_fire(player) {
                    player.seated = true;
                    player.place_at(if who == 0 { CAT_SEAT } else { DOG_SEAT });
                    player.facing = if who == 0 { vec2(1.0, 0.0) } else { vec2(-1.0, 0.0) };
                    music_mode = true;
                    toasts.push((format!("player {} sits by the fire", who + 1), 2.5));
                } else if player.pos.distance(DOOR_OUT) < 0.8 {
                    go_in = true;
                } else if at_dock_end(player) && !fishing.active {
                    player.place_at(DOCK_END);
                    player.facing = vec2(1.0, 0.0);
                    let lesson = !fish_lesson;
                    if lesson {
                        fish_card = 6.0;
                    }
                    let wait = if lesson { (fish_card / looper.step_seconds() / 4.0).ceil() * 4.0 } else { 0.0 };
                    fishing = Fishing {
                        active: true,
                        caster: who,
                        cue_at: next_beat(looper.position) + FISH_CUE_GAP + wait,
                        hits: 0,
                        tries: FISH_TRIES,
                        hit_by: [false; 2],
                        early: [false; 2],
                        lesson,
                        perfects: 0,
                        dip: 0.0,
                        jump: None,
                        tight: if lesson { 1.0 } else { 0.72 + 0.2 * hash((time * 41.0) as i32, 6, 2) },
                    };
                    audio.play_ch(3, &splash, 0.35);
                    if !lesson {
                        let key = if who == 0 { "E" } else { "O" };
                        toasts.push((format!("cast the line - {key} when the ring closes"), 2.5));
                    }
                } else if let Some(k) = {
                    let (ax, az) = player.ahead();
                    let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                    shrooms.iter().position(|(at, _)| at.distance(player.pos) < 0.9 || at.distance(probe) < 0.8)
                } {
                    let (at, species) = shrooms.remove(k);
                    let is_new = shroom_counts[species] == 0;
                    shroom_counts[species] += 1;
                    inv.acorns += mushroom::acorns(species);
                    let note = if mushroom::edible(species) {
                        inv.mush[mushroom::kind(species)] += 1;
                        "into the basket"
                    } else {
                        "not for eating - logged in the dex"
                    };
                    fish_pop = Some((species, 3.0, is_new, true));
                    quest_event(&mut quests, QuestKind::Mushroom, 1, &mut inv, &mut toasts);
                    player.sniff = 0.6;
                    audio.play_ch(3, &plink, 0.45);
                    audio.play_ch(who, &voices[player.character][3], 0.5);
                    player.voice_timer = 0.3;
                    let col = mushroom::look(species).cap;
                    for n in 0..10 {
                        spawn(&mut particles, vec3(at.x, 0.35, at.y), vec3(hash(n, 3, 5) * 1.2 - 0.6, 1.0, hash(n, 4, 5) * 1.2 - 0.6), 0.7, col, 0.03, 0.6);
                    }
                    toasts.push((format!("player {} found a {} - {} (+{} acorns)", who + 1, mushroom::SPECIES[species], note, mushroom::acorns(species)), 3.0));
                    save_clock = 99.0;
                } else if let Some(tile) = {
                    let (ax, az) = player.ahead();
                    let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                    if player.carrying.is_none() {
                        stick_trees.iter().copied().find(|t| stick_spot(&world, *t).map(|p| p.distance(player.pos) < 0.95 || p.distance(probe) < 0.8).unwrap_or(false))
                    } else {
                        None
                    }
                } {
                    stick_trees.retain(|t| *t != tile);
                    if !gather_lesson {
                        gather_card = 7.0;
                        gather_lesson = true;
                        save_clock = 99.0;
                    }
                    let spot = stick_spot(&world, tile).unwrap_or(player.pos);
                    player.carrying = Some(Carry::Stick);
                    player.sniff = 0.6;
                    puff(&mut particles, spot.x.floor() as i32, spot.y.floor() as i32, Color::from_rgba(110, 170, 80, 255), 8);
                    audio.play_ch(3, &chop, 0.35);
                    toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                    quest_event(&mut quests, QuestKind::Sticks, 1, &mut inv, &mut toasts);
                } else {
                    let (x, z) = player.ahead();
                    match gather_at(&world, &mut stick_trees, &mut tree_rest, (x, z), time, player.carrying.is_none()) {
                        Gathered::Stick => {
                            quest_event(&mut quests, QuestKind::Sticks, 1, &mut inv, &mut toasts);
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            player.carrying = Some(Carry::Stick);
                            player.sniff = 0.6;
                            puff(&mut particles, x, z, Color::from_rgba(110, 170, 80, 255), 8);
                            audio.play_ch(3, &chop, 0.35);
                            toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                        }
                        Gathered::Stone => {
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            player.carrying = Some(Carry::Stone);
                            player.sniff = 0.6;
                            puff(&mut particles, x, z, Color::from_rgba(170, 170, 180, 255), 8);
                            toasts.push((format!("player {} got a stone - bring it to the fire", who + 1), 2.5));
                        }
                        Gathered::Acorns(found) => {
                            quest_event(&mut quests, QuestKind::Shake, 1, &mut inv, &mut toasts);
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            inv.acorns += found;
                            audio.play_ch(3, &chop, 0.35);
                            audio.play_ch(3, &plink, 0.3);
                            player.sniff = 0.4;
                            puff(&mut particles, x, z, Color::from_rgba(110, 170, 80, 255), 10);
                            for k in 0..found as i32 * 2 {
                                spawn(&mut particles, vec3(x as f32 + 0.5, 1.6, z as f32 + 0.5), vec3(hash(k, x, 3) * 1.2 - 0.6, 0.6, hash(k, z, 4) * 1.2 - 0.6), 0.9, ACORN_COLOR, 0.06, 1.0);
                            }
                            toasts.push((format!("player {} shook the tree: +{} acorn{}", who + 1, found, if found == 1 { "" } else { "s" }), 2.5));
                        }
                        Gathered::Resting => toasts.push(("this tree needs a rest - try another".to_string(), 2.0)),
                        Gathered::Nothing => if !fishing.active { build_request = Some((who, x, z)); },
                    }
                }
            }
        }

        if inside && !picking {
            for who in 0..(if two_player { 2 } else { 1 }) {
                if seat[who].is_some() && !players[who].seated {
                    seat[who] = None;
                }
            }
            if let Some(piece) = holding.as_mut() {
                let ahead = ahead_local(&players[holding_by]);
                piece.x = ahead.0;
                piece.z = ahead.1;
            }
            let rows = home::shop_rows();
            let shop_left = if shop_owner == 0 { is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) } else { is_key_pressed(KeyCode::J) };
            let shop_right = if shop_owner == 0 { is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) } else { is_key_pressed(KeyCode::L) };
            let shop_up = if shop_owner == 0 { is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) } else { is_key_pressed(KeyCode::I) };
            let shop_down = if shop_owner == 0 { is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) } else { is_key_pressed(KeyCode::K) };
            let shop_act = if shop_owner == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
            let shop_turn = if shop_owner == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
            let ahead = ahead_local(&players[shop_owner]);
            let was_shop = shop_open;
            let was_dj = dj[0];
            if dj[0] && !was_shop {
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    looper.bpm = (looper.bpm + 2.0).min(170.0);
                }
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    looper.bpm = (looper.bpm - 2.0).max(80.0);
                }
                let n = BEAT_STYLES.len();
                let mut changed = false;
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    beat_style = (beat_style + n - 1) % n;
                    changed = true;
                }
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    beat_style = (beat_style + 1) % n;
                    changed = true;
                }
                if changed {
                    looper.beat = BEAT_STYLES[beat_style].name != "off";
                }
                if is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Escape) {
                    dj[0] = false;
                }
            } else if was_shop {
                if shop_up {
                    shop_tab = shop_tab.saturating_sub(1);
                }
                if shop_down {
                    shop_tab = (shop_tab + 1).min(3);
                }
                if shop_tab >= 2 {
                    dex_mode = shop_tab - 2;
                    shop_tab = 1;
                    shop_open = false;
                    dex_open = true;
                    dex_from_shop = true;
                    dex_query.clear();
                    dex_sel = 0;
                    while get_char_pressed().is_some() {}
                }
                if shop_tab == 0 {
                    if shop_left {
                        shop_row = (shop_row + rows - 1) % rows;
                    }
                    if shop_right {
                        shop_row = (shop_row + 1) % rows;
                    }
                } else if !diary.is_empty() {
                    let n = diary.len();
                    if shop_left {
                        diary_row = (diary_row + n - 1) % n;
                    }
                    if shop_right {
                        diary_row = (diary_row + 1) % n;
                    }
                    diary_row = diary_row.min(n - 1);
                    if shop_act && !picker_closed {
                        let idx = n - 1 - diary_row;
                        if !house.has(home::Item::RecordPlayer) {
                            toasts.push(("buy a record player to listen to your jams".to_string(), 2.5));
                        } else if diary[idx].events.is_empty() {
                            toasts.push(("this page has no recording left".to_string(), 2.0));
                        } else {
                            playback = Some((idx, get_time(), 0));
                            house.record_on = true;
                            shop_open = false;
                            toasts.push((format!("the record player plays your jam from {}", date_label(diary[idx].when)), 3.0));
                        }
                    }
                }
                if shop_turn || is_key_pressed(KeyCode::Escape) {
                    shop_open = false;
                }
                if shop_act && !picker_closed && shop_tab == 0 {
                    match home::row(shop_row) {
                        home::Row::Upgrade => match house.next_upgrade() {
                            None => toasts.push(("your home is fully built".to_string(), 2.0)),
                            Some((name, _, price)) => {
                                if inv.acorns < price {
                                    toasts.push((format!("the {} costs {} acorns", name, price), 2.0));
                                } else {
                                    inv.acorns -= price;
                                    let mut way = house.in_the_way();
                                    way.sort_unstable_by(|a, b| b.cmp(a));
                                    let moved = way.len();
                                    for i in way {
                                        stored.push(house.pieces.remove(i).item);
                                    }
                                    if moved > 0 {
                                        toasts.push((format!("{} piece{} in the way went into your bag (M)", moved, if moved == 1 { "" } else { "s" }), 4.0));
                                    }
                                    house.size += 1;
                                    shop_open = false;
                                    save_clock = 99.0;
                                    for k in 0..6 {
                                        audio.play_ch(3, &bells[k], 0.25);
                                    }
                                    let msg = if house.size > home::LOFT_SIZE { "the penthouse grows - your loft upstairs is twice as deep now".to_string() } else if house.loft() { "penthouse! take the stairs at the back right up to your loft".to_string() } else { format!("the walls slide out - your room is now {} x {}", house.w(), house.d()) };
                                    toasts.push((msg, 4.0));
                                }
                            }
                        },
                        home::Row::Item(item) => {
                            if inv.acorns >= item.price() {
                                inv.acorns -= item.price();
                                holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                                holding_by = shop_owner;
                                shop_open = false;
                                audio.play_ch(3, &plink, 0.6);
                                toasts.push((format!("bought a {} - {} to place it, {} to turn it", item.name(), if shop_owner == 0 { "E" } else { "O" }, if shop_owner == 0 { "X" } else { "U" }), 3.0));
                            } else {
                                toasts.push((format!("a {} costs {} acorns - you have {}", item.name(), item.price(), inv.acorns), 2.5));
                            }
                        }
                        home::Row::Wall(i) | home::Row::Floor(i) => {
                            let is_wall = matches!(home::row(shop_row), home::Row::Wall(_));
                            let bit = if is_wall { home::wall_bit(i) } else { home::floor_bit(i) };
                            let owned = house.owned_styles & bit != 0;
                            if owned || inv.acorns >= home::STYLE_PRICE {
                                if !owned {
                                    inv.acorns -= home::STYLE_PRICE;
                                    house.owned_styles |= bit;
                                }
                                if is_wall {
                                    house.wall = i as u8;
                                } else {
                                    house.floor = i as u8;
                                }
                                audio.play_ch(3, &bells[i % 6], 0.5);
                            } else {
                                toasts.push((format!("that costs {} acorns", home::STYLE_PRICE), 2.0));
                            }
                        }
                        home::Row::Set => {
                            let added = house.techno_club(&[home::local(players[0].pos), home::local(players[1].pos)]);
                            looper.beat = true;
                            looper.bpm = 132.0;
                            shop_open = false;
                            audio.play_ch(2, &kick, 0.9);
                            toasts.push((format!("techno club mode: concrete, red light, 132 bpm ({} pieces fit)", added), 3.5));
                        }
                    }
                }
            }
            for who in 0..(if two_player { 2 } else { 1 }) {
                let act = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
                let turn = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
                if was_shop || shop_open || (who == 0 && was_dj) {
                    continue;
                }
                if dj[who] {
                    if who == 1 && act {
                        dj[who] = false;
                    }
                    continue;
                }
                if ddr.on[who] || (bag_busy && bag_owner == who) || (sampler.open && sampler.owner == who) {
                    continue;
                }
                let ahead = ahead_local(&players[who]);
                let here = home::local(players[who].pos);
                let near_catalogue = here.distance(vec2(0.5, 0.5)) < 1.4;
                if act && !picker_closed {
                    if let Some(piece) = holding {
                        if holding_by == who {
                            if ahead == home::CATALOGUE {
                                let back = piece.item.price() / 2;
                                inv.acorns += back;
                                holding = None;
                                toasts.push((format!("sold the {} back for {} acorns", piece.item.name(), back), 2.5));
                            } else if house.can_place(&piece, &[home::local(players[1 - who].pos)]) {
                                house.pieces.push(piece);
                                holding = None;
                                audio.play_ch(3, &marimba[(piece.x + piece.z).rem_euclid(6) as usize], 0.7);
                                if piece.item == home::Item::Speaker {
                                    toasts.push(("the speaker makes your echo bigger (G for echo)".to_string(), 3.0));
                                }
                            } else {
                                toasts.push(("it doesn't fit there".to_string(), 1.8));
                            }
                        }
                    } else if seat[who].is_some() {
                        players[who].seated = false;
                        players[who].sleeping = false;
                        seat[who] = None;
                    } else if near_catalogue {
                        shop_open = true;
                        shop_owner = who;
                        audio.play_ch(3, &plink, 0.4);
                    } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                        let item = house.pieces[i].item;
                        match item {
                            home::Item::Piano | home::Item::Drums => {
                                seat[who] = Some(item);
                                players[who].seated = true;
                                toasts.push((format!("playing the {} - {} to play, move to get up", item.name(), if who == 0 { "1-6" } else { "7-=" }), 3.0));
                            }
                            home::Item::DjBooth => {
                                dj[who] = true;
                                music_mode = true;
                                if !looper.beat {
                                    looper.beat = true;
                                    looper.bpm = 132.0;
                                    beat_style = 0;
                                }
                            }
                            home::Item::RecordPlayer => {
                                playback = None;
                                house.record_on = !house.record_on;
                                toasts.push((if house.record_on { "the record spins".to_string() } else { "the record stops".to_string() }, 2.0));
                            }
                            home::Item::WindChime => {
                                for k in 0..3 {
                                    audio.play_ch(3, &bells[(k * 2 + (time as usize)) % 6], 0.3);
                                }
                            }
                            home::Item::Bed => {
                                let piece = house.pieces[i];
                                seat[who] = Some(item);
                                players[who].seated = true;
                                players[who].sleeping = true;
                                players[who].yaw = std::f32::consts::PI;
                                let front = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                                players[who].facing = vec2(front.x, front.z);
                                players[who].sleep_yaw = piece.yaw();
                                players[who].perch = 0.06;
                                let side = if who == 0 { -0.42 } else { 0.42 };
                                let offset = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
                                players[who].place_at(home::world(piece.center() + vec2(offset.x, offset.z)));
                                if who == 0 && !two_player {
                                    let friend_side = rotate_y(vec3(0.42, 0.0, 0.0), piece.yaw());
                                    let friend = &mut players[1];
                                    friend.carrying = None;
                                    friend.seated = true;
                                    friend.sleeping = true;
                                    friend.yaw = std::f32::consts::PI;
                                    friend.facing = vec2(front.x, front.z);
                                    friend.sleep_yaw = piece.yaw();
                                    friend.perch = 0.06;
                                    friend.place_at(home::world(piece.center() + vec2(friend_side.x, friend_side.z)));
                                }
                                for k in 0..3 {
                                    hearts.push((vec3(players[who].pos.x + k as f32 * 0.15 - 0.15, 1.32 + k as f32 * 0.12, players[who].pos.y), 1.4));
                                }
                                sleep_fade = 3.0;
                                save_clock = 99.0;
                            }
                            home::Item::Sofa | home::Item::ConcreteBench => {
                                let piece = house.pieces[i];
                                let out = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                                seat[who] = Some(item);
                                players[who].seated = true;
                                players[who].perch = if item == home::Item::Sofa { 0.2 } else { 0.24 };
                                let (w, _) = piece.item.size();
                                let side = if w > 1 { if who == 0 { -0.45 } else { 0.45 } } else { 0.0 };
                                let along = rotate_y(vec3(side, 0.0, 0.05), piece.yaw());
                                players[who].place_at(home::world(piece.center() + vec2(along.x, along.z)));
                                players[who].facing = vec2(out.x, out.z);
                                toasts.push((format!("sitting on the {} - move to get up", item.name()), 2.0));
                            }
                            home::Item::Lamp | home::Item::RedLight | home::Item::Speaker | home::Item::Subwoofer => {
                                let piece = &mut house.pieces[i];
                                piece.state = if piece.state == 0 { 1 } else { 0 };
                                let on = piece.state == 0;
                                audio.play_ch(3, &plink, 0.45);
                                let msg = match item {
                                    home::Item::Speaker => if on { "speaker on - bigger echo" } else { "speaker off" },
                                    home::Item::Subwoofer => if on { "subwoofer on - heavier kick" } else { "subwoofer off" },
                                    _ => if on { "light on" } else { "light off" },
                                };
                                toasts.push((msg.to_string(), 1.5));
                            }
                            home::Item::Plant => {
                                let piece = &mut house.pieces[i];
                                let revived = piece.state == home::WITHERED;
                                if revived {
                                    piece.state = 0;
                                }
                                let first_today = piece.day != sky().day as i32;
                                piece.day = sky().day as i32;
                                if !revived && piece.state < 3 && first_today {
                                    piece.state += 1;
                                }
                                audio.play_ch(3, &bells[(piece.state as usize + 1) % 6], 0.4);
                                let msg = if revived {
                                    "you water the withered plant - it slowly recovers"
                                } else if !first_today {
                                    "it already had water today - come back tomorrow"
                                } else {
                                    match piece.state {
                                        1 => "you water the plant - it perks up",
                                        2 => "you water the plant - it grows",
                                        _ => "the plant is in full bloom",
                                    }
                                };
                                quest_event(&mut quests, QuestKind::Water, 1, &mut inv, &mut toasts);
                                toasts.push((msg.to_string(), 2.0));
                            }
                            home::Item::Bookshelf => {
                                audio.play_ch(3, &plink, 0.3);
                                shelf_open = true;
                                open_book = None;
                                page_turn = 0.0;
                            }
                            home::Item::Table => {
                                let piece = &mut house.pieces[i];
                                let mut known: Vec<usize> = dishes.clone();
                                known.sort();
                                if known.is_empty() {
                                    toasts.push(("nothing to serve yet - cook by the fire first".to_string(), 2.5));
                                } else {
                                    let current = piece.state as usize;
                                    let next = known.iter().find(|d| **d + 1 > current).map(|d| *d + 1).unwrap_or(0);
                                    piece.state = next as u8;
                                    audio.play_ch(3, &plink, 0.4);
                                    if next == 0 {
                                        toasts.push(("you clear the table".to_string(), 1.5));
                                    } else {
                                        toasts.push((format!("you serve {} - a friend comes to eat", DISHES[next - 1]), 2.5));
                                        for k in 0..3 {
                                            hearts.push((vec3(players[1 - who].pos.x + k as f32 * 0.15 - 0.15, 1.0 + k as f32 * 0.12, players[1 - who].pos.y), 1.4));
                                        }
                                    }
                                }
                            }
                            home::Item::SteelPillar => {
                                audio.play_ch(3, &bells[0], 0.6);
                                audio.play_ch(3, &bells[3], 0.25);
                            }
                            home::Item::MirrorBall => {
                                let piece = &mut house.pieces[i];
                                piece.state = if piece.state == 0 { 1 } else { 0 };
                                audio.play_ch(3, &bells[5], 0.35);
                                toasts.push((if piece.state == 0 { "the mirror ball spins".to_string() } else { "the mirror ball rests".to_string() }, 1.5));
                            }
                            home::Item::Tv => {
                                let piece = &mut house.pieces[i];
                                piece.state = (piece.state + 1) % 6;
                                audio.play_ch(3, &plink, 0.35);
                                let msg = if piece.state == 0 { "tv off".to_string() } else { format!("channel {}: {}", piece.state, TV_CHANNELS[piece.state as usize - 1]) };
                                toasts.push((msg, 1.6));
                            }
                            home::Item::Arcade => {
                                ddr_request = Some((who, i));
                            }
                            home::Item::Sampler => {
                                sampler.open = true;
                                sampler.owner = who;
                                audio.play_ch(3, &plink, 0.35);
                            }
                            home::Item::Rug => {
                                for k in 0..4 {
                                    hearts.push((vec3(players[who].pos.x + k as f32 * 0.12 - 0.18, 0.9 + k as f32 * 0.1, players[who].pos.y), 1.2));
                                }
                                toasts.push(("a little dance on the rug".to_string(), 1.5));
                            }
                        }
                    } else if here.y > house.d() as f32 - 1.1 && (here.x - house.door().x).abs() < 0.9 {
                        go_out = true;
                    }
                }
                if turn && !shop_open {
                    if let Some(piece) = holding.as_mut() {
                        if holding_by == who {
                            piece.rot = (piece.rot + 1) % 4;
                        }
                    } else if seat[who].is_some() {
                        players[who].seated = false;
                        players[who].sleeping = false;
                        seat[who] = None;
                    } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                        let piece = house.pieces.remove(i);
                        toasts.push((format!("moving the {} - {} to place, {} to turn, or sell it at the catalogue", piece.item.name(), if who == 0 { "E" } else { "O" }, if who == 0 { "X" } else { "U" }), 3.0));
                        holding = Some(piece);
                        holding_by = who;
                    }
                }
            }
            if house.has(home::Item::WindChime) {
                chime_clock -= dt;
                if chime_clock <= 0.0 {
                    chime_clock = 2.5 + hash((time * 13.0) as i32, 4, 4) * 4.0;
                    audio.play_ch(3, &bells[(hash((time * 7.0) as i32, 2, 9) * 6.0) as usize % 6], 0.16);
                }
            }
        }

        let mut delivered: Option<Carry> = None;
        let mut delivered_by = 1usize;
        if !inside && players[0].carrying.is_some() && players[0].pos.distance(STICK_DROP) < 0.9 {
            delivered = players[0].carrying.take();
            delivered_by = 0;
        }
        if two_player && !inside {
            if players[1].carrying.is_some() && players[1].pos.distance(STICK_DROP) < 0.9 {
                delivered = players[1].carrying.take();
                delivered_by = 1;
            }
        } else if !inside {
            let cat_pos = players[0].pos;
            let dog = &mut players[1];
            let want_fire = camp.fuel < FUEL_WANT;
            let mut dir = Vec2::ZERO;
            let mut next: Option<Task> = None;
            brain.rest = (brain.rest - dt).max(0.0);
            if first_pick {
                brain.rest = brain.rest.max(0.5);
            }
            match &mut brain.task {
                Task::Idle => {
                    dog.seated = false;
                    let errand = if first_pick || (jam && camp.fuel > 40.0) {
                        None
                    } else if want_fire {
                        Some(Carry::Stick)
                    } else if brain.rest > 0.0 || fishing.active {
                        None
                    } else if inv.wood < 8 && inv.wood <= inv.stone {
                        Some(Carry::Stick)
                    } else if inv.stone < 8 {
                        Some(Carry::Stone)
                    } else if inv.wood < 8 {
                        Some(Carry::Stick)
                    } else {
                        Some(Carry::Acorn)
                    };
                    if let (Some(carry), None) = (errand, dog.carrying) {
                        let kind = if carry == Carry::Stone { Kind::Stone } else { Kind::Tree };
                        let marked = if carry == Carry::Stick { nearest_stick_tree(&world, &stick_trees, dog.pos) } else { None };
                        let found = match marked {
                            Some(m) => Some(m),
                            None => find_source(&mut world, dog.pos, kind),
                        };
                        if let Some((tile, spot)) = found {
                            next = Some(Task::Seek { spot, tile, carry });
                            brain.trip = 0.0;
                        }
                    } else if dog.carrying.is_some() {
                        next = Some(Task::Return);
                    } else if jam {
                        next = Some(Task::Sit);
                    } else {
                        let goal = if fishing.active { DOCK_END - vec2(1.0, 0.0) } else { cat_pos };
                        if dog.pos.distance(goal) > if fishing.active { 0.3 } else { 2.0 } {
                            dir = steer(&world, dog.pos, goal);
                        } else if fishing.active {
                            dog.facing = vec2(1.0, 0.0);
                        }
                    }
                }
                Task::Seek { spot, tile, carry } => {
                    dir = steer(&world, dog.pos, *spot);
                    if dog.pos.distance(*spot) < 0.3 {
                        dir = Vec2::ZERO;
                        let t = *tile;
                        dog.facing = (vec2(t.0 as f32 + 0.5, t.1 as f32 + 0.5) - dog.pos).normalize_or_zero();
                        next = Some(Task::Sniff { tile: t, timer: 1.2, carry: *carry });
                    }
                }
                Task::Sniff { tile, timer, carry } => {
                    *timer -= dt;
                    dog.sniff = 0.2;
                    if *timer <= 0.0 {
                        if *carry == Carry::Stick {
                            let t = *tile;
                            stick_trees.retain(|s| *s != t);
                        }
                        dog.carrying = Some(*carry);
                        let col = match *carry {
                            Carry::Stick => Color::from_rgba(110, 170, 80, 255),
                            Carry::Stone => Color::from_rgba(170, 170, 180, 255),
                            Carry::Acorn => ACORN_COLOR,
                        };
                        puff(&mut particles, tile.0, tile.1, col, 8);
                        next = Some(Task::Return);
                        brain.trip = 0.0;
                    }
                }
                Task::Return => {
                    dir = steer(&world, dog.pos, STICK_DROP);
                    if dog.pos.distance(STICK_DROP) < 0.45 {
                        delivered = dog.carrying.take();
                        brain.rest = if want_fire { 0.0 } else if delivered == Some(Carry::Acorn) { 14.0 } else { 5.0 };
                        next = Some(Task::Idle);
                    }
                }
                Task::Sit => {
                    if !jam || camp.fuel < 40.0 {
                        dog.seated = false;
                        next = Some(Task::Idle);
                    } else if !dog.seated {
                        dir = steer(&world, dog.pos, DOG_SEAT);
                        if dog.pos.distance(DOG_SEAT) < 0.45 {
                            dir = Vec2::ZERO;
                            dog.place_at(DOG_SEAT);
                            dog.seated = true;
                            dog.facing = vec2(-1.0, 0.0);
                        }
                    }
                }
            }
            let target = match &brain.task {
                Task::Seek { spot, .. } => Some(*spot),
                Task::Return => Some(STICK_DROP),
                _ => None,
            };
            let moved = dog.step(dir, &world, dt);
            if dir != Vec2::ZERO && moved < 0.3 * dt {
                brain.stuck += dt;
            } else {
                brain.stuck = 0.0;
            }
            brain.trip += dt;
            if let Some(goal) = target {
                if brain.stuck > 1.2 {
                    let hop = dog.pos + (goal - dog.pos).normalize_or_zero() * 1.5;
                    let spot = rescue(&world, hop);
                    dog.place_at(spot);
                    brain.stuck = 0.0;
                }
                if brain.trip > 25.0 {
                    let spot = rescue(&world, goal);
                    dog.place_at(spot);
                    brain.trip = 0.0;
                }
            } else if brain.stuck > 2.0 {
                let spot = rescue(&world, dog.pos + vec2(0.0, 1.0));
                dog.place_at(spot);
                brain.stuck = 0.0;
            }
            if let Some(t) = next {
                brain.task = t;
            }
        }

        if inside && !two_player && players[1].sleeping && !players[0].sleeping {
            players[1].sleeping = false;
            players[1].seated = false;
            players[1].perch = 0.0;
            let h = &house;
            let spot = h.free_spot_near(home::local(players[1].pos));
            players[1].place_at(home::world(spot));
        }
        if inside && !two_player && !players[1].sleeping {
            let h = &house;
            wander_clock -= dt;
                let goal = if seat[0].is_some() || dj[0] { players[0].pos + vec2(0.9, 0.4) } else { wander_to };
                let to_goal = goal - players[1].pos;
                let dir = if to_goal.length() > 0.35 { to_goal.normalize() } else { Vec2::ZERO };
                let moved = players[1].step_with(dir, dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
                if wander_clock <= 0.0 || (dir != Vec2::ZERO && moved < 0.2 * dt) {
                    wander_clock = 3.0 + hash((time * 11.0) as i32, 6, 1) * 4.0;
                    wander_to = home::world(h.free_spot(time * 0.731 + 0.17));
                }
                if dir == Vec2::ZERO && (seat[0].is_some() || dj[0]) {
                    players[1].facing = (players[0].pos - players[1].pos).normalize_or_zero();
                }
        }
        for p in players.iter_mut() {
            p.lift = if inside { house.height_at(home::local(p.pos)) } else { 0.0 };
        }
        if go_in && !inside {
            inside = true;
            fishing.active = false;
            cooking = false;
            for p in players.iter_mut() {
                p.seated = false;
                p.sleeping = false;
            }
            seat = [None, None];
            dj = [false, false];
            shop_open = false;
            let door = house.door();
            players[0].place_at(home::world(door - vec2(0.0, 0.45)));
            players[0].facing = vec2(0.0, -1.0);
            players[1].place_at(home::world(door - vec2(1.0, 0.35)));
            players[1].facing = vec2(0.0, -1.0);
            wander_clock = 2.0;
            audio.play_ch(3, &chop, 0.3);
            toasts.push(("home sweet home".to_string(), 2.5));
        }
        if go_out && inside {
            if holding.is_some() {
                players[holding_by].place_at(home::world(vec2(3.5, 4.0)));
                toasts.push(("put your furniture down first".to_string(), 2.0));
            } else {
                inside = false;
                seat = [None, None];
                for p in players.iter_mut() {
                    p.seated = false;
                    p.sleeping = false;
                }
                dj = [false, false];
                shop_open = false;
                players[0].place_at(DOOR_OUT + vec2(0.0, 0.25));
                players[0].facing = vec2(0.0, 1.0);
                let spot = rescue(&world, DOOR_OUT + vec2(1.0, 0.4));
                players[1].place_at(spot);
                brain = DogBrain::new();
                audio.play_ch(3, &chop, 0.3);
            }
        }

        let friend = cast_list[players[1].character].noun;
        let bringer = if delivered_by == 0 { "you".to_string() } else if two_player { "player 2".to_string() } else { format!("the {}", friend) };
        let bringer_voice = players[delivered_by].character;
        let bringer_ch = delivered_by;
        match delivered {
            Some(Carry::Stick) if camp.fuel < FUEL_WANT => {
                camp.fuel = (camp.fuel + STICK_FUEL).min(240.0);
                camp.pile = (camp.fuel / STICK_FUEL).ceil() as u32;
                audio.play_ch(bringer_ch, &voices[bringer_voice][2], 0.7);
                players[delivered_by].voice_timer = 0.4;
                if !camp.lit {
                    camp.lit = true;
                    toasts.push((format!("{} brought a stick - the fire is lit!", bringer), 3.5));
                    for k in 0..30 {
                        let a = k as f32 / 30.0 * 2.0 * PI;
                        spawn(&mut particles, fire_center() + vec3(0.0, 0.3, 0.0), vec3(a.cos() * 1.5, 2.0 + hash(k, 1, 3) * 1.5, a.sin() * 1.5), 0.9, Color::from_rgba(255, 190, 80, 255), 0.04, 0.6);
                    }
                } else {
                    toasts.push((format!("{} brought a stick for the fire", bringer), 2.5));
                }
            }
            Some(Carry::Stick) => {
                inv.wood += 1;
                audio.play_ch(bringer_ch, &voices[bringer_voice][3], 0.6);
                players[delivered_by].voice_timer = 0.4;
                toasts.push((format!("{} brought wood for building", bringer), 2.5));
            }
            Some(Carry::Acorn) => {
                inv.acorns += 2;
                audio.play_ch(1, &voices[players[1].character][1], 0.6);
                players[1].voice_timer = 0.4;
                toasts.push((format!("the {} brought you 2 acorns", friend), 2.5));
            }
            Some(Carry::Stone) => {
                inv.stone += 1;
                audio.play_ch(bringer_ch, &voices[bringer_voice][4], 0.6);
                players[delivered_by].voice_timer = 0.4;
                toasts.push((format!("{} brought a stone for building", bringer), 2.5));
            }
            None => {}
        }

        if let Some((who, x, z)) = build_request {
            let blocked = players.iter().any(|p| touches_cat(x, z, p.pos));
            match build(&mut world, &mut inv, tools[who], x, z, blocked) {
                Ok(Some((kind, stack))) => {
                    let i = ((x + z + stack as i32).rem_euclid(6)) as usize;
                    match kind {
                        Kind::WoodWall => audio.play_ch(3, &marimba[i], 0.8),
                        Kind::StoneWall => audio.play_ch(3, &bells[i], 0.8),
                        _ => audio.play_ch(3, &chop, 0.7),
                    }
                    puff(&mut particles, x, z, Color::from_rgba(230, 210, 170, 255), 6);
                    players[who].voice_timer = 0.2;
                }
                Ok(None) => {}
                Err(msg) => toasts.push((msg.to_string(), 2.5)),
            }
        }

        if sampler.open && !inside {
            sampler.open = false;
        }
        if sampler.open {
            let o = sampler.owner;
            let (l, r, u, d, act, close) = if o == 0 {
                (
                    is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left),
                    is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right),
                    is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up),
                    is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down),
                    is_key_pressed(KeyCode::E),
                    is_key_pressed(KeyCode::X),
                )
            } else {
                (is_key_pressed(KeyCode::J), is_key_pressed(KeyCode::L), is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::O), is_key_pressed(KeyCode::U))
            };
            let (mut cx, mut cy) = sampler.cursor;
            if l {
                cx = (cx + 15) % 16;
            }
            if r {
                cx = (cx + 1) % 16;
            }
            if u {
                cy = (cy + 3) % 4;
            }
            if d {
                cy = (cy + 1) % 4;
            }
            sampler.cursor = (cx, cy);
            if act {
                sampler.grid[cy][cx] = !sampler.grid[cy][cx];
                save_clock = 99.0;
            }
            if close {
                sampler.open = false;
            }
            if is_key_pressed(KeyCode::Space) && sampler.recording.is_none() {
                if sampler.pads[cy].is_some() {
                    sampler.pads[cy] = None;
                    toasts.push((format!("the {} row is back to the built-in sound", SAMPLER_ROWS[cy]), 2.0));
                } else {
                    if microphone.is_none() {
                        microphone = mic::start();
                    }
                    if let Some(m) = &microphone {
                        if let Ok(mut det) = m.detector.lock() {
                            det.capture_len = (det.rate * 1.5) as usize;
                            det.capture = Some(Vec::new());
                        }
                        sampler.recording = Some((cy, 0.0));
                    } else {
                        toasts.push(("no microphone - the built-in sounds still work".to_string(), 2.5));
                    }
                }
            }
        }
        if let Some((row, waited)) = sampler.recording {
            let waited = waited + dt;
            let mut finished: Option<Option<Vec<f32>>> = None;
            if let Some(m) = &microphone {
                if let Ok(mut det) = m.detector.lock() {
                    let full = det.capture.as_ref().map(|c| c.len() >= det.capture_len).unwrap_or(true);
                    if full || waited > 6.0 {
                        let raw = det.capture.take().unwrap_or_default();
                        finished = Some(tidy_recording(&raw, det.rate));
                    }
                }
            } else {
                finished = Some(None);
            }
            sampler.recording = Some((row, waited));
            if let Some(result) = finished {
                sampler.recording = None;
                match result {
                    Some(samples) => {
                        sampler.pads[row] = Some(audio.load(samples));
                        toasts.push((format!("the {} row now plays your own sound", SAMPLER_ROWS[row]), 2.5));
                    }
                    None => toasts.push(("didn't hear anything - try again a bit louder".to_string(), 2.5)),
                }
            }
        }
        if let Some((who, i)) = ddr_request.take() {
            if !ddr.on[who] && i < house.pieces.len() {
                let piece = house.pieces[i];
                let front = rotate_y(vec3(0.0, 0.0, 0.95), piece.yaw());
                let side = if two_player { if who == 0 { -0.22 } else { 0.22 } } else { 0.0 };
                let across = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
                let spot = piece.center() + vec2(front.x + across.x, front.z + across.z);
                players[who].pos = home::world(spot);
                players[who].facing = vec2(-front.x, -front.z).normalize_or_zero();
                if !ddr.on[0] && !ddr.on[1] {
                    ddr.arrows.clear();
                }
                ddr.on[who] = true;
                ddr.score[who] = 0;
                ddr.combo[who] = 0;
                if microphone.is_none() {
                    microphone = mic::start();
                }
                if microphone.is_some() && !listening {
                    listening = true;
                    locked = false;
                    bpm_history.clear();
                    mic_message = None;
                }
                if !looper.beat && microphone.is_none() {
                    looper.beat = true;
                }
                music_mode = true;
                audio.play_ch(3, &bells[4], 0.4);
                let keys = if who == 0 { "arrow keys or WASD" } else { "J K I L" };
                toasts.push((format!("dance! step with {keys} when the arrows reach the top - {} to stop", if who == 0 { "X" } else { "U" }), 3.5));
            }
        }
        if ddr.on[0] || ddr.on[1] {
            if !inside {
                ddr.on = [false; 2];
            }
            let step_s = looper.step_seconds();
            if !looper.beat && time - ddr.last_mic > 4.0 && looper.notes.is_empty() {
                looper.beat = true;
            }
            if listening {
                if let Some(m) = &microphone {
                    let hits: Vec<(usize, f32)> = m.detector.lock().map(|mut d| d.hits.drain(..).collect()).unwrap_or_default();
                    for (band, _) in hits {
                        ddr.last_mic = time;
                        let p = looper.position;
                        let g = (p / 2.0).round() * 2.0;
                        let target = time + (g - p) * step_s + 16.0 * step_s;
                        ddr.add(DDR_LANE_OF_BAND[band], target, step_s * 2.0);
                    }
                }
            }
            for who in 0..2 {
                if !ddr.on[who] {
                    continue;
                }
                let keys: [bool; 4] = if who == 0 {
                    [
                        is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
                        is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
                        is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
                        is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
                    ]
                } else {
                    [is_key_pressed(KeyCode::J), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::L)]
                };
                let quit = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
                if quit {
                    ddr.on[who] = false;
                    toasts.push((format!("player {} scored {}", who + 1, ddr.score[who]), 2.5));
                    continue;
                }
                for lane in 0..4 {
                    if !keys[lane] {
                        continue;
                    }
                    ddr.flash[who][lane] = 1.0;
                    let best = ddr.arrows.iter().enumerate().filter(|(_, a)| a.lane == lane && !a.hit[who] && !a.missed[who] && (a.at - time).abs() < 0.18).min_by(|a, b| (a.1.at - time).abs().partial_cmp(&(b.1.at - time).abs()).unwrap()).map(|(n, a)| (n, (a.at - time).abs()));
                    if let Some((n, off)) = best {
                        ddr.arrows[n].hit[who] = true;
                        ddr.combo[who] += 1;
                        let (word, col, points) = if off < 0.05 {
                            ("perfect", Color::from_rgba(255, 226, 120, 255), 300)
                        } else if off < 0.1 {
                            ("great", Color::from_rgba(140, 230, 170, 255), 200)
                        } else {
                            ("good", Color::from_rgba(170, 210, 255, 255), 100)
                        };
                        ddr.score[who] += points + ddr.combo[who].min(50) * 2;
                        ddr.judge[who] = Some((word, col, 0.6));
                        quest_event(&mut quests, QuestKind::Dance, 1, &mut inv, &mut toasts);
                        players[who].voice_timer = 0.15;
                        audio.play_ch(who, &voices[players[who].character][lane + 1], 0.18);
                    }
                }
                for a in ddr.arrows.iter_mut() {
                    if !a.hit[who] && !a.missed[who] && time - a.at > 0.18 {
                        a.missed[who] = true;
                        ddr.combo[who] = 0;
                        ddr.judge[who] = Some(("miss", Color::from_rgba(255, 140, 140, 255), 0.6));
                    }
                }
            }
            ddr.arrows.retain(|a| time - a.at < 0.5);
        } else {
            ddr.arrows.clear();
        }
        for who in 0..2 {
            for lane in 0..4 {
                ddr.flash[who][lane] = (ddr.flash[who][lane] - dt * 5.0).max(0.0);
            }
            if let Some(j) = ddr.judge[who].as_mut() {
                j.2 -= dt;
            }
            if ddr.judge[who].map(|j| j.2 <= 0.0).unwrap_or(false) {
                ddr.judge[who] = None;
            }
        }
        {
            let mic_bands = if listening { microphone.as_ref().and_then(|m| m.detector.lock().ok().map(|d| d.bands)) } else { None };
            let beat_frac = (looper.position / 4.0).fract();
            let pulse = if looper.beat { (1.0 - beat_frac).powi(3) } else { 0.0 };
            let glow: f32 = players.iter().map(|p| p.key_glow.iter().copied().fold(0.0, f32::max)).fold(0.0, f32::max);
            let bands = mic_bands.unwrap_or([pulse, pulse * 0.4 + glow * 0.6, glow * 0.7, if looper.beat { ((looper.position * 0.5).fract() < 0.3) as u8 as f32 * 0.6 } else { glow * 0.4 }]);
            let mut pads = [0.0f32; 4];
            for lane in 0..4 {
                pads[lane] = ddr.flash[0][lane].max(ddr.flash[1][lane]);
            }
            VIS.with(|v| v.set(Vis { bands, pulse: if mic_bands.is_some() { bands[0] } else { pulse }, pads, seq: sampler.step }));
        }
        if fishing.active && fish_card <= 0.0 {
            let fish_window = 0.22 * fishing.tight / looper.step_seconds();
            let perfect_window = 0.09 * fishing.tight / looper.step_seconds();
            for who in 0..(if two_player { 2 } else { 1 }) {
                if !fish_press[who] {
                    continue;
                }
                let d = looper.position - fishing.cue_at;
                if d.abs() <= fish_window && !fishing.hit_by[who] && !fishing.early[who] {
                    fishing.hit_by[who] = true;
                    fishing.dip = 1.0;
                    audio.play_ch(3, &plink, 0.7);
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * 2.0 * PI;
                        spawn(&mut particles, BOBBER + vec3(0.0, 0.05, 0.0), vec3(a.cos() * 0.8, 1.6, a.sin() * 0.8), 0.6, Color::from_rgba(200, 230, 255, 255), 0.025, 0.5);
                    }
                    let both = fishing.hit_by[0] && fishing.hit_by[1];
                    let (word, col) = if both {
                        ("together!", Color::from_rgba(255, 150, 190, 255))
                    } else if d.abs() <= perfect_window {
                        fishing.perfects += 1;
                        ("perfect!", Color::from_rgba(255, 226, 120, 255))
                    } else {
                        ("nice!", Color::from_rgba(170, 230, 255, 255))
                    };
                    fish_say = Some((word.to_string(), col, 1.0));
                } else if d < -fish_window && d > -4.0 && !fishing.hit_by[who] {
                    fishing.early[who] = true;
                    fish_say = Some(("too early - wait for the ring".to_string(), Color::from_rgba(255, 190, 150, 255), 1.2));
                }
            }
            if looper.position > fishing.cue_at + fish_window {
                let hit = fishing.hit_by[0] || fishing.hit_by[1];
                let both = fishing.hit_by[0] && fishing.hit_by[1];
                let friend_on_dock = !two_player && players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8;
                let helped = !hit && !fishing.lesson && friend_on_dock && hash((time * 29.0) as i32, 4, 8) < 0.5;
                if !hit && fishing.lesson {
                    let key = if fishing.caster == 0 { "E" } else { "O" };
                    fish_say = Some((format!("almost! press {key} as the ring touches the bobber"), Color::from_rgba(255, 210, 160, 255), 1.6));
                } else if hit || helped {
                    fishing.hits += if both { 2 } else { 1 };
                    if helped {
                        fishing.dip = 1.0;
                        audio.play_ch(1, &voices[players[1].character][2], 0.5);
                        players[1].voice_timer = 0.4;
                        let line = format!("the {} tugged the line for you!", cast_list[players[1].character].noun);
                        fish_say = Some((line, Color::from_rgba(200, 240, 170, 255), 1.6));
                    }
                    if both {
                        for k in 0..3 {
                            hearts.push((vec3(players[k % 2].pos.x, 1.0 + k as f32 * 0.12, players[k % 2].pos.y), 1.4));
                        }
                    }
                } else {
                    fishing.tries = fishing.tries.saturating_sub(1);
                    audio.play_ch(3, &splash, 0.35);
                    if fishing.tries > 0 {
                        fish_say = Some((format!("missed - {} {} left", fishing.tries, if fishing.tries == 1 { "try" } else { "tries" }), Color::from_rgba(255, 190, 150, 255), 1.4));
                    }
                }
                if fishing.hits >= 3 {
                    let roll = hash((time * 100.0) as i32, 7, 7);
                    let roll2 = hash((time * 61.0) as i32, 3, 13);
                    let luck = (fishing.perfects as f32 / 3.0).min(1.0) + if fishing.hit_by[0] && fishing.hit_by[1] { 0.3 } else { 0.0 };
                    let species = if fishing.lesson { fish::pick(roll * 0.5, roll2, 0.0) } else { fish::pick(roll, roll2, luck) };
                    let kind = fish::kind(species);
                    let is_new = dex_counts[species] == 0;
                    dex_counts[species] += 1;
                    fish_pop = Some((species, 3.0, is_new, false));
                    inv.fish[kind] += 1;
                    quest_event(&mut quests, QuestKind::Fish, 1, &mut inv, &mut toasts);
                    inv.acorns += fish::acorns(species);
                    fishing.active = false;
                    fishing.jump = Some((kind, 0.0));
                    audio.play_ch(3, &splash, 0.6);
                    let singer = if fishing.hit_by[1] { 1 } else { fishing.caster };
                    audio.play_ch(singer, &voices[players[singer].character][5], 0.7);
                    players[singer].voice_timer = 0.5;
                    let who = if fishing.hit_by[0] && fishing.hit_by[1] { "you both caught" } else if fishing.hit_by[1] || (fishing.caster == 1 && !fishing.hit_by[0]) { "player 2 caught" } else { "caught" };
                    toasts.push((format!("{who} a {}! +{} acorns", fish::SPECIES[species], fish::acorns(species)), 3.0));
                    if fishing.lesson {
                        fish_lesson = true;
                        save_clock = 99.0;
                        fish_say = Some(("your first fish!".to_string(), Color::from_rgba(255, 226, 120, 255), 2.0));
                    }
                    if friend_on_dock && hash((time * 37.0) as i32, 3, 11) < 0.55 {
                        let bonus = fish::pick(hash((time * 53.0) as i32, 5, 2) * 0.8, hash((time * 71.0) as i32, 2, 9), 0.0);
                        dex_counts[bonus] += 1;
                        inv.fish[fish::kind(bonus)] += 1;
                        inv.acorns += 2;
                        audio.play_ch(1, &voices[players[1].character][4], 0.6);
                        players[1].voice_timer = 0.5;
                        toasts.push((format!("the {} caught a {} too! +2 acorns", cast_list[players[1].character].noun, fish::SPECIES[bonus]), 3.0));
                    }
                } else if fishing.tries == 0 {
                    fishing.active = false;
                    audio.play_ch(3, &splash, 0.5);
                    toasts.push(("the fish got away - cast again!".to_string(), 2.5));
                } else {
                    let roll = hash((time * 47.0) as i32, 9, 3);
                    let gap = if fishing.lesson { FISH_CUE_GAP } else if roll < 0.25 { FISH_CUE_GAP - 2.0 } else if roll > 0.8 { FISH_CUE_GAP + 2.0 } else { FISH_CUE_GAP };
                    fishing.cue_at += gap;
                    fishing.hit_by = [false; 2];
                    fishing.early = [false; 2];
                    if !fishing.lesson {
                        fishing.tight = 0.72 + 0.2 * hash((time * 43.0) as i32, 1, 8);
                    }
                }
            }
        }
        fish_card = (fish_card - dt).max(0.0);
        cook_card = (cook_card - dt).max(0.0);
        gather_card = (gather_card - dt).max(0.0);
        if let Some(say) = fish_say.as_mut() {
            say.2 -= dt;
        }
        if fish_say.as_ref().map(|s| s.2 <= 0.0).unwrap_or(false) {
            fish_say = None;
        }
        fishing.dip = (fishing.dip - dt * 4.0).max(0.0);
        if let Some((kind, t)) = fishing.jump {
            fishing.jump = if t + dt > 1.0 { None } else { Some((kind, t + dt)) };
        }

        let was_jam = jam;
        jam = !inside && (players[0].seated || (two_player && players[1].seated));
        if jam && !was_jam {
            music_mode = true;
            if first_steps == 2 {
                first_steps = 3;
                toasts.push(("play together to earn your first star".to_string(), 3.0));
            }
        }

        if camp.lit {
            camp.fuel -= dt;
            camp.pile = (camp.fuel / STICK_FUEL).ceil().max(0.0) as u32;
            if camp.fuel <= 0.0 {
                camp.fuel = 0.0;
                camp.lit = false;
                cooking = false;
                toasts.push(("the fire went out".to_string(), 3.0));
                for p in players.iter_mut() {
                    p.seated = false;
                }
            }
            crackle_clock -= dt;
            if crackle_clock <= 0.0 {
                crackle_clock = 0.6 + hash((time * 10.0) as i32, 3, 9) * 1.2;
                let dist = players[0].pos.distance(FIRE_SPOT);
                let vol = (1.0 - dist / 7.0).max(0.0) * 0.35;
                if vol > 0.01 {
                    audio.play_ch(3, &crackle, vol);
                }
            }
            if hash((time * 60.0) as i32, 5, 1) > 0.7 {
                let c = fire_center();
                spawn(&mut particles, c + vec3(hash((time * 97.0) as i32, 1, 2) * 0.4 - 0.2, 0.4, 0.1), vec3(hash((time * 31.0) as i32, 2, 2) * 0.6 - 0.3, 1.6, 0.0), 0.9, Color::from_rgba(255, 180, 70, 255), 0.022, 0.05);
            }
        }
        if cooking && hash((time * 40.0) as i32, 9, 2) > 0.8 {
            let c = fire_center();
            spawn(&mut particles, c + vec3(hash((time * 53.0) as i32, 1, 3) * 0.3 - 0.15, 0.62, 0.06), vec3(0.0, 0.5, 0.0), 1.6, Color::from_rgba(240, 240, 245, 255), 0.05, -0.03);
        }
        if hash((time * 30.0) as i32, 8, 1) > 0.85 {
            let (_, z0, x1, _) = COTTAGE;
            spawn(&mut particles, vec3(x1 as f32 - 0.6, 2.7, z0 as f32 + 0.6), vec3(0.15, 0.5, 0.0), 2.6, Color::from_rgba(220, 220, 228, 255), 0.06, -0.02);
        }
        let real_night = update_sky().night * 0.95;
        {
            let sk = sky();
            if quests.day != sk.day {
                quests.day = sk.day;
                quests.list = daily_quests(sk.day, house.has(home::Item::Plant), house.has(home::Item::Arcade));
                let mut restored = false;
                if let Some((day, list)) = saved_quests.take() {
                    if day == sk.day {
                        for (q, (kind, target, progress, done)) in quests.list.iter_mut().zip(list.into_iter()) {
                            if q.kind as u8 == kind && q.target == target {
                                q.progress = progress;
                                q.done = done;
                                restored = true;
                            }
                        }
                    }
                }
                if !restored {
                    toasts.push(("3 new daily quests - see them in your bag (M)".to_string(), 3.5));
                }
            }
            for piece in house.pieces.iter_mut().filter(|p| p.item == home::Item::Plant) {
                if piece.day == 0 {
                    piece.day = sk.day as i32;
                } else if sk.day as i32 - piece.day >= 2 && piece.state != home::WITHERED {
                    piece.state = home::WITHERED;
                }
            }
            let mut batch = 1;
            if sk.day != shroom_day {
                shroom_day = sk.day;
                shroom_today = mushroom::today(sk.day);
                shrooms.retain(|(_, sp)| shroom_today.contains(sp));
                shroom_clock = 0.0;
                batch = 6;
            }
            shroom_clock -= dt;
            if shroom_clock <= 0.0 {
                shroom_clock = if sk.weather == 2 { 7.0 } else { 14.0 };
                for round in 0..batch {
                if shrooms.len() < 10 && !shroom_today.is_empty() {
                    let seed = (time * 97.0) as i32 + round * 131;
                    for attempt in 0..30 {
                        let tx = (hash(seed, attempt, 31) * MAP as f32) as i32;
                        let tz = (hash(seed, attempt, 32) * MAP as f32) as i32;
                        if world.get(tx, tz).map(|t| t.kind == Kind::Tree).unwrap_or(false) {
                            let dx = (hash(seed, attempt, 33) * 3.0) as i32 - 1;
                            let dz = (hash(seed, attempt, 34) * 3.0) as i32 - 1;
                            let (gx, gz) = (tx + dx, tz + dz);
                            let ok = world.get(gx, gz).map(|t| matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Tree)).unwrap_or(false);
                            let at = vec2(gx as f32 + 0.2 + hash(seed, attempt, 35) * 0.6, gz as f32 + 0.2 + hash(seed, attempt, 36) * 0.6);
                            let trunk_clear = (dx != 0 || dz != 0) || at.distance(vec2(tx as f32 + 0.5, tz as f32 + 0.5)) > 0.3;
                            if ok && trunk_clear && shrooms.iter().all(|(o, _)| o.distance(at) > 1.0) {
                                let species = mushroom::pick_from(&shroom_today, hash(seed, attempt, 37));
                                shrooms.push((at, species));
                                break;
                            }
                        }
                    }
                }
                }
            }
            SHROOMS.with(|c| *c.borrow_mut() = shrooms.clone());
        }
        let dusk_target = real_night.max(if jam { 0.45 } else if camp.lit { 0.12 } else { 0.0 });
        dusk += (dusk_target - dusk) * (dt * 0.4).min(1.0);
        if jam && !two_player {
            let c = players[0].cam;
            players[0].cam = c + (vec2(8.5, 17.0) - c) * (dt * 2.0).min(1.0);
        }

        let active = if two_player { 2 } else { 1 };
        let dog_jamming = jam && !two_player && players[1].seated;
        for who in 0..active {
            let keys = if who == 0 { NOTE_KEYS } else { P2_NOTE_KEYS };
            for (i, key) in keys.iter().enumerate() {
                if !is_key_pressed(*key) {
                    continue;
                }
                last_note_at = time;
                if who == 0 && first_steps == 1 && music_mode && !cooking && !(inside && dj[0]) {
                    first_steps = 2;
                    toasts.push(("your first note! find the campfire next".to_string(), 3.0));
                }
                if inside && dj[who] {
                    let id = 16 + who * 6 + i;
                    if looper.press(id) {
                        play_pad(who, i, 0.85, players[1].character);
                    }
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.2;
                    continue;
                }
                if inside && matches!(seat[who], Some(home::Item::Piano) | Some(home::Item::Drums)) {
                    if seat[who] == Some(home::Item::Piano) {
                        audio.play_ch(who, &piano[i], 0.8);
                    } else {
                        let drum = [&kick, &clap, &hat, &lid, &chop, &sizzle][i];
                        audio.play_ch(2, drum, 0.85);
                    }
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.3;
                    players[1 - who].voice_timer = players[1 - who].voice_timer.max(0.15);
                    continue;
                }
                if cooking && players[who].seated && i < 4 {
                    let id = if who == 0 { 12 + i } else { 28 + i };
                    let play_now = looper.press(id);
                    if play_now {
                        audio.play_ch(2, perc[i], 0.8);
                    }
                    let f = looper.position.fract();
                    if f < 0.3 || f > 0.7 {
                        cook_progress += 1;
                    }
                    players[who].key_glow[i] = 1.0;
                    let c = fire_center() + vec3(0.0, 0.6, 0.08);
                    let col = [Color::from_rgba(255, 170, 80, 255), Color::from_rgba(220, 220, 230, 255), Color::from_rgba(120, 200, 90, 255), Color::from_rgba(255, 220, 120, 255)][i];
                    for k in 0..6 {
                        let a = (k as f32 / 6.0 + time) * 2.0 * PI;
                        spawn(&mut particles, c, vec3(a.cos() * 0.7, 1.2, a.sin() * 0.3), 0.5, col, 0.025, 0.5);
                    }
                    if cook_progress >= COOK_GOAL {
                        cook_progress = 0;
                        let fish_n = inv.fish[0] + inv.fish[1] + inv.fish[2];
                        let mush_n = inv.mush[0] + inv.mush[1] + inv.mush[2];
                        let take = |stock: &mut [u32; 3], n: u32| {
                            for _ in 0..n {
                                if let Some(k) = (0..3).find(|k| stock[*k] > 0) {
                                    stock[k] -= 1;
                                }
                            }
                        };
                        let dish = if inv.mush[2] > 0 {
                            inv.mush[2] -= 1;
                            7
                        } else if inv.fish[2] > 0 {
                            inv.fish[2] -= 1;
                            3
                        } else if fish_n >= 1 && mush_n >= 1 {
                            take(&mut inv.fish, 1);
                            take(&mut inv.mush, 1);
                            6
                        } else if mush_n >= 2 {
                            take(&mut inv.mush, 2);
                            5
                        } else if fish_n >= 2 {
                            take(&mut inv.fish, 2);
                            2
                        } else if mush_n == 1 {
                            take(&mut inv.mush, 1);
                            4
                        } else if fish_n == 1 {
                            take(&mut inv.fish, 1);
                            1
                        } else {
                            0
                        };
                        let new = !dishes.contains(&dish);
                        if new {
                            dishes.push(dish);
                        }
                        let earned = if new { 10 } else { 5 };
                        inv.acorns += earned;
                        quest_event(&mut quests, QuestKind::Cook, 1, &mut inv, &mut toasts);
                        toasts.push((
                            if new { format!("new dish: {}! +{} acorns", DISHES[dish], earned) } else { format!("cooked {} again - yum, +{} acorns", DISHES[dish], earned) },
                            3.5,
                        ));
                        audio.play_ch(1, &voices[players[1].character][0], 0.6);
                        audio.play_ch(0, &voices[players[0].character][4], 0.6);
                        players[1].voice_timer = 0.5;
                        for k in 0..6 {
                            let who_pos = if k % 2 == 0 { players[0].pos } else { players[1].pos };
                            hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.2, 1.0 + k as f32 * 0.1, who_pos.y + 0.05), 1.6));
                        }
                    }
                    continue;
                }
                let id = who * 6 + i;
                let play_now = if music_mode { looper.press(id) } else { true };
                quest_event(&mut quests, QuestKind::Sing, 1, &mut inv, &mut toasts);
                if !music_mode {
                    let (pos, facing) = (players[who].pos, players[who].facing);
                    let (tx, tz) = players[who].ahead();
                    cast(i, &mut world, pos, facing, &mut particles);
                    if i == 0 && (tx, tz) == CAMPFIRE && camp.fuel > 0.0 && !camp.lit {
                        camp.lit = true;
                        toasts.push(("ember! the fire is lit".to_string(), 2.5));
                    }
                    if i == 2 && pos.distance(FIRE_SPOT) < 2.5 && camp.lit {
                        camp.lit = false;
                        cooking = false;
                        toasts.push(("the rain put the fire out".to_string(), 2.5));
                    }
                }
                if play_now {
                    let sound = &voices[players[who].character][i];
                    audio.play_ch(who, sound, 0.8);
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.4;
                    if who == 0 && dog_jamming {
                        cat_phrase_notes += 1;
                        recent_cat.push(i);
                        if recent_cat.len() > 8 {
                            recent_cat.remove(0);
                        }
                        if !partner_soloing && world.rand() < 0.6 {
                            echoes.push((looper.position + 8.0, i));
                        }
                    }
                }
            }
        }

        for id in looper.tick(get_frame_time()) {
            if id >= 28 && id < 32 {
                audio.play_ch(2, perc[id - 28], 0.7);
                continue;
            }
            if id >= 16 {
                play_pad((id - 16) / 6, (id - 16) % 6, 0.7, players[1].character);
                continue;
            }
            if id >= 12 {
                audio.play_ch(2, perc[(id - 12).min(3)], 0.7);
                continue;
            }
            let (who, i) = (id / 6, id % 6);
            let sound = &voices[players[who].character][i];
            audio.play_ch(who, sound, 0.75);
            players[who].key_glow[i] = 1.0;
            players[who].voice_timer = 0.3;
        }
        let position = looper.position;
        let friend_voice = players[1].character;
        echoes.retain(|&(at, i)| {
            if position >= at {
                audio.play_ch(1, &voices[friend_voice][i], 0.45);
                players[1].voice_timer = 0.3;
                false
            } else {
                position > at - 64.0
            }
        });
        let step = looper.current_step();
        if looper.last_step != Some(step) {
            let crossed: Vec<usize> = match looper.last_step {
                Some(prev) => {
                    let mut v = Vec::new();
                    let mut k = (prev + 1) % LOOP_STEPS;
                    while k != step && v.len() < LOOP_STEPS {
                        v.push(k);
                        k = (k + 1) % LOOP_STEPS;
                    }
                    v.push(step);
                    v
                }
                None => vec![step],
            };
            looper.last_step = Some(step);
            sampler.step = step % 16;
            if inside && house.has(home::Item::Sampler) {
                for row in 0..4 {
                    if sampler.grid[row][step % 16] {
                        if let Some(pad) = &sampler.pads[row] {
                            audio.play_private(pad, 0.8);
                        } else {
                            match row {
                                0 => audio.play_ch(2, &kick, 0.8),
                                1 => audio.play_ch(2, &clap, 0.55),
                                2 => audio.play_ch(2, &hat, 0.45),
                                _ => audio.play_ch(2, &voices[players[sampler.owner].character][(step / 4) % 6], 0.45),
                            }
                        }
                    }
                }
            }
            if (ddr.on[0] || ddr.on[1]) && time - ddr.last_mic > 3.0 && step % 2 == 0 {
                let step_s = looper.step_seconds();
                let frac = (looper.position - step as f32).rem_euclid(LOOP_STEPS as f32);
                let target = time + (16.0 - frac) * step_s;
                let slot_len = step_s * 2.0;
                if looper.beat {
                    let style = &BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)];
                    if velocity(style.kick, step).is_some() {
                        ddr.add(0, target, slot_len);
                    }
                    if velocity(style.clap, step).is_some() {
                        ddr.add(1, target, slot_len);
                    }
                    if velocity(style.open, step).is_some() {
                        ddr.add(3, target, slot_len);
                    } else if velocity(style.hat, step).is_some() && step % 4 == 2 {
                        ddr.add(2, target, slot_len);
                    }
                }
                let notes: Vec<usize> = looper.notes.iter().filter(|(at, _)| (at.round() as usize) % LOOP_STEPS == (step + 16) % LOOP_STEPS).map(|(_, id)| *id).collect();
                for id in notes {
                    ddr.add(note_slot(id) % 4, target, slot_len);
                }
            }
            if inside && house.record_on && house.has(home::Item::RecordPlayer) && playback.is_none() {
                if crossed.contains(&0) || record_tune.is_empty() {
                    let tunes = if songbook.is_empty() { vec![vec![(0, 0), (4, 2), (8, 4), (12, 3), (16, 2), (20, 1), (24, 0)]] } else { songbook.clone() };
                    record_pick = (record_pick + 1) % tunes.len();
                    record_tune = tunes[record_pick].clone();
                }
                for &(at, note) in &record_tune {
                    if crossed.contains(&at) {
                        audio.play_ch(1, &voices[players[1].character][note], 0.3);
                    }
                }
            }
            if looper.beat {
                let style = &BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)];
                if let Some(v) = velocity(style.kick, step) {
                    let sub = if house.switched_on(home::Item::Subwoofer) { 1.4 } else { 1.0 };
                    audio.play_ch(2, &kick, (0.75 * v * sub).min(1.0));
                }
                if let Some(v) = velocity(style.clap, step) {
                    audio.play_ch(2, &clap, 0.5 * v);
                }
                if let Some(v) = velocity(style.hat, step) {
                    audio.play_ch(2, &hat, 0.45 * v);
                }
                if let Some(v) = velocity(style.open, step) {
                    audio.play_ch(2, &open_hat, 0.4 * v);
                }
            } else if (looper.recording || fishing.active) && step % 4 == 0 {
                let accent = if step % 16 == 0 { 0.5 } else { 0.25 };
                audio.play_ch(2, &hat, accent);
            }
            if dog_jamming && crossed.contains(&0) && cat_phrase_notes >= 3 {
                inv.acorns += 1;
                toasts.push(("nice jam: +1 acorn".to_string(), 1.6));
            }
            if dog_jamming && crossed.contains(&0) {
                let chance = if partner_solo_count >= 2 {
                    0.15
                } else if cat_phrase_notes >= 2 {
                    0.75
                } else if partner_soloing {
                    0.3
                } else {
                    0.45
                };
                partner_soloing = world.rand() < chance;
                partner_solo_count = if partner_soloing { partner_solo_count + 1 } else { 0 };
                partner_phrase.clear();
                if partner_soloing {
                    let partner_mood: &str;
                    let pick = |world: &mut World, n: usize| (world.rand() * n as f32) as usize % n.max(1);
                    let longest = (2 + partner_xp as usize / 2).min(8);
                    let roll = world.rand();
                    let fresh = |world: &mut World, recent: &[usize]| {
                        let mut phrase = Vec::new();
                        let mut at = if world.rand() < 0.5 { 0 } else { 2 };
                        let length = 1 + (world.rand() * longest as f32) as usize;
                        let mut note = recent.last().copied().unwrap_or((world.rand() * 6.0) as usize % 6) as i32;
                        for k in 0..length {
                            if at >= LOOP_STEPS - 1 {
                                break;
                            }
                            let borrow = !recent.is_empty() && world.rand() < 0.35;
                            note = if borrow {
                                recent[k % recent.len()] as i32
                            } else {
                                let moves = [-2, -1, -1, 0, 1, 1, 2];
                                (note + moves[(world.rand() * 7.0) as usize % 7]).clamp(0, 5)
                            };
                            phrase.push((at, note as usize));
                            let gaps = [2, 2, 3, 4, 4, 6, 1];
                            at += gaps[(world.rand() * 7.0) as usize % 7];
                        }
                        phrase
                    };
                    if songbook.is_empty() || (roll < 0.3 && songbook.len() < 8) {
                        partner_phrase = fresh(&mut world, &recent_cat);
                        songbook.push(partner_phrase.clone());
                        partner_mood = if songbook.len() == 1 { "a first little tune" } else { "a new tune" };
                    } else if roll < 0.7 {
                        let k = pick(&mut world, songbook.len());
                        let mut tune = songbook[k].clone();
                        let change = world.rand();
                        if change < 0.4 && tune.len() < longest {
                            let last = tune.last().copied().unwrap_or((0, 2));
                            let gaps = [2, 3, 4];
                            let at = last.0 + gaps[pick(&mut world, 3)];
                            if at < LOOP_STEPS - 1 {
                                let moves = [-1, 1, 2, -2];
                                tune.push((at, (last.1 as i32 + moves[pick(&mut world, 4)]).clamp(0, 5) as usize));
                            }
                        } else if change < 0.75 && !tune.is_empty() {
                            let j = pick(&mut world, tune.len());
                            let moves = [-1, 1];
                            tune[j].1 = (tune[j].1 as i32 + moves[pick(&mut world, 2)]).clamp(0, 5) as usize;
                        } else {
                            let shift = if world.rand() < 0.5 { -1 } else { 1 };
                            for n in tune.iter_mut() {
                                n.1 = (n.1 as i32 + shift).clamp(0, 5) as usize;
                            }
                        }
                        songbook[k] = tune.clone();
                        partner_phrase = tune;
                        partner_mood = "growing a tune";
                    } else {
                        let k = pick(&mut world, songbook.len());
                        partner_phrase = songbook[k].clone();
                        partner_mood = "its favourite tune";
                    }
                    partner_xp += 1;
                    toasts.push((format!("the {} plays {}", cast_list[players[1].character].noun, partner_mood), 2.0));
                }
                cat_phrase_notes = 0;
            }
            if !dog_jamming {
                partner_soloing = false;
                partner_phrase.clear();
            }
            if dog_jamming && partner_soloing {
                for &(at, note) in &partner_phrase {
                    if crossed.contains(&at) {
                        audio.play_ch(1, &voices[players[1].character][note], 0.62);
                        players[1].voice_timer = 0.3;
                        players[1].key_glow[note] = 1.0;
                    }
                }
                if step % 4 == 2 && !looper.beat && !cooking {
                    audio.play_ch(2, &hat, 0.16);
                }
            } else if dog_jamming {
                let chord = [0, 4, 3, 1][(step / 8) % 4];
                if step % 8 == 0 {
                    audio.play_ch(1, &voices[players[1].character][chord], 0.42);
                    players[1].voice_timer = 0.25;
                } else if step % 16 == 11 {
                    audio.play_ch(1, &voices[players[1].character][chord], 0.26);
                }
                if step % 4 == 2 && !looper.beat && !cooking {
                    audio.play_ch(2, &hat, 0.16);
                }
            }
        }

        let together = !inside && if two_player { players[0].seated && players[1].seated } else { dog_jamming };
        let playing = together && time - last_note_at < 8.0;
        match session {
            None => {
                if playing {
                    session = Some(0.0);
                    session_stars = 0;
                    audio.start_jam();
                }
            }
            Some(t) => {
                if playing {
                    let t = t + dt;
                    session = Some(t);
                    let stars = stars_for(t);
                    if stars > session_stars {
                        session_stars = stars;
                        quest_event(&mut quests, QuestKind::Star, 1, &mut inv, &mut toasts);
                        let label = ["", "30 seconds together - 1 star!", "1 minute together - 2 stars!", "3 minutes together - 3 stars!!"][stars as usize];
                        toasts.push((label.to_string(), 3.0));
                        for k in 0..3 {
                            audio.play_ch(3, &bells[(k * 2 + stars as usize) % 6], 0.35);
                        }
                        for k in 0..(4 * stars as usize) {
                            let who_pos = if k % 2 == 0 { players[0].pos } else { players[1].pos };
                            hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.25, 1.0 + k as f32 * 0.08, who_pos.y + 0.05), 1.8));
                        }
                    }
                } else {
                    let events = audio.stop_jam();
                    if t >= 30.0 {
                        diary.push(DiaryEntry {
                            when: miniquad::date::now(),
                            seconds: t as u32,
                            stars: stars_for(t),
                            who: (players[0].character, players[1].character),
                            events,
                        });
                        toasts.push((format!("the jam was written in the diary - {} star{}", stars_for(t), if stars_for(t) == 1 { "" } else { "s" }), 3.5));
                    }
                    session = None;
                    session_stars = 0;
                }
            }
        }
        if first_steps == 3 && session_stars > 0 {
            first_steps = 4;
            toasts.push(("your first jam! explore the house and make more music".to_string(), 4.0));
        }
        if let Some((idx, start, next)) = playback {
            if !inside || idx >= diary.len() {
                playback = None;
            } else {
                let events = &diary[idx].events;
                let t = (get_time() - start) as f32;
                let mut k = next;
                while k < events.len() && events[k].0 <= t {
                    audio.play_id(events[k].1, events[k].2 * 0.85);
                    k += 1;
                }
                if k >= events.len() && t > events.last().map_or(0.0, |e| e.0) + 2.0 {
                    playback = None;
                    toasts.push(("the record ends".to_string(), 2.0));
                } else {
                    playback = Some((idx, start, k));
                }
            }
        }
        stick_trees.retain(|&(x, z)| world.get(x, z).map(|t| t.kind == Kind::Tree && t.fire <= 0.0).unwrap_or(false));
        stick_clock -= dt;
        if stick_clock <= 0.0 {
            stick_clock = if stick_trees.len() < 4 { 0.5 } else { 12.0 + world.rand() * 8.0 };
            if stick_trees.len() < 6 {
                grow_stick(&mut world, &mut stick_trees);
            }
        }
        tick_clock += dt;
        while tick_clock >= TICK {
            tick_clock -= TICK;
            world.tick(&mut particles);
        }
        world.wind_time = (world.wind_time - dt).max(0.0);

        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let t = world.get(x, z).unwrap();
                if t.fire > 0.0 && hash(x, z, (time * 20.0) as u32) > 0.8 {
                    let drift = if world.wind_time > 0.0 { world.wind * 1.2 } else { Vec2::ZERO };
                    spawn(
                        &mut particles,
                        vec3(x as f32 + 0.3 + hash(z, x, (time * 30.0) as u32) * 0.4, 0.9, z as f32 + 0.5),
                        vec3(drift.x, 1.4, drift.y),
                        0.8,
                        Color::from_rgba(255, 190, 80, 255),
                        0.035,
                        0.0,
                    );
                }
            }
        }
        for p in particles.iter_mut() {
            p.vel.y -= p.gravity * dt * 9.8;
            p.pos += p.vel * dt;
            p.life -= dt;
        }
        particles.retain(|p| p.life > 0.0);
        for h in hearts.iter_mut() {
            h.0.y += dt * 0.5;
            h.1 -= dt;
        }
        hearts.retain(|h| h.1 > 0.0);
        for t in toasts.iter_mut() {
            t.1 -= dt;
        }
        toasts.retain(|t| t.1 > 0.0);
        if toasts.len() > 3 {
            toasts.remove(0);
        }

        let cat = &players[0];
        let ahead = cat.ahead();
        let can_build_here = !inside && !cat.seated && !fishing.active && !(camp.lit && near_fire(cat)) && !at_dock_end(cat) && buildable(&world, ahead.0, ahead.1);
        let ring = if fish_card > 0.0 { 1.0 } else { ((fishing.cue_at - looper.position) / FISH_CUE_GAP).clamp(0.0, 1.0) };
        let extras = Extras {
            fishing: fishing.active,
            ring,
            in_window: fish_card <= 0.0 && (looper.position - fishing.cue_at).abs() <= 0.22 * fishing.tight / looper.step_seconds(),
            dip: fishing.dip,
            jump: fishing.jump,
            cat: players[fishing.caster].pos,
            pot: cooking,
            hearts: hearts.clone(),
            ghost: if can_build_here { Some((ahead.0, ahead.1, tools[0])) } else { None },
            sticks: stick_trees.iter().filter_map(|t| stick_spot(&world, *t)).collect(),
            friend_rod: if fishing.active && !two_player && players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8 { Some(players[1].pos) } else { None },
        };

        let dance = if looper.beat || dog_jamming { looper.beat_pulse() } else { 0.0 };
        if low_w() as u32 != view_w || low_h() as u32 != view_h {
            view_w = low_w() as u32;
            view_h = low_h() as u32;
            full_view = make_view(view_w, view_h);
        }
        let half = (low_w() / 2.0).floor();
        set_camera(&Camera2D {
            render_target: Some(full_view.clone()),
            zoom: vec2(2.0 / view_w as f32, -2.0 / view_h as f32),
            target: vec2(view_w as f32 / 2.0, view_h as f32 / 2.0),
            ..Default::default()
        });
        clear_background(FOG);
        let view = View {
            world: &world,
            players: &players,
            particles: &particles,
            camp: &camp,
            time,
            dance,
            dusk,
            extras: &extras,
        };
        let ghost = holding.map(|p| (p, ahead_local(&players[holding_by]) != home::CATALOGUE && house.can_place(&p, &[home::local(players[1 - holding_by].pos)])));
        if inside {
            let pov_for = |who: usize| if dj[who] { booth_pov(&house, if who == 0 { -1.0 } else { 1.0 }) } else { None };
            let beat_on = looper.beat || !now_playing.is_empty();
            let pov = if dj[0] ^ dj[1] { if dj[0] { pov_for(0) } else { pov_for(1) } } else { None };
            let hide = if dj[0] ^ dj[1] { if dj[0] { Some(0) } else { Some(1) } } else { None };
            render_home(&full_view, 0.0, low_w(), low_h(), &view, &house, ghost, pov, hide, beat_on);
        } else if two_player {
            render_view(&full_view, 0.0, half, low_h(), players[0].pos, players[0].cam, &view, true);
            render_view(&full_view, half, half, low_h(), players[1].pos, players[1].cam, &view, true);
        } else {
            render_view(&full_view, 0.0, low_w(), low_h(), players[0].pos, players[0].cam, &view, false);
        }
        if picking {
            render_pick_preview(&pick_view, &world, &cast_list, [players[0].character, players[1].character], pick_row, time);
        }
        if shop_open && shop_tab == 0 {
            render_shop_preview(
                &catalogue_view,
                0.0,
                0.0,
                catalogue_view.texture.width(),
                catalogue_view.texture.height(),
                home::row(shop_row),
                &house,
                time,
                looper.beat || !now_playing.is_empty(),
            );
        }

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &full_view.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        if two_player && !inside {
            ui_rect(half - 0.5, 0.0, 1.0, low_h(), Color::new(0.1, 0.08, 0.2, 0.9));
        }

        let ink = Color::from_rgba(40, 30, 60, 255);
        let touch_ui = radio::touch_ui() && !two_player;
        let portrait = low_h() > low_w();
        let edge = if touch_ui && !portrait { (low_w() * 0.235).round() } else { 0.0 };
        let right_edge = low_w() - edge;
        let bar_y = if touch_ui && portrait { 58.0 } else if touch_ui { low_h() - 2.0 } else { low_h() - 32.0 };
        let grid_x;
        let grid_w;
        if touch_ui {
            grid_w = 296f32.min(low_w() - 24.0);
            grid_x = (low_w() - grid_w) / 2.0;
        } else if two_player {
            for who in 0..2 {
                let box_w = 25.0;
                let start_x = who as f32 * half + (half - box_w * 6.0) / 2.0;
                let labels = if who == 0 { P1_KEY_LABELS } else { P2_KEY_LABELS };
                for i in 0..6 {
                    let x = start_x + i as f32 * box_w;
                    let glow = players[who].key_glow[i];
                    let mut bg = note_color(i);
                    bg.a = 0.55 + glow * 0.45;
                    let lift = glow * 3.0;
                    ui_rect(x + 1.0, bar_y - lift, box_w - 2.0, 28.0, bg);
                    ui_text(labels[i], x + 4.0, bar_y + 12.0 - lift, 12.0, ink);
                    let name = if inside && dj[who] { &DJ_PADS[who][i][..DJ_PADS[who][i].len().min(4)] } else if cooking && players[who].seated && i < 4 { &PERC_NAMES[i][..3] } else if inside && seat[who] == Some(home::Item::Piano) { "piano" } else if inside && seat[who] == Some(home::Item::Drums) { "drum" } else { NOTE_NAMES[i] };
                    ui_text(name, x + 4.0, bar_y + 24.0 - lift, 12.0, ink);
                }
            }
            grid_x = 12.0;
            grid_w = low_w() - 24.0;
        } else {
            let box_w = 50.0;
            let start_x = low_w() / 2.0 - box_w * 3.0;
            for i in 0..6 {
                let x = start_x + i as f32 * box_w;
                let glow = players[0].key_glow[i];
                let mut bg = note_color(i);
                bg.a = 0.55 + glow * 0.45;
                let lift = glow * 3.0;
                ui_rect(x + 2.0, bar_y - lift, box_w - 4.0, 28.0, bg);
                ui_text(&format!("{} {}", i + 1, NOTE_NAMES[i]), x + 6.0, bar_y + 12.0 - lift, 12.0, ink);
                let label = if inside && dj[0] {
                    DJ_PADS[0][i]
                } else if cooking && i < 4 {
                    PERC_NAMES[i]
                } else if music_mode {
                    voice_word(cast_list[players[0].character].voice)
                } else {
                    SPELL_NAMES[i]
                };
                ui_text(label, x + 6.0, bar_y + 24.0 - lift, 12.0, ink);
            }
            grid_x = start_x + 2.0;
            grid_w = box_w * 6.0 - 4.0;
        }

        let show_grid = music_mode || looper.recording || !looper.notes.is_empty();
        let lanes: Vec<(Option<usize>, f32, f32)> = if two_player && !touch_ui {
            (0..2).map(|who| (Some(who), who as f32 * half + (half - 150.0) / 2.0 + 1.0, 148.0)).collect()
        } else {
            vec![(None, grid_x, grid_w)]
        };
        let grid_y = bar_y - 18.0;
        if show_grid {
            for &(lane, lx, lw) in &lanes {
                let cell = lw / LOOP_STEPS as f32;
                ui_rect(lx, grid_y, lw, 14.0, Color::new(0.1, 0.08, 0.2, 0.5));
                for k in (0..LOOP_STEPS).step_by(4) {
                    let a = if k % 16 == 0 { 0.6 } else { 0.25 };
                    ui_rect(lx + k as f32 * cell, grid_y, 0.34, 14.0, Color::new(1.0, 1.0, 1.0, a));
                }
                ui_rect(lx + step as f32 * cell, grid_y, cell, 14.0, Color::new(1.0, 1.0, 1.0, 0.3));
                for &(at, id) in &looper.notes {
                    if lane.map(|who| note_owner(id) != who).unwrap_or(false) {
                        continue;
                    }
                    let slot = note_slot(id).min(5);
                    let nx = lx + at * cell + 0.3;
                    let ny = grid_y + 1.0 + (5 - slot) as f32 * 2.0;
                    let mut color = note_color(slot);
                    if lane.is_none() && note_owner(id) == 1 {
                        color = Color::new(color.r * 0.75, color.g * 0.75, color.b * 0.75, 1.0);
                    }
                    ui_rect(nx, ny, (cell - 0.6).max(1.5), 1.7, color);
                }
            }
        }

        let status_y = if show_grid { grid_y - 4.0 } else { bar_y - 4.0 };
        let mut status_x = grid_x;
        if looper.recording {
            if (time * 2.0).fract() < 0.6 {
                ui_rect(status_x, status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
            }
            ui_text("REC", status_x + 9.0, status_y, 12.0, WHITE);
            status_x += 34.0;
        }
        if listening {
            let dot = if locked {
                Color::from_rgba(90, 230, 120, 255)
            } else {
                let blink = 0.35 + 0.65 * (time * 4.0).sin().abs();
                Color::new(1.0, 0.75, 0.25, blink)
            };
            ui_rect(status_x + 1.0, status_y - 7.0, 6.0, 6.0, dot);
            let text = if locked { format!("synced {:.0} bpm", looper.bpm) } else { "syncing...".to_string() };
            ui_text(&text, status_x + 12.0, status_y, 12.0, if locked { Color::from_rgba(150, 255, 170, 255) } else { WHITE });
        } else if looper.beat {
            ui_text(&format!("beat {:.0} bpm", looper.bpm), status_x, status_y, 12.0, WHITE);
        }
        if let Some(msg) = mic_message {
            mic_message_clock += get_frame_time();
            if mic_message_clock > 3.0 {
                mic_message = None;
            } else {
                ui_rect(grid_x - 1.0, status_y - 21.0, 7.0, 7.0, Color::from_rgba(255, 90, 90, 255));
                ui_text(msg, grid_x + 9.0, status_y - 14.0, 12.0, Color::from_rgba(255, 210, 160, 255));
            }
        } else {
            mic_message_clock = 0.0;
        }
        if let Some(t) = session {
            let label = format!("together {}:{:02}", t as u32 / 60, t as u32 % 60);
            let tw = ui_measure(&label, 11.0) + 36.0;
            let sx = grid_x + (grid_w - tw) / 2.0;
            ui_text(&label, sx, status_y, 11.0, Color::from_rgba(255, 236, 200, 255));
            for k in 0..3u8 {
                let col = if k < session_stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.22) };
                ui_star(sx + ui_measure(&label, 11.0) + 4.0 + k as f32 * 11.0, status_y - 8.0, 8.0, col);
            }
        }
        let mut right_x = grid_x + grid_w;
        if ringing {
            let glow = 0.75 + 0.25 * (time * 4.0).sin();
            right_x -= ui_measure("~echo~", 12.0);
            ui_text("~echo~", right_x, status_y, 12.0, Color::new(0.75, 0.95, 1.0, glow));
            right_x -= 10.0;
        }
        if let Some(t) = audio.tape_seconds() {
            let label = format!("tape {}:{:02}", t as u32 / 60, t as u32 % 60);
            right_x -= ui_measure(&label, 12.0);
            ui_text(&label, right_x, status_y, 12.0, Color::from_rgba(255, 216, 216, 255));
            if (time * 2.0).fract() < 0.6 {
                ui_rect(right_x - 9.0, status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
            }
        }

        ui_rect(0.0, 0.0, low_w(), 17.0, Color::new(0.1, 0.08, 0.2, 0.4));
        let mode = if cooking { "cooking".to_string() } else if music_mode { format!("music  q:{}", looper.quantise.label()) } else { "spells".to_string() };
        if two_player {
            ui_text(cast_list[players[0].character].name, 6.0, 12.0, 12.0, Color::from_rgba(255, 226, 190, 255));
            ui_text(cast_list[players[1].character].name, half + 6.0, 12.0, 12.0, Color::from_rgba(240, 206, 140, 255));
            ui_text(&mode, 76.0, 12.0, 12.0, WHITE);
        } else {
            ui_text(&mode, edge + 6.0, 12.0, 12.0, WHITE);
            if !now_playing.is_empty() && right_edge - 144.0 - edge - ui_measure(&mode, 12.0) > 40.0 {
                let left = edge + 6.0 + ui_measure(&mode, 12.0) + 12.0;
                let room = right_edge - 118.0 - 8.0 - left;
                let line = ui_ticker(&format!("~ {now_playing}"), 12.0, room, time);
                ui_text(&line, left, 12.0, 12.0, Color::from_rgba(255, 214, 110, 255));
            }
        }
        let (meter_x, meter_y) = if two_player { (6.0, 27.0) } else { (right_edge - 118.0, 12.0) };
        ui_text("fire", meter_x, meter_y, 12.0, Color::from_rgba(255, 200, 140, 255));
        ui_rect(meter_x + 24.0, meter_y - 7.0, 40.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.35));
        let fill = (camp.fuel / 180.0).min(1.0);
        let fire_col = if camp.lit { Color::from_rgba(255, 150, 60, 255) } else { Color::from_rgba(120, 110, 110, 255) };
        ui_rect(meter_x + 24.0, meter_y - 7.0, 40.0 * fill, 6.0, fire_col);
        {
            let hx = right_edge - 46.0;
            ui_rect(hx - 2.0, 2.0, 12.0, 13.0, Color::from_rgba(255, 214, 110, 255));
            ui_text("H", hx + 1.0, 12.0, 12.0, Color::from_rgba(36, 26, 71, 255));
            ui_text("help", hx + 14.0, 12.0, 12.0, WHITE);
        }
        {
            let sk = sky();
            let temp = sk.temp.map(|t| format!(" {:.0}C", t)).unwrap_or_default();
            let line = format!("{:02}:{:02}  {}  {}{}", sk.hour as u32, ((sk.hour.fract()) * 60.0) as u32, SEASONS[sk.season], WEATHERS[sk.weather], temp);
            let wx = if two_player { half - ui_measure(&line, 9.0) - 6.0 } else { right_edge - ui_measure(&line, 9.0) - 4.0 };
            ui_text(&line, wx, 38.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        if ring_flash > 0.0 {
            let msg = if ringing { "echo on  (G)" } else { "echo off  (G)" };
            let a = ring_flash.min(1.0);
            let w = ui_measure(msg, 12.0) + 12.0;
            ui_rect((low_w() - w) / 2.0, 26.0, w, 16.0, Color::new(0.1, 0.08, 0.2, 0.7 * a));
            ui_text(msg, (low_w() - w) / 2.0 + 6.0, 38.0, 12.0, Color::new(0.8, 0.95, 1.0, a));
        }

        let fish_total: u32 = inv.fish.iter().sum();
        let items = [
            (Color::from_rgba(176, 124, 76, 255), format!("{}", inv.wood)),
            (Color::from_rgba(168, 170, 184, 255), format!("{}", inv.stone)),
            (Color::from_rgba(140, 190, 210, 255), format!("{}", fish_total)),
            (Color::from_rgba(255, 140, 160, 255), format!("{}/{}", dishes.len(), DISHES.len())),
            (ACORN_COLOR, format!("{}", inv.acorns)),
        ];
        let mut ix = right_edge - 6.0 - items.len() as f32 * 24.0;
        for (col, text) in items.iter() {
            ui_rect(ix, 21.0, 6.0, 6.0, *col);
            ui_text(text, ix + 8.0, 27.0, 11.0, WHITE);
            ix += 24.0;
        }

        let inside_hint = if inside {
            let ahead = ahead_local(&players[0]);
            let here = home::local(players[0].pos);
            Some(if shop_open || dj[0] {
                String::new()
            } else if let Some(piece) = holding {
                if ahead == home::CATALOGUE {
                    format!("E: sell the {} for {} acorns", piece.item.name(), piece.item.price() / 2)
                } else {
                    format!("E: place the {} - X: turn it - M: put in bag", piece.item.name())
                }
            } else if let Some(item) = seat[0] {
                match item {
                    home::Item::Piano | home::Item::Drums => format!("1-6: play the {} - move to get up", item.name()),
                    home::Item::Bed => "zzz - move to get up".to_string(),
                    _ => format!("sitting on the {} - move to get up", item.name()),
                }
            } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
                format!("E: catalogue & diary ({} acorns)", inv.acorns)
            } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                let item = house.pieces[i].item;
                match item {
                    home::Item::Piano | home::Item::Drums => format!("E: play the {} - X: move it", item.name()),
                    home::Item::RecordPlayer => format!("E: record {} - X: move it", if house.record_on { "off" } else { "on" }),
                    home::Item::DjBooth => "E: get on the decks - X: move it".to_string(),
                    home::Item::WindChime => "E: ring the chime - X: move it".to_string(),
                    home::Item::Lamp | home::Item::RedLight => format!("E: light {} - X: move it", if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Speaker | home::Item::Subwoofer => format!("E: {} {} - X: move it", item.name(), if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Bed => "E: sleep (saves the game) - X: move it".to_string(),
                    home::Item::Sofa | home::Item::ConcreteBench => format!("E: sit on the {} - X: move it", item.name()),
                    home::Item::Plant => {
                        let piece = &house.pieces[i];
                        if piece.state == home::WITHERED {
                            "withered - E: water it to bring it back".to_string()
                        } else if piece.day == sky().day as i32 {
                            "watered today - X: move it".to_string()
                        } else {
                            "thirsty - E: water the plant - X: move it".to_string()
                        }
                    }
                    home::Item::Bookshelf => "E: read a book - X: move it".to_string(),
                    home::Item::Table => "E: serve a dish - X: move it".to_string(),
                    home::Item::SteelPillar => "E: knock on it - X: move it".to_string(),
                    home::Item::MirrorBall => format!("E: mirror ball {} - X: move it", if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Tv => format!("E: next channel ({}) - X: move it", if house.pieces[i].state == 0 { "off" } else { TV_CHANNELS[house.pieces[i].state as usize - 1] }),
                    home::Item::Arcade => "E: dance! - X: move it".to_string(),
                    home::Item::Sampler => "E: open the sampler - X: move it".to_string(),
                    _ => format!("X: move the {}", item.name()),
                }
            } else if here.y > house.d() as f32 - 1.1 && (here.x - house.door().x).abs() < 0.9 {
                "walk out the door to go outside".to_string()
            } else {
                String::new()
            })
        } else {
            None
        };
        let carried_name = |c: Option<Carry>| match c {
            Some(Carry::Stick) => "stick",
            Some(Carry::Stone) => "stone",
            Some(Carry::Acorn) => "acorn",
            None => "",
        };
        let hint: String = if let Some(h) = inside_hint {
            h
        } else if players[0].carrying.is_some() && !players[0].seated {
            format!("bring the {} to the logs by the fire", carried_name(players[0].carrying))
        } else if two_player && players[1].carrying.is_some() {
            format!("player 2: bring the {} to the logs by the fire", carried_name(players[1].carrying))
        } else if fishing.active {
            let coop = two_player && players[0].pos.distance(DOCK_END) < 2.2 && players[1].pos.distance(DOCK_END) < 2.2;
            let who = if coop { "E and O" } else if fishing.caster == 1 { "O" } else { "E" };
            if fish_card > 0.0 {
                String::new()
            } else if fishing.lesson {
                format!("{} when the ring touches the bobber - catch {}/3", who, fishing.hits.min(3))
            } else {
                format!("{} when the ring closes - catch {}/3 - tries {}", who, fishing.hits.min(3), "o".repeat(fishing.tries as usize))
            }
        } else if cooking {
            format!("cook on the beat {}/{} - {}: done", cook_progress, COOK_GOAL, if two_player { "E / O" } else { "E" })
        } else if jam {
            if dog_jamming && partner_soloing {
                format!("the {} knows {} tune{} - answer back!", cast_list[players[1].character].noun, songbook.len(), if songbook.len() == 1 { "" } else { "s" })
            } else if dog_jamming || (two_player && players[1].seated) {
                "playing together by the fire - E / O: cook, move to stand up".to_string()
            } else {
                format!("playing by the fire - E: cook, the {} is on the way", cast_list[players[1].character].noun)
            }
        } else if players[0].pos.distance(DOOR_OUT) < 0.9 && !players[0].seated {
            if two_player { "E / O: go inside".to_string() } else { "E: go inside".to_string() }
        } else if camp.lit && near_fire(&players[0]) && !players[0].seated {
            "E: sit by the fire".to_string()
        } else if at_dock_end(&players[0]) {
            if two_player { "E / O: cast your fishing line".to_string() } else { "E: cast your fishing line".to_string() }
        } else if can_build_here {
            format!("E: build {}  X: change", tools[0].name())
        } else if !camp.lit && camp.fuel <= 0.0 {
            if two_player { "find a tree with a stick at its foot - E or O picks it up".to_string() } else { format!("the {} is looking for sticks - or grab one yourself with E", cast_list[players[1].character].noun) }
        } else {
            String::new()
        };
        if two_player && inside && !picking && !shop_open {
            let ahead = ahead_local(&players[1]);
            let here = home::local(players[1].pos);
            let mut p2_hint = if dj[1] {
                "O: leave DJ booth".to_string()
            } else if let Some(piece) = holding {
                if holding_by == 1 {
                    format!("O: place {} - U: turn - Bksp: bag", piece.item.name())
                } else {
                    String::new()
                }
            } else if let Some(item) = seat[1] {
                if item == home::Item::Bed { "O: get out of bed".to_string() } else { format!("O: get up from {}", item.name()) }
            } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
                "O: catalogue and diary".to_string()
            } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                format!("O: use {} - U: move", house.pieces[i].item.name())
            } else {
                String::new()
            };
            while ui_measure(&p2_hint, 11.0) > half - 12.0 && p2_hint.len() > 7 {
                p2_hint.pop();
            }
            if !p2_hint.is_empty() {
                let w = ui_measure(&p2_hint, 11.0);
                let x = half + (half - w) * 0.5;
                let y = if show_grid { grid_y - 31.0 } else { bar_y - 26.0 };
                ui_rect(x - 4.0, y - 9.0, w + 8.0, 13.0, Color::new(0.1, 0.08, 0.2, 0.68));
                ui_text(&p2_hint, x, y, 11.0, Color::from_rgba(255, 236, 200, 255));
            }
        }
        if !hint.is_empty() && !picking {
            let w = ui_measure(&hint, 12.0);
            let hy = if touch_ui && portrait { (low_h() * 0.56).round() } else if show_grid { grid_y - 16.0 } else { bar_y - 10.0 };
            ui_rect((low_w() - w) / 2.0 - 4.0, hy - 9.0, w + 8.0, 12.0, Color::new(0.1, 0.08, 0.2, 0.5));
            ui_text(&hint, (low_w() - w) / 2.0, hy, 12.0, Color::from_rgba(255, 236, 200, 255));
        }
        for (k, (text, life)) in toasts.iter().enumerate() {
            let a = life.min(1.0);
            let toast_y = if touch_ui && portrait { 70.0 } else { 40.0 };
            ui_text(text, edge + 8.0, toast_y + k as f32 * 11.0, 12.0, Color::new(1.0, 0.95, 0.85, a));
        }
        if show_inventory && !picking && !shop_open && !show_help {
            let lines = [
                format!("acorns  {}", inv.acorns),
                format!("wood    {}   stone  {}", inv.wood, inv.stone),
                format!("fish    {}   mushrooms  {}", inv.fish[0] + inv.fish[1] + inv.fish[2], inv.mush[0] + inv.mush[1] + inv.mush[2]),
                format!("fishdex {}/{}   shrooms {}/{}", dex_counts.iter().filter(|n| **n > 0).count(), fish::COUNT, shroom_counts.iter().filter(|n| **n > 0).count(), mushroom::COUNT),
                format!("dishes  {}/{}   tunes  {}", dishes.len(), DISHES.len(), songbook.len()),
            ];
            let w = 210f32.min(low_w() - 16.0);
            let shown_items = 5usize;
            let furn_rows = if stored.is_empty() { 1 } else { stored.len().min(shown_items) };
            let h = 30.0 + lines.len() as f32 * 11.0 + 16.0 + quests.list.len() as f32 * 11.0 + 16.0 + furn_rows as f32 * 11.0 + 18.0;
            let x = 8.0;
            let y = if touch_ui && portrait { 86.0 } else { 22.0 };
            let h = h.min(low_h() - y - 4.0);
            ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.92));
            ui_text(if two_player { if bag_owner == 0 { "bag - player 1" } else { "bag - player 2" } } else { "bag" }, x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            ui_rect(x + 8.0, y + 17.0, w - 16.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
            let mut ty = y + 29.0;
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::from_rgba(255, 236, 200, 255) } else { WHITE };
                ui_text(line, x + 8.0, ty, 10.0, col);
                ty += 11.0;
            }
            ty += 5.0;
            ui_text("today's quests", x + 8.0, ty, 10.0, Color::from_rgba(150, 215, 255, 255));
            ty += 11.0;
            for q in &quests.list {
                let mark = if q.done { "[x]" } else { "[ ]" };
                let line = format!("{} {}  {}/{}", mark, q.label(), q.progress, q.target);
                let reward = format!("+{}", q.reward());
                let col = if q.done { Color::new(0.6, 0.9, 0.6, 0.9) } else { WHITE };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.0) > w - 40.0 && shown.len() > 6 {
                    shown.pop();
                }
                ui_text(&shown, x + 8.0, ty, 9.0, col);
                ui_text(&reward, x + w - 8.0 - ui_measure(&reward, 9.0), ty, 9.0, ACORN_COLOR);
                ty += 11.0;
            }
            ty += 5.0;
            ui_text(&format!("furniture in the bag ({})", stored.len()), x + 8.0, ty, 10.0, Color::from_rgba(255, 190, 210, 255));
            ty += 11.0;
            if stored.is_empty() {
                ui_text(if inside { "empty - M while carrying a piece puts it here" } else { "empty" }, x + 8.0, ty, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
            } else {
                let first = bag_sel.saturating_sub(shown_items / 2).min(stored.len().saturating_sub(shown_items));
                for r in first..(first + shown_items).min(stored.len()) {
                    if inside && r == bag_sel {
                        ui_rect(x + 5.0, ty - 8.0, w - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.14));
                    }
                    ui_text(stored[r].name(), x + 10.0, ty, 9.0, WHITE);
                    ty += 11.0;
                }
            }
            let bag_hint = if !inside {
                if touch_ui { "bag to close" } else { "M to close" }
            } else if bag_clear_armed > 0.0 {
                "X again: pack ALL furniture away"
            } else if bag_owner == 0 {
                "W/S choose  E take out  X pack all  M close"
            } else {
                "I/K choose  O take out  U pack all  Bksp close"
            };
            ui_text(bag_hint, x + (w - ui_measure(bag_hint, 9.0)) / 2.0, y + h - 6.0, 9.0, if bag_clear_armed > 0.0 { Color::from_rgba(255, 150, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) });
        }

        for who in 0..2 {
            if !(inside && dj[who]) || (who == 1 && !two_player) {
                continue;
            }
            let style = BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)].name;
            let title = format!("{}  {:.0} bpm  {}{}", if dj[0] && dj[1] { "b2b" } else { "on the decks" }, looper.bpm, if looper.beat { style } else { "beat off" }, if looper.recording { "  rec" } else { "" });
            let area = if two_player { half } else { low_w() };
            let left = if two_player && who == 1 { half } else { 0.0 };
            let hint = if who == 0 { "W/S tempo  A/D style  R rec  X out" } else { "7 8 9 0 - = pads  O out" };
            let track = if now_playing.is_empty() { String::new() } else { format!("now playing: {now_playing}") };
            let two_decks = live_decks.len() >= 2;
            let widest = ui_measure(&title, 12.0).max(ui_measure(hint, 9.0)).max(ui_measure(&track, 10.0));
            let w = if two_decks { (area - 8.0).min(400.0) } else { (widest + 16.0).max(150.0).min(area - 8.0) };
            let col_w = (w - 24.0) / 2.0;
            let columns: Vec<Vec<String>> = if two_decks {
                live_decks.iter().take(2).map(|(_, name)| ui_wrap(name, 10.0, col_w).into_iter().take(4).collect()).collect()
            } else {
                Vec::new()
            };
            let lines = if two_decks || track.is_empty() { Vec::new() } else { ui_wrap(&track, 10.0, w - 16.0) };
            let body = if two_decks { 13.0 + columns.iter().map(|c| c.len()).max().unwrap_or(1) as f32 * 12.0 + 4.0 } else { lines.len() as f32 * 12.0 };
            let w = if live_next.is_empty() { w } else { w.max(220.0f32.min(area - 8.0)) };
            let heard_line = live_next.iter().find(|t| t.starts_with('=')).map(|t| t[1..].to_string()).filter(|t| two_decks || !track.contains(t.as_str()));
            let picks: Vec<String> = live_next.iter().filter(|t| !t.starts_with('=')).take(4).map(|t| {
                let mut t = format!("+ {}", t);
                while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                    t.pop();
                }
                t
            }).collect();
            let heard_line = heard_line.map(|t| {
                let mut t = format!("heard: {}", t);
                while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                    t.pop();
                }
                t
            });
            let heard_h = if heard_line.is_some() { 13.0 } else { 0.0 };
            let extra = heard_h + if picks.is_empty() { 0.0 } else { 14.0 + picks.len() as f32 * 11.0 };
            let h = 30.0 + body + extra;
            let x = left + (area - w) / 2.0;
            let y = if low_h() > low_w() { 84.0 } else if two_player { 64.0 } else { 52.0 };
            ui_rect(x, y, w, h, Color::new(0.06, 0.04, 0.1, 0.78));
            ui_text(&title, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 90, 100, 255));
            for (n, line) in lines.iter().enumerate() {
                ui_text(line, x + 8.0, y + 25.0 + n as f32 * 12.0, 10.0, Color::from_rgba(255, 214, 110, 255));
            }
            let deck_colors = [Color::from_rgba(220, 60, 70, 255), Color::from_rgba(240, 200, 80, 255)];
            for (c, column) in columns.iter().enumerate() {
                let cx = x + 8.0 + c as f32 * (col_w + 8.0);
                ui_rect(cx, y + 17.0, 2.0, body - 4.0, deck_colors[c]);
                ui_text(&format!("deck {}", live_decks[c].0), cx + 6.0, y + 26.0, 9.0, deck_colors[c]);
                for (n, line) in column.iter().enumerate() {
                    ui_text(line, cx + 6.0, y + 38.0 + n as f32 * 12.0, 10.0, WHITE);
                }
            }
            let lines = if two_decks { vec![String::new(); ((body - 4.0) / 12.0).round() as usize] } else { lines };
            let mut shown = hint.to_string();
            while ui_measure(&shown, 9.0) > w - 14.0 && shown.len() > 4 {
                shown.pop();
            }
            ui_text(&shown, x + 8.0, y + 25.0 + lines.len() as f32 * 12.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.75));
            if let Some(line) = &heard_line {
                ui_text(line, x + 8.0, y + 30.0 + body + 6.0, 9.0, Color::from_rgba(150, 200, 255, 255));
            }
            if !picks.is_empty() {
                let top = y + 30.0 + body + 6.0 + heard_h;
                ui_text("goes well with", x + 8.0, top, 9.0, Color::from_rgba(120, 220, 160, 255));
                for (n, pick) in picks.iter().enumerate() {
                    ui_text(pick, x + 8.0, top + 12.0 + n as f32 * 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.9));
                }
            }
        }
        if fishing.active && fish_card <= 0.0 && fishing.lesson && (looper.position - fishing.cue_at).abs() <= 0.22 / looper.step_seconds() {
            let pop = 1.0 + 0.15 * (time * 16.0).sin();
            let size = 22.0 * pop;
            let word = if fishing.caster == 1 { "O!" } else { "E!" };
            let cx = if two_player { half * (fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
            ui_text(word, cx - ui_measure(word, size) / 2.0, low_h() * 0.38, size, Color::from_rgba(255, 226, 120, 255));
        }
        if let Some((i, left)) = fish_pop.map(|(i, l, _, _)| (i, l)) {
            let is_new = fish_pop.map(|f| f.2).unwrap_or(false);
            let shroom = fish_pop.map(|f| f.3).unwrap_or(false);
            let a = (left.min(0.4) / 0.4).min(1.0) * ((3.0 - left) * 6.0).min(1.0);
            let w = 150.0;
            let cx = if two_player { half / 2.0 } else { low_w() / 2.0 };
            let x = (cx - w / 2.0).round();
            let y = if low_h() > low_w() { 120.0 } else { 46.0 };
            ui_rect(x, y, w, 48.0, Color::new(0.08, 0.12, 0.22, 0.9 * a));
            let rarity_now = if shroom { mushroom::rarity(i) } else { fish::rarity(i) };
            let rc = fish::RARITY_COLORS[rarity_now];
            ui_rect(x, y, w, 2.0, Color::new(rc.r, rc.g, rc.b, a));
            if a > 0.5 {
                if shroom {
                    draw_fish_art(&shroom_art[i], x + 8.0, y + 7.0, 2.8, true);
                } else {
                    draw_fish_art(&fish_art[i], x + 6.0, y + 8.0, 2.2, true);
                }
            }
            let tx = x + 50.0;
            ui_text(&if shroom { mushroom::code(i) } else { fish::code(i) }, tx, y + 14.0, 9.0, Color::new(0.75, 0.85, 1.0, a));
            if is_new {
                ui_rect(tx + 26.0, y + 6.0, 26.0, 10.0, Color::new(1.0, 0.45, 0.55, a));
                ui_text("new!", tx + 29.0, y + 14.0, 9.0, Color::new(1.0, 1.0, 1.0, a));
            }
            let mut name = if shroom { mushroom::SPECIES[i] } else { fish::SPECIES[i] }.to_string();
            while ui_measure(&name, 11.0) > w - 56.0 && name.len() > 3 {
                name.pop();
            }
            ui_text(&name, tx, y + 28.0, 11.0, Color::new(1.0, 1.0, 1.0, a));
            let r = rarity_now;
            let c = fish::RARITY_COLORS[r];
            ui_text(fish::RARITY_NAMES[r], tx, y + 41.0, 9.0, Color::new(c.r, c.g, c.b, a));
        }
        if let Some(pop) = fish_pop.as_mut() {
            pop.1 -= get_frame_time();
            if pop.1 <= 0.0 {
                fish_pop = None;
            }
        }
        for who in 0..2 {
            if let Some((id, t)) = emoji_choose[who] {
                let (sx, sy, cell) = emoji_strip(who, two_player);
                ui_rect(sx - 4.0, sy - 4.0, cell * emoji::COUNT as f32 + 8.0, cell + 8.0, Color::new(0.97, 0.94, 0.87, 0.92));
                for k in 0..emoji::COUNT {
                    let ex = sx + k as f32 * cell;
                    if k == id {
                        ui_rect(ex - 1.0, sy - 2.0, cell, cell + 2.0, Color::from_rgba(255, 214, 110, 255));
                    }
                    draw_emoji_ui(k, ex + 1.5, sy + 0.5, 1.15, 1.0);
                }
                let fill = (t / 1.1).min(1.0);
                ui_rect(sx, sy + cell + 1.5, cell * emoji::COUNT as f32 * fill, 1.5, Color::from_rgba(255, 214, 110, 255));
            }
        }
        let small_pop = fish_say.as_ref().map(|(w, _, _)| matches!(w.as_str(), "perfect!" | "nice!" | "together!")).unwrap_or(false);
        if let (true, Some((word, col, left))) = (small_pop, fish_say.as_ref()) {
            let view = if two_player { fishing.caster.min(1) } else { 0 };
            let (bx, by) = BOBBER_AT.with(|cell| cell.get()[view]);
            let age = 1.0 - left.clamp(0.0, 1.0);
            let a = (left / 0.35).min(1.0);
            let pop = 1.0 + 0.45 * (1.0 - (age * 6.0).min(1.0));
            let size = 9.0 * pop;
            let w = ui_measure(word, size);
            let y = by - 4.0 - age * 10.0;
            ui_text(word, bx - w / 2.0, y, size, Color::new(col.r, col.g, col.b, a));
            if word == "perfect!" {
                for k in 0..3 {
                    let ang = age * 5.0 + k as f32 * 2.1;
                    ui_star(bx + ang.cos() * (w * 0.5 + 5.0), y - 3.0 + ang.sin() * 5.0, 2.5, Color::new(1.0, 0.95, 0.6, a));
                }
            }
        } else if let Some((word, col, left)) = fish_say.as_ref() {
            let a = left.min(0.4) / 0.4;
            let rise = (1.0 - left.min(1.0)) * 6.0;
            let cx = if two_player { half * (fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
            let size = if word.len() < 12 { 16.0 } else { 11.0 };
            let w = ui_measure(word, size);
            ui_rect(cx - w / 2.0 - 6.0, low_h() * 0.3 - rise - size, w + 12.0, size + 6.0, Color::new(0.08, 0.06, 0.16, 0.7 * a));
            ui_text(word, cx - w / 2.0, low_h() * 0.3 - rise, size, Color::new(col.r, col.g, col.b, a));
        }
        if fish_card > 0.0 {
            let a = (fish_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(0.45, 0.75, 1.0, a));
            let friend = cast_list[players[1].character].noun;
            ui_text("fishing on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let cx = x + 34.0;
            let cy = y + 62.0;
            let t = (time * 0.5).fract();
            let r = 6.0 + (1.0 - t) * 18.0;
            let ring_col = if t > 0.85 { Color::new(1.0, 0.86, 0.3, a) } else { Color::new(0.95, 0.97, 1.0, a) };
            for k in 0..20 {
                let ang = k as f32 / 20.0 * 2.0 * PI;
                ui_rect(cx + ang.cos() * r - 1.0, cy + ang.sin() * r * 0.6 - 1.0, 2.0, 2.0, ring_col);
            }
            ui_rect(cx - 3.0, cy - 5.0, 6.0, 4.0, Color::new(0.9, 0.24, 0.24, a));
            ui_rect(cx - 3.0, cy - 1.0, 6.0, 3.0, Color::new(0.98, 0.98, 0.95, a));
            if t > 0.85 {
                ui_text(if fishing.caster == 1 { "O!" } else { "E!" }, cx - 6.0, cy - 22.0, 12.0, Color::new(1.0, 0.86, 0.3, a));
            }
            let lines = [
                if two_player { "the river whispers:".to_string() } else { format!("the {} whispers:", friend) },
                "watch the ring around the bobber.".to_string(),
                format!("when it shrinks onto it, press {}.", if fishing.caster == 0 { "E" } else { "O" }),
                "three good ones and the fish is yours!".to_string(),
                if two_player { "player 2 can press O too - together is better".to_string() } else { format!("miss? you get 3 tries, and the {} helps", friend) },
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.75, 0.9, 0.7, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 78.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 68.0, y + 38.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if fishing.caster == 0 { "E: got it" } else { "O: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if cook_card > 0.0 {
            let a = (cook_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(1.0, 0.62, 0.34, a));
            ui_text("cooking on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let lines = [
                "sit by the fire, then press E or O again".to_string(),
                "to put the pot on.".to_string(),
                "tap the first four notes on the beat.".to_string(),
                "fish makes bigger dishes, and new dishes".to_string(),
                "earn more acorns.".to_string(),
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if two_player { "E / O: got it" } else { "E: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if gather_card > 0.0 {
            let a = (gather_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(0.5, 0.88, 0.52, a));
            ui_text("gathering around the woods", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let lines = [
                "face a tree or stone and press E or O.".to_string(),
                "sparkly trees give sticks to carry.".to_string(),
                "other trees shake loose acorns.".to_string(),
                "stones can be picked up too.".to_string(),
                "bring your finds back to the fire.".to_string(),
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if two_player { "E / O: got it" } else { "E: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if shop_open {
            let rows = home::shop_rows();
            let w = 330f32.min(low_w() - 10.0);
            let x = ((low_w() - w) / 2.0).round();
            let y = if low_h() > low_w() { (low_h() * 0.1).round() } else { ((low_h() - 214.0) / 2.0 - 4.0).round().max(18.0) };
            let h = 214.0f32.min(low_h() - y - 6.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.2, 0.94));
            let tab_col = |on: bool| if on { Color::from_rgba(255, 226, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.45) };
            ui_text("catalogue", x + 8.0, y + 16.0, 14.0, tab_col(shop_tab == 0));
            let diary_x = x + 14.0 + ui_measure("catalogue", 14.0);
            ui_text("diary", diary_x, y + 16.0, 14.0, tab_col(shop_tab == 1));
            let fish_x = diary_x + ui_measure("diary", 14.0) + 6.0;
            ui_text("fishdex", fish_x, y + 16.0, 11.0, tab_col(false));
            ui_text("mushrooms", fish_x + ui_measure("fishdex", 11.0) + 5.0, y + 16.0, 11.0, tab_col(false));
            ui_rect(if shop_tab == 0 { x + 8.0 } else { diary_x }, y + 19.0, ui_measure(if shop_tab == 0 { "catalogue" } else { "diary" }, 14.0), 1.0, Color::from_rgba(255, 226, 150, 255));
            let wallet = format!("{}", inv.acorns);
            ui_rect(x + w - 14.0 - ui_measure(&wallet, 11.0), y + 8.0, 6.0, 6.0, ACORN_COLOR);
            ui_text(&wallet, x + w - 6.0 - ui_measure(&wallet, 11.0), y + 15.0, 11.0, WHITE);
            if shop_tab == 0 {
                let row = home::row(shop_row);
                let (name, price, extra) = shop_row_info(row, &house);
                let preview_x = x + 10.0;
                let preview_y = y + 28.0;
                let preview_w = w - 20.0;
                let preview_h = (preview_w * 150.0 / 400.0).min(118.0).round();
                let (preview_kx, preview_ky) = ui_scale();
                draw_texture_ex(
                    &catalogue_view.texture,
                    preview_x * preview_kx,
                    preview_y * preview_ky,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(preview_w * preview_kx, preview_h * preview_ky)),
                        flip_y: true,
                        ..Default::default()
                    },
                );
                ui_rect(preview_x, preview_y, preview_w, preview_h, Color::new(1.0, 1.0, 1.0, 0.06));
                let left_key = if shop_owner == 1 { "J" } else { "A" };
                let right_key = if shop_owner == 1 { "L" } else { "D" };
                let index = format!("{} / {}", shop_row + 1, rows);
                ui_text(&format!("{} <", left_key), preview_x + 6.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text(&format!("> {}", right_key), preview_x + preview_w - 20.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text(&index, preview_x + preview_w - ui_measure(&index, 9.0) - 6.0, preview_y + 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
                let title_y = preview_y + preview_h + 16.0;
                ui_text(&name, x + (w - ui_measure(&name, 15.0)) / 2.0, title_y, 15.0, Color::from_rgba(255, 240, 200, 255));
                let price_col = price.split_whitespace().next().and_then(|v| v.parse::<u32>().ok()).map(|p| if p <= inv.acorns { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) }).unwrap_or(Color::new(1.0, 1.0, 1.0, 0.75));
                ui_text(&price, x + (w - ui_measure(&price, 11.0)) / 2.0, title_y + 13.0, 11.0, price_col);
                let about = if extra.is_empty() { "a little room upgrade for your musical home" } else { &extra };
                let lines = ui_wrap(about, 10.0, w - 26.0);
                for (n, line) in lines.iter().take(2).enumerate() {
                    ui_text(line, x + (w - ui_measure(line, 10.0)) / 2.0, title_y + 28.0 + n as f32 * 11.0, 10.0, Color::new(0.88, 0.94, 1.0, 0.82));
                }
            } else if diary.is_empty() {
                ui_text("no jams yet - play together by the fire", x + 10.0, y + 46.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text("30s = 1 star, 1 min = 2, 3 min = 3", x + 10.0, y + 62.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.5));
            } else {
                let shown = 8usize;
                let row_h = 15.0;
                let total: u32 = diary.iter().map(|e| e.stars as u32).sum();
                let n = diary.len();
                let first = diary_row.saturating_sub(shown / 2).min(n.saturating_sub(shown));
                for r in first..(first + shown).min(n) {
                    let entry = &diary[n - 1 - r];
                    let ry = y + 30.0 + (r - first) as f32 * row_h;
                    if r == diary_row {
                        ui_rect(x + 4.0, ry - 2.0, w - 8.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
                    }
                    let mins = format!("{}:{:02}", entry.seconds / 60, entry.seconds % 60);
                    let who = format!("{} + {}", cast_list[entry.who.0.min(cast_list.len() - 1)].noun, cast_list[entry.who.1.min(cast_list.len() - 1)].noun);
                    ui_text(&date_label(entry.when), x + 10.0, ry + 9.0, 11.0, WHITE);
                    ui_text(&who, x + 56.0, ry + 9.0, 10.0, Color::new(0.85, 0.92, 1.0, 0.85));
                    ui_text(&mins, x + w - 52.0 - ui_measure(&mins, 10.0), ry + 9.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
                    for k in 0..3u8 {
                        let col = if k < entry.stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.18) };
                        ui_star(x + w - 42.0 + k as f32 * 11.0, ry + 1.0, 8.0, col);
                    }
                }
                let line = format!("{} jams, {} stars", n, total);
                ui_text(&line, x + w - 8.0 - ui_measure(&line, 9.0), y + 26.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
            }
            let help = if shop_owner == 1 {
                if shop_tab == 0 { "I/K tabs - J/L browse - O buy - U close" } else { "I/K tabs - J/L choose - O play - U close" }
            } else if shop_tab == 0 {
                "W/S tabs - A/D browse - E buy - X close"
            } else {
                "W/S tabs - A/D choose - E play - X close"
            };
            ui_text(help, x + (w - ui_measure(help, 10.0)) / 2.0, y + h - 6.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
        }

        if !picking && !show_help && first_steps > 0 && first_steps < 4 {
            let (heading, message) = match first_steps {
                1 => ("your first song - 1/3", "press any note 1-6 to sing"),
                2 if camp.lit => ("find the fire - 2/3", "walk to the campfire and press E to sit"),
                2 => ("find the fire - 2/3", "explore while your friend brings firewood"),
                _ => ("make a memory - 3/3", "play music together to earn a star"),
            };
            let w = (low_w() - 16.0).min(250.0);
            let x = (low_w() - w) / 2.0;
            let y = if touch_ui && portrait { 27.0 } else { 21.0 };
            ui_rect(x, y, w, 30.0, Color::new(0.1, 0.08, 0.2, 0.83));
            ui_text(heading, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            let line = ui_wrap(message, 10.0, w - 16.0);
            if let Some(first) = line.first() {
                ui_text(first, x + 8.0, y + 24.0, 10.0, WHITE);
            }
        }

        if picking {
            let w = 230.0f32.min(low_w() - 10.0);
            let preview_h = (w - 12.0) * 130.0 / 300.0;
            let top = if first_pick { 24.0 } else { 30.0 };
            let x = (low_w() - w) / 2.0;
            let h = preview_h + if first_pick { 100.0 } else { 76.0 };
            ui_rect(x, top, w, h, Color::new(0.1, 0.08, 0.2, 0.94));
            let mut y = top + 16.0;
            if first_pick {
                let title = "melody quack";
                let tw = ui_measure(title, 22.0);
                ui_text(title, (low_w() - tw) / 2.0, y + 6.0, 22.0, Color::from_rgba(255, 226, 150, 255));
                y += 24.0;
            }
            {
                let (kx, ky) = ui_scale();
                let px = x + 6.0;
                let py = y - 8.0;
                draw_texture_ex(&pick_view.texture, px * kx, py * ky, WHITE, DrawTextureParams { dest_size: Some(vec2((w - 12.0) * kx, preview_h * ky)), flip_y: true, ..Default::default() });
                y += preview_h + 6.0;
            }
            let labels = if two_player { ["player 1", "player 2"] } else { ["you", "friend"] };
            for row in 0..2 {
                let chosen = row == pick_row;
                let col = if chosen { Color::from_rgba(255, 236, 170, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) };
                if chosen {
                    ui_rect(x + 6.0, y - 9.0, w - 12.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.1));
                }
                ui_text(labels[row], x + 12.0, y, 12.0, col);
                let name = format!("<  {}  >", cast_list[players[row].character].name);
                ui_text(&name, x + 80.0, y, 12.0, col);
                y += 14.0;
            }
            ui_text("W/S row   A/D change   E start", x + 12.0, y + 6.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }

        if sleep_fade > 0.0 {
            sleep_fade -= get_frame_time();
            let a = (1.0 - (sleep_fade - 1.5).abs() / 1.5).clamp(0.0, 1.0);
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.03, 0.1, a * 0.92));
            let z = if (time * 2.0) as i32 % 2 == 0 { "z z z" } else { "z Z z" };
            ui_text(z, low_w() / 2.0 - ui_measure(z, 16.0) / 2.0, low_h() / 2.0, 16.0, Color::new(1.0, 1.0, 1.0, a));
            if a > 0.6 {
                let line = "sweet dreams - game saved";
                ui_text(line, low_w() / 2.0 - ui_measure(line, 11.0) / 2.0, low_h() / 2.0 + 18.0, 11.0, Color::new(1.0, 1.0, 1.0, a * 0.8));
            }
        }
        if sampler.open {
            let w = (low_w() - 16.0).min(330.0);
            let x = ((low_w() - w) / 2.0).round();
            let cell = ((w - 60.0) / 16.0).floor();
            let h = 34.0 + cell * 4.0 + 34.0;
            let y = low_h() - h - 44.0;
            ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.94));
            ui_text("sampler", x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            let bpm = format!("{:.0} bpm", looper.bpm);
            ui_text(&bpm, x + w - 8.0 - ui_measure(&bpm, 10.0), y + 13.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
            let gx = x + 52.0;
            let gy = y + 22.0;
            for row in 0..4 {
                let col = note_color(row);
                let label = SAMPLER_ROWS[row];
                ui_text(label, x + 8.0, gy + row as f32 * cell + cell * 0.7, 9.0, col);
                if sampler.pads[row].is_some() {
                    ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 120, 150, 255));
                    ui_text("mic", x + 35.0, gy + row as f32 * cell + cell * 0.68, 6.0, WHITE);
                }
                if matches!(sampler.recording, Some((r, _)) if r == row) && (time * 4.0).fract() < 0.6 {
                    ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 60, 70, 255));
                }
                for k in 0..16 {
                    let cx = gx + k as f32 * cell;
                    let cy = gy + row as f32 * cell;
                    let on = sampler.grid[row][k];
                    let beat = k % 4 == 0;
                    let base = if on { col } else if beat { Color::new(1.0, 1.0, 1.0, 0.16) } else { Color::new(1.0, 1.0, 1.0, 0.08) };
                    ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, base);
                    if k == sampler.step {
                        ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, Color::new(1.0, 1.0, 1.0, if on { 0.45 } else { 0.12 }));
                    }
                    if (k, row) == sampler.cursor {
                        let c = Color::from_rgba(255, 226, 150, 255);
                        ui_rect(cx, cy, cell, 1.0, c);
                        ui_rect(cx, cy + cell - 1.0, cell, 1.0, c);
                        ui_rect(cx, cy, 1.0, cell, c);
                        ui_rect(cx + cell - 1.0, cy, 1.0, cell, c);
                    }
                }
            }
            let hint = if sampler.recording.is_some() {
                "recording - make a sound into your mic"
            } else if sampler.owner == 0 {
                "arrows/WASD move  E step on/off  Space record your sound  X close"
            } else {
                "IJKL move  O step on/off  Space record your sound  U close"
            };
            ui_text(hint, x + (w - ui_measure(hint, 9.0)) / 2.0, y + h - 18.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.8));
            let note = "record only your own voice or sounds - recordings stay on this device and are left out of tapes";
            let mut shown = note.to_string();
            while ui_measure(&shown, 7.0) > w - 12.0 && shown.len() > 10 {
                shown.pop();
            }
            ui_text(&shown, x + (w - ui_measure(&shown, 7.0)) / 2.0, y + h - 6.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.5));
        }
        if ddr.on[0] || ddr.on[1] {
            let active: Vec<usize> = (0..2).filter(|w| ddr.on[*w]).collect();
            for (slot, &who) in active.iter().enumerate() {
                let cx = if active.len() == 2 { low_w() * (0.17 + slot as f32 * 0.66) } else { low_w() * 0.78 };
                let lane_w = 22.0;
                let x0 = cx - lane_w * 2.0;
                let top = 34.0;
                ui_rect(x0 - 4.0, top - 8.0, lane_w * 4.0 + 8.0, low_h() - top - 36.0, Color::new(0.04, 0.03, 0.1, 0.55));
                for lane in 0..4 {
                    let lx = x0 + lane as f32 * lane_w + 2.0;
                    let f = ddr.flash[who][lane];
                    let ring: Vec<(usize, usize, Color)> = ddr_art[lane].iter().map(|(x, y, _)| (*x, *y, Color::new(0.7 + 0.3 * f, 0.7 + 0.3 * f, 0.75 + 0.25 * f, 0.35 + 0.65 * f))).collect();
                    draw_pixel_grid(&ring, lx, top, 2.0, |c| c);
                }
                for a in ddr.arrows.iter().filter(|a| !a.hit[who]) {
                    let y = top + (a.at - time) * 75.0;
                    if y < top - 20.0 || y > low_h() - 40.0 {
                        continue;
                    }
                    let fade = if a.missed[who] { 0.35 } else { 1.0 };
                    draw_pixel_grid(&ddr_art[a.lane], x0 + a.lane as f32 * lane_w + 2.0, y, 2.0, |c| Color::new(c.r, c.g, c.b, fade));
                }
                let score = format!("{}", ddr.score[who]);
                ui_text(&score, cx - ui_measure(&score, 11.0) / 2.0, top - 12.0, 11.0, Color::from_rgba(255, 226, 150, 255));
                if let Some((word, col, left)) = ddr.judge[who] {
                    let size = 13.0 + 3.0 * (left - 0.4).max(0.0) * 5.0;
                    ui_text(word, cx - ui_measure(word, size) / 2.0, top + 38.0, size, Color::new(col.r, col.g, col.b, (left / 0.3).min(1.0)));
                }
                if ddr.combo[who] >= 4 {
                    let combo = format!("{} combo", ddr.combo[who]);
                    ui_text(&combo, cx - ui_measure(&combo, 10.0) / 2.0, top + 52.0, 10.0, WHITE);
                }
            }
            let source = if time - ddr.last_mic < 3.0 { "arrows from the music on your mic" } else if microphone.is_some() { "listening... arrows follow the game beat for now" } else { "no mic - arrows follow the game beat and your loop" };
            ui_text(source, (low_w() - ui_measure(source, 9.0)) / 2.0, low_h() - 30.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        if show_help {
            let pages: [(&str, &[(&str, &str)]); 3] = [
                ("basics", &[
                    ("WASD", "move  (player 2: IJKL)"),
                    ("E", "act  (player 2: O)"),
                    ("M", "bag, quests, furniture  (player 2: Backspace)"),
                    ("Z", "emoji  (player 2: .)"),
                    ("P", "1 or 2 players"),
                    ("V", "choose characters"),
                    ("Esc", "pause, mixer, fishdex"),
                    ("H / Esc", "close this help"),
                ]),
                ("music", &[
                    ("1 - 6", "sing  (player 2: 7 8 9 0 - =)"),
                    ("Tab", "spells / music"),
                    ("R / C", "record / clear  (player 2: Y)"),
                    ("Q", "quantise"),
                    ("B", "beat on / off"),
                    ("F", "follow the mic tempo"),
                    ("G", "echo on / off"),
                    ("T", "tape: record a .wav"),
                    ("shift+C", "friend forgets all tunes"),
                ]),
                ("world", &[
                    ("E at fire", "sit, again: cook"),
                    ("E at dock", "cast and fish"),
                    ("E by trees", "stick, mushroom, acorns"),
                    ("E in yard", "build walls and floors"),
                    ("X", "change build tool"),
                    ("E at door", "go inside"),
                    ("E / X inside", "use / move furniture"),
                    ("N twice", "fresh world"),
                ]),
            ];
            let (x, y, w, h) = help_box();
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.03, 0.02, 0.08, 0.55));
            ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.97));
            ui_rect(x, y, w, 3.0, Color::from_rgba(255, 214, 110, 255));
            ui_text("help", x + 10.0, y + 15.0, 13.0, Color::from_rgba(255, 226, 150, 255));
            let tab_w = (w - 20.0) / 3.0;
            for (k, (name, _)) in pages.iter().enumerate() {
                let tx = x + 10.0 + k as f32 * tab_w;
                let on = k == help_page;
                ui_rect(tx + 1.0, y + 20.0, tab_w - 2.0, 16.0, if on { Color::from_rgba(255, 214, 110, 255) } else { Color::new(1.0, 1.0, 1.0, 0.1) });
                ui_text(name, tx + (tab_w - ui_measure(name, 11.0)) / 2.0, y + 32.0, 11.0, if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE });
            }
            let rows = pages[help_page].1;
            let key_w = rows.iter().map(|(k, _)| ui_measure(k, 11.0)).fold(0.0, f32::max) + 10.0;
            for (n, (key, text)) in rows.iter().enumerate() {
                let ry = y + 44.0 + n as f32 * 16.0;
                ui_rect(x + 10.0, ry, ui_measure(key, 11.0) + 8.0, 13.0, Color::new(1.0, 0.84, 0.43, 0.2));
                ui_text(key, x + 14.0, ry + 10.0, 11.0, Color::from_rgba(255, 226, 150, 255));
                let mut t = text.to_string();
                while ui_measure(&t, 11.0) > w - key_w - 24.0 && t.len() > 4 {
                    t.pop();
                }
                ui_text(&t, x + 16.0 + key_w, ry + 10.0, 11.0, WHITE);
            }
            let foot = if radio::touch_ui() { "tap a tab - tap outside to close" } else { "left/right: page   H or Esc: close" };
            ui_text(foot, x + (w - ui_measure(foot, 9.0)) / 2.0, y + h - 6.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.6));
        }

        next_frame().await
    }
}
