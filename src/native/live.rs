use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

struct Cast {
    _stream: cpal::Stream,
    level: Arc<AtomicU32>,
    alive: Arc<AtomicBool>,
}

thread_local! {
    static CAST: RefCell<Option<Cast>> = const { RefCell::new(None) };
}

static LISTEN_GEN: AtomicU32 = AtomicU32::new(0);

pub fn inputs() -> Vec<String> {
    let host = cpal::default_host();
    let mut names: Vec<String> = host.input_devices().map(|d| d.map(|x| x.to_string()).collect()).unwrap_or_default();
    names.dedup();
    names
}

pub fn casting() -> bool {
    CAST.with(|c| c.borrow().is_some())
}

pub fn level() -> Option<f32> {
    CAST.with(|c| c.borrow().as_ref().map(|c| f32::from_bits(c.level.load(Ordering::Relaxed))))
}

pub fn stop() {
    CAST.with(|c| {
        if let Some(cast) = c.borrow_mut().take() {
            cast.alive.store(false, Ordering::Relaxed);
        }
    });
}

struct Resampler {
    step: f64,
    pos: f64,
    last: [f32; 2],
}

impl Resampler {
    fn push(&mut self, frame: [f32; 2], out: &mut Vec<i16>) {
        let pcm = |v: f32| (v.clamp(-1.0, 1.0) * 32767.0) as i16;
        if self.step == 1.0 {
            out.extend([pcm(frame[0]), pcm(frame[1])]);
            return;
        }
        while self.pos < 1.0 {
            let t = self.pos as f32;
            for k in 0..2 {
                out.push(pcm(self.last[k] + (frame[k] - self.last[k]) * t));
            }
            self.pos += self.step;
        }
        self.pos -= 1.0;
        self.last = frame;
    }
}

pub fn start(device_name: &str, send: impl Fn(Vec<u8>) + Send + 'static) -> Result<(), String> {
    stop();
    let host = cpal::default_host();
    let device = host
        .input_devices()
        .ok()
        .and_then(|mut d| d.find(|x| x.to_string() == device_name))
        .or_else(|| if device_name == "default input" { host.default_input_device() } else { None })
        .ok_or(format!("{device_name} isn't there right now. is it switched on?"))?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = (config.channels() as usize).max(1);
    let in_rate = config.sample_rate();
    let rate = if in_rate == 44100 || in_rate == 48000 || in_rate == 32000 { in_rate } else { 44100 };
    let (tx, rx) = mpsc::channel::<Vec<i16>>();
    let level = Arc::new(AtomicU32::new(0));
    let alive = Arc::new(AtomicBool::new(true));
    let meter = level.clone();
    let step = in_rate as f64 / rate as f64;
    let resample = std::sync::Mutex::new(Resampler { step, pos: 0.0, last: [0.0; 2] });
    let push = Arc::new(move |data: &[f32], channels: usize| {
        let mut out = Vec::new();
        let mut peak = 0.0f32;
        let mut state = resample.lock().unwrap();
        for c in data.chunks(channels) {
            let frame = [c[0], if c.len() >= 2 { c[1] } else { c[0] }];
            peak = peak.max(frame[0].abs()).max(frame[1].abs());
            state.push(frame, &mut out);
        }
        let old = f32::from_bits(meter.load(Ordering::Relaxed));
        meter.store((peak.max(old * 0.9)).to_bits(), Ordering::Relaxed);
        let _ = tx.send(out);
    });
    let format = config.sample_format();
    let config: cpal::StreamConfig = config.into();
    macro_rules! input {
        ($t:ty) => {{
            let push = push.clone();
            device.build_input_stream(
                config.clone(),
                move |data: &[$t], _| {
                    let f: Vec<f32> = data.iter().map(|s| <f32 as cpal::FromSample<$t>>::from_sample_(*s)).collect();
                    push(&f, channels)
                },
                |e| eprintln!("live input error: {e}"),
                None,
            )
        }};
    }
    let stream = match format {
        cpal::SampleFormat::F32 => input!(f32),
        cpal::SampleFormat::F64 => input!(f64),
        cpal::SampleFormat::I8 => input!(i8),
        cpal::SampleFormat::I16 => input!(i16),
        cpal::SampleFormat::I32 => input!(i32),
        cpal::SampleFormat::I64 => input!(i64),
        cpal::SampleFormat::U8 => input!(u8),
        cpal::SampleFormat::U16 => input!(u16),
        cpal::SampleFormat::U32 => input!(u32),
        cpal::SampleFormat::U64 => input!(u64),
        other => return Err(format!("this input sends {other} audio, which isn't supported yet")),
    }
    .map_err(|e| e.to_string())?;
    let mut builder = mp3lame_encoder::Builder::new().ok_or("the mp3 encoder couldn't start")?;
    builder.set_num_channels(2).map_err(|e| format!("{e:?}"))?;
    builder.set_sample_rate(rate).map_err(|e| format!("{e:?}"))?;
    builder.set_brate(mp3lame_encoder::Bitrate::Kbps128).map_err(|e| format!("{e:?}"))?;
    builder.set_quality(mp3lame_encoder::Quality::Good).map_err(|e| format!("{e:?}"))?;
    let mut encoder = builder.build().map_err(|e| format!("{e:?}"))?;
    let running = alive.clone();
    std::thread::spawn(move || {
        let mut out: Vec<u8> = Vec::new();
        while running.load(Ordering::Relaxed) {
            let Ok(chunk) = rx.recv_timeout(Duration::from_millis(500)) else {
                continue;
            };
            out.reserve(mp3lame_encoder::max_required_buffer_size(chunk.len() / 2));
            let _ = encoder.encode_to_vec(mp3lame_encoder::InterleavedPcm(&chunk), &mut out);
            if out.len() >= 3000 {
                send(std::mem::take(&mut out));
            }
        }
    });
    stream.play().map_err(|e| e.to_string())?;
    CAST.with(|c| *c.borrow_mut() = Some(Cast { _stream: stream, level, alive }));
    Ok(())
}

