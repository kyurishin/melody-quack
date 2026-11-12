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
