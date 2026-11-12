use crate::SAMPLE_RATE;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn audio_init();
    fn audio_add_buffer(content: *const u8, content_len: u32) -> u32;
    fn audio_play_buffer(buffer: u32, volume: f32, repeat: bool) -> u32;
    fn audio_source_is_loaded(buffer: u32) -> bool;
}

#[unsafe(no_mangle)]
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
