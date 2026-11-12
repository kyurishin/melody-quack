use std::collections::VecDeque;
use std::f32::consts::PI;

pub struct OnsetDetector {
    hop: usize,
    coef: f32,
    lp1: f32,
    lp2: f32,
    low_energy: f32,
    full_energy: f32,
    count: usize,
    prev_low: f32,
    prev_full: f32,
    max_len: usize,
    pub onsets: VecDeque<f32>,
    pub fps: f32,
    pub level: f32,
    band_coef: [f32; 3],
    band_lp: [f32; 3],
    band_energy: [f32; 4],
    band_prev: [f32; 4],
    band_mean: [f32; 4],
    band_wait: [u32; 4],
    pub bands: [f32; 4],
    pub hits: VecDeque<(usize, f32)>,
    pub capture: Option<Vec<f32>>,
    pub capture_len: usize,
    pub rate: f32,
}

impl OnsetDetector {
    pub fn new(sample_rate: f32) -> OnsetDetector {
        let hop = (sample_rate / 100.0).round() as usize;
        let fps = sample_rate / hop as f32;
        OnsetDetector {
            hop,
            coef: 1.0 - (-2.0 * PI * 150.0 / sample_rate).exp(),
            lp1: 0.0,
            lp2: 0.0,
            low_energy: 0.0,
            full_energy: 0.0,
            count: 0,
            prev_low: 0.0,
            prev_full: 0.0,
            max_len: (fps * 10.0) as usize,
            onsets: VecDeque::new(),
            fps,
            level: 0.0,
            band_coef: [150.0, 600.0, 3000.0].map(|f: f32| 1.0 - (-2.0 * PI * f / sample_rate).exp()),
            band_lp: [0.0; 3],
            band_energy: [0.0; 4],
            band_prev: [0.0; 4],
            band_mean: [0.05; 4],
            band_wait: [0; 4],
            bands: [0.0; 4],
            hits: VecDeque::new(),
            capture: None,
            capture_len: 0,
            rate: sample_rate,
        }
    }

    pub fn push(&mut self, sample: f32) {
        if let Some(c) = self.capture.as_mut() {
            if c.len() < self.capture_len {
                c.push(sample);
            }
        }
        self.lp1 += (sample - self.lp1) * self.coef;
        self.lp2 += (self.lp1 - self.lp2) * self.coef;
        self.low_energy += self.lp2 * self.lp2;
        self.full_energy += sample * sample;
        self.count += 1;
        for k in 0..3 {
            self.band_lp[k] += (sample - self.band_lp[k]) * self.band_coef[k];
        }
        let split = [self.band_lp[0], self.band_lp[1] - self.band_lp[0], self.band_lp[2] - self.band_lp[1], sample - self.band_lp[2]];
        for k in 0..4 {
            self.band_energy[k] += split[k] * split[k];
        }

        if self.count == self.hop {
            let low = (self.low_energy / self.hop as f32 + 1e-9).ln();
            let full = (self.full_energy / self.hop as f32 + 1e-9).ln();
            let onset = (low - self.prev_low).max(0.0) + 0.3 * (full - self.prev_full).max(0.0);
            self.prev_low = low;
            self.prev_full = full;
            self.level = (self.full_energy / self.hop as f32).sqrt().max(self.level * 0.995);
            self.onsets.push_back(onset);
            while self.onsets.len() > self.max_len {
                self.onsets.pop_front();
            }
            for k in 0..4 {
                let e = (self.band_energy[k] / self.hop as f32 + 1e-9).ln();
                let rise = (e - self.band_prev[k]).max(0.0);
                self.band_prev[k] = e;
                self.band_mean[k] += (rise - self.band_mean[k]) * 0.02;
                let rms = (self.band_energy[k] / self.hop as f32).sqrt();
                self.bands[k] = (rms * [6.0, 10.0, 18.0, 30.0][k]).min(1.0).max(self.bands[k] * 0.9);
                self.band_energy[k] = 0.0;
                if self.band_wait[k] > 0 {
                    self.band_wait[k] -= 1;
                } else if rise > self.band_mean[k] * 3.0 + 0.6 && rms > 0.004 {
                    self.hits.push_back((k, rise));
                    self.band_wait[k] = (self.fps * [0.22, 0.18, 0.16, 0.14][k]) as u32;
                    while self.hits.len() > 32 {
                        self.hits.pop_front();
                    }
                }
            }
            self.low_energy = 0.0;
            self.full_energy = 0.0;
            self.count = 0;
        }
    }
}

pub struct Tempo {
    pub bpm: f32,
    pub confidence: f32,
    pub since_beat: f32,
}

fn sample_at(signal: &[f32], position: f32) -> f32 {
    if position < 0.0 {
        return 0.0;
    }
    let i = position.floor() as usize;
    if i + 1 >= signal.len() {
        return 0.0;
    }
    let f = position - i as f32;
    signal[i] * (1.0 - f) + signal[i + 1] * f
}

fn periodicity(signal: &[f32], lag: f32) -> f32 {
    let start = (lag * 2.0).ceil() as usize;
    let mut sum = 0.0;
    for i in start..signal.len() {
        let p = i as f32;
        sum += signal[i] * (sample_at(signal, p - lag) + 0.5 * sample_at(signal, p - lag * 2.0));
    }
    sum / (signal.len() - start).max(1) as f32
}

pub fn estimate(onsets: &[f32], fps: f32) -> Option<Tempo> {
    if (onsets.len() as f32) < fps * 4.0 {
        return None;
    }
    let mean = onsets.iter().sum::<f32>() / onsets.len() as f32;
    let signal: Vec<f32> = onsets.iter().map(|o| (o - mean).max(0.0)).collect();
    if signal.iter().all(|s| *s < 1e-4) {
        return None;
    }

    let mut best_bpm = 0.0;
    let mut best_score = f32::MIN;
    let mut total = 0.0;
    let mut count = 0.0;
    let mut bpm = 80.0;
    while bpm <= 160.0 {
        let lag = fps * 60.0 / bpm;
        let prior = (-0.5 * ((bpm / 124.0).log2() / 0.35).powi(2)).exp();
        let score = periodicity(&signal, lag) * (0.6 + 0.4 * prior);
        if score > best_score {
            best_score = score;
            best_bpm = bpm;
        }
        total += score;
        count += 1.0;
        bpm += 0.25;
    }
    let confidence = best_score / (total / count).max(1e-9);

    let score_at = |bpm: f32| periodicity(&signal, fps * 60.0 / bpm);
    let (left, right) = (score_at(best_bpm - 0.25), score_at(best_bpm + 0.25));
    let curve = left - 2.0 * score_at(best_bpm) + right;
    if curve < 0.0 {
        best_bpm += (0.5 * (left - right) / curve * 0.25).clamp(-0.25, 0.25);
    }

    let lag = fps * 60.0 / best_bpm;
    let steps = lag.floor() as usize;
    let last = signal.len() as f32 - 1.0;
    let mut best_offset = 0;
    let mut best_sum = f32::MIN;
    for offset in 0..steps {
        let mut sum = 0.0;
        let mut k = 0.0;
        while last - offset as f32 - k * lag >= 0.0 {
            let weight = 1.0 / (1.0 + k * 0.15);
            sum += sample_at(&signal, last - offset as f32 - k * lag) * weight;
            k += 1.0;
        }
        if sum > best_sum {
            best_sum = sum;
            best_offset = offset;
        }
    }

    Some(Tempo {
        bpm: best_bpm,
        confidence,
        since_beat: best_offset as f32 / fps,
    })
}
