#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn paint_page(target: &RenderTarget, book: &books::Book, page: usize, left: bool) {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / w, 2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        ..Default::default()
    });
    clear_background(Color::from_rgba(247, 239, 219, 255));
    for i in 0..24 {
        let a = 0.016 * (24 - i) as f32;
        let x = if left { w - (i + 1) as f32 * 2.0 } else { i as f32 * 2.0 };
        draw_rectangle(x, 0.0, 2.0, h, Color::new(0.4, 0.28, 0.16, a));
    }
    let ink = Color::from_rgba(52, 38, 32, 255);
    let soft = Color::from_rgba(150, 128, 108, 255);
    let accent = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    let margin = 44.0;
    draw_text(book.title, margin, 54.0, 24.0, soft);
    draw_rectangle(margin, 68.0, 64.0, 5.0, accent);
    let (heading, body) = book.pages[page % book.pages.len()];
    let mut y = 128.0;
    for line in wrap_px(heading, 44, w - margin * 2.0) {
        draw_text(&line, margin, y, 44.0, ink);
        y += 44.0;
    }
    y += 18.0;
    for line in wrap_px(body, 30, w - margin * 2.0) {
        draw_text(&line, margin, y, 30.0, Color::from_rgba(78, 62, 50, 255));
        y += 37.0;
    }
    let dots = (page * 7 + 3) % 5 + 3;
    for d in 0..dots {
        let dx = margin + d as f32 * 22.0;
        draw_rectangle(dx, h - 92.0 - ((d * 13 + page * 5) % 4) as f32 * 8.0, 12.0, 12.0, Color::new(accent.r, accent.g, accent.b, 0.55));
    }
    let number = format!("{}", page + 1);
    let nx = if left { margin } else { w - margin - measure_text(&number, None, 24, 1.0).width };
    draw_text(&number, nx, h - 34.0, 24.0, soft);
}

pub(crate) fn paint_spine(target: &RenderTarget, book: &books::Book) {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / w, 2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        ..Default::default()
    });
    let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    clear_background(c);
    let band = Color::new(c.r * 0.7, c.g * 0.7, c.b * 0.7, 1.0);
    draw_rectangle(0.0, 0.0, 26.0, h, band);
    draw_rectangle(w - 26.0, 0.0, 26.0, h, band);
    let size = 54u16;
    let tw = measure_text(book.title, None, size, 1.0).width;
    draw_text(book.title, (w - tw) / 2.0, h / 2.0 + 18.0, size as f32, Color::from_rgba(255, 248, 230, 255));
}

pub(crate) fn reading_light() -> Light {
    Light {
        dusk: 0.15,
        fire: vec3(0.8, 1.2, -0.5),
        fire_strength: 0.55,
        tint: vec3(1.0, 0.75, 0.45),
        lamps: Vec::new(),
    }
}

pub(crate) fn draw_shelf_scene(sel: usize, time: f32, spines: &[RenderTarget]) {
    let eye = vec3(0.0, 0.62, 1.35);
    set_camera(&Camera3D {
        position: eye,
        target: vec3(0.0, 0.42, 0.0),
        up: Vec3::Y,
        fovy: 0.85,
        ..Default::default()
    });
    clear_background(Color::from_rgba(38, 28, 34, 255));
    let mut b = Batch::new(eye, reading_light());
    let wood = Color::from_rgba(140, 92, 58, 255);
    let dark = Color::from_rgba(96, 62, 40, 255);
    b.cube(vec3(0.0, 0.55, -0.32), vec3(1.7, 1.3, 0.04), dark);
    b.cube(vec3(0.0, 0.07, 0.0), vec3(1.7, 0.06, 0.66), wood);
    b.cube(vec3(0.0, 1.02, 0.0), vec3(1.7, 0.05, 0.66), wood);
    b.cube(vec3(-0.83, 0.55, 0.0), vec3(0.05, 1.0, 0.66), wood);
    b.cube(vec3(0.83, 0.55, 0.0), vec3(0.05, 1.0, 0.66), wood);
    b.cube(vec3(0.72, 0.15, -0.05), vec3(0.12, 0.1, 0.12), Color::from_rgba(200, 110, 80, 255));
    b.cube(vec3(0.72, 0.26, -0.05), vec3(0.07, 0.14, 0.07), Color::from_rgba(90, 160, 80, 255));
    b.cube(vec3(-0.72, 0.13, -0.02), vec3(0.14, 0.06, 0.2), Color::from_rgba(230, 220, 200, 255));
    let mut faces = Vec::new();
    for (i, book) in books::BOOKS.iter().enumerate() {
        let hgt = 0.6 + ((i * 7) % 3) as f32 * 0.04;
        let x = (i as f32 - (books::BOOKS.len() as f32 - 1.0) / 2.0) * 0.2;
        let out = if i == sel { 0.12 + 0.012 * (time * 3.0).sin() } else { 0.0 };
        let lift = if i == sel { 0.03 } else { 0.0 };
        let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
        let center = vec3(x, 0.1 + hgt / 2.0 + lift, out);
        b.cube(center, vec3(0.16, hgt, 0.5), c);
        b.cube(center + vec3(0.0, 0.0, -0.005), vec3(0.17, hgt * 0.96, 0.48), Color::from_rgba(240, 230, 205, 255));
        faces.push((vec3(x - 0.08, 0.1 + lift, out + 0.251), hgt));
    }
    b.flush();
    for (i, (corner, hgt)) in faces.iter().enumerate() {
        draw_affine_parallelogram(*corner, vec3(0.16, 0.0, 0.0), vec3(0.0, *hgt, 0.0), Some(&spines[i].texture), WHITE);
    }
}

