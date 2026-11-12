mod audio;
mod autopilot;
mod computer;
mod draw;
mod game;
mod gfx;
mod music;
mod net;
mod poster;
mod settings;
mod sprites;
mod ui;
mod world;

use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "Melody Quack".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut game = game::Game::new();
    if game.settings.fullscreen {
        set_fullscreen(true);
    }
    loop {
        game.ui.begin();
        game.update();
        game.draw();
        game.ui.end(&game.fonts);
        if game.autopilot.is_some() {
            autopilot::step(&mut game);
        }
        next_frame().await;
    }
}
