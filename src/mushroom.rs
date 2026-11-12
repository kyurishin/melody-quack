use macroquad::prelude::*;

pub const COUNT: usize = 120;
pub const PER_DAY: usize = 7;

pub const SPECIES: [&str; COUNT] = [
    "button mushroom", "chestnut mushroom", "field mushroom", "horse mushroom", "meadow puffball", "common puffball", "shaggy ink cap", "fairy ring champignon", "wood blewit", "field blewit",
    "oyster mushroom", "velvet shank", "honey fungus", "the deceiver", "amethyst deceiver", "tawny grisette", "parasol mushroom", "shaggy parasol", "scaly wood mushroom", "the miller",
    "bay bolete", "orange birch bolete", "brown birch bolete", "slippery jack", "weeping bolete", "larch bolete", "summer cep", "penny bun", "hedgehog fungus", "terracotta hedgehog",
    "chanterelle", "trumpet chanterelle", "horn of plenty", "king oyster", "enoki", "shiitake", "nameko", "shimeji", "wood ear", "witch's butter",
    "cauliflower fungus", "beefsteak fungus", "chicken of the woods", "hen of the woods", "lion's mane", "st george's mushroom", "charcoal burner", "the blusher", "saffron milk cap", "false saffron milk cap",
    "crab brittlegill", "green brittlegill", "yellow swamp brittlegill", "ochre brittlegill", "porcelain fungus", "velvet bolete", "snowy waxcap", "meadow waxcap", "parrot waxcap", "scarlet waxcap",
    "golden waxcap", "crimson waxcap", "pink waxcap", "verdigris agaric", "sulphur tuft", "candlesnuff fungus", "dead man's fingers", "turkey tail", "birch polypore", "dryad's saddle",
    "bear's head tooth", "coral tooth", "yellow stagshorn", "white coral fungus", "violet coral", "earthstar", "bird's nest fungus", "stinkhorn", "morel", "black morel",
    "false morel", "scarlet elf cup", "orange peel fungus", "hare's ear", "jelly tongue", "bleeding tooth", "indigo milk cap", "lilac bonnet", "bleeding bonnet", "orange mosscap",
    "bitter oyster", "ghost fungus", "fly agaric", "panther cap", "caesar's mushroom", "matsutake", "devil's bolete", "shaggy scalycap", "golden scalycap", "rooting shank",
    "black truffle", "white truffle", "summer truffle", "blue chanterelle", "golden chanterelle", "ruby bolete", "amethyst puffball", "moonlight mycena", "bassline bolete", "hi-hat hedgehog",
    "kick drum puffball", "vinyl cap", "echo chanterelle", "synth shimeji", "acid ink cap", "golden dock oyster", "aurora morel", "quack cap", "rainbow coral", "melody truffle",
];

const INEDIBLE: [&str; 22] = [
    "the deceiver", "verdigris agaric", "sulphur tuft", "candlesnuff fungus", "dead man's fingers", "turkey tail", "birch polypore", "yellow stagshorn", "earthstar", "bird's nest fungus", "stinkhorn",
    "false morel", "bleeding tooth", "lilac bonnet", "bleeding bonnet", "orange mosscap", "bitter oyster", "ghost fungus", "fly agaric", "panther cap", "devil's bolete", "shaggy scalycap",
];

pub fn rarity(i: usize) -> usize {
    match i {
        0..=59 => 0,
        60..=99 => 1,
        100..=114 => 2,
        _ => 3,
    }
}

pub fn edible(i: usize) -> bool {
    !INEDIBLE.contains(&SPECIES[i % COUNT])
}

pub fn acorns(i: usize) -> u32 {
    [1, 3, 7, 14][rarity(i)]
}

pub fn kind(i: usize) -> usize {
    [0, 1, 2, 2][rarity(i)]
}

pub fn code(i: usize) -> String {
    format!("M{:03}", i + 1)
}

pub fn hint(i: usize) -> &'static str {
    if !edible(i) {
        return "not for the pot - look, don't eat";
    }
    match rarity(i) {
        0 => "pops up near trees most days",
        1 => "only some days - check often",
        2 => "rare - a lucky day under the trees",
        _ => "legendary - almost never grows",
    }
}

