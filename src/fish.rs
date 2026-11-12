use macroquad::prelude::*;

pub const COUNT: usize = 120;

pub const SPECIES: [&str; COUNT] = [
    "bluegill", "perch", "minnow", "roach", "rudd", "dace", "chub", "bream", "tench", "gudgeon",
    "bleak", "ruffe", "crucian carp", "common carp", "mirror carp", "grass carp", "rainbow trout", "brown trout", "brook trout", "lake trout",
    "grayling", "pike", "zander", "walleye", "smallmouth bass", "largemouth bass", "rock bass", "crappie", "pumpkinseed", "green sunfish",
    "catfish", "bullhead", "channel catfish", "eel", "loach", "stone loach", "weatherfish", "goldfish", "shubunkin", "comet",
    "ide", "golden orfe", "asp", "barbel", "nase", "vimba", "smelt", "stickleback", "sculpin", "goby",
    "oranda", "ryukin", "wels catfish", "lamprey", "burbot", "sterlet", "gar", "bowfin", "piranha", "pacu",
    "neon tetra", "guppy", "molly", "platy", "swordtail", "betta", "gourami", "angelfish", "discus", "oscar",
    "tilapia", "cichlid", "danio", "rasbora", "tiger barb", "killifish", "medaka", "ayu", "arctic char", "whitefish",
    "vendace", "herring", "sardine", "anchovy", "mackerel", "flounder", "sole", "plaice", "blenny", "pipefish",
    "salmon", "sockeye salmon", "coho salmon", "king salmon", "steelhead", "huchen", "taimen", "sturgeon", "paddlefish", "snakehead",
    "mudskipper", "pufferfish", "seahorse", "lionfish", "clownfish", "blue tang", "parrotfish", "butterflyfish", "arowana", "arapaima",
    "tuna", "swordfish", "marlin", "ocean sunfish", "opah", "golden koi", "platinum koi", "ghost koi", "rainbow koi", "dragon koi",
];

pub const RARITY_NAMES: [&str; 4] = ["common", "uncommon", "rare", "legendary"];
pub const RARITY_COLORS: [Color; 4] = [
    Color::new(0.85, 0.88, 0.92, 1.0),
    Color::new(0.55, 0.9, 0.6, 1.0),
    Color::new(0.5, 0.75, 1.0, 1.0),
    Color::new(1.0, 0.8, 0.3, 1.0),
];

pub fn rarity(i: usize) -> usize {
    match i {
        0..=49 => 0,
        50..=89 => 1,
        90..=111 => 2,
        _ => 3,
    }
}

pub fn acorns(i: usize) -> u32 {
    [2, 4, 8, 15][rarity(i)]
}

pub fn kind(i: usize) -> usize {
    [0, 1, 2, 2][rarity(i)]
}

pub fn code(i: usize) -> String {
    format!("#{:03}", i + 1)
}

pub fn pick(roll: f32, pick_roll: f32, luck: f32) -> usize {
    let weights = [10.0, 4.0 * (1.0 + luck), 1.4 * (1.0 + luck * 2.0), 0.35 * (1.0 + luck * 3.0)];
    let total: f32 = weights.iter().sum();
    let mut r = roll.clamp(0.0, 0.9999) * total;
    let mut tier = 0;
    for (k, w) in weights.iter().enumerate() {
        if r < *w {
            tier = k;
            break;
        }
        r -= w;
    }
    let range = match tier {
        0 => 0..50,
        1 => 50..90,
        2 => 90..112,
        _ => 112..120,
    };
    let span = range.end - range.start;
    range.start + ((pick_roll.clamp(0.0, 0.9999) * span as f32) as usize).min(span - 1)
}

pub fn hint(i: usize) -> &'static str {
    match rarity(i) {
        0 => "bites all day long",
        1 => "shows up now and then",
        2 => "rare - perfect timing helps",
        _ => "legendary - hit every ring perfectly",
    }
}

