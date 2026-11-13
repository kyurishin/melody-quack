#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn draw_cat(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, meowing: bool, dance: f32, look: &CatLook) {
    let fur = look.fur;
    let dark = tinted(fur, -0.5);
    let pink = Color::from_rgba(255, 160, 175, 255);

    let bob = if walking { (time * 14.0).sin().abs() * 0.05 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 14.0).sin() * 0.05 } else { 0.0 };

    for (lx, lz, phase) in [(-0.1, 0.12, 1.0), (0.1, 0.12, -1.0), (-0.1, -0.18, -1.0), (0.1, -0.18, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.08, lz + step * phase), vec3(0.08, 0.16, 0.08), fur);
    }
    b.box_at(o, yaw, vec3(0.0, 0.27, -0.03), vec3(0.32, 0.24, 0.44), fur);
    if let Some(st) = look.stripes {
        for k in 0..3 {
            b.box_at(o, yaw, vec3(0.0, 0.392, -0.16 + k as f32 * 0.12), vec3(0.3, 0.01, 0.05), st);
        }
    }

    for k in 0..3 {
        let kf = k as f32;
        let sway = (time * 3.0 - kf * 0.6).sin() * 0.05 * (kf + 1.0);
        let c = match look.stripes {
            Some(st) if k == 1 => st,
            _ => fur,
        };
        b.box_at(o, yaw, vec3(sway, 0.36 + kf * 0.1, -0.28 - kf * 0.04), vec3(0.07, 0.1, 0.07), c);
    }

    b.box_at(o, yaw, vec3(0.0, 0.54, 0.16), vec3(0.4, 0.32, 0.32), fur);
    if let Some(st) = look.stripes {
        for dx in [-0.08_f32, 0.0, 0.08] {
            b.box_at(o, yaw, vec3(dx, 0.68, 0.321), vec3(0.03, 0.06, 0.01), st);
        }
    }
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.15, 0.76, 0.16), vec3(0.1, 0.13, 0.08), fur);
        b.box_at(o, yaw, vec3(side * 0.15, 0.75, 0.205), vec3(0.05, 0.08, 0.01), pink);
    }

    let blinking = time % 3.4 < 0.12;
    for side in [-1.0_f32, 1.0] {
        if blinking || meowing {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.08, 0.025, 0.01), dark);
        } else if let Some(eye) = look.eye {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.07, 0.085, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.325), vec3(0.055, 0.07, 0.01), eye);
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.328), vec3(0.022, 0.065, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09 - 0.012, 0.58, 0.331), vec3(0.016, 0.016, 0.01), WHITE);
        } else {
            b.box_at(o, yaw, vec3(side * 0.09, 0.56, 0.322), vec3(0.06, 0.08, 0.01), Color::from_rgba(25, 20, 30, 255));
            b.box_at(o, yaw, vec3(side * 0.09 + 0.012, 0.575, 0.328), vec3(0.02, 0.02, 0.01), WHITE);
        }
        b.box_at(o, yaw, vec3(side * 0.14, 0.49, 0.323), vec3(0.06, 0.035, 0.01), Color::from_rgba(255, 175, 185, 255));
    }
    b.box_at(o, yaw, vec3(0.0, 0.5, 0.324), vec3(0.04, 0.03, 0.01), pink);
    if meowing {
        b.box_at(o, yaw, vec3(0.0, 0.45, 0.324), vec3(0.05, 0.04, 0.01), Color::from_rgba(150, 70, 80, 255));
    }

}

pub(crate) fn draw_carry(b: &mut Batch, o: Vec3, yaw: f32, at: Vec3, carry: Option<Carry>) {
    match carry {
        Some(Carry::Stick) => {
            b.box_at(o, yaw, at, vec3(0.42, 0.05, 0.05), Color::from_rgba(130, 90, 56, 255));
            b.box_at(o, yaw, at + vec3(0.1, 0.04, 0.0), vec3(0.08, 0.06, 0.06), Color::from_rgba(104, 170, 78, 255));
        }
        Some(Carry::Stone) => b.box_at(o, yaw, at + vec3(0.0, 0.0, 0.02), vec3(0.13, 0.1, 0.11), Color::from_rgba(160, 162, 176, 255)),
        Some(Carry::Acorn) => {
            b.box_at(o, yaw, at + vec3(0.0, -0.01, 0.02), vec3(0.09, 0.1, 0.09), ACORN_COLOR);
            b.box_at(o, yaw, at + vec3(0.0, 0.05, 0.02), vec3(0.11, 0.04, 0.11), Color::from_rgba(120, 84, 50, 255));
        }
        None => {}
    }
}