pub fn search(query: &str) -> Vec<usize> {
    let q = query.trim().trim_start_matches('#').trim_start_matches('m').to_lowercase();
    let raw = query.trim().to_lowercase();
    if raw.is_empty() {
        return (0..COUNT).collect();
    }
    if let Ok(n) = q.parse::<usize>() {
        let exact: Vec<usize> = (0..COUNT).filter(|i| i + 1 == n).collect();
        let partial: Vec<usize> = (0..COUNT).filter(|i| i + 1 != n && format!("{:03}", i + 1).contains(&q)).collect();
        return exact.into_iter().chain(partial).collect();
    }
    let starts: Vec<usize> = (0..COUNT).filter(|i| SPECIES[*i].starts_with(&raw)).collect();
    let contains: Vec<usize> = (0..COUNT).filter(|i| !SPECIES[*i].starts_with(&raw) && SPECIES[*i].contains(&raw)).collect();
    starts.into_iter().chain(contains).collect()
}

pub fn today(day: i64) -> Vec<usize> {
    let mut picks = Vec::new();
    let mut salt = 0u32;
    while picks.len() < PER_DAY && salt < 500 {
        let roll = hash(day as usize, 11 + salt);
        let tier = if roll < 0.62 { 0 } else if roll < 0.9 { 1 } else if roll < 0.985 { 2 } else { 3 };
        let range = match tier {
            0 => 0..60,
            1 => 60..100,
            2 => 100..115,
            _ => 115..120,
        };
        let span = range.end - range.start;
        let i = range.start + ((hash(day as usize, 97 + salt) * span as f32) as usize).min(span - 1);
        if !picks.contains(&i) {
            picks.push(i);
        }
        salt += 1;
    }
    picks
}

pub fn pick_from(today: &[usize], roll: f32) -> usize {
    let weights: Vec<f32> = today.iter().map(|i| [6.0, 3.0, 1.0, 0.3][rarity(*i)]).collect();
    let total: f32 = weights.iter().sum();
    let mut r = roll.clamp(0.0, 0.9999) * total;
    for (k, w) in weights.iter().enumerate() {
        if r < *w {
            return today[k];
        }
        r -= w;
    }
    today[0]
}

pub struct Look {
    pub cap: Color,
    pub spot: Option<Color>,
    pub stem: Color,
    pub shape: u8,
}