pub fn search(query: &str) -> Vec<usize> {
    let q = query.trim().trim_start_matches('#').to_lowercase();
    if q.is_empty() {
        return (0..COUNT).collect();
    }
    if let Ok(n) = q.parse::<usize>() {
        let exact: Vec<usize> = (0..COUNT).filter(|i| i + 1 == n).collect();
        let partial: Vec<usize> = (0..COUNT).filter(|i| i + 1 != n && format!("{:03}", i + 1).contains(&q)).collect();
        return exact.into_iter().chain(partial).collect();
    }
    let starts: Vec<usize> = (0..COUNT).filter(|i| SPECIES[*i].starts_with(&q)).collect();
    let contains: Vec<usize> = (0..COUNT).filter(|i| !SPECIES[*i].starts_with(&q) && SPECIES[*i].contains(&q)).collect();
    starts.into_iter().chain(contains).collect()
}

fn hash(i: usize, salt: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(374761393).wrapping_add(salt.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn shade(c: Color, k: f32) -> Color {
    Color::new((c.r * k).min(1.0), (c.g * k).min(1.0), (c.b * k).min(1.0), 1.0)
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Normal,
    Long,
    Round,
    Flat,
    Horse,
}

#[derive(Clone, Copy, PartialEq)]
enum Pattern {
    Plain,
    Spots,
    Stripes,
    Patches,
    Bands,
}

struct Look {
    shape: Shape,
    body: Color,
    belly: Color,
    fin: Color,
    mark: Color,
    pattern: Pattern,
}

fn look(i: usize) -> Look {
    let name = SPECIES[i];
    let has = |words: &[&str]| words.iter().any(|w| if w.contains(' ') { name.contains(w) } else { name.split(' ').any(|part| part == *w) });
    let shape = if has(&["eel", "lamprey", "pipefish", "gar", "loach", "weatherfish", "snakehead", "burbot", "paddlefish", "needle"]) {
        Shape::Long
    } else if has(&["puffer", "ocean sunfish", "opah", "discus", "angelfish", "pumpkinseed", "bluegill", "butterflyfish", "tang", "green sunfish", "crappie", "pacu", "piranha", "oranda", "ryukin", "bream"]) {
        Shape::Round
    } else if has(&["flounder", "sole", "plaice"]) {
        Shape::Flat
    } else if has(&["seahorse"]) {
        Shape::Horse
    } else {
        Shape::Normal
    };
    let h = hash(i, 3);
    let palettes = [
        (rgb(150, 160, 120), rgb(220, 220, 190), rgb(110, 120, 90)),
        (rgb(170, 180, 190), rgb(235, 235, 240), rgb(120, 130, 145)),
        (rgb(120, 140, 100), rgb(210, 200, 150), rgb(90, 105, 75)),
        (rgb(190, 160, 110), rgb(235, 220, 180), rgb(150, 120, 80)),
        (rgb(110, 130, 160), rgb(210, 220, 230), rgb(80, 95, 125)),
        (rgb(160, 120, 100), rgb(225, 200, 170), rgb(120, 85, 70)),
    ];
    let (mut body, mut belly, mut fin) = palettes[(h * palettes.len() as f32) as usize % palettes.len()];
    let mut mark = shade(body, 0.6);
    let mut pattern = match (hash(i, 9) * 5.0) as usize {
        0 => Pattern::Spots,
        1 => Pattern::Stripes,
        _ => Pattern::Plain,
    };
    let set = |b: Color, be: Color, f: Color, m: Color, p: Pattern, body: &mut Color, belly: &mut Color, fin: &mut Color, mark: &mut Color, pattern: &mut Pattern| {
        *body = b;
        *belly = be;
        *fin = f;
        *mark = m;
        *pattern = p;
    };
    if has(&["golden koi"]) {
        set(rgb(250, 190, 60), rgb(255, 230, 140), rgb(240, 160, 40), rgb(255, 245, 200), Pattern::Patches, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["platinum koi", "ghost koi"]) {
        set(rgb(235, 238, 245), rgb(255, 255, 255), rgb(200, 205, 215), rgb(170, 180, 195), Pattern::Patches, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["rainbow koi"]) {
        set(rgb(255, 120, 150), rgb(255, 230, 120), rgb(120, 200, 255), rgb(140, 230, 160), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["dragon koi"]) {
        set(rgb(200, 40, 50), rgb(255, 210, 90), rgb(255, 170, 40), rgb(40, 20, 30), Pattern::Patches, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["goldfish", "comet", "orfe", "oranda", "ryukin"]) {
        set(rgb(250, 140, 50), rgb(255, 200, 120), rgb(255, 170, 90), rgb(255, 240, 220), Pattern::Plain, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["shubunkin"]) {
        set(rgb(120, 150, 220), rgb(240, 240, 250), rgb(250, 150, 80), rgb(40, 40, 60), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["trout", "char", "huchen", "taimen", "steelhead"]) {
        set(rgb(140, 150, 110), rgb(240, 200, 190), rgb(120, 130, 95), rgb(60, 60, 50), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
        if has(&["rainbow", "steelhead"]) {
            pattern = Pattern::Bands;
            mark = rgb(230, 120, 140);
        }
    } else if has(&["salmon"]) {
        set(rgb(200, 205, 215), rgb(245, 200, 190), rgb(160, 165, 175), rgb(230, 110, 100), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
        if has(&["sockeye"]) {
            body = rgb(210, 60, 60);
            fin = rgb(90, 130, 90);
            pattern = Pattern::Plain;
        }
    } else if has(&["catfish", "bullhead", "burbot", "sculpin", "goby", "mudskipper"]) {
        set(rgb(110, 95, 75), rgb(190, 175, 150), rgb(85, 72, 58), rgb(70, 60, 48), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["clownfish"]) {
        set(rgb(250, 130, 40), rgb(255, 170, 80), rgb(250, 130, 40), rgb(255, 255, 255), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["lionfish"]) {
        set(rgb(240, 230, 220), rgb(250, 245, 240), rgb(200, 70, 60), rgb(190, 60, 50), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["tiger", "perch", "zander", "walleye"]) {
        set(rgb(170, 175, 90), rgb(230, 230, 180), rgb(220, 120, 70), rgb(70, 75, 40), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["neon", "tang", "betta", "gourami"]) {
        set(rgb(60, 110, 230), rgb(120, 200, 255), rgb(230, 70, 90), rgb(90, 230, 255), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
        if has(&["tang"]) {
            fin = rgb(255, 220, 60);
            mark = rgb(30, 40, 90);
        }
    } else if has(&["parrotfish", "guppy", "swordtail", "platy", "molly"]) {
        set(rgb(90, 200, 170), rgb(250, 220, 120), rgb(240, 120, 180), rgb(250, 240, 120), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["butterflyfish", "angelfish", "discus"]) {
        set(rgb(250, 220, 90), rgb(255, 245, 200), rgb(250, 200, 60), rgb(40, 40, 50), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["piranha"]) {
        set(rgb(140, 145, 150), rgb(230, 80, 60), rgb(110, 115, 120), rgb(80, 85, 90), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["tuna", "mackerel", "herring", "sardine", "anchovy", "marlin", "swordfish", "bleak", "smelt", "vendace", "whitefish"]) {
        set(rgb(60, 90, 140), rgb(225, 230, 240), rgb(50, 70, 110), rgb(30, 50, 90), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
        if !has(&["mackerel"]) {
            pattern = Pattern::Plain;
        }
    } else if has(&["pufferfish"]) {
        set(rgb(230, 210, 150), rgb(250, 245, 225), rgb(210, 190, 130), rgb(120, 100, 60), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["seahorse"]) {
        set(rgb(250, 180, 80), rgb(255, 220, 140), rgb(240, 150, 60), rgb(200, 120, 40), Pattern::Stripes, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["arowana", "arapaima"]) {
        set(rgb(200, 170, 110), rgb(240, 225, 190), rgb(200, 80, 60), rgb(220, 60, 50), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["ocean sunfish", "opah"]) {
        set(rgb(160, 170, 190), rgb(220, 225, 235), rgb(200, 90, 90), rgb(240, 240, 255), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["sturgeon", "sterlet", "paddlefish"]) {
        set(rgb(110, 115, 120), rgb(215, 215, 210), rgb(85, 90, 95), rgb(230, 230, 225), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["eel", "lamprey"]) {
        set(rgb(80, 85, 60), rgb(170, 165, 120), rgb(60, 65, 45), rgb(50, 55, 40), Pattern::Plain, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["pike", "gar", "snakehead", "bowfin"]) {
        set(rgb(100, 130, 80), rgb(220, 220, 170), rgb(150, 120, 70), rgb(200, 210, 150), Pattern::Spots, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    } else if has(&["rudd", "roach"]) {
        fin = rgb(220, 70, 60);
    } else if has(&["bluegill", "pumpkinseed", "green sunfish"]) {
        set(rgb(90, 130, 140), rgb(240, 170, 80), rgb(70, 100, 115), rgb(40, 60, 90), Pattern::Bands, &mut body, &mut belly, &mut fin, &mut mark, &mut pattern);
    }
    Look { shape, body, belly, fin, mark, pattern }
}

pub const ART_W: usize = 18;
pub const ART_H: usize = 11;

pub fn art(i: usize) -> Vec<(usize, usize, Color)> {
    let l = look(i);
    let mut px: Vec<(usize, usize, Color)> = Vec::new();
    let cy = ART_H as f32 / 2.0 - 0.5;
    let (len, half_h, x0) = match l.shape {
        Shape::Normal => (11.0 + hash(i, 1) * 3.0, 2.3 + hash(i, 2) * 1.2, 3.0),
        Shape::Long => (14.5, 1.2, 1.0),
        Shape::Round => (9.0, 3.8, 4.0),
        Shape::Flat => (11.0, 2.6, 3.0),
        Shape::Horse => (0.0, 0.0, 0.0),
    };
    if l.shape == Shape::Horse {
        let rows = ["....xxx...", "...xxxxe..", "....xxxxxx", "....xxx...", "...xxxx...", "...xxxf...", "....xxx...", ".....xx...", "....x.x...", ".....xx..."];
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let col = match ch {
                    'x' => if (x + y) % 3 == 0 { l.mark } else { l.body },
                    'e' => rgb(20, 20, 30),
                    'f' => l.fin,
                    _ => continue,
                };
                px.push((x + 4, y, col));
            }
        }
        return px;
    }
    let head = x0 + len;
    for y in 0..ART_H {
        for x in 0..ART_W {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5 - 0.5;
            let t = (fx - x0) / len;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let profile = match l.shape {
                Shape::Round => (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt(),
                Shape::Long => if t > 0.9 { (1.0 - t) * 10.0 } else { 1.0 },
                _ => (t.powf(0.7) * (1.0 - t).powf(0.45) * 2.3).min(1.0),
            };
            let reach = half_h * profile;
            let dy = fy - cy;
            if dy.abs() > reach + 0.15 {
                continue;
            }
            let mut col = if dy > reach * 0.25 { l.belly } else { l.body };
            match l.pattern {
                Pattern::Spots if hash(x * 31 + y, i as u32) > 0.78 && dy < reach * 0.4 => col = l.mark,
                Pattern::Stripes if x % 3 == 0 && t > 0.15 && t < 0.85 => col = l.mark,
                Pattern::Patches if hash(x / 2 + (y / 2) * 7, i as u32 + 5) > 0.55 => col = l.mark,
                Pattern::Bands if (dy + reach * 0.1).abs() < 0.6 && t > 0.12 && t < 0.88 => col = l.mark,
                _ => {}
            }
            if dy.abs() > reach - 0.6 && dy < 0.0 {
                col = shade(col, 0.82);
            }
            px.push((x, y, col));
        }
    }
    let tail_x = x0 as usize;
    let spread = match l.shape {
        Shape::Long => 1,
        Shape::Round => 2,
        _ => 3,
    };
    for k in 0..3usize {
        let x = tail_x.saturating_sub(k + 1);
        let span = (k + 1).min(spread) as i32;
        for d in -span..=span {
            let y = cy.round() as i32 + d;
            if y >= 0 && (y as usize) < ART_H && (d.abs() as usize) >= k.saturating_sub(1) {
                px.push((x, y as usize, l.fin));
            }
        }
    }
    let top = (cy - half_h - 0.2).round().max(0.0) as usize;
    let fin_start = (x0 + len * 0.35) as usize;
    for x in fin_start..fin_start + 3 {
        if x < ART_W {
            px.push((x, top, l.fin));
        }
    }
    let eye_x = (head - 2.2).max(0.0) as usize;
    let eye_y = (cy - half_h * 0.3).round().max(0.0) as usize;
    if eye_x < ART_W {
        px.push((eye_x, eye_y, rgb(255, 255, 255)));
        if eye_x + 1 < ART_W {
            px.push((eye_x + 1, eye_y, rgb(20, 20, 30)));
        }
    }
    if name_has(i, "swordfish") || name_has(i, "marlin") {
        for x in head as usize..(head as usize + 3).min(ART_W) {
            px.push((x, cy.round() as usize, shade(l.body, 0.8)));
        }
    }
    px
}

fn name_has(i: usize, w: &str) -> bool {
    SPECIES[i].contains(w)
}