pub(crate) fn draw_dog(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, barking: bool, dance: f32, carry: Option<Carry>, look: &DogLook) {
    let fur = look.fur;
    let nose = Color::from_rgba(56, 38, 32, 255);
    let ink = Color::from_rgba(30, 22, 20, 255);

    let bob = if walking { (time * 12.0).sin().abs() * 0.05 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 12.0).sin() * 0.06 } else { 0.0 };

    for (lx, lz, phase) in [(-0.11, 0.16, 1.0), (0.11, 0.16, -1.0), (-0.11, -0.2, -1.0), (0.11, -0.2, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.1, lz + step * phase), vec3(0.09, 0.2, 0.09), fur);
    }
    b.box_at(o, yaw, vec3(0.0, 0.32, -0.02), vec3(0.3, 0.25, 0.56), fur);
    b.box_at(o, yaw, vec3(0.0, 0.25, 0.1), vec3(0.24, 0.12, 0.3), look.muzzle);

    let wag = (time * 14.0).sin() * 0.09;
    match look.tail {
        Some(false) => {
            b.box_at(o, yaw, vec3(wag * 0.5, 0.42, -0.33), vec3(0.06, 0.06, 0.1), fur);
            b.box_at(o, yaw, vec3(wag, 0.47, -0.4), vec3(0.06, 0.06, 0.1), fur);
        }
        Some(true) => {
            b.box_at(o, yaw, vec3(wag * 0.3, 0.48, -0.3), vec3(0.1, 0.1, 0.1), fur);
            b.box_at(o, yaw, vec3(wag * 0.5, 0.56, -0.22), vec3(0.09, 0.08, 0.1), fur);
            b.box_at(o, yaw, vec3(wag * 0.6, 0.52, -0.13), vec3(0.08, 0.07, 0.08), look.muzzle);
        }
        None => {}
    }

    b.box_at(o, yaw, vec3(0.0, 0.58, 0.25), vec3(0.32, 0.28, 0.3), fur);
    b.box_at(o, yaw, vec3(0.0, 0.52, 0.44), vec3(0.18, 0.13, 0.16), look.muzzle);
    b.box_at(o, yaw, vec3(0.0, 0.57, 0.525), vec3(0.08, 0.06, 0.03), nose);
    if let Some(c) = look.collar {
        b.box_at(o, yaw, vec3(0.0, 0.45, 0.2), vec3(0.32, 0.05, 0.2), c);
        b.box_at(o, yaw, vec3(0.0, 0.41, 0.31), vec3(0.05, 0.05, 0.02), Color::from_rgba(240, 190, 70, 255));
    }

    let flop = if walking { (time * 12.0).sin() * 0.02 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        if look.tail == Some(true) {
            b.box_at(o, yaw, vec3(side * 0.11, 0.76, 0.22), vec3(0.09, 0.12, 0.06), look.ear);
        } else if look.tail.is_none() {
            b.box_at(o, yaw, vec3(side * 0.2, 0.7 + flop * side, 0.22), vec3(0.14, 0.12, 0.05), look.ear);
            b.box_at(o, yaw, vec3(side * 0.25, 0.77 + flop * side, 0.22), vec3(0.08, 0.08, 0.05), tinted(look.ear, -0.15));
        } else {
            b.box_at(o, yaw, vec3(side * 0.185, 0.54 + flop * side, 0.24), vec3(0.06, 0.2, 0.14), look.ear);
        }
        let blinking = time % 4.1 < 0.12;
        if blinking || barking {
            b.box_at(o, yaw, vec3(side * 0.08, 0.63, 0.402), vec3(0.06, 0.02, 0.01), ink);
        } else {
            b.box_at(o, yaw, vec3(side * 0.08, 0.63, 0.402), vec3(0.05, 0.06, 0.01), ink);
            b.box_at(o, yaw, vec3(side * 0.08 + 0.012, 0.645, 0.408), vec3(0.018, 0.018, 0.01), WHITE);
        }
    }
    if barking {
        b.box_at(o, yaw, vec3(0.0, 0.445, 0.47), vec3(0.14, 0.03, 0.1), Color::from_rgba(120, 50, 50, 255));
        b.box_at(o, yaw, vec3(0.0, 0.43, 0.5), vec3(0.06, 0.02, 0.08), Color::from_rgba(255, 140, 160, 255));
    }
    draw_carry(b, o, yaw, vec3(0.0, 0.47, 0.5), carry);
}

