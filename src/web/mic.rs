use crate::tempo::OnsetDetector;
use std::sync::{Arc, Mutex};

pub struct Mic {
    pub detector: Arc<Mutex<OnsetDetector>>,
    buf: Vec<f32>,
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
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
