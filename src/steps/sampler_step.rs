#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn sampler(g: &mut Game, _fr: &mut Frame) {
    if g.sampler.open && !g.inside {
        g.sampler.open = false;
    }
    if g.sampler.open {
        let o = g.sampler.owner;
        let (l, r, u, d, act, close) = if o == 0 {
            (
                is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left),
                is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right),
                is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up),
                is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down),
                is_key_pressed(KeyCode::E),
                is_key_pressed(KeyCode::X),
            )
        } else {
            (is_key_pressed(KeyCode::J), is_key_pressed(KeyCode::L), is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::O), is_key_pressed(KeyCode::U))
        };
        let (mut cx, mut cy) = g.sampler.cursor;
        if l {
            cx = (cx + 15) % 16;
        }
        if r {
            cx = (cx + 1) % 16;
        }
        if u {
            cy = (cy + 3) % 4;
        }
        if d {
            cy = (cy + 1) % 4;
        }
        g.sampler.cursor = (cx, cy);
        if act {
            g.sampler.grid[cy][cx] = !g.sampler.grid[cy][cx];
            g.save_clock = 99.0;
        }
        if close {
            g.sampler.open = false;
        }
        if o == 0 && is_key_pressed(KeyCode::Space) {
            if get_time() - g.sampler.clear_armed < 2.5 {
                g.sampler.grid = [[false; 16]; 4];
                g.sampler.pads = [None, None, None, None];
                g.sampler.clear_armed = 0.0;
                g.save_clock = 99.0;
                g.toasts.push(("the sampler is cleared".to_string(), 2.0));
            } else {
                g.sampler.clear_armed = get_time();
            }
        }
    }
}