pub(crate) fn draw_duck(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, quacking: bool, dance: f32, carry: Option<Carry>) {
    let white = Color::from_rgba(250, 250, 244, 255);
    let wing = Color::from_rgba(222, 224, 228, 255);
    let orange = Color::from_rgba(250, 156, 50, 255);
    let ink = Color::from_rgba(26, 24, 30, 255);
    let waddle = if walking { (time * 12.0).sin() * 0.12 } else { 0.0 };
    let bob = if walking { (time * 12.0).sin().abs() * 0.03 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let y2 = yaw + waddle * 0.6;
    for (side, phase) in [(-1.0_f32, 1.0_f32), (1.0, -1.0)] {
        let lift = if walking { ((time * 12.0).sin() * phase).max(0.0) * 0.04 } else { 0.0 };
        b.box_at(o, yaw, vec3(side * 0.07, 0.015 + lift, 0.06), vec3(0.1, 0.03, 0.14), orange);
        b.box_at(o, yaw, vec3(side * 0.07, 0.06 + lift, 0.02), vec3(0.04, 0.07, 0.04), orange);
    }
    b.box_at(o, y2, vec3(0.0, 0.22, -0.02), vec3(0.3, 0.22, 0.38), white);
    b.box_at(o, y2, vec3(0.0, 0.3, -0.24), vec3(0.14, 0.1, 0.1), white);
    let flap = if quacking || dance > 0.3 { (time * 30.0).sin().abs() * 0.08 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, y2, vec3(side * (0.165 + flap * 0.5), 0.24 + flap, -0.02), vec3(0.04, 0.13, 0.26), wing);
    }
    b.box_at(o, y2, vec3(0.0, 0.38, 0.1), vec3(0.14, 0.14, 0.12), white);
    b.box_at(o, y2, vec3(0.0, 0.5, 0.12), vec3(0.24, 0.22, 0.22), white);
    let open = if quacking { 0.035 } else { 0.0 };
    b.box_at(o, y2, vec3(0.0, 0.47 + open * 0.5, 0.27), vec3(0.15, 0.04, 0.12), orange);
    b.box_at(o, y2, vec3(0.0, 0.43 - open * 0.5, 0.26), vec3(0.13, 0.03, 0.1), tinted(orange, -0.15));
    let blinking = time % 3.9 < 0.12;
    for side in [-1.0_f32, 1.0] {
        let h = if blinking { 0.012 } else { 0.045 };
        b.box_at(o, y2, vec3(side * 0.07, 0.54, 0.232), vec3(0.04, h, 0.01), ink);
        b.box_at(o, y2, vec3(side * 0.12, 0.49, 0.232), vec3(0.04, 0.025, 0.01), Color::from_rgba(255, 180, 190, 255));
    }
    draw_carry(b, o, y2, vec3(0.0, 0.45, 0.32), carry);
}

pub(crate) fn draw_owl(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, singing: bool, dance: f32, carry: Option<Carry>) {
    let brown = Color::from_rgba(140, 100, 70, 255);
    let brown_dark = Color::from_rgba(104, 72, 50, 255);
    let belly = Color::from_rgba(214, 186, 146, 255);
    let face = Color::from_rgba(232, 214, 186, 255);
    let ink = Color::from_rgba(26, 22, 24, 255);
    let phones = Color::from_rgba(52, 52, 64, 255);
    let led = mix(Color::from_rgba(60, 200, 190, 255), Color::from_rgba(255, 240, 140, 255), if singing { 0.5 + 0.5 * (time * 20.0).sin() } else { 0.0 });
    let hop = if walking { (time * 10.0).sin().abs() * 0.08 } else { dance * 0.06 };
    let o = pos + vec3(0.0, hop, 0.0);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.07, 0.02, 0.08), vec3(0.08, 0.04, 0.08), Color::from_rgba(230, 190, 80, 255));
    }
    b.box_at(o, yaw, vec3(0.0, 0.26, 0.0), vec3(0.36, 0.42, 0.32), brown);
    b.box_at(o, yaw, vec3(0.0, 0.22, 0.161), vec3(0.24, 0.28, 0.01), belly);
    for (dx, dy) in [(-0.06, 0.28), (0.05, 0.24), (-0.03, 0.16), (0.07, 0.12)] {
        b.box_at(o, yaw, vec3(dx, dy, 0.167), vec3(0.04, 0.025, 0.01), brown_dark);
    }
    let flap = if singing || dance > 0.3 { (time * 18.0).sin() * 0.06 } else { 0.0 };
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * (0.2 + flap.abs()), 0.27 + flap, -0.01), vec3(0.05, 0.3, 0.26), brown_dark);
    }
    b.box_at(o, yaw, vec3(0.0, 0.58, 0.0), vec3(0.4, 0.26, 0.32), brown);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.14, 0.74, -0.01), vec3(0.07, 0.1, 0.06), brown_dark);
    }
    b.box_at(o, yaw, vec3(0.0, 0.57, 0.161), vec3(0.32, 0.2, 0.01), face);
    let blinking = time % 4.4 < 0.15;
    for side in [-1.0_f32, 1.0] {
        if blinking {
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.168), vec3(0.1, 0.02, 0.01), ink);
        } else {
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.166), vec3(0.11, 0.11, 0.01), Color::from_rgba(250, 196, 50, 255));
            b.box_at(o, yaw, vec3(side * 0.075, 0.58, 0.169), vec3(0.055, 0.055, 0.01), ink);
            b.box_at(o, yaw, vec3(side * 0.075 - 0.015, 0.595, 0.172), vec3(0.018, 0.018, 0.01), WHITE);
        }
    }
    b.box_at(o, yaw, vec3(0.0, 0.51, 0.175), vec3(0.04, if singing { 0.07 } else { 0.05 }, 0.03), Color::from_rgba(200, 150, 80, 255));
    b.box_at(o, yaw, vec3(0.0, 0.725, 0.0), vec3(0.46, 0.03, 0.06), phones);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.215, 0.6, 0.0), vec3(0.06, 0.13, 0.13), phones);
        b.box_at(o, yaw, vec3(side * 0.247, 0.6, 0.0), vec3(0.01, 0.05, 0.05), led);
    }
    draw_carry(b, o, yaw, vec3(0.0, 0.46, 0.22), carry);
}

