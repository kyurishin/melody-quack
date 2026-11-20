#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    Rest,
    Word(String),
    Seq(Vec<(Node, f64)>),
    Alt(Vec<Node>),
    Stack(Vec<Node>),
    Fast(Box<Node>, f64),
    Maybe(Box<Node>, u32),
    Pick(Vec<Node>, u32),
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
        let mut choices: Vec<Node> = Vec::new();
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
                Some(b'|') => {
                    self.i += 1;
                    choices.push(Node::Seq(std::mem::take(&mut steps)));
                }
                Some(b'_') if !self.s.get(self.i + 1).map(|c| c.is_ascii_alphanumeric()).unwrap_or(false) => {
                    self.i += 1;
                    if let Some(last) = steps.last_mut() {
                        last.1 += 1.0;
                    }
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
        if !choices.is_empty() {
            choices.push(Node::Seq(steps));
            self.salt += 1;
            steps = vec![(Node::Pick(choices, self.salt), 1.0)];
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
                Some(b'(') => {
                    self.i += 1;
                    let mut nums = Vec::new();
                    loop {
                        self.ws();
                        let neg = self.peek() == Some(b'-');
                        if neg {
                            self.i += 1;
                        }
                        let start = self.i;
                        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                            self.i += 1;
                        }
                        let n: i64 = std::str::from_utf8(&self.s[start..self.i]).ok().and_then(|t| t.parse().ok()).ok_or_else(|| "euclid needs numbers, like bd(3,8)".to_string())?;
                        nums.push(if neg { -n } else { n });
                        self.ws();
                        match self.peek() {
                            Some(b',') => self.i += 1,
                            Some(b')') => {
                                self.i += 1;
                                break;
                            }
                            _ => return Err("missing ')' in euclid".to_string()),
                        }
                    }
                    if nums.len() < 2 || nums[1] <= 0 || nums[1] > 64 {
                        return Err("euclid looks like bd(3,8)".to_string());
                    }
                    let (k, n) = (nums[0].clamp(0, nums[1]), nums[1]);
                    let rot = nums.get(2).copied().unwrap_or(0);
                    let on: Vec<bool> = (0..n).map(|i| ((i + rot).rem_euclid(n) * k) % n < k).collect();
                    node = Node::Seq(on.into_iter().map(|b| (if b { node.clone() } else { Node::Rest }, 1.0)).collect());
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
        Node::Pick(items, salt) => {
            if !items.is_empty() {
                let k = (hash01(cycle, *salt) * items.len() as f64) as usize;
                query(&items[k.min(items.len() - 1)], cycle, start, len, out);
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
    Raw,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Layer {
    pub(crate) kind: Kind,
    pub(crate) pat: Node,
    pub(crate) sound: Option<Node>,
    pub(crate) gain: f32,
    pub(crate) fast: f64,
    pub(crate) rev: bool,
    pub(crate) shift: i64,
    pub(crate) lpf: Option<Param>,
    pub(crate) room: Option<Param>,
    pub(crate) line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Param {
    Num(f64),
    Pat(Node),
    Sig { shape: String, lo: f64, hi: f64, speed: f64 },
}

fn smooth_noise(t: f64) -> f64 {
    let i = t.floor();
    let f = t - i;
    let a = hash01(i as i64, 77);
    let b = hash01(i as i64 + 1, 77);
    let u = f * f * (3.0 - 2.0 * f);
    a + (b - a) * u
}

impl Param {
    fn at(&self, cycle: i64, at: f64) -> Option<f64> {
        match self {
            Param::Num(n) => Some(*n),
            Param::Pat(node) => {
                let mut v = Vec::new();
                query(node, cycle, 0.0, 1.0, &mut v);
                let e = v.iter().filter(|e| e.at <= at + 1e-9).last().or(v.first())?;
                e.value.split(':').next()?.parse::<f64>().ok()
            }
            Param::Sig { shape, lo, hi, speed } => {
                let t = (cycle as f64 + at) * speed;
                let tau = std::f64::consts::TAU;
                let u = match shape.as_str() {
                    "sine" | "sine2" => 0.5 + 0.5 * (tau * t).sin(),
                    "cosine" | "cosine2" => 0.5 + 0.5 * (tau * t).cos(),
                    "saw" | "saw2" => t.rem_euclid(1.0),
                    "isaw" | "isaw2" => 1.0 - t.rem_euclid(1.0),
                    "tri" | "tri2" => 1.0 - (2.0 * t.rem_euclid(1.0) - 1.0).abs(),
                    "square" | "square2" => if t.rem_euclid(1.0) < 0.5 { 0.0 } else { 1.0 },
                    "rand" | "rand2" => hash01((t * 4096.0) as i64, 91),
                    _ => smooth_noise(t),
                };
                Some(lo + (hi - lo) * u)
            }
        }
    }
}

const SIGNALS: [&str; 15] = ["sine", "cosine", "saw", "isaw", "tri", "square", "rand", "perlin", "sine2", "cosine2", "saw2", "isaw2", "tri2", "square2", "rand2"];

fn param(v: &Val) -> Option<Param> {
    match v {
        Val::Num(n) => Some(Param::Num(*n)),
        Val::Str(t) => parse_mini(t).ok().map(Param::Pat),
        Val::Call(name, args, methods) if args.is_empty() && SIGNALS.contains(&name.as_str()) => {
            let (mut lo, mut hi, mut speed) = (0.0, 1.0, 1.0);
            for (m, a) in methods {
                let n: Vec<f64> = a.iter().filter_map(first_number).collect();
                match (m.as_str(), n.as_slice()) {
                    ("range" | "rangex", [a, b, ..]) => {
                        lo = *a;
                        hi = *b;
                    }
                    ("slow", [k, ..]) if *k > 0.0 => speed /= k,
                    ("fast", [k, ..]) if *k > 0.0 => speed *= k,
                    _ => {}
                }
            }
            Some(Param::Sig { shape: name.clone(), lo, hi, speed })
        }
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program {
    pub(crate) layers: Vec<Layer>,
    pub(crate) cps: Option<f64>,
    pub(crate) skipped: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Hit {
    pub(crate) at: f64,
    pub(crate) sound: String,
    pub(crate) note: Option<usize>,
    pub(crate) gain: f32,
    pub(crate) lpf: Option<f32>,
    pub(crate) room: f32,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
enum Val {
    Str(String),
    Num(f64),
    Call(String, Vec<Val>, Vec<(String, Vec<Val>)>),
}

struct Expr<'a> {
    c: Vec<char>,
    i: usize,
    _src: &'a str,
}

impl<'a> Expr<'a> {
    fn ws(&mut self) {
        while self.i < self.c.len() && self.c[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    fn ident(&mut self) -> String {
        let start = self.i;
        while self.i < self.c.len() && (self.c[self.i].is_alphanumeric() || self.c[self.i] == '_' || self.c[self.i] == '$') {
            self.i += 1;
        }
        self.c[start..self.i].iter().collect()
    }

    fn args(&mut self) -> Result<Vec<Val>, String> {
        let mut out = Vec::new();
        loop {
            self.ws();
            if self.c.get(self.i) == Some(&')') {
                self.i += 1;
                return Ok(out);
            }
            let start = self.i;
            match self.expr() {
                Ok(v) => {
                    self.ws();
                    if matches!(self.c.get(self.i), Some(',') | Some(')')) {
                        out.push(v);
                    } else {
                        self.i = start;
                        self.skip_arg()?;
                        out.push(Val::Call("?".to_string(), Vec::new(), Vec::new()));
                    }
                }
                Err(_) => {
                    self.i = start;
                    self.skip_arg()?;
                    out.push(Val::Call("?".to_string(), Vec::new(), Vec::new()));
                }
            }
            self.ws();
            match self.c.get(self.i) {
                Some(',') => self.i += 1,
                Some(')') => {
                    self.i += 1;
                    return Ok(out);
                }
                _ => return Err("missing ')'".to_string()),
            }
        }
    }

    fn skip_arg(&mut self) -> Result<(), String> {
        let mut depth = 0i32;
        while let Some(&c) = self.c.get(self.i) {
            match c {
                '"' | '\'' | '`' => {
                    self.i += 1;
                    while self.i < self.c.len() && self.c[self.i] != c {
                        self.i += 1;
                    }
                }
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth == 0 => return Ok(()),
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => return Ok(()),
                _ => {}
            }
            self.i += 1;
        }
        Err("missing ')'".to_string())
    }

    fn atom(&mut self) -> Result<Val, String> {
        self.ws();
        match self.c.get(self.i).copied() {
            Some(q) if q == '"' || q == '\'' || q == '`' => {
                self.i += 1;
                let start = self.i;
                while self.i < self.c.len() && self.c[self.i] != q {
                    self.i += 1;
                }
                if self.i >= self.c.len() {
                    return Err("the quotes aren't closed".to_string());
                }
                let text: String = self.c[start..self.i].iter().collect();
                self.i += 1;
                Ok(Val::Str(text))
            }
            Some(c) if c.is_ascii_digit() || c == '-' || c == '.' => {
                let start = self.i;
                self.i += 1;
                while self.i < self.c.len() && (self.c[self.i].is_ascii_digit() || self.c[self.i] == '.') {
                    self.i += 1;
                }
                let t: String = self.c[start..self.i].iter().collect();
                t.parse::<f64>().map(Val::Num).map_err(|_| format!("\"{t}\" isn't a number"))
            }
            Some('(') => {
                self.i += 1;
                let v = self.expr()?;
                self.ws();
                if self.c.get(self.i) != Some(&')') {
                    return Err("missing ')'".to_string());
                }
                self.i += 1;
                Ok(v)
            }
            Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {
                let name = self.ident();
                self.ws();
                if self.c.get(self.i) == Some(&'(') {
                    self.i += 1;
                    let args = self.args()?;
                    Ok(Val::Call(name, args, Vec::new()))
                } else {
                    Ok(Val::Call(name, Vec::new(), Vec::new()))
                }
            }
            Some(c) => Err(format!("unexpected '{c}'")),
            None => Err("the line ends too early".to_string()),
        }
    }

    fn sum(&mut self) -> Result<Val, String> {
        let mut v = self.product()?;
        loop {
            let save = self.i;
            self.ws();
            let op = self.c.get(self.i).copied();
            if !matches!(op, Some('+') | Some('-')) || !matches!(v, Val::Num(_)) {
                self.i = save;
                return Ok(v);
            }
            self.i += 1;
            let Val::Num(b) = self.product()? else { return Err("only numbers add up".to_string()) };
            if let Val::Num(a) = v {
                v = Val::Num(if op == Some('+') { a + b } else { a - b });
            }
        }
    }

    fn product(&mut self) -> Result<Val, String> {
        let mut v = self.chain()?;
        loop {
            let save = self.i;
            self.ws();
            let op = self.c.get(self.i).copied();
            if !matches!(op, Some('*') | Some('/')) || !matches!(v, Val::Num(_)) || self.c.get(self.i + 1) == Some(&'/') || self.c.get(self.i + 1) == Some(&'*') {
                self.i = save;
                return Ok(v);
            }
            self.i += 1;
            let Val::Num(b) = self.chain()? else { return Err("only numbers multiply".to_string()) };
            if let Val::Num(a) = v {
                v = Val::Num(if op == Some('*') { a * b } else if b != 0.0 { a / b } else { a });
            }
        }
    }

    fn expr(&mut self) -> Result<Val, String> {
        self.sum()
    }

    fn chain(&mut self) -> Result<Val, String> {
        let mut v = self.atom()?;
        loop {
            let save = self.i;
            self.ws();
            if self.c.get(self.i) != Some(&'.') {
                self.i = save;
                return Ok(v);
            }
            self.i += 1;
            self.ws();
            let name = self.ident();
            if name.is_empty() {
                return Err("a '.' needs a method after it".to_string());
            }
            self.ws();
            let args = if self.c.get(self.i) == Some(&'(') {
                self.i += 1;
                self.args()?
            } else {
                Vec::new()
            };
            v = match v {
                Val::Call(n, a, mut m) => {
                    m.push((name, args));
                    Val::Call(n, a, m)
                }
                Val::Str(t) if args.is_empty() && PATTERN_FNS[..4].contains(&name.as_str()) => Val::Call(name, vec![Val::Str(t)], Vec::new()),
                Val::Str(t) => Val::Call("s".to_string(), vec![Val::Str(t)], vec![(name, args)]),
                Val::Num(_) => return Err("a number can't have methods".to_string()),
            };
        }
    }
}

fn statements(code: &str) -> Vec<(usize, String)> {
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut start_line = 0usize;
    let mut line = 0usize;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut i = 0;
    let push = |cur: &mut String, out: &mut Vec<(usize, String)>, at: usize| {
        let t = cur.trim().to_string();
        if !t.is_empty() {
            out.push((at, t));
        }
        cur.clear();
    };
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            cur.push(c);
            if c == '\n' {
                line += 1;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                if chars[i] == '\n' {
                    line += 1;
                    cur.push(' ');
                }
                i += 1;
            }
            i += 2;
            continue;
        }
        match c {
            '"' | '\'' | '`' => {
                quote = Some(c);
                cur.push(c);
            }
            '(' | '[' | '{' => {
                depth += 1;
                cur.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                cur.push(c);
            }
            ';' if depth <= 0 => push(&mut cur, &mut out, start_line),
            '\n' => {
                line += 1;
                let mut k = i + 1;
                let mut next_line_dot = false;
                while k < chars.len() {
                    if chars[k] == '\n' || chars[k] == ' ' || chars[k] == '\t' || chars[k] == '\r' {
                        k += 1;
                        continue;
                    }
                    if chars[k] == '/' && chars.get(k + 1) == Some(&'/') {
                        while k < chars.len() && chars[k] != '\n' {
                            k += 1;
                        }
                        continue;
                    }
                    next_line_dot = chars[k] == '.' && !chars.get(k + 1).map(|d| d.is_ascii_digit()).unwrap_or(false);
                    break;
                }
                if depth > 0 || next_line_dot {
                    cur.push(' ');
                } else {
                    push(&mut cur, &mut out, start_line);
                }
            }
            '\r' => {}
            _ => {
                if cur.trim().is_empty() {
                    start_line = line;
                }
                cur.push(c);
            }
        }
        i += 1;
    }
    push(&mut cur, &mut out, start_line);
    out
}

fn first_number(v: &Val) -> Option<f64> {
    match v {
        Val::Num(n) => Some(*n),
        Val::Str(s) => s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).find_map(|t| t.parse::<f64>().ok()),
        _ => None,
    }
}

const PATTERN_FNS: [&str; 5] = ["s", "sound", "note", "n", "freq"];

fn to_layers(v: &Val, line: usize, skipped: &mut Vec<String>) -> Result<Vec<Layer>, String> {
    if let Val::Str(t) = v {
        return Ok(vec![Layer { kind: Kind::Raw, pat: parse_mini(t)?, sound: None, gain: 1.0, fast: 1.0, rev: false, shift: 0, lpf: None, room: None, line }]);
    }
    let Val::Call(name, args, methods) = v else {
        return Err("start with s(\"...\"), note(\"...\") or n(\"...\")".to_string());
    };
    let mut layers = match name.as_str() {
        "s" | "sound" | "note" | "n" => {
            let text = match args.first() {
                Some(Val::Str(t)) => Some(t),
                Some(Val::Call(n, a, _)) if n == "s" => match a.first() {
                    Some(Val::Str(t)) => Some(t),
                    _ => None,
                },
                _ => None,
            };
            let Some(text) = text else {
                return Err(format!("{name} needs a pattern in quotes, like {name}(\"bd sd\")"));
            };
            let kind = match name.as_str() {
                "note" => Kind::Note,
                "n" => Kind::Index,
                _ => Kind::Sound,
            };
            vec![Layer { kind, pat: parse_mini(text)?, sound: None, gain: 1.0, fast: 1.0, rev: false, shift: 0, lpf: None, room: None, line }]
        }
        "stack" => {
            let mut all = Vec::new();
            for a in args {
                all.extend(to_layers(a, line, skipped)?);
            }
            all
        }
        "cat" | "slowcat" | "seq" | "fastcat" | "sequence" => {
            let mut parts = Vec::new();
            for a in args {
                parts.extend(to_layers(a, line, skipped)?);
            }
            let Some(first) = parts.first().cloned() else { return Ok(Vec::new()) };
            let pats: Vec<Node> = parts.into_iter().map(|l| l.pat).collect();
            let pat = if name == "cat" || name == "slowcat" { Node::Alt(pats) } else { Node::Seq(pats.into_iter().map(|p| (p, 1.0)).collect()) };
            vec![Layer { pat, ..first }]
        }
        "silence" | "hush" => Vec::new(),
        other => {
            let tag = format!("{other}()");
            if !skipped.contains(&tag) {
                skipped.push(tag);
            }
            Vec::new()
        }
    };
    for (m, margs) in methods {
        let num = margs.first().and_then(first_number);
        match m.as_str() {
            "note" | "n" | "s" | "sound" if margs.is_empty() => {
                for l in layers.iter_mut() {
                    if l.kind == Kind::Raw {
                        l.kind = match m.as_str() {
                            "note" => Kind::Note,
                            "n" => Kind::Index,
                            _ => Kind::Sound,
                        };
                    }
                }
            }
            "transpose" | "add" if num.is_some() && matches!(margs.first(), Some(Val::Num(_))) || m == "transpose" && num.is_some() => {
                let k = num.unwrap_or(0.0).round() as i64;
                for l in layers.iter_mut() {
                    l.shift += k;
                }
            }
            "s" | "sound" => {
                if let Some(Val::Str(t)) = margs.first() {
                    let pat = parse_mini(t)?;
                    for l in layers.iter_mut() {
                        if l.kind == Kind::Sound {
                            l.pat = pat.clone();
                        } else {
                            l.sound = Some(pat.clone());
                        }
                    }
                }
            }
            "note" | "n" if margs.first().map(|a| matches!(a, Val::Str(_))).unwrap_or(false) => {
                if let Some(Val::Str(t)) = margs.first() {
                    let pat = parse_mini(t)?;
                    for l in layers.iter_mut() {
                        if l.kind == Kind::Sound {
                            l.sound = Some(l.pat.clone());
                            l.pat = pat.clone();
                            l.kind = if m == "note" { Kind::Note } else { Kind::Index };
                        }
                    }
                }
            }
            "gain" | "velocity" | "amp" => {
                if let Some(n) = num {
                    for l in layers.iter_mut() {
                        l.gain = (l.gain * n as f32).clamp(0.0, 2.0);
                    }
                }
            }
            "fast" | "slow" => {
                if let Some(n) = num.filter(|n| *n > 0.0) {
                    for l in layers.iter_mut() {
                        l.fast = if m == "fast" { (l.fast * n).clamp(1.0 / 256.0, 64.0) } else { (l.fast / n).clamp(1.0 / 256.0, 64.0) };
                    }
                }
            }
            "rev" => {
                for l in layers.iter_mut() {
                    l.rev = !l.rev;
                }
            }
            "lpf" | "cutoff" | "ctf" | "lp" | "room" => {
                if let Some(p) = margs.first().and_then(param) {
                    for l in layers.iter_mut() {
                        if m == "room" {
                            l.room = Some(p.clone());
                        } else {
                            l.lpf = Some(p.clone());
                        }
                    }
                }
            }
            "bank" | "scale" | "color" | "colour" | "pianoroll" | "punchcard" | "scope" | "spiral" | "orbit" | "p" | "q" | "_pianoroll" | "_scope" | "_punchcard" | "analyze" | "fft" => {}
            other => {
                if !skipped.iter().any(|x| x == other) && !PATTERN_FNS.contains(&other) {
                    skipped.push(other.to_string());
                }
            }
        }
    }
    Ok(layers)
}

pub(crate) fn parse(code: &str) -> Result<Program, (usize, String)> {
    let mut layers = Vec::new();
    let mut cps = None;
    let mut skipped = Vec::new();
    for (line_no, raw) in statements(code) {
        let err = |m: String| (line_no, m);
        let mut st = raw.trim().trim_end_matches(';').trim().to_string();
        if st.starts_with("_$:") || (st.starts_with('_') && st.split_once(':').map(|(l, _)| l.chars().all(|c| c.is_alphanumeric() || c == '_')).unwrap_or(false)) {
            continue;
        }
        if let Some(rest) = st.strip_prefix("$:") {
            st = rest.trim().to_string();
        }
        let label: String = st.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !label.is_empty() && st[label.len()..].starts_with(':') && !st[label.len()..].starts_with("::") {
            st = st[label.len() + 1..].trim().to_string();
        }
        for kw in ["const ", "let ", "var "] {
            if st.starts_with(kw) {
                st.clear();
                if !skipped.iter().any(|x: &String| x == "variables") {
                    skipped.push("variables".to_string());
                }
            }
        }
        let st = st.trim_start_matches("await ").trim().to_string();
        if st.is_empty() {
            continue;
        }
        let mut e = Expr { c: st.chars().collect(), i: 0, _src: &st };
        let v = e.expr().map_err(err)?;
        e.ws();
        if e.i < e.c.len() {
            return Err(err(format!("unexpected \"{}\"", e.c[e.i..].iter().take(12).collect::<String>())));
        }
        if let Val::Call(name, args, _) = &v {
            match name.as_str() {
                "setcps" | "setCps" => {
                    cps = args.first().and_then(first_number).map(|n| n.clamp(0.05, 4.0));
                    continue;
                }
                "setcpm" | "setCpm" => {
                    cps = args.first().and_then(first_number).map(|n| (n / 60.0).clamp(0.05, 4.0));
                    continue;
                }
                "setbpm" | "bpm" | "setBpm" => {
                    cps = args.first().and_then(first_number).map(|n| (n / 240.0).clamp(0.05, 4.0));
                    continue;
                }
                "samples" | "register" | "hush" | "silence" | "initAudioOnFirstClick" | "loadOrc" | "useRNG" => continue,
                _ => {}
            }
        }
        layers.extend(to_layers(&v, line_no, &mut skipped).map_err(err)?);
        if layers.len() > 24 {
            return Err(err("24 patterns at most".to_string()));
        }
    }
    if layers.is_empty() && !skipped.is_empty() {
        return Err((0, format!("nothing here this computer can play yet ({}) - try s(), note(), n() or stack()", skipped.join(" "))));
    }
    Ok(Program { layers, cps, skipped })
}

const NOTE_PCS: [i32; 6] = [2, 4, 6, 9, 11, 0];

fn midi_index(n: i64) -> usize {
    let pc = n.rem_euclid(12) as i32;
    NOTE_PCS.iter().enumerate().min_by_key(|(_, p)| {
        let d = (pc - **p).rem_euclid(12);
        d.min(12 - d)
    }).map(|(i, _)| i).unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn note_index(word: &str) -> Option<usize> {
    note_pc(word).map(midi_index)
}

fn note_pc(word: &str) -> Option<i64> {
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
    Some((pc as i64).rem_euclid(12))
}

pub(crate) fn hits(p: &Program, cycle: i64) -> Vec<Hit> {
    let mut out = Vec::new();
    for layer in &p.layers {
        let mut evs = Vec::new();
        query(&Node::Fast(Box::new(layer.pat.clone()), layer.fast), cycle, 0.0, 1.0, &mut evs);
        let sounds = layer.sound.as_ref().map(|s| {
            let mut v = Vec::new();
            query(s, cycle, 0.0, 1.0, &mut v);
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
            let kind = match layer.kind {
                Kind::Raw if layer.sound.is_some() => Kind::Note,
                Kind::Raw => Kind::Sound,
                k => k,
            };
            let (sound, note) = match kind {
                Kind::Sound => {
                    let (name, idx) = e.value.split_once(':').map(|(a, b)| (a.to_string(), b.parse::<usize>().ok())).unwrap_or((e.value.clone(), None));
                    (name, idx.map(|i| i % 6))
                }
                Kind::Note => (pick("piano"), note_pc(&e.value).or_else(|| e.value.parse::<f64>().ok().map(|n| n.round() as i64)).map(|n| midi_index(n + layer.shift))),
                _ => (pick("piano"), e.value.parse::<i64>().ok().map(|n| n.rem_euclid(6) as usize)),
            };
            if kind != Kind::Sound && note.is_none() {
                continue;
            }
            let lpf = layer.lpf.as_ref().and_then(|p| p.at(cycle, e.at)).map(|f| f.clamp(30.0, 20000.0) as f32);
            let room = layer.room.as_ref().and_then(|p| p.at(cycle, e.at)).unwrap_or(0.0).clamp(0.0, 1.0) as f32;
            out.push(Hit { at, sound, note, gain: layer.gain, lpf, room, line: layer.line });
        }
    }
    out.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
    out
}

pub(crate) fn lpf_bucket(f: f32) -> Option<i16> {
    if f >= 16000.0 {
        return None;
    }
    Some(((f.max(40.0) / 40.0).log2() * 3.0).round() as i16)
}

pub(crate) fn bucket_hz(b: i16) -> f32 {
    40.0 * 2f32.powf(b as f32 / 3.0)
}

pub(crate) fn room_bucket(r: f32) -> u8 {
    (r.clamp(0.0, 1.0) * 8.0).round() as u8
}

pub(crate) fn render_fx(dry: &[f32], lpf: Option<f32>, room: f32, rate: f32) -> Vec<f32> {
    let mut x: Vec<f32> = dry.to_vec();
    if let Some(f) = lpf {
        let w = std::f32::consts::TAU * f.min(rate * 0.45) / rate;
        let q = 1.0f32;
        let alpha = w.sin() / (2.0 * q);
        let c = w.cos();
        let a0 = 1.0 + alpha;
        let (b0, b1, b2) = ((1.0 - c) / 2.0 / a0, (1.0 - c) / a0, (1.0 - c) / 2.0 / a0);
        let (a1, a2) = (-2.0 * c / a0, (1.0 - alpha) / a0);
        let (mut x1, mut x2, mut y1, mut y2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for v in x.iter_mut() {
            let y = b0 * *v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            x2 = x1;
            x1 = *v;
            y2 = y1;
            y1 = y;
            *v = y;
        }
    }
    if room > 0.01 {
        let k = rate / 44100.0;
        let tail = ((0.5 + 1.8 * room) * rate) as usize;
        let n = x.len() + tail;
        let mut wet = vec![0.0f32; n];
        let feedback = 0.78 + 0.16 * room;
        let damp = 0.3f32;
        for d in [1116.0f32, 1188.0, 1277.0, 1356.0, 1422.0, 1491.0] {
            let len = (d * k) as usize;
            let mut buf = vec![0.0f32; len];
            let (mut i, mut low) = (0usize, 0.0f32);
            for t in 0..n {
                let input = x.get(t).copied().unwrap_or(0.0);
                let out = buf[i];
                low = out * (1.0 - damp) + low * damp;
                buf[i] = input + low * feedback;
                wet[t] += out;
                i = (i + 1) % len;
            }
        }
        for d in [556.0f32, 441.0, 341.0, 225.0] {
            let len = (d * k) as usize;
            let mut buf = vec![0.0f32; len];
            let mut i = 0usize;
            for v in wet.iter_mut() {
                let b = buf[i];
                let out = -*v + b;
                buf[i] = *v + b * 0.5;
                *v = out;
                i = (i + 1) % len;
            }
        }
        let fade = (0.25 * rate) as usize;
        let level = 0.045 * room;
        let dry_level = 1.0 - 0.35 * room;
        let mut out = Vec::with_capacity(n);
        for t in 0..n {
            let mut v = x.get(t).copied().unwrap_or(0.0) * dry_level + wet[t] * level;
            if t + fade > n {
                v *= (n - t) as f32 / fade as f32;
            }
            out.push(v);
        }
        x = out;
    }
    x
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
        assert_eq!(parse("s(\"bd\")\ns(\"[sd\")").unwrap_err().0, 1);
        assert_eq!(parse("s(\"bd\").wobble(2)").unwrap().skipped, vec!["wobble".to_string()]);
        assert!(parse("foo(\"x\")").is_err());
        assert_eq!(note_index("f#3"), Some(2));
        assert_eq!(note_index("g"), Some(2).or(Some(3)).and(note_index("g")));
        let code = "s(\"bd\")\nnote(\"d\")";
        assert_eq!(unescape(&escape(code)), code);
    }

    #[test]
    fn mini_notation_has_euclid_elongation_and_choice() {
        let e: Vec<f64> = at("bd(3,8)", 0).into_iter().map(|x| x.0).collect();
        assert_eq!(e, vec![0.0, 0.375, 0.75]);
        assert_eq!(at("bd(3,8,2)", 0).len(), 3);
        assert_eq!(at("bd _ _ sd", 0)[1].0, 0.75);
        for c in 0..8 {
            let v = at("bd | sd | hh", c);
            assert_eq!(v.len(), 1);
        }
    }

    #[test]
    fn code_pasted_from_strudel_cc_plays() {
        let tune = r#"// "coastline" @by someone
// @license CC BY-NC-SA 4.0
samples('github:tidalcycles/dirt-samples')
setcpm(120/4)

$: s("bd*2, ~ sd, hh*8")
  .bank("RolandTR909")
  .gain(.8)

$: note("<c2 eb2 g1 bb1>(3,8)")
  .sound("sawtooth")
  .lpf(perlin.range(300, 2000).slow(4))
  .room(.5)

_$: s("cp*4")

melody: n("0 2 4 <6 7>".add("<0 3>"))
  .scale("C:minor")
  .s("gm_epiano1")
  .every(4, x => x.fast(2))
  .delay(.25)
"#;
        let p = parse(tune).unwrap();
        assert_eq!(p.cps, Some(0.5));
        assert_eq!(p.layers.len(), 3);
        for m in ["every", "delay"] {
            assert!(p.skipped.iter().any(|s| s == m), "{m} in {:?}", p.skipped);
        }
        let h = hits(&p, 0);
        assert!(h.iter().any(|h| h.sound == "bd"));
        assert!(h.iter().any(|h| h.sound == "sawtooth" && h.note.is_some() && h.lpf.is_some() && h.room > 0.4));
        assert!(!p.skipped.iter().any(|s| s == "lpf" || s == "room"));
        assert!(h.iter().any(|h| h.sound == "gm_epiano1"));
        assert!(!h.iter().any(|h| h.sound == "cp"));

        let stacked = parse("stack(\n  s(\"bd sd\"),\n  note(\"c3 e3 g3\").s(\"piano\")\n).fast(2);").unwrap();
        assert_eq!(stacked.layers.len(), 2);
        assert!(stacked.layers.iter().all(|l| l.fast == 2.0));
        let midi = parse("note(\"60 64 67\")").unwrap();
        assert_eq!(hits(&midi, 0).iter().map(|h| h.note.unwrap()).collect::<Vec<_>>(), vec![5, 1, 2]);
        assert!(parse("let x = 1\nsilence").is_err());
    }

    #[test]
    fn more_strudel_cc_styles_parse() {
        let tunes = [
            "setcps(1)\nn(\"<0 1 2 3 4>*8\").scale('G4 minor')\n.s(\"gm_lead_6_voice\")\n.clip(sine.range(.2,.8).slow(8))\n.jux(rev)\n.room(2)\n.sometimes(add(note(\"12\")))\n.lpf(perlin.range(200,20000).slow(4))",
            "\"<c3 [e3 g3]>*2\".note().s('piano').gain(\"<.5 1>\")",
            "stack(\n  s(\"bd*2, [~ hh]*2, ~ sd\").bank('RolandTR808'),\n  note(\"<[c2 c3]*4 [bb1 bb2]*4>\").s('sawtooth').lpf(800),\n  n(\"[0 2 4 6]/2\").scale(\"C4:minor\").s(\"triangle\")\n).color('cyan')",
            "await samples('github:tidalcycles/dirt-samples')\nsetcps(.7)\nconst drums = s(\"bd sd\")\n$: s(\"bd ~ [~ bd] sd\").speed(\"1 2\") // kick\n$: s(\"hh*16\").gain(\"[.25 1]*4\").pan(sine)",
        ];
        for t in tunes {
            let p = parse(t).unwrap_or_else(|e| panic!("{t}\n{e:?}"));
            assert!(!p.layers.is_empty());
            assert!(!hits(&p, 0).is_empty() || !hits(&p, 1).is_empty(), "{t}");
        }
    }

    #[test]
    fn a_stack_of_plain_strings_turned_into_notes_plays() {
        let code = "// two parts\nstack(\n  // top\n  `[B3@2 D4] [A3]\n  [G3 A3]`,\n  // bottom\n  `[[C2 G2] E3@2] [F#3]`\n).transpose(12).slow(8)\n  .superimpose(x=>x.add(0.06)) // detune\n  .note()\n  .gain(.1)\n  .s('triangle')\n  .room(1)\n  //.pianoroll({fold:1})\n";
        let p = parse(code).unwrap();
        assert_eq!(p.layers.len(), 2);
        assert_eq!(p.skipped, vec!["superimpose".to_string()]);
        let all: Vec<Hit> = (0..8).flat_map(|c| hits(&p, c)).collect();
        assert!(all.iter().all(|h| h.sound == "triangle"));
        assert!(all.iter().any(|h| h.note == Some(4)));
        assert!(all.iter().any(|h| h.note == Some(2)));
        let up = parse("note(\"c3\").transpose(2)").unwrap();
        assert_eq!(hits(&up, 0)[0].note, Some(0));
    }

    #[test]
    fn lpf_and_room_are_parsed_and_rendered() {
        let p = parse("note(\"c3 e3\").s(\"sawtooth\").lpf(800).room(.5)\ns(\"hh*4\").lpf(\"<300 4000>\")\ns(\"bd\").cutoff(sine.range(200, 2000).slow(4))").unwrap();
        assert!(p.skipped.is_empty(), "{:?}", p.skipped);
        let h = hits(&p, 0);
        assert!(h.iter().any(|h| h.lpf == Some(800.0) && (h.room - 0.5).abs() < 1e-6));
        assert!(h.iter().any(|h| h.sound == "hh" && h.lpf == Some(300.0)));
        assert!(hits(&p, 1).iter().any(|h| h.sound == "hh" && h.lpf == Some(4000.0)));
        assert!(h.iter().any(|h| h.sound == "bd" && h.lpf.map(|f| f > 200.0 && f < 2000.0).unwrap_or(false)));

        let rate = 44100.0;
        let mut seed = 1u32;
        let noise: Vec<f32> = (0..4410).map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32 * 1.2 - 0.6
        }).collect();
        let hf = |v: &[f32]| v.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / v.len() as f32;
        let dark = render_fx(&noise, Some(400.0), 0.0, rate);
        assert_eq!(dark.len(), noise.len());
        assert!(hf(&dark) < hf(&noise) * 0.2, "{} {}", hf(&dark), hf(&noise));
        let wet = render_fx(&noise, None, 1.0, rate);
        assert!(wet.len() > noise.len() + rate as usize);
        let tail = &wet[noise.len() + 4410..noise.len() + 8820];
        let rms = |v: &[f32]| (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt();
        println!("dry {} wet-body {} tail {} peak {}", rms(&noise), rms(&wet[..noise.len()]), rms(tail), wet.iter().fold(0.0f32, |m, x| m.max(x.abs())));
        assert!(rms(tail) > rms(&noise) * 0.05);
        assert!(wet.iter().all(|x| x.is_finite() && x.abs() < 1.5));
        assert_eq!(lpf_bucket(20000.0), None);
        assert!((bucket_hz(lpf_bucket(800.0).unwrap()) - 800.0).abs() < 80.0);
    }
}
