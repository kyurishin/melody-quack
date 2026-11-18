use std::collections::HashMap;

pub const MAX_HOUSE: usize = 12_000;

#[derive(Default)]
pub struct Homes {
    pub hosts: HashMap<String, u32>,
    pub knocks: HashMap<u32, (String, usize, String)>,
    pub visits: HashMap<u32, String>,
    pub faces: HashMap<u32, (usize, String)>,
}

pub fn valid_home(code: &str) -> bool {
    code.len() == 11 && code.chars().enumerate().all(|(i, c)| if i == 3 || i == 7 { c == '-' } else { c.is_ascii_uppercase() || c.is_ascii_digit() })
}

pub fn clean_name(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').take(16).collect()
}

impl Homes {
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
