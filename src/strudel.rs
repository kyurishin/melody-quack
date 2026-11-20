#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    Rest,
    Word(String),
    Seq(Vec<(Node, f64)>),
    Alt(Vec<Node>),
    Stack(Vec<Node>),
    Fast(Box<Node>, f64),
    Maybe(Box<Node>, u32),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Event {
    pub(crate) at: f64,
    pub(crate) len: f64,
    pub(crate) value: String,
}

struct Mini<'a> {
    s: &'a [u8],
    i: usize,
    salt: u32,
}

impl<'a> Mini<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] == b' ' || self.s[self.i] == b'\t' || self.s[self.i] == b'\n') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn number(&mut self) -> Result<f64, String> {
        let start = self.i;
        while self.i < self.s.len() && (self.s[self.i].is_ascii_digit() || self.s[self.i] == b'.') {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok().and_then(|t| t.parse::<f64>().ok()).filter(|n| *n > 0.0 && *n <= 64.0).ok_or_else(|| "expected a number from 1 to 64".to_string())
    }

    fn seq(&mut self, close: Option<u8>) -> Result<Node, String> {
        let mut layers: Vec<Node> = Vec::new();
        let mut steps: Vec<(Node, f64)> = Vec::new();
        loop {
            self.ws();
            match self.peek() {
                None => {
                    if close.is_some() {
                        return Err(format!("missing '{}'", close.unwrap() as char));
                    }
                    break;
                }
                Some(c) if Some(c) == close => {
                    self.i += 1;
                    break;
                }
                Some(b',') if close != Some(b'>') => {
                    self.i += 1;
                    layers.push(Node::Seq(std::mem::take(&mut steps)));
                }
                Some(b']') | Some(b'>') => return Err(format!("unexpected '{}'", self.peek().unwrap() as char)),
                _ => {
                    let (node, weight, copies) = self.term()?;
                    for _ in 0..copies {
                        steps.push((node.clone(), weight));
                    }
                }
            }
            if steps.len() > 256 {
                return Err("pattern too long".to_string());
            }
        }
        if close == Some(b'>') {
            return Ok(Node::Alt(steps.into_iter().map(|(n, _)| n).collect()));
        }
        if layers.is_empty() {
            Ok(Node::Seq(steps))
        } else {
            layers.push(Node::Seq(steps));
            Ok(Node::Stack(layers))
        }
    }

    fn term(&mut self) -> Result<(Node, f64, usize), String> {
        let mut node = match self.peek() {
            Some(b'[') => {
                self.i += 1;
                self.seq(Some(b']'))?
            }
            Some(b'<') => {
                self.i += 1;
                self.seq(Some(b'>'))?
            }
            Some(b'~') | Some(b'-') => {
                self.i += 1;
                Node::Rest
            }
            Some(c) if c.is_ascii_alphanumeric() || c == b'#' || c == b'.' || c == b'_' => {
                let start = self.i;
                while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || b"#._:".contains(&self.s[self.i])) {
                    self.i += 1;
                }
                Node::Word(String::from_utf8_lossy(&self.s[start..self.i]).to_lowercase())
            }
            Some(c) => return Err(format!("unexpected '{}'", c as char)),
            None => return Err("pattern ends too early".to_string()),
        };
        let mut weight = 1.0;
        let mut copies = 1usize;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.i += 1;
                    node = Node::Fast(Box::new(node), self.number()?);
                }
                Some(b'/') => {
                    self.i += 1;
                    node = Node::Fast(Box::new(node), 1.0 / self.number()?);
                }
                Some(b'!') => {
                    self.i += 1;
                    copies = if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) { self.number()? as usize } else { copies + 1 };
                }
                Some(b'@') => {
                    self.i += 1;
                    weight = self.number()?;
                }
                Some(b'?') => {
                    self.i += 1;
                    self.salt += 1;
                    node = Node::Maybe(Box::new(node), self.salt);
                }
                _ => break,
            }
        }
        Ok((node, weight, copies.min(64)))
    }
}

pub(crate) fn parse_mini(text: &str) -> Result<Node, String> {
    let mut m = Mini { s: text.as_bytes(), i: 0, salt: 0 };
    m.seq(None)
}

