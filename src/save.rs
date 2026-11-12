#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const KINDS: [Kind; 15] = [
    Kind::Grass, Kind::Flowers, Kind::Sprout, Kind::Tree, Kind::Water, Kind::Ice, Kind::Ash, Kind::Stone,
    Kind::House, Kind::Campfire, Kind::Path, Kind::WoodWall, Kind::StoneWall, Kind::Floor, Kind::Dock,
];

pub(crate) struct Progress {
    pub(crate) tiles: Vec<(u8, u8)>,
    pub(crate) wood: u32,
    pub(crate) stone: u32,
    pub(crate) fish: [u32; 3],
    pub(crate) fuel: f32,
    pub(crate) lit: bool,
    pub(crate) pile: u32,
    pub(crate) dishes: Vec<usize>,
    pub(crate) cook_progress: u32,
    pub(crate) characters: [usize; 2],
    pub(crate) spots: [Vec2; 2],
    pub(crate) echo: bool,
    pub(crate) beat: bool,
    pub(crate) bpm: f32,
    pub(crate) quantise: u8,
    pub(crate) notes: Vec<(f32, usize)>,
    pub(crate) songbook: Vec<Vec<(usize, usize)>>,
    pub(crate) partner_xp: u32,
    pub(crate) mix: [f32; 5],
    pub(crate) acorns: u32,
    pub(crate) pieces: Vec<home::Piece>,
    pub(crate) styles: (u8, u8, u16, bool),
    pub(crate) diary: Vec<DiaryEntry>,
    pub(crate) fish_lesson: bool,
    pub(crate) cook_lesson: bool,
    pub(crate) gather_lesson: bool,
    pub(crate) room: u8,
    pub(crate) dex: Vec<u32>,
    pub(crate) mush: [u32; 3],
    pub(crate) shroomdex: Vec<u32>,
    pub(crate) quests: Option<(i64, Vec<(u8, u32, u32, bool)>)>,
    pub(crate) stored: Vec<u8>,
    pub(crate) seq: [u16; 4],
    pub(crate) name: String,
}

pub(crate) fn kind_index(kind: Kind) -> u8 {
    KINDS.iter().position(|k| *k == kind).unwrap_or(0) as u8
}