pub(crate) fn draw_book_scene(book: &books::Book, pages: &[RenderTarget], turn: f32, dir: f32, time: f32) {
    let eye = vec3(0.0, 1.12, 0.3);
    set_camera(&Camera3D {
        position: eye,
        target: vec3(0.0, 0.0, -0.02),
        up: Vec3::Y,
        fovy: 0.74,
        ..Default::default()
    });
    clear_background(Color::from_rgba(38, 28, 34, 255));
    let mut b = Batch::new(eye, reading_light());
    let cover = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
    b.cube(vec3(0.0, -0.09, 0.0), vec3(3.2, 0.06, 2.2), Color::from_rgba(122, 80, 50, 255));
    b.cube(vec3(0.0, -0.045, 0.0), vec3(1.0, 0.03, 0.7), cover);
    b.cube(vec3(-0.235, -0.012, 0.0), vec3(0.46, 0.04, 0.64), Color::from_rgba(226, 216, 190, 255));
    b.cube(vec3(0.235, -0.012, 0.0), vec3(0.46, 0.04, 0.64), Color::from_rgba(226, 216, 190, 255));
    b.cube(vec3(-0.78, -0.02, -0.28), vec3(0.14, 0.12, 0.14), Color::from_rgba(236, 236, 236, 255));
    b.cube(vec3(-0.78, 0.035, -0.28), vec3(0.11, 0.01, 0.11), Color::from_rgba(120, 76, 46, 255));
    b.cube(vec3(0.8, -0.04, -0.4), vec3(0.18, 0.04, 0.18), Color::from_rgba(60, 60, 70, 255));
    b.cube(vec3(0.8, 0.12, -0.42), vec3(0.03, 0.3, 0.03), Color::from_rgba(60, 60, 70, 255));
    b.unlit = true;
    b.cube(vec3(0.74, 0.27, -0.36), vec3(0.16, 0.08, 0.16), Color::from_rgba(255, 220, 140, 255));
    b.unlit = false;
    let fur = Color::from_rgba(244, 224, 196, 255);
    let pad = Color::from_rgba(240, 160, 170, 255);
    let breathe = 0.006 * (time * 1.6).sin();
    for side in [-1.0f32, 1.0] {
        let paw = vec3(side * 0.4, 0.05 + breathe, 0.3);
        b.cube(paw, vec3(0.13, 0.07, 0.12), fur);
        b.cube(paw + vec3(0.0, 0.0, 0.1), vec3(0.11, 0.06, 0.1), fur);
        for t in 0..3 {
            b.cube(paw + vec3(-0.035 + t as f32 * 0.035, 0.04, -0.035), vec3(0.025, 0.012, 0.025), pad);
        }
    }
    b.flush();
    draw_affine_parallelogram(vec3(-0.46, 0.042, -0.31), vec3(0.0, 0.0, 0.62), vec3(0.46, -0.03, 0.0), Some(&pages[0].texture), WHITE);
    draw_affine_parallelogram(vec3(0.0, 0.012, -0.31), vec3(0.0, 0.0, 0.62), vec3(0.46, 0.03, 0.0), Some(&pages[1].texture), WHITE);
    if turn > 0.0 {
        let p = 1.0 - turn;
        let angle = if dir > 0.0 { p * PI } else { PI - p * PI };
        let reach = vec3(angle.cos() * 0.46, 0.012 + angle.sin() * 0.3, 0.0);
        let shade = 0.82 + 0.18 * angle.sin();
        draw_affine_parallelogram(vec3(0.0, 0.016, -0.31), vec3(0.0, 0.0, 0.62), reach, None, Color::new(0.97 * shade, 0.94 * shade, 0.86 * shade, 1.0));
    }
}