pub(crate) fn draw_fox(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, calling: bool, dance: f32, carry: Option<Carry>) {
    let orange = Color::from_rgba(232, 112, 46, 255);
    let orange_dark = Color::from_rgba(196, 86, 34, 255);
    let cream = Color::from_rgba(250, 240, 226, 255);
    let sock = Color::from_rgba(60, 40, 36, 255);
    let ink = Color::from_rgba(26, 20, 20, 255);
    let scarf = Color::from_rgba(70, 150, 140, 255);
    let bob = if walking { (time * 13.0).sin().abs() * 0.045 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let step = if walking { (time * 13.0).sin() * 0.06 } else { 0.0 };
    for (lx, lz, phase) in [(-0.09, 0.15, 1.0), (0.09, 0.15, -1.0), (-0.09, -0.17, -1.0), (0.09, -0.17, 1.0)] {
        b.box_at(o, yaw, vec3(lx, 0.1, lz + step * phase), vec3(0.07, 0.2, 0.07), orange);
        b.box_at(o, yaw, vec3(lx, 0.035, lz + step * phase), vec3(0.075, 0.07, 0.075), sock);
    }
    b.box_at(o, yaw, vec3(0.0, 0.3, -0.01), vec3(0.24, 0.2, 0.5), orange);
    b.box_at(o, yaw, vec3(0.0, 0.25, 0.12), vec3(0.18, 0.1, 0.26), cream);
    let swish = (time * 4.0).sin() * 0.08 + if calling { (time * 20.0).sin() * 0.05 } else { 0.0 };
    b.box_at(o, yaw, vec3(swish * 0.4, 0.34, -0.33), vec3(0.14, 0.14, 0.18), orange);
    b.box_at(o, yaw, vec3(swish * 0.8, 0.31, -0.48), vec3(0.17, 0.16, 0.18), orange_dark);
    b.box_at(o, yaw, vec3(swish, 0.28, -0.62), vec3(0.13, 0.13, 0.12), cream);
    let lift = if calling { 0.06 } else { 0.0 };
    b.box_at(o, yaw, vec3(0.0, 0.44, 0.2), vec3(0.28, 0.06, 0.16), scarf);
    b.box_at(o, yaw, vec3(0.1, 0.38, 0.27), vec3(0.06, 0.12, 0.04), scarf);
    b.box_at(o, yaw, vec3(0.0, 0.56 + lift, 0.24), vec3(0.28, 0.24, 0.26), orange);
    b.box_at(o, yaw, vec3(0.0, 0.5 + lift, 0.41), vec3(0.14, 0.1, 0.16), cream);
    b.box_at(o, yaw, vec3(0.0, 0.53 + lift, 0.49), vec3(0.05, 0.04, 0.03), ink);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.13, 0.48 + lift, 0.3), vec3(0.05, 0.08, 0.14), cream);
        b.box_at(o, yaw, vec3(side * 0.1, 0.74 + lift, 0.22), vec3(0.1, 0.14, 0.05), orange);
        b.box_at(o, yaw, vec3(side * 0.1, 0.83 + lift, 0.22), vec3(0.06, 0.06, 0.05), sock);
        b.box_at(o, yaw, vec3(side * 0.1, 0.73 + lift, 0.247), vec3(0.05, 0.08, 0.01), Color::from_rgba(250, 200, 190, 255));
        let blinking = time % 3.6 < 0.12;
        let h = if blinking || calling { 0.015 } else { 0.05 };
        b.box_at(o, yaw, vec3(side * 0.07, 0.6 + lift, 0.372), vec3(0.05, h, 0.01), ink);
    }
    if calling {
        b.box_at(o, yaw, vec3(0.0, 0.46 + lift, 0.45), vec3(0.1, 0.03, 0.08), Color::from_rgba(130, 50, 50, 255));
    }
    draw_carry(b, o, yaw, vec3(0.0, 0.48 + lift, 0.5), carry);
}