fn has(name: &str, words: &[&str]) -> bool {
    words.iter().any(|w| name.contains(w))
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

pub fn look(i: usize) -> Look {
    let name = SPECIES[i % COUNT];
    let cap = if has(name, &["fly agaric", "scarlet", "crimson", "ruby", "blusher", "bleeding", "beefsteak", "caesar"]) {
        rgb(214, 52, 48)
    } else if has(name, &["golden", "yellow", "sulphur", "chanterelle", "witch", "chicken", "saffron"]) {
        rgb(236, 180, 50)
    } else if has(name, &["orange", "terracotta", "tawny", "acid"]) {
        rgb(232, 120, 50)
    } else if has(name, &["amethyst", "violet", "lilac", "blewit", "indigo", "blue", "echo", "synth"]) {
        rgb(130, 100, 200)
    } else if has(name, &["green", "verdigris", "parrot", "aurora"]) {
        rgb(90, 170, 120)
    } else if has(name, &["pink", "rainbow", "quack"]) {
        rgb(240, 140, 180)
    } else if has(name, &["white", "snowy", "porcelain", "ghost", "button", "puffball", "ink cap", "lion", "moonlight", "coral", "miller", "enoki"]) {
        rgb(238, 234, 222)
    } else if has(name, &["black", "charcoal", "truffle", "horn of plenty", "dead man", "vinyl", "kick drum"]) {
        rgb(62, 52, 56)
    } else {
        let earthy = [rgb(150, 100, 60), rgb(176, 122, 72), rgb(120, 84, 56), rgb(196, 150, 100), rgb(140, 110, 80)];
        earthy[(hash(i, 3) * earthy.len() as f32) as usize % earthy.len()]
    };
    let spot = if has(name, &["fly agaric", "panther", "blusher", "quack", "melody"]) {
        Some(rgb(250, 246, 236))
    } else if has(name, &["bassline", "hi-hat"]) {
        Some(rgb(255, 214, 110))
    } else {
        None
    };
    let shape = if has(name, &["puffball", "truffle", "earthstar"]) {
        1
    } else if has(name, &["morel", "ink cap", "stinkhorn", "candlesnuff", "fingers", "bonnet", "mycena", "mosscap"]) {
        2
    } else if has(name, &["oyster", "polypore", "saddle", "turkey", "beefsteak", "chicken", "hen of", "ear", "porcelain", "tongue"]) {
        3
    } else if has(name, &["coral", "stagshorn", "cauliflower", "lion", "tooth", "enoki", "shimeji", "nameko"]) {
        4
    } else if has(name, &["parasol", "grisette", "caesar", "fly agaric", "panther", "blusher"]) {
        5
    } else {
        0
    };
    Look { cap, spot, stem: rgb(240, 230, 210), shape }
}

pub const ART: usize = 12;

pub fn art(i: usize) -> Vec<(usize, usize, Color)> {
    let l = look(i);
    let mut px = Vec::new();
    let dark = Color::new(l.cap.r * 0.75, l.cap.g * 0.75, l.cap.b * 0.75, 1.0);
    let mut put = |x: i32, y: i32, c: Color| {
        if x >= 0 && y >= 0 && (x as usize) < ART && (y as usize) < ART {
            px.push((x as usize, y as usize, c));
        }
    };
    match l.shape {
        1 => {
            for y in 4..11 {
                for x in 2..10 {
                    let dx = x as f32 - 5.5;
                    let dy = y as f32 - 7.5;
                    if dx * dx / 16.0 + dy * dy / 10.0 <= 1.0 {
                        put(x, y, if dy > 1.5 { dark } else { l.cap });
                    }
                }
            }
        }
        2 => {
            for y in 6..11 {
                put(5, y, l.stem);
                put(6, y, l.stem);
            }
            for y in 1..7 {
                let half = if y < 3 { 1 } else { 2 };
                for x in (6 - half)..(6 + half) {
                    put(x, y, if (x + y) % 2 == 0 { dark } else { l.cap });
                }
            }
        }
        3 => {
            for (y, (a, b)) in [(3, (2, 10)), (4, (1, 11)), (5, (2, 10)), (7, (3, 9)), (8, (2, 10))].iter() {
                for x in *a..*b {
                    put(x, *y, if x == *a || x == b - 1 { dark } else { l.cap });
                }
            }
            for y in 3..10 {
                put(1, y, l.stem);
            }
        }
        4 => {
            for k in 0..5 {
                let x = 2 + k * 2;
                for y in (2 + (k % 2))..10 {
                    put(x, y, if y % 3 == 0 { dark } else { l.cap });
                }
                put(x - 1, 3 + (k % 3), l.cap);
            }
            for x in 2..11 {
                put(x, 10, l.stem);
            }
        }
        5 => {
            for y in 5..11 {
                put(5, y, l.stem);
                put(6, y, l.stem);
            }
            for x in 0..12 {
                put(x, 4, dark);
            }
            for x in 1..11 {
                put(x, 3, l.cap);
            }
            for x in 3..9 {
                put(x, 2, l.cap);
            }
        }
        _ => {
            for y in 6..11 {
                put(4, y, l.stem);
                put(5, y, l.stem);
                put(6, y, l.stem);
            }
            for x in 1..11 {
                put(x, 5, dark);
            }
            for x in 1..11 {
                put(x, 4, l.cap);
            }
            for x in 2..10 {
                put(x, 3, l.cap);
            }
            for x in 3..9 {
                put(x, 2, l.cap);
            }
        }
    }
    if let Some(s) = l.spot {
        for (x, y) in [(3, 3), (7, 2), (8, 4), (5, 4)] {
            px.push((x, y, s));
        }
    }
    px
}

fn hash(i: usize, salt: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(374761393).wrapping_add(salt.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}
