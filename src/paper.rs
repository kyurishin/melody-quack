use macroquad::prelude::*;

thread_local! {
    static FONT: Option<fontdue::Font> = fontdue::Font::from_bytes(&include_bytes!("../assets/fonts/GeistPixel-Regular.ttf")[..], fontdue::FontSettings::default()).ok();
}

pub(crate) struct Paper {
    pub(crate) w: usize,
    pub(crate) h: usize,
    px: Vec<[f32; 4]>,
}

impl Paper {
    pub(crate) fn new(w: usize, h: usize, bg: Color) -> Paper {
        Paper { w, h, px: vec![[bg.r, bg.g, bg.b, 1.0]; w * h] }
    }

    fn blend(&mut self, x: i32, y: i32, c: Color, a: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h || a <= 0.0 {
            return;
        }
        let p = &mut self.px[y as usize * self.w + x as usize];
        let a = (a * c.a).clamp(0.0, 1.0);
        p[0] = p[0] * (1.0 - a) + c.r * a;
        p[1] = p[1] * (1.0 - a) + c.g * a;
        p[2] = p[2] * (1.0 - a) + c.b * a;
    }

    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let (x0, y0) = (x.round() as i32, y.round() as i32);
        let (x1, y1) = ((x + w).round() as i32, (y + h).round() as i32);
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend(xx, yy, c, 1.0);
            }
        }
    }

    pub(crate) fn text(&mut self, text: &str, x: f32, baseline: f32, size: f32, c: Color) {
        FONT.with(|f| {
            let Some(font) = f else { return };
            let mut pen = x;
            for ch in text.chars() {
                let (m, bitmap) = font.rasterize(ch, size);
                let gx = pen.round() as i32 + m.xmin;
                let gy = baseline.round() as i32 - m.height as i32 - m.ymin;
                for j in 0..m.height {
                    for i in 0..m.width {
                        let cov = bitmap[j * m.width + i] as f32 / 255.0;
                        self.blend(gx + i as i32, gy + j as i32, c, cov);
                    }
                }
                pen += m.advance_width;
            }
        })
    }

    pub(crate) fn image(&self) -> Image {
        let mut bytes = Vec::with_capacity(self.w * self.h * 4);
        for row in 0..self.h {
            for p in &self.px[row * self.w..(row + 1) * self.w] {
                bytes.extend_from_slice(&[(p[0] * 255.0) as u8, (p[1] * 255.0) as u8, (p[2] * 255.0) as u8, 255]);
            }
        }
        Image { bytes, width: self.w as u16, height: self.h as u16 }
    }
}

pub(crate) fn measure(text: &str, size: f32) -> f32 {
    FONT.with(|f| f.as_ref().map(|font| text.chars().map(|ch| font.metrics(ch, size).advance_width).sum()).unwrap_or(0.0))
}

pub(crate) fn wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if !line.is_empty() && measure(&candidate, size) > width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
