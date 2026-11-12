#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn echo_and_quantise(g: &mut Game, _fr: &mut Frame) {
    if is_key_pressed(KeyCode::G) {
        g.ringing = !g.ringing;
        g.audio.set_ring(g.ringing);
        g.ring_flash = 1.5;
    }
    g.audio.set_echo_gap(g.looper.step_seconds() * 3.0);
    g.audio.tick();
    g.ring_flash = (g.ring_flash - get_frame_time()).max(0.0);
    if is_key_pressed(KeyCode::Q) {
        g.looper.quantise = g.looper.quantise.next();
    }
}

pub(crate) fn tool_keys(g: &mut Game, _fr: &mut Frame) {
    if is_key_pressed(KeyCode::X) && !g.picking && !g.inside {
        g.tools[0] = g.tools[0].next();
    }
    if is_key_pressed(KeyCode::U) && g.two_player && !g.inside {
        g.tools[1] = g.tools[1].next();
    }
}

pub(crate) fn listening(g: &mut Game, fr: &mut Frame) {
    if is_key_pressed(KeyCode::F) {
        if g.listening {
            g.listening = false;
            g.looper.bpm = (g.looper.bpm * 10.0).round() / 10.0;
        } else {
            if g.microphone.is_none() {
                g.microphone = mic::start();
            }
            if g.microphone.is_some() {
                g.listening = true;
                g.locked = false;
                g.bpm_history.clear();
                g.mic_message = None;
            } else {
                g.mic_message = Some("no microphone");
            }
        }
    }

    if g.listening || g.sampler.recording.is_some() {
        if let Some(m) = g.microphone.as_mut() {
            m.pump();
            if m.blocked() {
                g.listening = false;
                g.locked = false;
                g.microphone = None;
                g.mic_message = Some("microphone blocked");
            }
        }
    }
    if g.listening {
        g.listen_clock += fr.dt;
        if g.listen_clock >= 0.5 {
            g.listen_clock = 0.0;
            if let Some(m) = &g.microphone {
                let (onsets, fps, level) = match m.detector.lock() {
                    Ok(d) => (d.onsets.iter().copied().collect::<Vec<f32>>(), d.fps, d.level),
                    Err(_) => (Vec::new(), 100.0, 0.0),
                };
                match tempo::estimate(&onsets, fps) {
                    Some(t) if t.confidence > 2.2 && level > 0.002 => {
                        g.bpm_history.push(t.bpm);
                        if g.bpm_history.len() > 6 {
                            g.bpm_history.remove(0);
                        }
                        let mut sorted = g.bpm_history.clone();
                        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        g.looper.bpm = sorted[sorted.len() / 2];
                        g.looper.sync_to_beat(t.since_beat);
                        g.locked = true;
                    }
                    _ => g.locked = false,
                }
            }
        }
    }
}

pub(crate) fn visual_bands(g: &mut Game, _fr: &mut Frame) {
    {
        let mic_bands = if g.listening { g.microphone.as_ref().and_then(|m| m.detector.lock().ok().map(|d| d.bands)) } else { None };
        let beat_frac = (g.looper.position / 4.0).fract();
        let pulse = if g.looper.beat { (1.0 - beat_frac).powi(3) } else { 0.0 };
        let glow: f32 = g.players.iter().map(|p| p.key_glow.iter().copied().fold(0.0, f32::max)).fold(0.0, f32::max);
        let bands = mic_bands.unwrap_or([pulse, pulse * 0.4 + glow * 0.6, glow * 0.7, if g.looper.beat { ((g.looper.position * 0.5).fract() < 0.3) as u8 as f32 * 0.6 } else { glow * 0.4 }]);
        let mut pads = [0.0f32; 4];
        for lane in 0..4 {
            pads[lane] = g.ddr.flash[0][lane].max(g.ddr.flash[1][lane]);
        }
        VIS.with(|v| v.set(Vis { bands, pulse: if mic_bands.is_some() { bands[0] } else { pulse }, pads, seq: g.sampler.step }));
    }
}