pub fn listen(url: &str) {
    let gen = LISTEN_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    crate::audio::stream_clear();
    if url.is_empty() {
        return;
    }
    let url = url.to_string();
    std::thread::spawn(move || {
        while LISTEN_GEN.load(Ordering::SeqCst) == gen {
            if play_stream(&url, gen).is_err() {
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    });
}

fn play_stream(url: &str, gen: u32) -> Result<(), String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    let agent = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(8)).timeout_read(Duration::from_secs(15)).build();
    let reader = agent.get(url).call().map_err(|e| e.to_string())?.into_reader();
    let source = MediaSourceStream::new(Box::new(ReadOnlySource::new(reader)), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let probed = symphonia::default::get_probe().format(&hint, source, &FormatOptions::default(), &MetadataOptions::default()).map_err(|e| e.to_string())?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("no audio in the stream")?.clone();
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default()).map_err(|e| e.to_string())?;
    let mut buf: Option<SampleBuffer<f32>> = None;
    while LISTEN_GEN.load(Ordering::SeqCst) == gen {
        let packet = format.next_packet().map_err(|e| e.to_string())?;
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.to_string()),
        };
        let spec = *decoded.spec();
        let b = buf.get_or_insert_with(|| SampleBuffer::new(decoded.capacity() as u64, spec));
        if b.capacity() < decoded.capacity() * spec.channels.count() {
            *b = SampleBuffer::new(decoded.capacity() as u64, spec);
        }
        b.copy_interleaved_ref(decoded);
        crate::audio::stream_push(b.samples(), spec.channels.count(), spec.rate as f32);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn a_96k_input_comes_out_at_44k_with_the_same_pitch() {
        let mut r = Resampler { step: 96000.0 / 44100.0, pos: 0.0, last: [0.0; 2] };
        let mut out = Vec::new();
        for i in 0..96000 {
            let v = ((i as f32 / 96000.0) * 440.0 * std::f32::consts::TAU).sin() * 0.5;
            r.push([v, v], &mut out);
        }
        let frames = out.len() / 2;
        assert!((44090..=44110).contains(&frames), "{frames} frames");
        let left: Vec<i16> = out.iter().step_by(2).copied().collect();
        let crossings = left.windows(2).filter(|w| w[0] < 0 && w[1] >= 0).count();
        assert!((438..=442).contains(&crossings), "{crossings}");
    }

    #[test]
    fn a_tone_survives_encoding_streaming_and_decoding() {
        let rate = 44100u32;
        let mut builder = mp3lame_encoder::Builder::new().unwrap();
        builder.set_num_channels(2).unwrap();
        builder.set_sample_rate(rate).unwrap();
        builder.set_brate(mp3lame_encoder::Bitrate::Kbps128).unwrap();
        builder.set_quality(mp3lame_encoder::Quality::Good).unwrap();
        let mut enc = builder.build().unwrap();
        let mut pcm = Vec::new();
        for i in 0..(rate * 4) {
            let v = ((i as f32 / rate as f32) * 440.0 * std::f32::consts::TAU).sin() * 0.5;
            pcm.push((v * 32767.0) as i16);
            pcm.push((v * 32767.0) as i16);
        }
        let mut mp3 = Vec::new();
        mp3.reserve(mp3lame_encoder::max_required_buffer_size(pcm.len() / 2));
        enc.encode_to_vec(mp3lame_encoder::InterleavedPcm(&pcm), &mut mp3).unwrap();
        assert!(mp3.len() > 30_000);
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = server.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut s, _) = server.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = std::io::Read::read(&mut s, &mut buf);
            s.write_all(b"HTTP/1.1 200 OK\r\ncontent-type: audio/mpeg\r\ntransfer-encoding: chunked\r\n\r\n").unwrap();
            for c in mp3.chunks(4000) {
                s.write_all(format!("{:x}\r\n", c.len()).as_bytes()).unwrap();
                s.write_all(c).unwrap();
                s.write_all(b"\r\n").unwrap();
            }
            s.write_all(b"0\r\n\r\n").unwrap();
        });
        listen(&format!("http://127.0.0.1:{port}/live/TEST.mp3"));
        let mut got: Vec<[f32; 2]> = Vec::new();
        let until = std::time::Instant::now() + Duration::from_secs(10);
        while got.len() < 88_200 && std::time::Instant::now() < until {
            got.extend(crate::audio::stream_take(100_000));
            std::thread::sleep(Duration::from_millis(50));
        }
        listen("");
        assert!(got.len() >= 88_200, "only {} frames decoded", got.len());
        let part = &got[22_050..66_150];
        let crossings = part.windows(2).filter(|w| w[0][0] < 0.0 && w[1][0] >= 0.0).count();
        assert!((430..=450).contains(&crossings), "{crossings} rising zero crossings in one second");
        let peak = part.iter().map(|f| f[0].abs()).fold(0.0f32, f32::max);
        assert!(peak > 0.4 && peak < 0.6, "peak {peak}");
    }
}