fn hash01(a: i64, b: u32) -> f64 {
    let mut x = (a as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (b as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 31;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 29;
    (x % 10_000) as f64 / 10_000.0
}

pub(crate) fn query(node: &Node, cycle: i64, start: f64, len: f64, out: &mut Vec<Event>) {
    if out.len() > 4096 || len <= 1e-6 {
        return;
    }
    match node {
        Node::Rest => {}
        Node::Word(w) => out.push(Event { at: start, len, value: w.clone() }),
        Node::Seq(steps) => {
            let total: f64 = steps.iter().map(|(_, w)| *w).sum();
            if total <= 0.0 {
                return;
            }
            let mut t = start;
            for (n, w) in steps {
                let d = len * w / total;
                query(n, cycle, t, d, out);
                t += d;
            }
        }
        Node::Alt(items) => {
            if items.is_empty() {
                return;
            }
            let k = items.len() as i64;
            query(&items[cycle.rem_euclid(k) as usize], cycle.div_euclid(k), start, len, out);
        }
        Node::Stack(items) => {
            for n in items {
                query(n, cycle, start, len, out);
            }
        }
        Node::Fast(n, k) => {
            if *k >= 1.0 {
                let whole = k.floor() as i64;
                let step = len / k;
                for i in 0..whole {
                    query(n, cycle * whole + i, start + i as f64 * step, step, out);
                }
                let frac = k - whole as f64;
                if frac > 1e-6 {
                    let mut tail = Vec::new();
                    query(n, cycle * whole + whole, start + whole as f64 * step, step, &mut tail);
                    let end = start + len;
                    out.extend(tail.into_iter().filter(|e| e.at < end - 1e-9));
                }
            } else {
                let span = 1.0 / k;
                let sub = (cycle as f64 / span).floor() as i64;
                let offset = cycle as f64 - sub as f64 * span;
                let mut tmp = Vec::new();
                query(n, sub, start - offset * len, len * span, &mut tmp);
                let end = start + len;
                out.extend(tmp.into_iter().filter(|e| e.at >= start - 1e-9 && e.at < end - 1e-9));
            }
        }
        Node::Maybe(n, salt) => {
            let mut tmp = Vec::new();
            query(n, cycle, start, len, &mut tmp);
            out.extend(tmp.into_iter().filter(|e| hash01(cycle * 1000 + (e.at * 997.0) as i64, *salt) >= 0.5));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Sound,
    Note,
    Index,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Layer {
    pub(crate) kind: Kind,
    pub(crate) pat: Node,
    pub(crate) sound: Option<Node>,
    pub(crate) gain: f32,
    pub(crate) fast: f64,
    pub(crate) rev: bool,
    pub(crate) line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program {
    pub(crate) layers: Vec<Layer>,
    pub(crate) cps: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Hit {
    pub(crate) at: f64,
    pub(crate) sound: String,
    pub(crate) note: Option<usize>,
    pub(crate) gain: f32,
    pub(crate) line: usize,
}

fn string_arg(call: &str, rest: &str) -> Result<(String, usize), String> {
    let r = rest.trim_start();
    let skip = rest.len() - r.len();
    let quote = r.chars().next().filter(|c| *c == '"' || *c == '\'' || *c == '`').ok_or_else(|| format!("{call} needs text in quotes, like {call}(\"bd sd\")"))?;
    let body = &r[1..];
    let end = body.find(quote).ok_or_else(|| format!("{call}: the quotes aren't closed"))?;
    let after = body[end + 1..].trim_start();
    if !after.starts_with(')') {
        return Err(format!("{call}: missing ')'"));
    }
    let used = skip + 1 + end + 1 + (body[end + 1..].len() - after.len()) + 1;
    Ok((body[..end].to_string(), used))
}

fn number_arg(call: &str, rest: &str) -> Result<(f64, usize), String> {
    let close = rest.find(')').ok_or_else(|| format!("{call}: missing ')'"))?;
    let inner = rest[..close].trim().trim_matches(|c| c == '"' || c == '\'');
    let n = inner.parse::<f64>().map_err(|_| format!("{call} needs a number"))?;
    Ok((n, close + 1))
}

pub(crate) fn parse(code: &str) -> Result<Program, (usize, String)> {
    let mut layers = Vec::new();
    let mut cps = None;
    for (line_no, raw) in code.lines().enumerate() {
        let mut line = raw.split("//").next().unwrap_or("").trim();
        if let Some(rest) = line.strip_prefix("$:") {
            line = rest.trim();
        }
        let line = line.trim_end_matches(';').trim();
        if line.is_empty() {
            continue;
        }
        let err = |m: String| (line_no, m);
        if let Some(rest) = line.strip_prefix("setcps(") {
            let (n, _) = number_arg("setcps", rest).map_err(err)?;
            cps = Some(n.clamp(0.05, 4.0));
            continue;
        }
        if let Some(rest) = line.strip_prefix("setbpm(").or_else(|| line.strip_prefix("bpm(")) {
            let (n, _) = number_arg("setbpm", rest).map_err(err)?;
            cps = Some((n / 240.0).clamp(0.05, 4.0));
            continue;
        }
        if line == "hush()" || line == "silence" {
            continue;
        }
        let (kind, after) = if let Some(r) = line.strip_prefix("s(").or_else(|| line.strip_prefix("sound(")) {
            (Kind::Sound, r)
        } else if let Some(r) = line.strip_prefix("note(") {
            (Kind::Note, r)
        } else if let Some(r) = line.strip_prefix("n(") {
            (Kind::Index, r)
        } else {
            let word: String = line.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            return Err(err(format!("start with s(\"...\"), note(\"...\") or n(\"...\") - \"{word}\" isn't known here")));
        };
        let (text, used) = string_arg(match kind { Kind::Sound => "s", Kind::Note => "note", Kind::Index => "n" }, after).map_err(err)?;
        let pat = parse_mini(&text).map_err(|m| err(m))?;
        let mut layer = Layer { kind, pat, sound: None, gain: 1.0, fast: 1.0, rev: false, line: line_no };
        let mut rest = &after[used..];
        loop {
            rest = rest.trim_start();
            if rest.is_empty() {
                break;
            }
            let Some(r) = rest.strip_prefix('.') else {
                return Err(err(format!("unexpected \"{}\"", rest.chars().take(12).collect::<String>())));
            };
            let name: String = r.chars().take_while(|c| c.is_alphanumeric()).collect();
            let r = &r[name.len()..];
            let Some(r) = r.strip_prefix('(') else {
                return Err(err(format!(".{name} needs brackets")));
            };
            match name.as_str() {
                "s" | "sound" => {
                    let (t, u) = string_arg(".s", r).map_err(err)?;
                    layer.sound = Some(parse_mini(&t).map_err(err)?);
                    rest = &r[u..];
                }
                "gain" | "velocity" => {
                    let (n, u) = number_arg(".gain", r).map_err(err)?;
                    layer.gain = (n as f32).clamp(0.0, 2.0);
                    rest = &r[u..];
                }
                "fast" => {
                    let (n, u) = number_arg(".fast", r).map_err(err)?;
                    layer.fast = (layer.fast * n).clamp(0.0625, 16.0);
                    rest = &r[u..];
                }
                "slow" => {
                    let (n, u) = number_arg(".slow", r).map_err(err)?;
                    if n <= 0.0 {
                        return Err(err(".slow needs a number above 0".to_string()));
                    }
                    layer.fast = (layer.fast / n).clamp(0.0625, 16.0);
                    rest = &r[u..];
                }
                "rev" => {
                    let close = r.find(')').ok_or_else(|| err(".rev: missing ')'".to_string()))?;
                    layer.rev = !layer.rev;
                    rest = &r[close + 1..];
                }
                other => return Err(err(format!(".{other} isn't supported yet - try .s .gain .fast .slow .rev"))),
            }
        }
        layers.push(layer);
        if layers.len() > 16 {
            return Err(err("16 lines of patterns at most".to_string()));
        }
    }
    Ok(Program { layers, cps })
}

const NOTE_PCS: [i32; 6] = [2, 4, 6, 9, 11, 0];

pub(crate) fn note_index(word: &str) -> Option<usize> {
    let w = word.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-');
    let mut chars = w.chars();
    let base = match chars.next()? {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };
    let mut pc = base;
    for c in chars {
        match c {
            '#' | 's' => pc += 1,
            'b' => pc -= 1,
            _ => return None,
        }
    }
    let pc = (pc as i32).rem_euclid(12);
    NOTE_PCS.iter().enumerate().min_by_key(|(_, p)| {
        let d = (pc - **p).rem_euclid(12);
        d.min(12 - d)
    }).map(|(i, _)| i)
}

pub(crate) fn hits(p: &Program, cycle: i64) -> Vec<Hit> {
    let mut out = Vec::new();
    for layer in &p.layers {
        let mut evs = Vec::new();
        query(&Node::Fast(Box::new(layer.pat.clone()), layer.fast), cycle, 0.0, 1.0, &mut evs);
        let sounds = layer.sound.as_ref().map(|s| {
            let mut v = Vec::new();
            query(&Node::Fast(Box::new(s.clone()), layer.fast), cycle, 0.0, 1.0, &mut v);
            v
        });
        for e in evs {
            let at = if layer.rev { (1.0 - e.at - e.len).max(0.0) } else { e.at };
            let pick = |default: &str| -> String {
                sounds
                    .as_ref()
                    .and_then(|v| v.iter().filter(|s| s.at <= e.at + 1e-9).last().or(v.first()))
                    .map(|s| s.value.clone())
                    .unwrap_or_else(|| default.to_string())
            };
            let (sound, note) = match layer.kind {
                Kind::Sound => {
                    let (name, idx) = e.value.split_once(':').map(|(a, b)| (a.to_string(), b.parse::<usize>().ok())).unwrap_or((e.value.clone(), None));
                    (name, idx.map(|i| i % 6))
                }
                Kind::Note => (pick("piano"), note_index(&e.value).or_else(|| e.value.parse::<i64>().ok().map(|n| n.rem_euclid(6) as usize))),
                Kind::Index => (pick("piano"), e.value.parse::<i64>().ok().map(|n| n.rem_euclid(6) as usize)),
            };
            if (layer.kind != Kind::Sound) && note.is_none() {
                continue;
            }
            out.push(Hit { at, sound, note, gain: layer.gain, line: layer.line });
        }
    }
    out.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
    out
}

pub(crate) fn escape(code: &str) -> String {
    code.replace('\\', "\\\\").replace('\n', "\\n")
}

pub(crate) fn unescape(text: &str) -> String {
    let mut out = String::new();
    let mut it = text.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some(o) => out.push(o),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub(crate) const EXAMPLE: &str = "// ctrl+enter plays, ctrl+. stops
setcps(0.55)
s(\"bd*4\")
s(\"~ cp ~ cp\")
s(\"hh*8\").gain(0.5)
note(\"<d a> [f# e] b ~\").s(\"piano\")";

#[cfg(test)]
mod tests {
    use super::*;

    fn at(p: &str, cycle: i64) -> Vec<(f64, String)> {
        let mut v = Vec::new();
        query(&parse_mini(p).unwrap(), cycle, 0.0, 1.0, &mut v);
        v.into_iter().map(|e| ((e.at * 1000.0).round() / 1000.0, e.value)).collect()
    }

    #[test]
    fn mini_notation_places_steps_like_strudel() {
        assert_eq!(at("bd sd", 0), vec![(0.0, "bd".into()), (0.5, "sd".into())]);
        assert_eq!(at("bd*2 ~ sd", 0), vec![(0.0, "bd".into()), (0.167, "bd".into()), (0.667, "sd".into())]);
        assert_eq!(at("[bd bd] sd", 0).len(), 3);
        assert_eq!(at("<bd sd>", 0), vec![(0.0, "bd".into())]);
        assert_eq!(at("<bd sd>", 1), vec![(0.0, "sd".into())]);
        assert_eq!(at("bd!3", 0).len(), 3);
        assert_eq!(at("[bd, hh hh]", 0).len(), 3);
        assert_eq!(at("bd/2", 0).len(), 1);
        assert_eq!(at("bd/2", 1).len(), 0);
        assert_eq!(at("bd@3 sd", 0)[1].0, 0.75);
        assert!(parse_mini("[bd sd").is_err());
    }

    #[test]
    fn programs_parse_with_methods_and_errors_point_at_the_line() {
        let p = parse(EXAMPLE).unwrap();
        assert_eq!(p.layers.len(), 4);
        assert_eq!(p.cps, Some(0.55));
        let h = hits(&p, 0);
        assert!(h.iter().any(|h| h.sound == "piano" && h.note == Some(0)));
        assert_eq!(h.iter().filter(|h| h.sound == "bd").count(), 4);
        assert_eq!(parse("s(\"bd\")\nfoo(\"x\")").unwrap_err().0, 1);
        assert!(parse("s(\"bd\").wobble(2)").is_err());
        assert_eq!(note_index("f#3"), Some(2));
        assert_eq!(note_index("g"), Some(2).or(Some(3)).and(note_index("g")));
        let code = "s(\"bd\")\nnote(\"d\")";
        assert_eq!(unescape(&escape(code)), code);
    }
}
