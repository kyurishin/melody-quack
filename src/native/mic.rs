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
