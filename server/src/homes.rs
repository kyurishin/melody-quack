use std::collections::HashMap;

pub const MAX_HOUSE: usize = 12_000;

#[derive(Default)]
pub struct Homes {
    pub hosts: HashMap<String, u32>,
    pub knocks: HashMap<u32, (String, usize, String)>,
    pub visits: HashMap<u32, String>,
    pub faces: HashMap<u32, (usize, String)>,
    pub watch: HashMap<u32, Vec<String>>,
    pub me: HashMap<u32, (usize, String)>,
    pub spoke: HashMap<u32, i64>,
}

pub const MAX_FRIENDS: usize = 50;
pub const SAY_GAP_MS: i64 = 600;

pub fn clean_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(160).collect::<String>().trim().to_string()
}

pub fn valid_home(code: &str) -> bool {
    code.len() == 11 && code.chars().enumerate().all(|(i, c)| if i == 3 || i == 7 { c == '-' } else { c.is_ascii_uppercase() || c.is_ascii_digit() })
}

pub fn clean_sounds(rest: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for pair in rest.trim().split(',').take(64) {
        if let Some((id, v)) = pair.split_once(':') {
            if let (Ok(id), Ok(v)) = (id.parse::<u16>(), v.parse::<f32>()) {
                if v.is_finite() && id < 2048 {
                    out.push(format!("{}:{:.2}", id, v.clamp(0.0, 1.5)));
                }
            }
        }
    }
    out.join(",")
}

pub fn clean_name(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').take(16).collect()
}

#[cfg(test)]
mod sound_tests {
    use super::*;

    #[test]
    fn shared_sounds_are_cleaned() {
        assert_eq!(clean_sounds("3:0.5,x:1,9:9,4:nan"), "3:0.50,9:1.50");
        assert_eq!(clean_sounds(""), "");
        assert_eq!(clean_sounds(&vec!["1:1"; 100].join(",")).split(',').count(), 64);
    }
}

impl Homes {
    pub fn status(&self, code: &str) -> String {
        match self.hosts.get(code) {
            Some(id) => {
                let st = if self.visits.contains_key(id) { 2 } else { 1 };
                let (ch, name) = self.me.get(id).cloned().unwrap_or((0, String::new()));
                format!("fstat {code} {st} {ch} {}", if name.is_empty() { "-".to_string() } else { name })
            }
            None => format!("fstat {code} 0 0 -"),
        }
    }

    pub fn watchers(&self, code: &str) -> Vec<u32> {
        self.watch.iter().filter(|(_, list)| list.iter().any(|c| c == code)).map(|(id, _)| *id).collect()
    }

    pub fn mutual(&self, a: u32, b: u32) -> bool {
        let (Some(ca), Some(cb)) = (self.code_of(a), self.code_of(b)) else { return false };
        self.watch.get(&a).map(|l| l.contains(&cb)).unwrap_or(false) && self.watch.get(&b).map(|l| l.contains(&ca)).unwrap_or(false)
    }

    pub fn may_speak(&mut self, id: u32, now: i64) -> bool {
        let last = self.spoke.get(&id).copied().unwrap_or(i64::MIN / 2);
        if now - last < SAY_GAP_MS {
            return false;
        }
        self.spoke.insert(id, now);
        true
    }

    pub fn name_of(&self, id: u32) -> (usize, String) {
        self.me.get(&id).or_else(|| self.faces.get(&id)).cloned().unwrap_or((0, "friend".to_string()))
    }

    pub fn code_of(&self, id: u32) -> Option<String> {
        self.hosts.iter().find(|(_, h)| **h == id).map(|(c, _)| c.clone())
    }

    pub fn group(&self, id: u32) -> Vec<u32> {
        let code = match self.visits.get(&id) {
            Some(c) => c.clone(),
            None => match self.code_of(id) {
                Some(c) => c,
                None => return Vec::new(),
            },
        };
        let mut out: Vec<u32> = self.visits.iter().filter(|(_, c)| **c == code).map(|(g, _)| *g).collect();
        if let Some(h) = self.hosts.get(&code) {
            out.push(*h);
        }
        out.retain(|x| *x != id);
        out.sort();
        out
    }

    pub fn guests_of(&self, host: u32) -> Vec<u32> {
        match self.code_of(host) {
            Some(code) => self.visits.iter().filter(|(_, c)| **c == code).map(|(g, _)| *g).collect(),
            None => Vec::new(),
        }
    }
}
