use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

struct Voice {
    samples: Arc<Vec<f32>>,
    position: f32,
    volume: f32,
}

struct StreamBuf {
    data: std::collections::VecDeque<[f32; 2]>,
    rate: f32,
    pos: f32,
    playing: bool,
    volume: f32,
}

static STREAM: std::sync::OnceLock<Mutex<StreamBuf>> = std::sync::OnceLock::new();

fn stream_buf() -> &'static Mutex<StreamBuf> {
    STREAM.get_or_init(|| Mutex::new(StreamBuf { data: std::collections::VecDeque::new(), rate: 44100.0, pos: 0.0, playing: false, volume: 1.0 }))
}

pub fn stream_push(samples: &[f32], channels: usize, rate: f32) {
    if channels == 0 {
        return;
    }
    if let Ok(mut s) = stream_buf().lock() {
        s.rate = rate;
        for c in samples.chunks(channels) {
            let l = c[0];
            let r = if channels > 1 { c[1] } else { c[0] };
            s.data.push_back([l, r]);
        }
        let most = (rate * 6.0) as usize;
        while s.data.len() > most {
            s.data.pop_front();
        }
    }
}

#[cfg(test)]
pub fn stream_take(n: usize) -> Vec<[f32; 2]> {
    stream_buf().lock().map(|mut s| {
        let k = n.min(s.data.len());
        s.data.drain(..k).collect()
    }).unwrap_or_default()
}

pub fn stream_clear() {
    if let Ok(mut s) = stream_buf().lock() {
        s.data.clear();
        s.playing = false;
        s.pos = 0.0;
    }
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
        if let Ok(mut s) = stream_buf().lock() {
            s.volume = mix[4];
        }
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
    let device_rate = config.sample_rate() as f32;
    let step = crate::SAMPLE_RATE as f32 / device_rate;
    let small = match config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => Some(256u32.clamp(*min, *max)),
        cpal::SupportedBufferSize::Unknown => None,
    };
    let mut stream_config: cpal::StreamConfig = config.into();
    let fallback = stream_config.clone();
    if let Some(frames) = small {
        stream_config.buffer_size = cpal::BufferSize::Fixed(frames);
    }
    let voices_again = voices.clone();
    let stream = build(&device, stream_config, channels, step, device_rate, voices).or_else(|| build(&device, fallback, channels, step, device_rate, voices_again))?;
    stream.play().ok()?;
    Some(stream)
}

fn build(device: &cpal::Device, config: cpal::StreamConfig, channels: usize, step: f32, device_rate: f32, voices: Arc<Mutex<Vec<Voice>>>) -> Option<cpal::Stream> {
    device
        .build_output_stream(
            config,
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
                if let Ok(mut st) = stream_buf().lock() {
                    if !st.playing && st.data.len() as f32 > st.rate * 1.2 {
                        st.playing = true;
                    }
                    if st.playing {
                        let advance = st.rate / device_rate;
                        let vol = st.volume * 0.8;
                        for frame in data.chunks_mut(channels) {
                            if st.data.len() < 2 {
                                st.playing = false;
                                break;
                            }
                            let f = st.pos;
                            let (a, b) = (st.data[0], st.data[1]);
                            let l = (a[0] * (1.0 - f) + b[0] * f) * vol;
                            let r = (a[1] * (1.0 - f) + b[1] * f) * vol;
                            if channels >= 2 {
                                frame[0] += l;
                                frame[1] += r;
                            } else {
                                frame[0] += (l + r) * 0.5;
                            }
                            st.pos += advance;
                            while st.pos >= 1.0 && !st.data.is_empty() {
                                st.data.pop_front();
                                st.pos -= 1.0;
                            }
                        }
                    }
                }
                for s in data.iter_mut() {
                    *s = s.tanh();
                }
            },
            |e| eprintln!("audio error: {e}"),
            None,
        )
        .ok()
}
