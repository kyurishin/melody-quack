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

pub fn start(device_name: &str, send: impl Fn(Vec<u8>) + Send + 'static) -> Result<(), String> {
    stop();
    let host = cpal::default_host();
    let device = host
        .input_devices()
        .ok()
        .and_then(|mut d| d.find(|x| x.to_string() == device_name))
        .or_else(|| host.default_input_device())
        .ok_or("no audio input found")?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = config.channels() as usize;
    let rate = config.sample_rate();
    let (tx, rx) = mpsc::channel::<Vec<i16>>();
    let level = Arc::new(AtomicU32::new(0));
    let alive = Arc::new(AtomicBool::new(true));
    let meter = level.clone();
    let push = move |frames: &mut dyn Iterator<Item = (f32, f32)>| {
        let mut out = Vec::new();
        let mut peak = 0.0f32;
        for (l, r) in frames {
            peak = peak.max(l.abs()).max(r.abs());
            out.push((l.clamp(-1.0, 1.0) * 32767.0) as i16);
            out.push((r.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
        let old = f32::from_bits(meter.load(Ordering::Relaxed));
        meter.store((peak.max(old * 0.9)).to_bits(), Ordering::Relaxed);
        let _ = tx.send(out);
    };
    let pair = move |c: &[f32]| if c.len() >= 2 { (c[0], c[1]) } else { (c[0], c[0]) };
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                config.into(),
                move |data: &[f32], _| push(&mut data.chunks(channels).map(pair)),
                |e| eprintln!("live input error: {e}"),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            device.build_input_stream(
                config.into(),
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|s| *s as f32 / 32768.0).collect();
                    push(&mut f.chunks(channels).map(pair))
                },
                |e| eprintln!("live input error: {e}"),
                None,
            )
        }
        _ => return Err("this input's sample format isn't supported".to_string()),
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