pub(crate) fn draw_otter(b: &mut Batch, pos: Vec3, yaw: f32, time: f32, walking: bool, smashing: bool, dance: f32, carry: Option<Carry>, voice: f32) {
    let brown = Color::from_rgba(118, 82, 58, 255);
    let brown_dark = Color::from_rgba(90, 62, 44, 255);
    let face = Color::from_rgba(214, 196, 170, 255);
    let ink = Color::from_rgba(24, 20, 20, 255);
    let shell = Color::from_rgba(240, 196, 180, 255);
    let rock = Color::from_rgba(150, 152, 164, 255);
    let bob = if walking { (time * 11.0).sin().abs() * 0.04 } else { dance * 0.06 };
    let o = pos + vec3(0.0, bob, 0.0);
    let sway = if walking { (time * 11.0).sin() * 0.08 } else { 0.0 };
    let y2 = yaw + sway;
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, yaw, vec3(side * 0.08, 0.03, 0.06), vec3(0.09, 0.06, 0.12), brown_dark);
    }
    b.box_at(o, y2, vec3(0.0, 0.08, -0.22), vec3(0.1, 0.06, 0.24), brown_dark);
    b.box_at(o, y2, vec3(0.0, 0.26, 0.0), vec3(0.3, 0.36, 0.26), brown);
    b.box_at(o, y2, vec3(0.0, 0.26, 0.131), vec3(0.2, 0.26, 0.01), tinted(brown, 0.15));
    b.box_at(o, y2, vec3(0.0, 0.29, 0.15), vec3(0.16, 0.05, 0.08), shell);
    b.box_at(o, y2, vec3(0.0, 0.32, 0.16), vec3(0.1, 0.03, 0.06), tinted(shell, -0.12));
    let strike = if smashing { (1.0 - (voice / 0.4).min(1.0)).powi(2) } else { 1.0 };
    let rock_y = 0.36 + 0.14 * strike;
    b.box_at(o, y2, vec3(0.1, rock_y, 0.16), vec3(0.07, 0.07, 0.07), brown_dark);
    b.box_at(o, y2, vec3(0.04, rock_y + 0.01, 0.2), vec3(0.08, 0.07, 0.07), rock);
    b.box_at(o, y2, vec3(-0.1, 0.33, 0.15), vec3(0.07, 0.08, 0.07), brown_dark);
    b.box_at(o, y2, vec3(0.0, 0.55, 0.02), vec3(0.3, 0.24, 0.24), brown);
    b.box_at(o, y2, vec3(0.0, 0.51, 0.13), vec3(0.22, 0.13, 0.04), face);
    b.box_at(o, y2, vec3(0.0, 0.53, 0.155), vec3(0.06, 0.04, 0.02), ink);
    for side in [-1.0_f32, 1.0] {
        b.box_at(o, y2, vec3(side * 0.15, 0.66, 0.0), vec3(0.06, 0.05, 0.05), brown_dark);
        let blinking = time % 4.0 < 0.12;
        b.box_at(o, y2, vec3(side * 0.07, 0.6, 0.141), vec3(0.04, if blinking { 0.012 } else { 0.04 }, 0.01), ink);
        for dy in [0.0_f32, 0.02] {
            b.box_at(o, y2, vec3(side * 0.16, 0.5 + dy, 0.14), vec3(0.09, 0.006, 0.006), Color::from_rgba(240, 236, 226, 255));
        }
    }
    draw_carry(b, o, y2, vec3(0.0, 0.47, 0.2), carry);
}