pub(crate) fn encode_progress(p: &Progress) -> String {
    let list = |v: Vec<String>| v.join(",");
    let mut out = String::from("melody-quack-save 1\n");
    out += &format!("tiles {}\n", list(p.tiles.iter().map(|(k, s)| format!("{k}:{s}")).collect()));
    out += &format!("inv {} {} {} {} {}\n", p.wood, p.stone, p.fish[0], p.fish[1], p.fish[2]);
    out += &format!("camp {:.1} {} {}\n", p.fuel, p.lit as u8, p.pile);
    out += &format!("dishes {}\n", list(p.dishes.iter().map(|d| d.to_string()).collect()));
    out += &format!("cook {}\n", p.cook_progress);
    out += &format!("chars {} {}\n", p.characters[0], p.characters[1]);
    out += &format!("spots {:.2} {:.2} {:.2} {:.2}\n", p.spots[0].x, p.spots[0].y, p.spots[1].x, p.spots[1].y);
    out += &format!("sound {} {} {:.1} {}\n", p.echo as u8, p.beat as u8, p.bpm, p.quantise);
    out += &format!("notes {}\n", list(p.notes.iter().map(|(t, id)| format!("{t:.3}:{id}")).collect()));
    let tunes: Vec<String> = p.songbook.iter().map(|tune| list(tune.iter().map(|(at, n)| format!("{at}:{n}")).collect())).collect();
    out += &format!("songbook {}\n", tunes.join(";"));
    out += &format!("partner {}\n", p.partner_xp);
    out += &format!("mix {:.2} {:.2} {:.2} {:.2} {:.2}\n", p.mix[0], p.mix[1], p.mix[2], p.mix[3], p.mix[4]);
    out += &format!("acorns {}\n", p.acorns);
    out += &format!("lessons {} {} {}\n", p.fish_lesson as u8, p.cook_lesson as u8, p.gather_lesson as u8);
    out += &format!("room {}\n", p.room);
    out += &format!("dex {}\n", list(p.dex.iter().map(|n| n.to_string()).collect()));
    out += &format!("mush {} {} {}\n", p.mush[0], p.mush[1], p.mush[2]);
    out += &format!("shroomdex {}\n", list(p.shroomdex.iter().map(|n| n.to_string()).collect()));
    if !p.name.is_empty() {
        out += &format!("name {}\n", p.name);
    }
    if p.seq.iter().any(|r| *r != 0) {
        out += &format!("seq {} {} {} {}\n", p.seq[0], p.seq[1], p.seq[2], p.seq[3]);
    }
    if !p.stored.is_empty() {
        out += &format!("stored {}\n", list(p.stored.iter().map(|n| n.to_string()).collect()));
    }
    if let Some((day, qs)) = &p.quests {
        out += &format!("quests {} {}\n", day, list(qs.iter().map(|(k, t, pr, d)| format!("{}:{}:{}:{}", k, t, pr, *d as u8)).collect()));
    }
    out += &format!("home {}\n", list(p.pieces.iter().map(|q| if q.day != 0 { format!("{}:{}:{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot, q.state, q.day) } else if q.state == 0 { format!("{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot) } else { format!("{}:{}:{}:{}:{}", q.item.index(), q.x, q.z, q.rot, q.state) }).collect()));
    out += &format!("style {} {} {} {}\n", p.styles.0, p.styles.1, p.styles.2, p.styles.3 as u8);
    let keep_from = p.diary.len().saturating_sub(10);
    for (k, e) in p.diary.iter().enumerate() {
        let events = if k >= keep_from { e.events.iter().map(|(t, id, v)| format!("{:.2}:{}:{:.2}", t, id, v)).collect::<Vec<_>>().join(";") } else { String::new() };
        out += &format!("diary {:.0}|{}|{}|{}|{}|{}\n", e.when, e.seconds, e.stars, e.who.0, e.who.1, events);
    }
    out
}

pub(crate) fn decode_progress(text: &str, characters: usize) -> Option<Progress> {
    let mut lines = text.lines();
    if lines.next()? != "melody-quack-save 1" {
        return None;
    }
    let mut p = Progress {
        tiles: Vec::new(),
        wood: 0,
        stone: 0,
        fish: [0; 3],
        fuel: 0.0,
        lit: false,
        pile: 0,
        dishes: Vec::new(),
        cook_progress: 0,
        characters: [0, 4],
        spots: [vec2(6.5, 16.5), vec2(7.5, 16.5)],
        echo: false,
        beat: false,
        bpm: DEFAULT_BPM,
        quantise: 1,
        notes: Vec::new(),
        songbook: Vec::new(),
        partner_xp: 0,
        mix: [1.0; 5],
        acorns: 0,
        pieces: Vec::new(),
        styles: (0, 0, 1 | (1 << 4), true),
        diary: Vec::new(),
        fish_lesson: false,
        cook_lesson: false,
        gather_lesson: false,
        room: 0,
        dex: vec![0; fish::COUNT],
        mush: [0; 3],
        shroomdex: vec![0; mushroom::COUNT],
        quests: None,
        stored: Vec::new(),
        seq: [0; 4],
        name: String::new(),
    };
    for line in lines {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        let nums: Vec<f32> = rest.split_whitespace().filter_map(|v| v.parse().ok()).collect();
        let pairs = || rest.split(',').filter_map(|v| v.split_once(':')).collect::<Vec<_>>();
        match key {
            "tiles" => {
                p.tiles = pairs().iter().filter_map(|(k, s)| Some((k.parse().ok()?, s.parse().ok()?))).collect();
            }
            "inv" if nums.len() == 5 => {
                p.wood = nums[0] as u32;
                p.stone = nums[1] as u32;
                p.fish = [nums[2] as u32, nums[3] as u32, nums[4] as u32];
            }
            "camp" if nums.len() == 3 => {
                p.fuel = nums[0];
                p.lit = nums[1] > 0.0;
                p.pile = nums[2] as u32;
            }
            "dishes" => p.dishes = rest.split(',').filter_map(|v| v.parse().ok()).filter(|d| *d < DISHES.len()).collect(),
            "cook" if nums.len() == 1 => p.cook_progress = (nums[0] as u32).min(COOK_GOAL),
            "chars" if nums.len() == 2 => p.characters = [(nums[0] as usize).min(characters - 1), (nums[1] as usize).min(characters - 1)],
            "spots" if nums.len() == 4 => p.spots = [vec2(nums[0], nums[1]), vec2(nums[2], nums[3])],
            "sound" if nums.len() == 4 => {
                p.echo = nums[0] > 0.0;
                p.beat = nums[1] > 0.0;
                p.bpm = nums[2].clamp(60.0, 200.0);
                p.quantise = nums[3] as u8;
            }
            "notes" => {
                p.notes = pairs().iter().filter_map(|(t, id)| Some((t.parse::<f32>().ok()?, id.parse::<usize>().ok()?))).filter(|(t, id)| *id < 32 && t.is_finite()).collect();
            }
            "songbook" => {
                p.songbook = rest
                    .split(';')
                    .map(|tune| tune.split(',').filter_map(|v| v.split_once(':')).filter_map(|(a, n)| Some((a.parse::<usize>().ok()?, n.parse::<usize>().ok()?))).filter(|(a, n)| *a < LOOP_STEPS && *n < 6).collect::<Vec<_>>())
                    .filter(|tune| !tune.is_empty())
                    .take(8)
                    .collect();
            }
            "partner" if nums.len() == 1 => p.partner_xp = nums[0] as u32,
            "acorns" if nums.len() == 1 => p.acorns = nums[0] as u32,
            "lessons" if !nums.is_empty() => {
                p.fish_lesson = nums[0] > 0.0;
                if nums.len() > 1 {
                    p.cook_lesson = nums[1] > 0.0;
                }
                if nums.len() > 2 {
                    p.gather_lesson = nums[2] > 0.0;
                }
            },
            "room" if nums.len() == 1 => p.room = (nums[0].max(0.0) as u8).min(home::MAX_SIZE),
            "dex" => {
                for (k, v) in rest.split(',').enumerate().take(fish::COUNT) {
                    p.dex[k] = v.trim().parse().unwrap_or(0);
                }
            }
            "mush" if nums.len() == 3 => p.mush = [nums[0] as u32, nums[1] as u32, nums[2] as u32],
            "name" => p.name = rest.chars().filter(|c| !c.is_control()).take(12).collect::<String>().trim().to_string(),
            "seq" if nums.len() == 4 => p.seq = [nums[0] as u16, nums[1] as u16, nums[2] as u16, nums[3] as u16],
            "stored" => p.stored = rest.split(',').filter_map(|v| v.trim().parse::<u8>().ok()).filter(|v| (*v as usize) < home::ITEMS.len()).collect(),
            "quests" => {
                let mut parts = rest.splitn(2, ' ');
                if let (Some(day), Some(items)) = (parts.next().and_then(|d| d.trim().parse::<i64>().ok()), parts.next()) {
                    let qs: Vec<(u8, u32, u32, bool)> = items
                        .split(',')
                        .filter_map(|q| {
                            let f: Vec<u32> = q.split(':').filter_map(|n| n.trim().parse().ok()).collect();
                            if f.len() == 4 { Some((f[0] as u8, f[1], f[2], f[3] > 0)) } else { None }
                        })
                        .collect();
                    p.quests = Some((day, qs));
                }
            }
            "shroomdex" => {
                for (k, v) in rest.split(',').enumerate().take(mushroom::COUNT) {
                    p.shroomdex[k] = v.trim().parse().unwrap_or(0);
                }
            }
            "diary" => {
                let parts: Vec<&str> = rest.splitn(6, '|').collect();
                if parts.len() == 6 {
                    let events = parts[5]
                        .split(';')
                        .filter_map(|ev| {
                            let f: Vec<&str> = ev.split(':').collect();
                            if f.len() == 3 {
                                Some((f[0].parse::<f32>().ok()?, f[1].parse::<u16>().ok()?, f[2].parse::<f32>().ok()?.clamp(0.0, 1.5)))
                            } else {
                                None
                            }
                        })
                        .collect();
                    if let (Ok(when), Ok(seconds), Ok(stars), Ok(a), Ok(b)) = (parts[0].parse::<f64>(), parts[1].parse::<u32>(), parts[2].parse::<u8>(), parts[3].parse::<usize>(), parts[4].parse::<usize>()) {
                        p.diary.push(DiaryEntry { when, seconds, stars: stars.min(3), who: (a.min(characters - 1), b.min(characters - 1)), events });
                    }
                }
            }
            "home" => {
                p.pieces = rest
                    .split(',')
                    .filter_map(|v| {
                        let parts: Vec<i32> = v.split(':').filter_map(|n| n.parse().ok()).collect();
                        if (4..=6).contains(&parts.len()) {
                            Some(home::Piece { item: home::Item::from_index(parts[0].max(0) as u8), x: parts[1], z: parts[2], rot: (parts[3].rem_euclid(4)) as u8, state: parts.get(4).copied().unwrap_or(0).clamp(0, 255) as u8, day: parts.get(5).copied().unwrap_or(0) })
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            "style" if nums.len() == 4 => {
                p.styles = ((nums[0] as u8).min(home::WALLS.len() as u8 - 1), (nums[1] as u8).min(home::FLOORS.len() as u8 - 1), nums[2] as u16, nums[3] > 0.0);
            }
            "mix" if nums.len() == 5 => {
                for (k, v) in nums.iter().enumerate() {
                    p.mix[k] = v.clamp(0.0, 1.0);
                }
            }
            _ => {}
        }
    }
    Some(p)
}
