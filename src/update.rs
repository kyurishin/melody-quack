pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) const RELEASES: &str = "https://github.com/kyurishin/melody-quack/releases/latest";

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) fn parse(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let mut it = v.split(['.', '-', '+']).map(|p| p.parse::<u32>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) fn newer(tag: &str, current: &str) -> bool {
    match (parse(tag), parse(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::sync::{Mutex, Once};

    static FOUND: Mutex<Option<(String, String)>> = Mutex::new(None);
    static START: Once = Once::new();

    pub(crate) fn check() {
        START.call_once(|| {
            std::thread::spawn(|| {
                let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(10)).build();
                let reply = agent
                    .get(&std::env::var("MQ_UPDATE_URL").unwrap_or_else(|_| "https://api.github.com/repos/kyurishin/melody-quack/releases/latest".to_string()))
                    .set("User-Agent", "melody-quack-update-check")
                    .set("Accept", "application/vnd.github+json")
                    .call();
                let Ok(reply) = reply else { return };
                let Ok(json) = reply.into_json::<ureq::serde_json::Value>() else { return };
                let tag = json.get("tag_name").and_then(|t| t.as_str()).unwrap_or("");
                let url = json.get("html_url").and_then(|t| t.as_str()).unwrap_or(super::RELEASES);
                if super::newer(tag, super::VERSION) {
                    if let Ok(mut f) = FOUND.lock() {
                        *f = Some((tag.trim_start_matches('v').to_string(), url.to_string()));
                    }
                }
            });
        });
    }

    pub(crate) fn found() -> Option<(String, String)> {
        FOUND.lock().ok().and_then(|f| f.clone())
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    pub(crate) fn check() {}

    pub(crate) fn found() -> Option<(String, String)> {
        None
    }
}

pub(crate) use imp::{check, found};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_higher_version_counts_as_newer() {
        assert!(newer("v0.2.1", "0.2.0"));
        assert!(newer("v0.3.0", "0.2.9"));
        assert!(newer("1.0.0", "0.9.9"));
        assert!(!newer("v0.2.0", "0.2.0"));
        assert!(!newer("v0.1.9", "0.2.0"));
        assert!(!newer("preview-184", "0.2.0"));
        assert!(!newer("", "0.2.0"));
    }
}
