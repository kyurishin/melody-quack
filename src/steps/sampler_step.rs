#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// The step sampler and recording your own sounds.
pub(crate) fn sampler(g: &mut Game, fr: &mut Frame) {
    if g.sampler.open && !g.inside {
        g.sampler.open = false;
    }
    if g.sampler.open {
        let o = g.sampler.owner;
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
        let (mut cx, mut cy) = g.sampler.cursor;
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
        g.sampler.cursor = (cx, cy);
        if act {
            g.sampler.grid[cy][cx] = !g.sampler.grid[cy][cx];
            g.save_clock = 99.0;
        }
        if close {
            g.sampler.open = false;
        }
        if is_key_pressed(KeyCode::Space) && g.sampler.recording.is_none() {
            if g.sampler.pads[cy].is_some() {
                g.sampler.pads[cy] = None;
                g.toasts.push((format!("the {} row is back to the built-in sound", SAMPLER_ROWS[cy]), 2.0));
            } else {
                if g.microphone.is_none() {
                    g.microphone = mic::start();
                }
                if let Some(m) = &g.microphone {
                    if let Ok(mut det) = m.detector.lock() {
                        det.capture_len = (det.rate * 1.5) as usize;
                        det.capture = Some(Vec::new());
                    }
                    g.sampler.recording = Some((cy, 0.0));
                } else {
                    g.toasts.push(("no microphone - the built-in sounds still work".to_string(), 2.5));
                }
            }
        }
    }
    if let Some((row, waited)) = g.sampler.recording {
        let waited = waited + fr.dt;
        let mut finished: Option<Option<Vec<f32>>> = None;
        if let Some(m) = &g.microphone {
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
        g.sampler.recording = Some((row, waited));
        if let Some(result) = finished {
            g.sampler.recording = None;
            match result {
                Some(samples) => {
                    g.sampler.pads[row] = Some(g.audio.load(samples));
                    g.toasts.push((format!("the {} row now plays your own sound", SAMPLER_ROWS[row]), 2.5));
                }
                None => g.toasts.push(("didn't hear anything - try again a bit louder".to_string(), 2.5)),
            }
        }
    }
}
