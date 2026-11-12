use macroquad::prelude::*;

pub const COUNT: usize = 12;

pub const NAMES: [&str; COUNT] = ["heart", "smile", "laugh", "wow", "sad", "sleepy", "music", "star", "fish", "acorn", "fire", "huh"];

const ART: [[&str; 8]; COUNT] = [
    [".rr..rr.", "rwrrrrrr", "rwrrrrrr", "rrrrrrrr", ".rrrrrr.", "..rrrr..", "...rr...", "........"],
    ["..yyyy..", ".yyyyyy.", "yykyykyy", "yyyyyyyy", "ykyyyyky", "yykkkkyy", ".yyyyyy.", "..yyyy.."],
    ["..yyyy..", ".yyyyyy.", "ykkyykky", "yyyyyyyy", "ykkkkkky", "ykppppky", ".ykkkky.", "..yyyy.."],
    ["..yyyy..", ".yyyyyy.", "ywkyywky", "yyyyyyyy", "yyykkyyy", "yyykkyyy", ".yyyyyy.", "..yyyy.."],
    ["..yyyy..", ".yyyyyy.", "yykyykyy", "ybyyyyyy", "ybykkyyy", "yykyykyy", ".yyyyyy.", "..yyyy.."],
    ["......kk", "..yyyyk.", ".yyyyykk", "ykkyykky", "yyyyyyyy", "yyykkyyy", ".yyyyyy.", "..yyyy.."],
    ["...ppppp", "...p...p", "...p...p", "...p...p", ".ppp.ppp", "pppp.ppp", ".pp...pp", "........"],
    ["...yy...", "...yy...", "yyyyyyyy", ".yyyyyy.", "..yyyy..", ".yyyyyy.", ".yy..yy.", "y......y"],
    ["........", "b...bbb.", "bb.bbbbb", "bbbbbbwk", "bbbbbbbb", "bb.bbbbb", "b...bbb.", "........"],
    ["...g....", "..nnnn..", ".nnnnnn.", ".hhhhhh.", ".hhhhhh.", ".hhhhhh.", "..hhhh..", "...hh..."],
    ["...r....", "..rr..r.", "..rro.rr", ".rroorr.", ".rooyorr", "rroyyorr", ".rooyor.", "..rrrr.."],
    ["..kkkk..", ".kk..kk.", ".....kk.", "....kk..", "...kk...", "...kk...", "........", "...kk..."],
];

fn color(ch: char) -> Option<Color> {
    Some(match ch {
        'r' => Color::from_rgba(240, 70, 90, 255),
        'w' => Color::from_rgba(255, 255, 255, 255),
        'y' => Color::from_rgba(255, 212, 70, 255),
        'k' => Color::from_rgba(50, 36, 30, 255),
        'p' => Color::from_rgba(150, 110, 240, 255),
        'b' => Color::from_rgba(90, 170, 240, 255),
        'g' => Color::from_rgba(110, 180, 80, 255),
        'n' => Color::from_rgba(120, 84, 50, 255),
        'h' => Color::from_rgba(200, 140, 70, 255),
        'o' => Color::from_rgba(255, 150, 50, 255),
        _ => return None,
    })
}

pub fn pixels(id: usize) -> Vec<(usize, usize, Color)> {
    let mut out = Vec::new();
    for (y, row) in ART[id % COUNT].iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some(c) = color(ch) {
                out.push((x, y, c));
            }
        }
    }
    out
}

pub fn reply(id: usize, roll: f32) -> usize {
    let pick = |a: usize, b: usize| if roll < 0.5 { a } else { b };
    match NAMES[id % COUNT] {
        "heart" => pick(0, 1),
        "smile" => pick(2, 0),
        "laugh" => pick(2, 1),
        "wow" => pick(3, 7),
        "sad" => pick(0, 0),
        "sleepy" => pick(5, 0),
        "music" => pick(6, 7),
        "star" => pick(7, 2),
        "fish" => pick(8, 3),
        "acorn" => pick(9, 1),
        "fire" => pick(10, 6),
        _ => pick(11, 1),
    }
}
