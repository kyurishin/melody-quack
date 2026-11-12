use macroquad::prelude::*;
use std::f32::consts::PI;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
mod tempo;
mod fish;
mod emoji;
mod home;
mod books;
mod mushroom;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/audio.rs"]
mod audio;

#[cfg(target_arch = "wasm32")]
#[path = "web/audio.rs"]
mod audio;

mod util;
mod ui;
mod sound;
mod voices;
mod drums;
mod music;
mod world;
mod spells;
mod batch;
mod world_draw;
mod home_draw;
mod reading;
mod render;
mod diary;
mod gathering;
mod camp;
mod characters;
mod critters;
mod sky;
mod save;
mod player;
mod dog;
mod building;
mod inventory;
mod fishing;
mod quests;
mod visuals;
mod sampler;
mod ddr;

use util::*;
use ui::*;
use sound::*;
use voices::*;
use drums::*;
use music::*;
use world::*;
use spells::*;
use batch::*;
use world_draw::*;
use home_draw::*;
use reading::*;
use render::*;
use diary::*;
use gathering::*;
use camp::*;
use characters::*;
use critters::*;
use sky::*;
use save::*;
use player::*;
use dog::*;
use building::*;
use inventory::*;
use fishing::*;
use quests::*;
use visuals::*;
use sampler::*;
use ddr::*;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/mic.rs"]
mod mic;

#[cfg(target_arch = "wasm32")]
#[path = "web/mic.rs"]
mod mic;

#[cfg(target_arch = "wasm32")]
#[path = "web/radio.rs"]
mod radio;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/radio.rs"]
mod radio;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/store.rs"]
mod store;

#[cfg(target_arch = "wasm32")]
#[path = "web/store.rs"]
mod store;

#[cfg(target_arch = "wasm32")]
#[path = "web/clock.rs"]
mod clock;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/clock.rs"]
mod clock;

fn icon_pixels<const N: usize>(png: &[u8]) -> [u8; N] {
    let img = Image::from_file_with_format(png, None).expect("could not read icon");
    img.bytes.try_into().expect("icon has the wrong size")
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Melody Quack".to_owned(),
        window_width: (VIEW_W * 3) as i32,
        window_height: (VIEW_H * 3) as i32,
        icon: Some(miniquad::conf::Icon {
            small: icon_pixels(include_bytes!("../assets/icon16.png")),
            medium: icon_pixels(include_bytes!("../assets/icon32.png")),
            big: icon_pixels(include_bytes!("../assets/icon64.png")),
        }),
        ..Default::default()
    }
}

struct Extras {
    fishing: bool,
    ring: f32,
    in_window: bool,
    dip: f32,
    jump: Option<(usize, f32)>,
    cat: Vec2,
    pot: bool,
    hearts: Vec<(Vec3, f32)>,
    ghost: Option<(i32, i32, Tool)>,
    friend_rod: Option<Vec2>,
    sticks: Vec<Vec2>,
}

#[macroquad::main(window_conf)]
async fn main() {
    let audio = audio::Audio::new();
    let cast_list = roster();
    let voices: Vec<Vec<audio::Sound>> = cast_list.iter().map(|c| NOTES.iter().map(|n| audio.load_ringing(render_voice(c.voice, *n))).collect()).collect();
    let marimba: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_marimba(midi_to_freq(*n - 12)))).collect();
    let bells: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_bell(midi_to_freq(*n)))).collect();
    let piano: Vec<audio::Sound> = NOTES.iter().map(|n| audio.load_ringing(render_piano(midi_to_freq(*n)))).collect();
    let open_hat = audio.load_ringing(render_open_hat());
    let tom = audio.load_ringing(render_tom());
    let rim = audio.load_ringing(render_rim());
    let bass = audio.load_ringing(render_bass());
    let kick = audio.load_ringing(render_kick());
    let clap = audio.load_ringing(render_clap());
    let hat = audio.load(render_hat());
    let crackle = audio.load(render_crackle());
    let lid = audio.load_ringing(render_lid());
    let chop = audio.load_ringing(render_chop());
    let sizzle = audio.load_ringing(render_sizzle());
    let plink = audio.load(render_plink());
    let splash = audio.load(render_splash());
    let fixed = [&kick, &clap, &hat, &crackle, &lid, &chop, &sizzle, &plink, &splash];
    let dj_sounds = [&open_hat, &tom, &rim, &bass];
    while !voices.iter().flatten().chain(marimba.iter()).chain(bells.iter()).chain(piano.iter()).chain(fixed).chain(dj_sounds).all(|s| audio.is_loaded(s)) {
        next_frame().await;
    }
    let perc = [&kick, &lid, &chop, &sizzle];
    let play_pad = |who: usize, i: usize, volume: f32, friend_voice: usize| {
        if who == 0 {
            let s = [&kick, &clap, &hat, &open_hat, &tom, &rim][i.min(5)];
            audio.play_ch(2, s, volume);
        } else {
            match i {
                0 => audio.play_ch(2, &bass, volume),
                1 => {
                    for n in [0, 2, 4] {
                        audio.play_ch(1, &piano[n], volume * 0.55);
                    }
                }
                2 => audio.play_ch(1, &voices[friend_voice][4], volume),
                3 => audio.play_ch(2, &sizzle, volume),
                4 => audio.play_ch(2, &bells[5], volume * 0.8),
                _ => {
                    audio.play_ch(2, &clap, volume * 0.8);
                    audio.play_ch(2, &chop, volume * 0.6);
                }
            }
        }
    };

    let make_view = |w: u32, h: u32| {
        let target = render_target_ex(w, h, RenderTargetParams { sample_count: 1, depth: true });
        target.texture.set_filter(FilterMode::Nearest);
        target
    };
    let mut view_w = low_w() as u32;
    let mut view_h = low_h() as u32;
    let mut full_view = make_view(view_w, view_h);
    let catalogue_view = make_view(400, 150);
    let pick_view = make_view(300, 130);

    let mut world = World::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut players = vec![Player::new(0, cast_list[0].body, vec2(6.5, 16.5)), Player::new(4, cast_list[4].body, vec2(7.5, 16.5))];
    let mut picking = true;
    let mut first_pick = true;
    let mut first_steps = 0u8;
    let mut pick_row = 0usize;
    let mut brain = DogBrain::new();
    let mut camp = Camp { fuel: 0.0, lit: false, pile: 0 };
    let mut inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
    let mut tools = [Tool::WoodWall, Tool::StoneWall];
    let mut fishing = Fishing { active: false, caster: 0, cue_at: 0.0, hits: 0, tries: FISH_TRIES, hit_by: [false; 2], early: [false; 2], lesson: false, perfects: 0, dip: 0.0, jump: None, tight: 1.0 };
    let mut fish_lesson = false;
    let mut cook_lesson = false;
    let mut gather_lesson = false;
    let mut fish_card = 0.0_f32;
    let mut cook_card = 0.0_f32;
    let mut gather_card = 0.0_f32;
    let mut fish_say: Option<(String, Color, f32)> = None;
    let mut dex_counts: Vec<u32> = vec![0; fish::COUNT];
    let mut shroom_counts: Vec<u32> = vec![0; mushroom::COUNT];
    let shroom_art: Vec<Vec<(usize, usize, Color)>> = (0..mushroom::COUNT).map(mushroom::art).collect();
    let mut shrooms: Vec<(Vec2, usize)> = Vec::new();
    let mut shroom_clock = 3.0f32;
    let mut shroom_day = i64::MIN;
    let mut shroom_today: Vec<usize> = Vec::new();
    let mut dex_mode = 0usize;
    let mut dex_from_shop = false;
    let fish_art: Vec<Vec<(usize, usize, Color)>> = (0..fish::COUNT).map(fish::art).collect();
    let mut fish_pop: Option<(usize, f32, bool, bool)> = None;
    let mut dex_open = false;
    let mut dex_query = String::new();
    let mut dex_sel = 0usize;
    let mut dex_drag: Option<(f32, usize)> = None;
    let mut emoji_choose: [Option<(usize, f32)>; 2] = [None, None];
    let mut emoji_last = [0usize, 0usize];
    let mut emoji_reply: Option<(usize, f32)> = None;
    let mut cooking = false;
    let mut cook_progress = 0u32;
    let mut dishes: Vec<usize> = Vec::new();
    let mut hearts: Vec<(Vec3, f32)> = Vec::new();
    let mut jam = false;
    let mut dusk = update_sky().night * 0.95;
    let mut echoes: Vec<(f32, usize)> = Vec::new();
    let mut partner_phrase: Vec<(usize, usize)> = Vec::new();
    let mut partner_soloing = false;
    let mut partner_solo_count = 0u32;
    let mut songbook: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut partner_xp = 0u32;
    let mut cat_phrase_notes = 0u32;
    let mut recent_cat: Vec<usize> = Vec::new();
    let mut crackle_clock = 0.0_f32;
    let mut toasts: Vec<(String, f32)> = Vec::new();
    let mut two_player = false;
    let mut ringing = false;
    let mut ring_flash = 0.0_f32;
    let mut show_help = false;
    let mut help_page = 0usize;
    let mut show_inventory = false;
    let mut bag_owner = 0usize;
    let mut bag_sel = 0usize;
    let mut bag_clear_armed = 0.0f32;
    let mut stored: Vec<home::Item> = Vec::new();
    let mut tick_clock = 0.0_f32;
    let mut looper = Looper::new();
    let mut music_mode = true;
    let mut microphone: Option<mic::Mic> = None;
    let mut listening = false;
    let mut listen_clock = 0.0_f32;
    let mut bpm_history: Vec<f32> = Vec::new();
    let mut locked = false;
    let mut mic_message: Option<&str> = None;
    let mut save_clock = 0.0_f32;
    let mut sleep_fade = 0.0_f32;
    let mut shelf_open = false;
    let mut shelf_sel = 0usize;
    let mut open_book: Option<usize> = None;
    let mut book_spread = 0usize;
    let mut page_turn = 0.0f32;
    let mut turn_dir = 1.0f32;
    let mut painted: Option<(usize, usize)> = None;
    let mut spines_painted = false;
    let page_rt: Vec<RenderTarget> = (0..2).map(|_| {
        let rt = render_target(512, 680);
        rt.texture.set_filter(FilterMode::Linear);
        rt
    }).collect();
    let spine_rt: Vec<RenderTarget> = (0..books::BOOKS.len()).map(|_| {
        let rt = render_target(640, 96);
        rt.texture.set_filter(FilterMode::Linear);
        rt
    }).collect();
    let mut mic_message_clock = 0.0_f32;
    let mut tree_rest: std::collections::HashMap<(i32, i32), f32> = std::collections::HashMap::new();
    let mut stick_trees: Vec<(i32, i32)> = Vec::new();
    let mut diary: Vec<DiaryEntry> = Vec::new();
    let mut session: Option<f32> = None;
    let mut session_stars = 0u8;
    let mut last_note_at = -100.0_f32;
    let mut shop_tab = 0usize;
    let mut diary_row = 0usize;
    let mut playback: Option<(usize, f64, usize)> = None;
    let mut dj = [false, false];
    let mut ddr = Ddr { on: [false; 2], arrows: Vec::new(), score: [0; 2], combo: [0; 2], judge: [None; 2], flash: [[0.0; 4]; 2], last_mic: -99.0 };
    let mut ddr_request: Option<(usize, usize)> = None;
    let mut quests = Quests { day: i64::MIN, list: Vec::new() };
    let mut sampler = Sampler { open: false, owner: 0, grid: [[false; 16]; 4], cursor: (0, 0), pads: [None, None, None, None], recording: None, step: 0 };
    let mut saved_quests: Option<(i64, Vec<(u8, u32, u32, bool)>)> = None;
    let ddr_art: Vec<Vec<(usize, usize, Color)>> = (0..4).map(ddr_arrow_pixels).collect();
    let mut beat_style = 3usize;
    let mut stick_clock = 0.0_f32;
    let mut house = home::Home::new();
    let mut inside = false;
    let mut holding: Option<home::Piece> = None;
    let mut holding_by = 0usize;
    let mut shop_open = false;
    let mut shop_owner = 0usize;
    let mut shop_row = 0usize;
    let mut seat: [Option<home::Item>; 2] = [None, None];
    let mut chime_clock = 3.0_f32;
    let mut record_tune: Vec<(usize, usize)> = Vec::new();
    let mut record_pick = 0usize;
    let mut wander_clock = 0.0_f32;
    let mut wander_to = home::world(home::Home::new().door());
    let mut paused = false;
    let mut restart_now = false;
    let mut restart_confirm = 0.0_f32;
    let mut mix = [1.0_f32; 5];
    let mut mix_drag: Option<usize> = None;
    let mut first_click_done = false;
    let mut radio_clock = 0.0_f32;
    let mut last_report = String::new();
    let mut now_playing = String::new();
    let mut live_decks: Vec<(String, String)> = Vec::new();
    let mut live_next: Vec<String> = Vec::new();
    let mut radio_bpm = 0.0_f32;
    let mut fresh_confirm = 0.0_f32;
    if let Some(saved) = store::read().and_then(|t| decode_progress(&t, cast_list.len())) {
        if saved.tiles.len() == world.tiles.len() {
            for (tile, (k, st)) in world.tiles.iter_mut().zip(saved.tiles.iter()) {
                tile.kind = KINDS[(*k as usize).min(KINDS.len() - 1)];
                tile.stack = *st;
                tile.fire = 0.0;
            }
        }
        inv.wood = saved.wood;
        inv.stone = saved.stone;
        inv.fish = saved.fish;
        camp.fuel = saved.fuel;
        camp.lit = saved.lit;
        camp.pile = saved.pile;
        dishes = saved.dishes;
        cook_progress = saved.cook_progress;
        for who in 0..2 {
            players[who].character = saved.characters[who];
            players[who].body = cast_list[saved.characters[who]].body;
            let spot = rescue(&world, saved.spots[who]);
            players[who].place_at(spot);
        }
        ringing = saved.echo;
        audio.set_ring(ringing);
        looper.beat = saved.beat;
        looper.bpm = saved.bpm;
        looper.quantise = [Quantise::Off, Quantise::Soft, Quantise::Tight][(saved.quantise as usize).min(2)];
        looper.notes = saved.notes;
        mix = saved.mix;
        audio.set_mix(mix);
        inv.acorns = saved.acorns;
        house.pieces = saved.pieces;
        house.wall = saved.styles.0;
        house.floor = saved.styles.1;
        house.owned_styles = saved.styles.2 | 1 | (1 << 4);
        house.record_on = saved.styles.3;
        house.size = saved.room;
        dex_counts = saved.dex;
        dex_counts.resize(fish::COUNT, 0);
        inv.mush = saved.mush;
        shroom_counts = saved.shroomdex;
        saved_quests = saved.quests;
        stored = saved.stored.iter().map(|i| home::Item::from_index(*i)).collect();
        for row in 0..4 {
            for k in 0..16 {
                sampler.grid[row][k] = saved.seq[row] & (1 << k) != 0;
            }
        }
        shroom_counts.resize(mushroom::COUNT, 0);
        diary = saved.diary;
        fish_lesson = saved.fish_lesson;
        cook_lesson = saved.cook_lesson;
        gather_lesson = saved.gather_lesson;
        songbook = saved.songbook;
        partner_xp = saved.partner_xp;
        picking = false;
        first_pick = false;
        first_steps = 4;
        toasts.push(("welcome back".to_string(), 3.5));
    }
    prevent_quit();

    loop {
        let dt = get_frame_time().min(0.05);
        let time = get_time() as f32;

        let mut click = is_mouse_button_pressed(MouseButton::Left);
        if click && !first_click_done {
            first_click_done = true;
            if cfg!(target_arch = "wasm32") {
                click = false;
            }
        }
        let (ux, uy) = {
            let (kx, ky) = ui_scale();
            let (mx, my) = mouse_position();
            (mx / kx, my / ky)
        };
        let mut emoji_send: [Option<usize>; 2] = [None, None];
        for who in 0..2 {
            if who == 1 && !two_player {
                emoji_choose[1] = None;
                continue;
            }
            let pressed = if who == 0 { is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Apostrophe) } else { is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Slash) };
            if pressed && !dex_open {
                emoji_choose[who] = Some(match emoji_choose[who] {
                    None => (emoji_last[who], 0.0),
                    Some((id, _)) => ((id + 1) % emoji::COUNT, 0.0),
                });
            }
            if let Some((id, t)) = emoji_choose[who].as_mut() {
                *t += dt;
                let (sx, sy, cell) = emoji_strip(who, two_player);
                if click && uy > sy - 3.0 && uy < sy + cell + 3.0 && ux > sx && ux < sx + cell * emoji::COUNT as f32 {
                    *id = (((ux - sx) / cell) as usize).min(emoji::COUNT - 1);
                    *t = 99.0;
                    click = false;
                }
                if *t > 1.1 {
                    emoji_send[who] = Some(*id);
                }
            }
            if let Some(id) = emoji_send[who] {
                emoji_choose[who] = None;
                emoji_last[who] = id;
                players[who].emote = Some((id, 3.0));
                quest_event(&mut quests, QuestKind::Emoji, 1, &mut inv, &mut toasts);
                if who == 0 && !two_player {
                    emoji_reply = Some((emoji::reply(id, hash((time * 17.0) as i32, 8, 3)), 0.8));
                }
            }
        }
        if let Some((id, t)) = emoji_reply.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                players[1].emote = Some((*id, 3.0));
                players[1].voice_timer = 0.3;
                emoji_reply = None;
            }
        }
        for p in players.iter_mut() {
            if let Some((_, left)) = p.emote.as_mut() {
                *left -= dt;
            }
            if p.emote.map(|e| e.1 <= 0.0).unwrap_or(false) {
                p.emote = None;
            }
        }

        radio::overlay(if paused || dex_open || picking || shelf_open { 0.0 } else if inside && (dj[0] || dj[1]) { 0.075 } else { 0.52 });
        if shelf_open {
            let left_k = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::J);
            let right_k = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::L);
            let pick_k = is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::O) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
            let back_k = is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace);
            let close_hit = click && ux > low_w() - 30.0 && uy < 26.0;
            let tap = click && !close_hit;
            let (tap_left, tap_right) = (tap && ux < low_w() * 0.33, tap && ux > low_w() * 0.67);
            let tap_mid = tap && !tap_left && !tap_right;
            if open_book.is_none() || !spines_painted {
                for (i, book) in books::BOOKS.iter().enumerate() {
                    paint_spine(&spine_rt[i], book);
                }
                spines_painted = painted.is_some();
            }
            let count = books::BOOKS.len();
            match open_book {
                None => {
                    if left_k || tap_left {
                        shelf_sel = (shelf_sel + count - 1) % count;
                        audio.play_ch(3, &plink, 0.2);
                    }
                    if right_k || tap_right {
                        shelf_sel = (shelf_sel + 1) % count;
                        audio.play_ch(3, &plink, 0.2);
                    }
                    if pick_k || tap_mid {
                        open_book = Some(shelf_sel);
                        book_spread = 0;
                        page_turn = 0.0;
                        audio.play_ch(3, &bells[3], 0.25);
                    }
                    if back_k || close_hit {
                        shelf_open = false;
                        save_clock = 99.0;
                    }
                }
                Some(_) => {
                    if (left_k || tap_left) && book_spread > 0 && page_turn <= 0.0 {
                        book_spread -= 1;
                        page_turn = 1.0;
                        turn_dir = -1.0;
                        audio.play_ch(3, &plink, 0.25);
                    }
                    if (right_k || tap_right || pick_k || tap_mid) && book_spread + 1 < books::SPREADS && page_turn <= 0.0 {
                        book_spread += 1;
                        page_turn = 1.0;
                        turn_dir = 1.0;
                        audio.play_ch(3, &plink, 0.25);
                    }
                    if back_k || close_hit {
                        open_book = None;
                    }
                }
            }
            page_turn = (page_turn - get_frame_time() * 2.6).max(0.0);
            if let Some(b) = open_book {
                paint_page(&page_rt[0], &books::BOOKS[b], book_spread * 2, true);
                paint_page(&page_rt[1], &books::BOOKS[b], book_spread * 2 + 1, false);
                painted = Some((b, book_spread));
                draw_book_scene(&books::BOOKS[b], &page_rt, page_turn, turn_dir, time);
            } else {
                draw_shelf_scene(shelf_sel, time, &spine_rt);
            }
            set_default_camera();
            let touch = radio::touch_ui();
            ui_rect(low_w() - 26.0, 6.0, 20.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.16));
            ui_text("x", low_w() - 19.0, 19.0, 12.0, WHITE);
            match open_book {
                None => {
                    let book = &books::BOOKS[shelf_sel];
                    let c = Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255);
                    ui_text("the bookshelf", 10.0, 18.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                    ui_text(book.title, (low_w() - ui_measure(book.title, 18.0)) / 2.0, low_h() - 30.0, 18.0, c);
                    let hint = if touch { "tap a side to choose, the middle to open" } else { "A/D choose   E open   X leave" };
                    ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 12.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
                }
                Some(b) => {
                    let book = &books::BOOKS[b];
                    ui_text(book.title, 10.0, 18.0, 12.0, Color::from_rgba(book.cover.0, book.cover.1, book.cover.2, 255));
                    let at = format!("pages {}-{} of {}", book_spread * 2 + 1, book_spread * 2 + 2, book.pages.len());
                    ui_text(&at, (low_w() - ui_measure(&at, 10.0)) / 2.0, low_h() - 24.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.8));
                    let hint = if touch { "tap left or right to turn the page" } else { "A/D turn the page   X back to the shelf" };
                    ui_text(hint, (low_w() - ui_measure(hint, 10.0)) / 2.0, low_h() - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
                }
            }
            next_frame().await;
            continue;
        }

        if dex_open {
            let shrooms_mode = dex_mode == 1;
            let counts: &Vec<u32> = if shrooms_mode { &shroom_counts } else { &dex_counts };
            let total = if shrooms_mode { mushroom::COUNT } else { fish::COUNT };
            let name_of = |i: usize| if shrooms_mode { mushroom::SPECIES[i] } else { fish::SPECIES[i] };
            let code_of = |i: usize| if shrooms_mode { mushroom::code(i) } else { fish::code(i) };
            let rarity_of = |i: usize| if shrooms_mode { mushroom::rarity(i) } else { fish::rarity(i) };
            let hint_of = |i: usize| if shrooms_mode { mushroom::hint(i) } else { fish::hint(i) };
            let acorns_of = |i: usize| if shrooms_mode { mushroom::acorns(i) } else { fish::acorns(i) };
            let art_of = |i: usize| if shrooms_mode { &shroom_art[i] } else { &fish_art[i] };
            let (art_w, art_h) = if shrooms_mode { (mushroom::ART, mushroom::ART) } else { (fish::ART_W, fish::ART_H) };
            let verb = if shrooms_mode { "found" } else { "caught" };
            let numeric = dex_query.trim().trim_start_matches('#').trim_start_matches('m').parse::<usize>().is_ok();
            let found: Vec<usize> = (if shrooms_mode { mushroom::search(&dex_query) } else { fish::search(&dex_query) }).into_iter().filter(|i| dex_query.trim().is_empty() || numeric || counts[*i] > 0).collect();
            let tab_names = ["catalogue", "diary", "fishdex", "mushrooms"];
            let mut tab_go: Option<usize> = None;
            if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Right) {
                tab_go = Some(if dex_mode == 0 { 3 } else if dex_from_shop { 0 } else { 2 });
            }
            if is_key_pressed(KeyCode::Left) {
                tab_go = Some(if dex_mode == 1 { 2 } else if dex_from_shop { 1 } else { 3 });
            }
            if is_key_pressed(KeyCode::Up) && dex_sel == 0 && dex_from_shop {
                tab_go = Some(if dex_mode == 1 { 2 } else { 1 });
            }
            while let Some(ch) = get_char_pressed() {
                if (ch.is_ascii_alphanumeric() || ch == ' ' || ch == '#') && dex_query.len() < 20 {
                    dex_query.push(ch.to_ascii_lowercase());
                    dex_sel = 0;
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                dex_query.pop();
                dex_sel = 0;
            }
            let n = found.len();
            if n > 0 {
                if is_key_pressed(KeyCode::Down) {
                    dex_sel = (dex_sel + 1).min(n - 1);
                }
                if is_key_pressed(KeyCode::Up) && tab_go.is_none() {
                    dex_sel = dex_sel.saturating_sub(1);
                }
                let wheel = mouse_wheel().1;
                if wheel < 0.0 {
                    dex_sel = (dex_sel + 1).min(n - 1);
                } else if wheel > 0.0 {
                    dex_sel = dex_sel.saturating_sub(1);
                }
            }
            dex_sel = dex_sel.min(n.saturating_sub(1));
            let caught = counts.iter().filter(|c| **c > 0).count();
            let w = (low_w() - 12.0).min(390.0);
            let h = (low_h() - 16.0).min(260.0);
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0).round().max(8.0);
            let wide = w >= 300.0;
            let detail_h = if wide { 0.0 } else { 74.0 };
            let list_w = if wide { (w * 0.52).round() } else { w - 16.0 };
            let search_y = y + 24.0 + detail_h;
            let list_y = search_y + 20.0;
            let row_h = 14.0;
            let shown = (((y + h - 16.0) - list_y) / row_h).floor().max(1.0) as usize;
            let first = dex_sel.saturating_sub(shown / 2).min(n.saturating_sub(shown));
            let on_search = ux > x + 8.0 && ux < x + 8.0 + list_w && uy > search_y && uy < search_y + 15.0;
            let on_list = ux > x + 8.0 && ux < x + 8.0 + list_w && uy > list_y && uy < list_y + shown as f32 * row_h;
            let inside = ux > x && ux < x + w && uy > y && uy < y + h;
            if click && on_search && radio::touch_ui() {
                if let Some(text) = radio::ask_text(if shrooms_mode { "search the mushrooms: a name or a number" } else { "search the fishdex: a name or a number" }, &dex_query) {
                    dex_query = text.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '#').take(20).collect();
                    dex_sel = 0;
                }
            } else if click && on_list {
                let r = first + ((uy - list_y) / row_h) as usize;
                if r < n {
                    dex_sel = r;
                }
                dex_drag = Some((uy, dex_sel));
            } else if click && !inside {
                dex_open = false;
            }
            {
                let mut tx = x + 8.0;
                for (k, name) in tab_names.iter().enumerate() {
                    let tw = ui_measure(name, 11.0);
                    let usable = k >= 2 || dex_from_shop;
                    if click && usable && ux > tx - 2.0 && ux < tx + tw + 2.0 && uy > y + 4.0 && uy < y + 20.0 {
                        tab_go = Some(k);
                    }
                    tx += tw + 10.0;
                }
            }
            if let Some(k) = tab_go {
                if k >= 2 {
                    if k - 2 != dex_mode {
                        dex_mode = k - 2;
                        dex_sel = 0;
                        dex_query.clear();
                    }
                } else {
                    dex_open = false;
                    shop_open = true;
                    shop_tab = k;
                }
            }
            if is_mouse_button_down(MouseButton::Left) {
                if let Some((start, from)) = dex_drag {
                    let moved = ((start - uy) / row_h) as i32;
                    dex_sel = (from as i32 + moved).clamp(0, n.saturating_sub(1) as i32) as usize;
                }
            } else {
                dex_drag = None;
            }
            if is_key_pressed(KeyCode::Escape) {
                dex_open = false;
            }
            set_default_camera();
            clear_background(BLACK);
            draw_texture_ex(
                &full_view.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.06, 0.14, 0.6));
            ui_rect(x, y, w, h, Color::new(0.08, 0.12, 0.22, 0.96));
            ui_rect(x, y, w, 3.0, Color::from_rgba(110, 190, 255, 255));
            {
                let mut tx = x + 8.0;
                for (k, name) in tab_names.iter().enumerate() {
                    let active = k == dex_mode + 2;
                    let usable = k >= 2 || dex_from_shop;
                    let col = if active { Color::from_rgba(150, 215, 255, 255) } else if usable { Color::new(1.0, 1.0, 1.0, 0.55) } else { Color::new(1.0, 1.0, 1.0, 0.2) };
                    ui_text(name, tx, y + 16.0, 11.0, col);
                    if active {
                        ui_rect(tx, y + 18.0, ui_measure(name, 11.0), 1.0, col);
                    }
                    tx += ui_measure(name, 11.0) + 10.0;
                }
            }
            let count = format!("{}/{} {}", caught, total, verb);
            ui_text(&count, x + w - 8.0 - ui_measure(&count, 11.0), y + 16.0, 11.0, WHITE);
            ui_rect(x + 8.0, search_y, list_w, 15.0, Color::new(1.0, 1.0, 1.0, if on_search { 0.16 } else { 0.1 }));
            let placeholder = if radio::touch_ui() { "tap to search a name or #number" } else { "type a name or #number" };
            let cursor = if (time * 2.0).fract() < 0.5 && !radio::touch_ui() { "_" } else { "" };
            if dex_query.is_empty() {
                ui_text(placeholder, x + 13.0, search_y + 11.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.45));
            } else {
                ui_text(&format!("{}{}", dex_query, cursor), x + 13.0, search_y + 11.0, 11.0, WHITE);
            }
            if n == 0 {
                ui_text(if shrooms_mode { "no mushrooms found" } else { "no fish found" }, x + 13.0, list_y + 11.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
            for r in first..(first + shown).min(n) {
                let i = found[r];
                let known = counts[i] > 0;
                let ry = list_y + (r - first) as f32 * row_h;
                if r == dex_sel {
                    ui_rect(x + 6.0, ry - 1.0, list_w + 4.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
                }
                ui_text(&code_of(i), x + 10.0, ry + 9.5, 9.0, Color::new(0.75, 0.85, 1.0, 0.8));
                draw_fish_art(art_of(i), x + 34.0 + if shrooms_mode { 2.0 } else { 0.0 }, ry + if shrooms_mode { 0.0 } else { 1.0 }, if shrooms_mode { 1.05 } else { 0.95 }, known);
                let name = if known { name_of(i) } else { "???" };
                ui_text(name, x + 54.0, ry + 9.5, 10.0, if known { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) });
                if known {
                    let c = format!("x{}", counts[i]);
                    ui_text(&c, x + 6.0 + list_w - ui_measure(&c, 9.0), ry + 9.5, 9.0, fish::RARITY_COLORS[rarity_of(i)]);
                }
            }
            if let Some(&i) = found.get(dex_sel) {
                let known = counts[i] > 0;
                let (dx, dy, dw) = if wide { (x + list_w + 18.0, y + 30.0, w - list_w - 26.0) } else { (x + 8.0, y + 24.0, w - 16.0) };
                let px = if wide { (dw / art_w as f32).min(5.5) } else { 3.6 };
                let art_w = px * art_w as f32;
                let art_x = if wide { dx + (dw - art_w) / 2.0 } else { dx };
                ui_rect(art_x - 4.0, dy - 2.0, art_w + 8.0, px * art_h as f32 + 6.0, Color::new(0.3, 0.55, 0.8, 0.25));
                draw_fish_art(art_of(i), art_x, dy + 1.0, px, known);
                let tx = if wide { dx } else { dx + art_w + 12.0 };
                let mut ty = if wide { dy + px * art_h as f32 + 18.0 } else { dy + 10.0 };
                let rarity = rarity_of(i);
                ui_text(&code_of(i), tx, ty, 11.0, Color::new(0.75, 0.85, 1.0, 0.9));
                ty += 14.0;
                ui_text(if known { name_of(i) } else { "???" }, tx, ty, 13.0, WHITE);
                ty += 13.0;
                ui_text(fish::RARITY_NAMES[rarity], tx, ty, 10.0, fish::RARITY_COLORS[rarity]);
                ty += 12.0;
                let line = if known { format!("{} {} - {} acorns", verb, counts[i], acorns_of(i)) } else { format!("not {} yet", verb) };
                ui_text(&line, tx, ty, 10.0, Color::new(1.0, 1.0, 1.0, 0.75));
                ty += 12.0;
                ui_text(hint_of(i), tx, ty, 9.0, Color::new(0.8, 0.9, 1.0, 0.6));
            }
            let help = if radio::touch_ui() { "tap a row - tap the tabs - tap outside to close" } else { "type to search - up/down - left/right tabs - esc to close" };
            ui_text(help, x + (w - ui_measure(help, 9.0)) / 2.0, y + h - 5.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
            next_frame().await;
            continue;
        }
        let mut just_paused = false;
        let mut help_closed = false;
        if show_help && !paused {
            let (hx, hy, hw, hh) = help_box();
            let tab_hit = (0..3).find(|k| click && uy > hy + 20.0 && uy < hy + 36.0 && ux > hx + 10.0 + *k as f32 * (hw - 20.0) / 3.0 && ux < hx + 10.0 + (*k + 1) as f32 * (hw - 20.0) / 3.0);
            if let Some(k) = tab_hit {
                help_page = k;
                click = false;
            } else if is_key_pressed(KeyCode::Escape) || (click && !(ux > hx && ux < hx + hw && uy > hy && uy < hy + hh)) {
                show_help = false;
                help_closed = true;
                click = false;
            } else if click {
                click = false;
            }
            if is_key_pressed(KeyCode::Right) {
                help_page = (help_page + 1) % 3;
            }
            if is_key_pressed(KeyCode::Left) {
                help_page = (help_page + 2) % 3;
            }
        }
        if sampler.open && !paused && is_key_pressed(KeyCode::Escape) {
            sampler.open = false;
            help_closed = true;
        }
        if (ddr.on[0] || ddr.on[1]) && !paused && is_key_pressed(KeyCode::Escape) {
            ddr.on = [false; 2];
            help_closed = true;
        }
        if !paused && !picking && !help_closed {
            let tap_ok = if radio::touch_ui() && !two_player {
                if low_h() > low_w() { uy < low_h() * 0.55 && uy > 20.0 } else { ux > low_w() * 0.235 && ux < low_w() * 0.765 && uy > 20.0 }
            } else {
                true
            };
            if (click && tap_ok) || is_key_pressed(KeyCode::Escape) {
                paused = true;
                just_paused = true;
                mix_drag = None;
            }
        }
        if paused {
            let labels: [String; 5] = if two_player {
                [
                    cast_list[players[0].character].name.to_string(),
                    cast_list[players[1].character].name.to_string(),
                    "beat & pot".to_string(),
                    "world".to_string(),
                    "master".to_string(),
                ]
            } else {
                [
                    "you".to_string(),
                    format!("the {}", cast_list[players[1].character].noun),
                    "beat & pot".to_string(),
                    "world".to_string(),
                    "master".to_string(),
                ]
            };
            let w = 230f32.min(low_w() - 10.0);
            let row_h = 24.0;
            let h = 40.0 + row_h * 5.0 + 74.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = if radio::touch_ui() && low_h() > low_w() { (low_h() * 0.16).round() } else { ((low_h() - h) / 2.0).round() };
            let label_w = 70.0;
            let bar_x = x + 10.0 + label_w;
            let bar_w = w - 20.0 - label_w - 30.0;
            let row_y = |r: usize| y + 40.0 + r as f32 * row_h + row_h / 2.0;
            if is_mouse_button_down(MouseButton::Left) {
                if mix_drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
                    for r in 0..5 {
                        if (uy - row_y(r)).abs() < row_h / 2.0 && ux > bar_x - 10.0 && ux < bar_x + bar_w + 10.0 {
                            mix_drag = Some(r);
                        }
                    }
                }
                if let Some(r) = mix_drag {
                    mix[r] = ((ux - bar_x) / bar_w).clamp(0.0, 1.0);
                    audio.set_mix(mix);
                }
            } else {
                mix_drag = None;
            }
            let inside = ux > x && ux < x + w && uy > y && uy < y + h;
            let (dex_x, btn_y, dex_w, btn_h) = (x + 10.0, y + 40.0 + row_h * 5.0 + 4.0, ((w - 26.0) * 0.42).round(), 20.0);
            let (btn_x, btn_w) = (dex_x + dex_w + 6.0, w - 26.0 - dex_w);
            let on_button = ux > btn_x && ux < btn_x + btn_w && uy > btn_y && uy < btn_y + btn_h;
            let on_dex = ux > dex_x && ux < dex_x + dex_w && uy > btn_y && uy < btn_y + btn_h;
            if !just_paused && ((click && on_dex) || is_key_pressed(KeyCode::F)) {
                paused = false;
                dex_open = true;
                dex_from_shop = false;
                dex_query.clear();
                dex_sel = 0;
                while get_char_pressed().is_some() {}
                click = false;
            }
            let live_y = btn_y + btn_h + 4.0;
            let live_w = ((w - 26.0) / 2.0).round();
            let live_x = [dex_x, dex_x + live_w + 6.0];
            let on_live = |i: usize| ux > live_x[i] && ux < live_x[i] + live_w && uy > live_y && uy < live_y + btn_h;
            for (i, key) in [KeyCode::J, KeyCode::K].into_iter().enumerate() {
                if !just_paused && ((click && on_live(i) && mix_drag.is_none()) || is_key_pressed(key)) {
                    radio::set_flag(i as i32, !radio::flag(i as i32));
                    click = false;
                }
            }
            restart_confirm = (restart_confirm - get_frame_time()).max(0.0);
            if !just_paused && ((click && on_button && mix_drag.is_none()) || is_key_pressed(KeyCode::R)) {
                if restart_confirm > 0.0 {
                    restart_now = true;
                    paused = false;
                } else {
                    restart_confirm = 3.0;
                }
            }
            if !just_paused && ((click && !inside && mix_drag.is_none()) || is_key_pressed(KeyCode::Escape)) {
                paused = false;
                restart_confirm = 0.0;
                save_clock = 99.0;
            }
            set_default_camera();
            clear_background(BLACK);
            draw_texture_ex(
                &full_view.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.06, 0.04, 0.14, 0.55));
            ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.92));
            let title = "paused";
            ui_text(title, x + (w - ui_measure(title, 18.0)) / 2.0, y + 22.0, 18.0, Color::from_rgba(255, 226, 150, 255));
            ui_text("mixer", x + 10.0, y + 34.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.55));
            for r in 0..5 {
                let cy = row_y(r);
                let mut name = labels[r].clone();
                while ui_measure(&name, 11.0) > label_w - 4.0 && name.len() > 3 {
                    name.pop();
                }
                ui_text(&name, x + 10.0, cy + 4.0, 11.0, WHITE);
                ui_rect(bar_x, cy - 2.0, bar_w, 4.0, Color::new(1.0, 1.0, 1.0, 0.18));
                let fill = if r == 4 { Color::from_rgba(255, 214, 110, 255) } else { note_color([0, 2, 4, 3][r]) };
                ui_rect(bar_x, cy - 2.0, bar_w * mix[r], 4.0, fill);
                ui_rect(bar_x + bar_w * mix[r] - 4.0, cy - 6.0, 8.0, 12.0, if mix_drag == Some(r) { WHITE } else { Color::new(0.95, 0.92, 0.85, 1.0) });
                ui_text(&format!("{:.0}", mix[r] * 100.0), bar_x + bar_w + 8.0, cy + 4.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
            }
            ui_rect(dex_x, btn_y, dex_w, btn_h, if on_dex { Color::from_rgba(90, 160, 230, 255) } else { Color::from_rgba(60, 110, 170, 255) });
            let dex_label = if radio::touch_ui() { "fishdex" } else { "fishdex (F)" };
            ui_text(dex_label, dex_x + (dex_w - ui_measure(dex_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
            let armed = restart_confirm > 0.0;
            ui_rect(btn_x, btn_y, btn_w, btn_h, if armed { Color::from_rgba(220, 70, 80, 255) } else if on_button { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) });
            let btn_label = if armed {
                if radio::touch_ui() { "tap again" } else { "again to restart" }
            } else if radio::touch_ui() {
                "restart"
            } else {
                "restart (R)"
            };
            let mut btn_label = btn_label.to_string();
            while ui_measure(&btn_label, 11.0) > btn_w - 6.0 && btn_label.len() > 4 {
                btn_label.pop();
            }
            ui_text(&btn_label, btn_x + (btn_w - ui_measure(&btn_label, 11.0)) / 2.0, btn_y + 14.0, 11.0, WHITE);
            for i in 0..2 {
                let on = radio::flag(i as i32);
                let name = if i == 0 { "dj set" } else { "karaoke" };
                let key = if radio::touch_ui() { "" } else if i == 0 { " (J)" } else { " (K)" };
                let mut label = format!("{}: {}{}", name, if on { "on" } else { "off" }, key);
                while ui_measure(&label, 10.0) > live_w - 6.0 && label.len() > 4 {
                    label.pop();
                }
                let fill = if on { Color::from_rgba(255, 214, 110, 255) } else if on_live(i) { Color::new(1.0, 1.0, 1.0, 0.22) } else { Color::new(1.0, 1.0, 1.0, 0.12) };
                ui_rect(live_x[i], live_y, live_w, btn_h, fill);
                let ink = if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE };
                ui_text(&label, live_x[i] + (live_w - ui_measure(&label, 10.0)) / 2.0, live_y + 14.0, 10.0, ink);
            }
            let hint = if radio::touch_ui() { "tap outside to play" } else { "click outside or esc to play" };
            ui_text(hint, x + (w - ui_measure(hint, 10.0)) / 2.0, y + h - 8.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
            next_frame().await;
            continue;
        }

        if is_key_pressed(KeyCode::P) {
            two_player = !two_player;
            brain = DogBrain::new();
            players[1].seated = false;
            players[1].sleeping = false;
            seat[1] = None;
            dj[1] = false;
            players[1].carrying = None;
            if !two_player {
                if shop_open && shop_owner == 1 {
                    shop_owner = 0;
                }
                if holding.is_some() && holding_by == 1 {
                    holding_by = 0;
                }
                if fishing.active && fishing.caster == 1 {
                    fishing.active = false;
                    fish_card = 0.0;
                    toasts.push(("the fishing line was reeled in".to_string(), 1.5));
                }
            }
            if two_player {
                let start = if inside {
                    home::world(house.free_spot_near(home::local(players[0].pos) + vec2(1.0, 0.0)))
                } else {
                    rescue(&world, players[0].pos + vec2(-1.0, 0.0))
                };
                players[1].place_at(start);
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            music_mode = !music_mode;
        }
        if is_key_pressed(KeyCode::R) {
            looper.recording = !looper.recording;
        }
        let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if is_key_pressed(KeyCode::C) && shift_held {
            songbook.clear();
            partner_xp = 0;
            partner_phrase.clear();
            partner_soloing = false;
            toasts.push((format!("the {} forgot every tune - starting from scratch", cast_list[players[1].character].noun), 3.0));
        } else if is_key_pressed(KeyCode::C) || (two_player && is_key_pressed(KeyCode::Y)) {
            let only = if !two_player { None } else if is_key_pressed(KeyCode::C) { Some(0) } else { Some(1) };
            match only {
                None => {
                    looper.notes.clear();
                    looper.skip.clear();
                }
                Some(who) => {
                    looper.notes.retain(|&(_, id)| note_owner(id) != who);
                    looper.skip.retain(|&(_, id)| note_owner(id) != who);
                    looper.pending.retain(|&(_, id)| note_owner(id) != who);
                    toasts.push((format!("player {} cleared their loop", who + 1), 1.5));
                }
            }
        }
        if is_key_pressed(KeyCode::B) {
            looper.beat = !looper.beat;
        }
        if is_key_pressed(KeyCode::H) {
            show_help = !show_help;
        }
        let bag_keys = [is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Enter), two_player && is_key_pressed(KeyCode::Backspace)];
        for who in 0..2 {
            if !bag_keys[who] || picking {
                continue;
            }
            if inside && holding.is_some() && holding_by == who {
                if let Some(piece) = holding.take() {
                    stored.push(piece.item);
                    audio.play_ch(3, &plink, 0.35);
                    toasts.push((format!("the {} goes into your bag", piece.item.name()), 2.0));
                    save_clock = 99.0;
                }
            } else if show_inventory && bag_owner != who {
                bag_owner = who;
            } else {
                show_inventory = !show_inventory;
                bag_owner = who;
                bag_sel = 0;
                bag_clear_armed = 0.0;
            }
        }
        bag_clear_armed = (bag_clear_armed - dt).max(0.0);
        let bag_busy = show_inventory && inside && !shop_open;
        if bag_busy && holding.is_none() {
            let owner = bag_owner;
            let (up, down, act, clear) = if owner == 0 {
                (is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up), is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down), is_key_pressed(KeyCode::E), is_key_pressed(KeyCode::X))
            } else {
                (is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::O), is_key_pressed(KeyCode::U))
            };
            if up {
                bag_sel = bag_sel.saturating_sub(1);
            }
            if down && bag_sel + 1 < stored.len() {
                bag_sel += 1;
            }
            if act && bag_sel < stored.len() {
                let item = stored.remove(bag_sel);
                let ahead = ahead_local(&players[owner]);
                holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                holding_by = owner;
                show_inventory = false;
                bag_sel = bag_sel.min(stored.len().saturating_sub(1));
                toasts.push((format!("took the {} out - {} to place it", item.name(), if owner == 0 { "E" } else { "O" }), 2.5));
            }
            if clear && !house.pieces.is_empty() {
                if bag_clear_armed > 0.0 {
                    let n = house.pieces.len();
                    for piece in house.pieces.drain(..) {
                        stored.push(piece.item);
                    }
                    for k in 0..2 {
                        seat[k] = None;
                        players[k].seated = false;
                        players[k].sleeping = false;
                        dj[k] = false;
                        ddr.on[k] = false;
                    }
                    bag_clear_armed = 0.0;
                    save_clock = 99.0;
                    toasts.push((format!("packed away all {} pieces into your bag", n), 2.5));
                } else {
                    bag_clear_armed = 3.0;
                }
            }
        }
        if is_key_pressed(KeyCode::T) {
            if audio.tape_seconds().is_some() {
                let msg = match audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
                    Some(place) => format!("jam saved: {place}"),
                    None => "could not save the jam".to_string(),
                };
                toasts.push((msg, 4.0));
            } else {
                audio.start_tape();
                toasts.push(("tape rolling - T again to stop and save".to_string(), 2.5));
            }
        }
        if audio.tape_seconds().map_or(false, |t| t > TAPE_LIMIT) {
            if let Some(place) = audio.stop_tape().and_then(|mix| store::save_jam(&wav_bytes(&mix))) {
                toasts.push((format!("tape full (5 min) - jam saved: {place}"), 4.0));
            }
        }
        fresh_confirm = (fresh_confirm - get_frame_time()).max(0.0);
        let mut start_fresh = false;
        if is_key_pressed(KeyCode::N) && !picking {
            if fresh_confirm > 0.0 {
                start_fresh = true;
            } else {
                fresh_confirm = 3.0;
                toasts.push(("press N again to start a fresh world".to_string(), 3.0));
            }
        }
        if start_fresh {
            store::clear();
            world = World::new();
            inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
            camp = Camp { fuel: 0.0, lit: false, pile: 0 };
            dishes.clear();
            cook_progress = 0;
            cooking = false;
            fishing.active = false;
            fish_lesson = false;
            cook_lesson = false;
            gather_lesson = false;
            fish_card = 0.0;
            cook_card = 0.0;
            gather_card = 0.0;
            looper.notes.clear();
            looper.skip.clear();
            songbook.clear();
            partner_xp = 0;
            house = home::Home::new();
            dj = [false, false];
            stick_trees.clear();
            diary.clear();
            playback = None;
            inside = false;
            holding = None;
            shop_open = false;
            seat = [None, None];
            brain = DogBrain::new();
            for (who, spot) in [vec2(6.5, 16.5), vec2(7.5, 16.5)].into_iter().enumerate() {
                players[who].place_at(spot);
                players[who].seated = false;
                players[who].carrying = None;
            }
            fresh_confirm = 0.0;
            picking = true;
            first_pick = true;
            first_steps = 0;
            pick_row = 0;
        }
        {
            let labels: Vec<&str> = (0..6)
                .map(|i| {
                    if inside && dj[0] {
                        DJ_PADS[0][i]
                    } else if seat[0] == Some(home::Item::Piano) {
                        "piano"
                    } else if seat[0] == Some(home::Item::Drums) {
                        ["kick", "clap", "hat", "lid", "chop", "sizzle"][i]
                    } else if cooking && players[0].seated && i < 4 {
                        PERC_NAMES[i]
                    } else if music_mode {
                        voice_word(cast_list[players[0].character].voice)
                    } else {
                        SPELL_NAMES[i]
                    }
                })
                .collect();
            audio.set_boost(house.switched_on(home::Item::Speaker));
            let tool_label = if inside {
                let ahead = ahead_local(&players[0]);
                if shop_open {
                    "close".to_string()
                } else if dj[0] {
                    "leave".to_string()
                } else if holding.is_some() {
                    "turn".to_string()
                } else if seat[0].is_some() {
                    "get up".to_string()
                } else if house.piece_at(ahead.0, ahead.1).is_some() {
                    "move".to_string()
                } else {
                    "-".to_string()
                }
            } else {
                tools[0].name().to_string()
            };
            let report = format!(
                "mode={}|rec={}|echo={}|tape={}|tool={}|notes={}",
                if cooking { "cooking" } else if music_mode { "music" } else { "spells" },
                looper.recording as u8,
                ringing as u8,
                audio.tape_seconds().map(|t| t as u32).map_or(-1, |t| t as i64),
                tool_label,
                labels.join(",")
            );
            if report != last_report {
                radio::report(&report);
                last_report = report;
            }
        }
        radio_clock += get_frame_time();
        if radio_clock > 0.5 {
            radio_clock = 0.0;
            live_decks = radio::decks();
            live_next = radio::next();
            let title = radio::now_playing();
            if title != now_playing {
                if !title.is_empty() {
                    toasts.push((format!("now playing: {title}"), 4.0));
                }
                now_playing = title;
            }
            let bpm = radio::bpm();
            if bpm > 0.0 && (bpm - radio_bpm).abs() > 0.05 {
                looper.bpm = bpm.clamp(60.0, 200.0);
                toasts.push((format!("the beat follows the mix: {:.0} bpm", looper.bpm), 2.5));
            }
            radio_bpm = bpm;
        }
        save_clock += get_frame_time();
        let quitting = is_quit_requested();
        if (save_clock > 5.0 || quitting || restart_now) && !first_pick {
            save_clock = 0.0;
            let progress = Progress {
                tiles: world.tiles.iter().map(|t| (kind_index(t.kind), t.stack)).collect(),
                wood: inv.wood,
                stone: inv.stone,
                fish: inv.fish,
                fuel: camp.fuel,
                lit: camp.lit,
                pile: camp.pile,
                dishes: dishes.clone(),
                cook_progress,
                characters: [players[0].character, players[1].character],
                spots: if inside { [DOOR_OUT + vec2(0.0, 0.25), DOOR_OUT + vec2(1.0, 0.4)] } else { [players[0].pos, players[1].pos] },
                echo: ringing,
                beat: looper.beat,
                bpm: looper.bpm,
                quantise: [Quantise::Off, Quantise::Soft, Quantise::Tight].iter().position(|q| *q == looper.quantise).unwrap_or(1) as u8,
                notes: looper.notes.clone(),
                songbook: songbook.clone(),
                partner_xp,
                mix,
                acorns: inv.acorns + holding.map_or(0, |h| h.item.price() / 2),
                pieces: house.pieces.clone(),
                styles: (house.wall, house.floor, house.owned_styles, house.record_on),
                diary: diary.clone(),
                fish_lesson,
                cook_lesson,
                gather_lesson,
                room: house.size,
                dex: dex_counts.clone(),
                mush: inv.mush,
                shroomdex: shroom_counts.clone(),
                stored: stored.iter().map(|i| i.index()).collect(),
                seq: sampler.grid.map(|row| row.iter().enumerate().fold(0u16, |m, (k, on)| if *on { m | (1 << k) } else { m })),
                quests: if quests.day == i64::MIN { None } else { Some((quests.day, quests.list.iter().map(|q| (q.kind as u8, q.target, q.progress, q.done)).collect())) },
            };
            store::write(&encode_progress(&progress));
        }
        if quitting || restart_now {
            if let Some(mix) = audio.stop_tape() {
                store::save_jam(&wav_bytes(&mix));
            }
            if restart_now {
                restart_now = false;
                store::restart();
            } else {
                break;
            }
        }
        if is_key_pressed(KeyCode::G) {
            ringing = !ringing;
            audio.set_ring(ringing);
            ring_flash = 1.5;
        }
        audio.set_echo_gap(looper.step_seconds() * 3.0);
        audio.tick();
        ring_flash = (ring_flash - get_frame_time()).max(0.0);
        if is_key_pressed(KeyCode::Q) {
            looper.quantise = looper.quantise.next();
        }
        let mut picker_closed = false;
        if is_key_pressed(KeyCode::V) {
            picking = !picking;
            picker_closed = !picking;
        }
        if picking {
            if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                pick_row = 0;
            }
            if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                pick_row = 1;
            }
            let step = if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                cast_list.len() - 1
            } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                1
            } else {
                0
            };
            if step > 0 {
                let idx = (players[pick_row].character + step) % cast_list.len();
                players[pick_row].character = idx;
                players[pick_row].body = cast_list[idx].body;
                players[pick_row].voice_timer = 0.4;
                audio.play_ch(pick_row, &voices[idx][2], 0.7);
            }
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                picking = false;
                picker_closed = true;
            }
        }
        if picker_closed && first_pick {
            first_pick = false;
            first_steps = 1;
            toasts.push((format!("the {} trots off to find sticks", cast_list[players[1].character].noun), 4.0));
        }
        if is_key_pressed(KeyCode::X) && !picking && !inside {
            tools[0] = tools[0].next();
        }
        if is_key_pressed(KeyCode::U) && two_player && !inside {
            tools[1] = tools[1].next();
        }
        if is_key_pressed(KeyCode::F) {
            if listening {
                listening = false;
                looper.bpm = (looper.bpm * 10.0).round() / 10.0;
            } else {
                if microphone.is_none() {
                    microphone = mic::start();
                }
                if microphone.is_some() {
                    listening = true;
                    locked = false;
                    bpm_history.clear();
                    mic_message = None;
                } else {
                    mic_message = Some("no microphone");
                }
            }
        }

        if listening || sampler.recording.is_some() {
            if let Some(m) = microphone.as_mut() {
                m.pump();
                if m.blocked() {
                    listening = false;
                    locked = false;
                    microphone = None;
                    mic_message = Some("microphone blocked");
                }
            }
        }
        if listening {
            listen_clock += dt;
            if listen_clock >= 0.5 {
                listen_clock = 0.0;
                if let Some(m) = &microphone {
                    let (onsets, fps, level) = match m.detector.lock() {
                        Ok(d) => (d.onsets.iter().copied().collect::<Vec<f32>>(), d.fps, d.level),
                        Err(_) => (Vec::new(), 100.0, 0.0),
                    };
                    match tempo::estimate(&onsets, fps) {
                        Some(t) if t.confidence > 2.2 && level > 0.002 => {
                            bpm_history.push(t.bpm);
                            if bpm_history.len() > 6 {
                                bpm_history.remove(0);
                            }
                            let mut sorted = bpm_history.clone();
                            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            looper.bpm = sorted[sorted.len() / 2];
                            looper.sync_to_beat(t.since_beat);
                            locked = true;
                        }
                        _ => locked = false,
                    }
                }
            }
        }

        let p1_dir = if two_player {
            direction(&[KeyCode::W], &[KeyCode::S], &[KeyCode::A], &[KeyCode::D])
        } else {
            direction(
                &[KeyCode::W, KeyCode::Up],
                &[KeyCode::S, KeyCode::Down],
                &[KeyCode::A, KeyCode::Left],
                &[KeyCode::D, KeyCode::Right],
            )
        };
        let p2_dir = direction(
            &[KeyCode::I, KeyCode::Up],
            &[KeyCode::K, KeyCode::Down],
            &[KeyCode::J, KeyCode::Left],
            &[KeyCode::L, KeyCode::Right],
        );
        let (p1_dir, p2_dir) = if picking { (Vec2::ZERO, Vec2::ZERO) } else { (p1_dir, p2_dir) };
        if fishing.active && [p1_dir, p2_dir][fishing.caster] != Vec2::ZERO {
            fishing.active = false;
            toasts.push(("reeled in".to_string(), 1.5));
        }
        let mut go_in = false;
        let mut go_out = false;
        if inside {
            let h = &house;
            for who in 0..(if two_player { 2 } else { 1 }) {
                let dir = [p1_dir, p2_dir][who];
                let walk_dir = if (shop_open && shop_owner == who) || dj[who] || ddr.on[who] || (bag_busy && bag_owner == who) || (sampler.open && sampler.owner == who) { Vec2::ZERO } else { dir };
                players[who].step_with(walk_dir, dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
                if home::local(players[who].pos).y > house.d() as f32 + 0.15 {
                    go_out = true;
                }
            }
        } else {
            players[0].step(p1_dir, &world, dt);
            if two_player {
                players[1].step(p2_dir, &world, dt);
            }
            for who in 0..(if two_player { 2 } else { 1 }) {
                let dir = [p1_dir, p2_dir][who];
                if dir.y < -0.5 && players[who].pos.distance(DOOR_OUT) < 0.55 && !picking {
                    go_in = true;
                }
            }
        }
        if !players[0].seated && !(two_player && players[1].seated) {
            cooking = false;
        }
        let near_fire = |p: &Player| p.pos.distance(FIRE_SPOT) < 2.2;
        let at_dock_end = |p: &Player| p.pos.distance(DOCK_END) < 0.7;
        let mut fish_press = [false; 2];
        let mut build_request: Option<(usize, i32, i32)> = None;

        if !inside && !picking && !picker_closed {
            for who in 0..(if two_player { 2 } else { 1 }) {
                let pressed = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
                if !pressed {
                    continue;
                }
                if fish_card > 0.0 {
                    fish_card = 0.0;
                    fishing.cue_at = next_beat(looper.position) + FISH_CUE_GAP;
                    continue;
                }
                if cook_card > 0.0 {
                    cook_card = 0.0;
                    continue;
                }
                if gather_card > 0.0 {
                    gather_card = 0.0;
                    continue;
                }
                let player = &mut players[who];
                if fishing.active && player.pos.distance(DOCK_END) < 2.2 {
                    if fish_card > 0.0 {
                        fish_card = 0.0;
                        fishing.cue_at = next_beat(looper.position) + FISH_CUE_GAP;
                    } else {
                        fish_press[who] = true;
                    }
                } else if player.seated {
                    if camp.lit && near_fire(player) {
                        cooking = !cooking;
                        cook_progress = 0;
                        music_mode = true;
                        if cooking && !cook_lesson {
                            cook_card = 7.0;
                            cook_lesson = true;
                            save_clock = 99.0;
                        }
                        toasts.push((if cooking { "the pot goes on the fire".to_string() } else { "the pot comes off".to_string() }, 2.0));
                    }
                } else if camp.lit && near_fire(player) {
                    player.seated = true;
                    player.place_at(if who == 0 { CAT_SEAT } else { DOG_SEAT });
                    player.facing = if who == 0 { vec2(1.0, 0.0) } else { vec2(-1.0, 0.0) };
                    music_mode = true;
                    toasts.push((format!("player {} sits by the fire", who + 1), 2.5));
                } else if player.pos.distance(DOOR_OUT) < 0.8 {
                    go_in = true;
                } else if at_dock_end(player) && !fishing.active {
                    player.place_at(DOCK_END);
                    player.facing = vec2(1.0, 0.0);
                    let lesson = !fish_lesson;
                    if lesson {
                        fish_card = 6.0;
                    }
                    let wait = if lesson { (fish_card / looper.step_seconds() / 4.0).ceil() * 4.0 } else { 0.0 };
                    fishing = Fishing {
                        active: true,
                        caster: who,
                        cue_at: next_beat(looper.position) + FISH_CUE_GAP + wait,
                        hits: 0,
                        tries: FISH_TRIES,
                        hit_by: [false; 2],
                        early: [false; 2],
                        lesson,
                        perfects: 0,
                        dip: 0.0,
                        jump: None,
                        tight: if lesson { 1.0 } else { 0.72 + 0.2 * hash((time * 41.0) as i32, 6, 2) },
                    };
                    audio.play_ch(3, &splash, 0.35);
                    if !lesson {
                        let key = if who == 0 { "E" } else { "O" };
                        toasts.push((format!("cast the line - {key} when the ring closes"), 2.5));
                    }
                } else if let Some(k) = {
                    let (ax, az) = player.ahead();
                    let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                    shrooms.iter().position(|(at, _)| at.distance(player.pos) < 0.9 || at.distance(probe) < 0.8)
                } {
                    let (at, species) = shrooms.remove(k);
                    let is_new = shroom_counts[species] == 0;
                    shroom_counts[species] += 1;
                    inv.acorns += mushroom::acorns(species);
                    let note = if mushroom::edible(species) {
                        inv.mush[mushroom::kind(species)] += 1;
                        "into the basket"
                    } else {
                        "not for eating - logged in the dex"
                    };
                    fish_pop = Some((species, 3.0, is_new, true));
                    quest_event(&mut quests, QuestKind::Mushroom, 1, &mut inv, &mut toasts);
                    player.sniff = 0.6;
                    audio.play_ch(3, &plink, 0.45);
                    audio.play_ch(who, &voices[player.character][3], 0.5);
                    player.voice_timer = 0.3;
                    let col = mushroom::look(species).cap;
                    for n in 0..10 {
                        spawn(&mut particles, vec3(at.x, 0.35, at.y), vec3(hash(n, 3, 5) * 1.2 - 0.6, 1.0, hash(n, 4, 5) * 1.2 - 0.6), 0.7, col, 0.03, 0.6);
                    }
                    toasts.push((format!("player {} found a {} - {} (+{} acorns)", who + 1, mushroom::SPECIES[species], note, mushroom::acorns(species)), 3.0));
                    save_clock = 99.0;
                } else if let Some(tile) = {
                    let (ax, az) = player.ahead();
                    let probe = vec2(ax as f32 + 0.5, az as f32 + 0.5);
                    if player.carrying.is_none() {
                        stick_trees.iter().copied().find(|t| stick_spot(&world, *t).map(|p| p.distance(player.pos) < 0.95 || p.distance(probe) < 0.8).unwrap_or(false))
                    } else {
                        None
                    }
                } {
                    stick_trees.retain(|t| *t != tile);
                    if !gather_lesson {
                        gather_card = 7.0;
                        gather_lesson = true;
                        save_clock = 99.0;
                    }
                    let spot = stick_spot(&world, tile).unwrap_or(player.pos);
                    player.carrying = Some(Carry::Stick);
                    player.sniff = 0.6;
                    puff(&mut particles, spot.x.floor() as i32, spot.y.floor() as i32, Color::from_rgba(110, 170, 80, 255), 8);
                    audio.play_ch(3, &chop, 0.35);
                    toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                    quest_event(&mut quests, QuestKind::Sticks, 1, &mut inv, &mut toasts);
                } else {
                    let (x, z) = player.ahead();
                    match gather_at(&world, &mut stick_trees, &mut tree_rest, (x, z), time, player.carrying.is_none()) {
                        Gathered::Stick => {
                            quest_event(&mut quests, QuestKind::Sticks, 1, &mut inv, &mut toasts);
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            player.carrying = Some(Carry::Stick);
                            player.sniff = 0.6;
                            puff(&mut particles, x, z, Color::from_rgba(110, 170, 80, 255), 8);
                            audio.play_ch(3, &chop, 0.35);
                            toasts.push((format!("player {} got a stick - bring it to the fire", who + 1), 2.5));
                        }
                        Gathered::Stone => {
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            player.carrying = Some(Carry::Stone);
                            player.sniff = 0.6;
                            puff(&mut particles, x, z, Color::from_rgba(170, 170, 180, 255), 8);
                            toasts.push((format!("player {} got a stone - bring it to the fire", who + 1), 2.5));
                        }
                        Gathered::Acorns(found) => {
                            quest_event(&mut quests, QuestKind::Shake, 1, &mut inv, &mut toasts);
                            if !gather_lesson {
                                gather_card = 7.0;
                                gather_lesson = true;
                                save_clock = 99.0;
                            }
                            inv.acorns += found;
                            audio.play_ch(3, &chop, 0.35);
                            audio.play_ch(3, &plink, 0.3);
                            player.sniff = 0.4;
                            puff(&mut particles, x, z, Color::from_rgba(110, 170, 80, 255), 10);
                            for k in 0..found as i32 * 2 {
                                spawn(&mut particles, vec3(x as f32 + 0.5, 1.6, z as f32 + 0.5), vec3(hash(k, x, 3) * 1.2 - 0.6, 0.6, hash(k, z, 4) * 1.2 - 0.6), 0.9, ACORN_COLOR, 0.06, 1.0);
                            }
                            toasts.push((format!("player {} shook the tree: +{} acorn{}", who + 1, found, if found == 1 { "" } else { "s" }), 2.5));
                        }
                        Gathered::Resting => toasts.push(("this tree needs a rest - try another".to_string(), 2.0)),
                        Gathered::Nothing => if !fishing.active { build_request = Some((who, x, z)); },
                    }
                }
            }
        }

        if inside && !picking {
            for who in 0..(if two_player { 2 } else { 1 }) {
                if seat[who].is_some() && !players[who].seated {
                    seat[who] = None;
                }
            }
            if let Some(piece) = holding.as_mut() {
                let ahead = ahead_local(&players[holding_by]);
                piece.x = ahead.0;
                piece.z = ahead.1;
            }
            let rows = home::shop_rows();
            let shop_left = if shop_owner == 0 { is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) } else { is_key_pressed(KeyCode::J) };
            let shop_right = if shop_owner == 0 { is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) } else { is_key_pressed(KeyCode::L) };
            let shop_up = if shop_owner == 0 { is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) } else { is_key_pressed(KeyCode::I) };
            let shop_down = if shop_owner == 0 { is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) } else { is_key_pressed(KeyCode::K) };
            let shop_act = if shop_owner == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
            let shop_turn = if shop_owner == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
            let ahead = ahead_local(&players[shop_owner]);
            let was_shop = shop_open;
            let was_dj = dj[0];
            if dj[0] && !was_shop {
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    looper.bpm = (looper.bpm + 2.0).min(170.0);
                }
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    looper.bpm = (looper.bpm - 2.0).max(80.0);
                }
                let n = BEAT_STYLES.len();
                let mut changed = false;
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    beat_style = (beat_style + n - 1) % n;
                    changed = true;
                }
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    beat_style = (beat_style + 1) % n;
                    changed = true;
                }
                if changed {
                    looper.beat = BEAT_STYLES[beat_style].name != "off";
                }
                if is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Escape) {
                    dj[0] = false;
                }
            } else if was_shop {
                if shop_up {
                    shop_tab = shop_tab.saturating_sub(1);
                }
                if shop_down {
                    shop_tab = (shop_tab + 1).min(3);
                }
                if shop_tab >= 2 {
                    dex_mode = shop_tab - 2;
                    shop_tab = 1;
                    shop_open = false;
                    dex_open = true;
                    dex_from_shop = true;
                    dex_query.clear();
                    dex_sel = 0;
                    while get_char_pressed().is_some() {}
                }
                if shop_tab == 0 {
                    if shop_left {
                        shop_row = (shop_row + rows - 1) % rows;
                    }
                    if shop_right {
                        shop_row = (shop_row + 1) % rows;
                    }
                } else if !diary.is_empty() {
                    let n = diary.len();
                    if shop_left {
                        diary_row = (diary_row + n - 1) % n;
                    }
                    if shop_right {
                        diary_row = (diary_row + 1) % n;
                    }
                    diary_row = diary_row.min(n - 1);
                    if shop_act && !picker_closed {
                        let idx = n - 1 - diary_row;
                        if !house.has(home::Item::RecordPlayer) {
                            toasts.push(("buy a record player to listen to your jams".to_string(), 2.5));
                        } else if diary[idx].events.is_empty() {
                            toasts.push(("this page has no recording left".to_string(), 2.0));
                        } else {
                            playback = Some((idx, get_time(), 0));
                            house.record_on = true;
                            shop_open = false;
                            toasts.push((format!("the record player plays your jam from {}", date_label(diary[idx].when)), 3.0));
                        }
                    }
                }
                if shop_turn || is_key_pressed(KeyCode::Escape) {
                    shop_open = false;
                }
                if shop_act && !picker_closed && shop_tab == 0 {
                    match home::row(shop_row) {
                        home::Row::Upgrade => match house.next_upgrade() {
                            None => toasts.push(("your home is fully built".to_string(), 2.0)),
                            Some((name, _, price)) => {
                                if inv.acorns < price {
                                    toasts.push((format!("the {} costs {} acorns", name, price), 2.0));
                                } else {
                                    inv.acorns -= price;
                                    let mut way = house.in_the_way();
                                    way.sort_unstable_by(|a, b| b.cmp(a));
                                    let moved = way.len();
                                    for i in way {
                                        stored.push(house.pieces.remove(i).item);
                                    }
                                    if moved > 0 {
                                        toasts.push((format!("{} piece{} in the way went into your bag (M)", moved, if moved == 1 { "" } else { "s" }), 4.0));
                                    }
                                    house.size += 1;
                                    shop_open = false;
                                    save_clock = 99.0;
                                    for k in 0..6 {
                                        audio.play_ch(3, &bells[k], 0.25);
                                    }
                                    let msg = if house.size > home::LOFT_SIZE { "the penthouse grows - your loft upstairs is twice as deep now".to_string() } else if house.loft() { "penthouse! take the stairs at the back right up to your loft".to_string() } else { format!("the walls slide out - your room is now {} x {}", house.w(), house.d()) };
                                    toasts.push((msg, 4.0));
                                }
                            }
                        },
                        home::Row::Item(item) => {
                            if inv.acorns >= item.price() {
                                inv.acorns -= item.price();
                                holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                                holding_by = shop_owner;
                                shop_open = false;
                                audio.play_ch(3, &plink, 0.6);
                                toasts.push((format!("bought a {} - {} to place it, {} to turn it", item.name(), if shop_owner == 0 { "E" } else { "O" }, if shop_owner == 0 { "X" } else { "U" }), 3.0));
                            } else {
                                toasts.push((format!("a {} costs {} acorns - you have {}", item.name(), item.price(), inv.acorns), 2.5));
                            }
                        }
                        home::Row::Wall(i) | home::Row::Floor(i) => {
                            let is_wall = matches!(home::row(shop_row), home::Row::Wall(_));
                            let bit = if is_wall { home::wall_bit(i) } else { home::floor_bit(i) };
                            let owned = house.owned_styles & bit != 0;
                            if owned || inv.acorns >= home::STYLE_PRICE {
                                if !owned {
                                    inv.acorns -= home::STYLE_PRICE;
                                    house.owned_styles |= bit;
                                }
                                if is_wall {
                                    house.wall = i as u8;
                                } else {
                                    house.floor = i as u8;
                                }
                                audio.play_ch(3, &bells[i % 6], 0.5);
                            } else {
                                toasts.push((format!("that costs {} acorns", home::STYLE_PRICE), 2.0));
                            }
                        }
                        home::Row::Set => {
                            let added = house.techno_club(&[home::local(players[0].pos), home::local(players[1].pos)]);
                            looper.beat = true;
                            looper.bpm = 132.0;
                            shop_open = false;
                            audio.play_ch(2, &kick, 0.9);
                            toasts.push((format!("techno club mode: concrete, red light, 132 bpm ({} pieces fit)", added), 3.5));
                        }
                    }
                }
            }
            for who in 0..(if two_player { 2 } else { 1 }) {
                let act = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
                let turn = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
                if was_shop || shop_open || (who == 0 && was_dj) {
                    continue;
                }
                if dj[who] {
                    if who == 1 && act {
                        dj[who] = false;
                    }
                    continue;
                }
                if ddr.on[who] || (bag_busy && bag_owner == who) || (sampler.open && sampler.owner == who) {
                    continue;
                }
                let ahead = ahead_local(&players[who]);
                let here = home::local(players[who].pos);
                let near_catalogue = here.distance(vec2(0.5, 0.5)) < 1.4;
                if act && !picker_closed {
                    if let Some(piece) = holding {
                        if holding_by == who {
                            if ahead == home::CATALOGUE {
                                let back = piece.item.price() / 2;
                                inv.acorns += back;
                                holding = None;
                                toasts.push((format!("sold the {} back for {} acorns", piece.item.name(), back), 2.5));
                            } else if house.can_place(&piece, &[home::local(players[1 - who].pos)]) {
                                house.pieces.push(piece);
                                holding = None;
                                audio.play_ch(3, &marimba[(piece.x + piece.z).rem_euclid(6) as usize], 0.7);
                                if piece.item == home::Item::Speaker {
                                    toasts.push(("the speaker makes your echo bigger (G for echo)".to_string(), 3.0));
                                }
                            } else {
                                toasts.push(("it doesn't fit there".to_string(), 1.8));
                            }
                        }
                    } else if seat[who].is_some() {
                        players[who].seated = false;
                        players[who].sleeping = false;
                        seat[who] = None;
                    } else if near_catalogue {
                        shop_open = true;
                        shop_owner = who;
                        audio.play_ch(3, &plink, 0.4);
                    } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                        let item = house.pieces[i].item;
                        match item {
                            home::Item::Piano | home::Item::Drums => {
                                seat[who] = Some(item);
                                players[who].seated = true;
                                toasts.push((format!("playing the {} - {} to play, move to get up", item.name(), if who == 0 { "1-6" } else { "7-=" }), 3.0));
                            }
                            home::Item::DjBooth => {
                                dj[who] = true;
                                music_mode = true;
                                if !looper.beat {
                                    looper.beat = true;
                                    looper.bpm = 132.0;
                                    beat_style = 0;
                                }
                            }
                            home::Item::RecordPlayer => {
                                playback = None;
                                house.record_on = !house.record_on;
                                toasts.push((if house.record_on { "the record spins".to_string() } else { "the record stops".to_string() }, 2.0));
                            }
                            home::Item::WindChime => {
                                for k in 0..3 {
                                    audio.play_ch(3, &bells[(k * 2 + (time as usize)) % 6], 0.3);
                                }
                            }
                            home::Item::Bed => {
                                let piece = house.pieces[i];
                                seat[who] = Some(item);
                                players[who].seated = true;
                                players[who].sleeping = true;
                                players[who].yaw = std::f32::consts::PI;
                                let front = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                                players[who].facing = vec2(front.x, front.z);
                                players[who].sleep_yaw = piece.yaw();
                                players[who].perch = 0.06;
                                let side = if who == 0 { -0.42 } else { 0.42 };
                                let offset = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
                                players[who].place_at(home::world(piece.center() + vec2(offset.x, offset.z)));
                                if who == 0 && !two_player {
                                    let friend_side = rotate_y(vec3(0.42, 0.0, 0.0), piece.yaw());
                                    let friend = &mut players[1];
                                    friend.carrying = None;
                                    friend.seated = true;
                                    friend.sleeping = true;
                                    friend.yaw = std::f32::consts::PI;
                                    friend.facing = vec2(front.x, front.z);
                                    friend.sleep_yaw = piece.yaw();
                                    friend.perch = 0.06;
                                    friend.place_at(home::world(piece.center() + vec2(friend_side.x, friend_side.z)));
                                }
                                for k in 0..3 {
                                    hearts.push((vec3(players[who].pos.x + k as f32 * 0.15 - 0.15, 1.32 + k as f32 * 0.12, players[who].pos.y), 1.4));
                                }
                                sleep_fade = 3.0;
                                save_clock = 99.0;
                            }
                            home::Item::Sofa | home::Item::ConcreteBench => {
                                let piece = house.pieces[i];
                                let out = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                                seat[who] = Some(item);
                                players[who].seated = true;
                                players[who].perch = if item == home::Item::Sofa { 0.2 } else { 0.24 };
                                let (w, _) = piece.item.size();
                                let side = if w > 1 { if who == 0 { -0.45 } else { 0.45 } } else { 0.0 };
                                let along = rotate_y(vec3(side, 0.0, 0.05), piece.yaw());
                                players[who].place_at(home::world(piece.center() + vec2(along.x, along.z)));
                                players[who].facing = vec2(out.x, out.z);
                                toasts.push((format!("sitting on the {} - move to get up", item.name()), 2.0));
                            }
                            home::Item::Lamp | home::Item::RedLight | home::Item::Speaker | home::Item::Subwoofer => {
                                let piece = &mut house.pieces[i];
                                piece.state = if piece.state == 0 { 1 } else { 0 };
                                let on = piece.state == 0;
                                audio.play_ch(3, &plink, 0.45);
                                let msg = match item {
                                    home::Item::Speaker => if on { "speaker on - bigger echo" } else { "speaker off" },
                                    home::Item::Subwoofer => if on { "subwoofer on - heavier kick" } else { "subwoofer off" },
                                    _ => if on { "light on" } else { "light off" },
                                };
                                toasts.push((msg.to_string(), 1.5));
                            }
                            home::Item::Plant => {
                                let piece = &mut house.pieces[i];
                                let revived = piece.state == home::WITHERED;
                                if revived {
                                    piece.state = 0;
                                }
                                let first_today = piece.day != sky().day as i32;
                                piece.day = sky().day as i32;
                                if !revived && piece.state < 3 && first_today {
                                    piece.state += 1;
                                }
                                audio.play_ch(3, &bells[(piece.state as usize + 1) % 6], 0.4);
                                let msg = if revived {
                                    "you water the withered plant - it slowly recovers"
                                } else if !first_today {
                                    "it already had water today - come back tomorrow"
                                } else {
                                    match piece.state {
                                        1 => "you water the plant - it perks up",
                                        2 => "you water the plant - it grows",
                                        _ => "the plant is in full bloom",
                                    }
                                };
                                quest_event(&mut quests, QuestKind::Water, 1, &mut inv, &mut toasts);
                                toasts.push((msg.to_string(), 2.0));
                            }
                            home::Item::Bookshelf => {
                                audio.play_ch(3, &plink, 0.3);
                                shelf_open = true;
                                open_book = None;
                                page_turn = 0.0;
                            }
                            home::Item::Table => {
                                let piece = &mut house.pieces[i];
                                let mut known: Vec<usize> = dishes.clone();
                                known.sort();
                                if known.is_empty() {
                                    toasts.push(("nothing to serve yet - cook by the fire first".to_string(), 2.5));
                                } else {
                                    let current = piece.state as usize;
                                    let next = known.iter().find(|d| **d + 1 > current).map(|d| *d + 1).unwrap_or(0);
                                    piece.state = next as u8;
                                    audio.play_ch(3, &plink, 0.4);
                                    if next == 0 {
                                        toasts.push(("you clear the table".to_string(), 1.5));
                                    } else {
                                        toasts.push((format!("you serve {} - a friend comes to eat", DISHES[next - 1]), 2.5));
                                        for k in 0..3 {
                                            hearts.push((vec3(players[1 - who].pos.x + k as f32 * 0.15 - 0.15, 1.0 + k as f32 * 0.12, players[1 - who].pos.y), 1.4));
                                        }
                                    }
                                }
                            }
                            home::Item::SteelPillar => {
                                audio.play_ch(3, &bells[0], 0.6);
                                audio.play_ch(3, &bells[3], 0.25);
                            }
                            home::Item::MirrorBall => {
                                let piece = &mut house.pieces[i];
                                piece.state = if piece.state == 0 { 1 } else { 0 };
                                audio.play_ch(3, &bells[5], 0.35);
                                toasts.push((if piece.state == 0 { "the mirror ball spins".to_string() } else { "the mirror ball rests".to_string() }, 1.5));
                            }
                            home::Item::Tv => {
                                let piece = &mut house.pieces[i];
                                piece.state = (piece.state + 1) % 6;
                                audio.play_ch(3, &plink, 0.35);
                                let msg = if piece.state == 0 { "tv off".to_string() } else { format!("channel {}: {}", piece.state, TV_CHANNELS[piece.state as usize - 1]) };
                                toasts.push((msg, 1.6));
                            }
                            home::Item::Arcade => {
                                ddr_request = Some((who, i));
                            }
                            home::Item::Sampler => {
                                sampler.open = true;
                                sampler.owner = who;
                                audio.play_ch(3, &plink, 0.35);
                            }
                            home::Item::Rug => {
                                for k in 0..4 {
                                    hearts.push((vec3(players[who].pos.x + k as f32 * 0.12 - 0.18, 0.9 + k as f32 * 0.1, players[who].pos.y), 1.2));
                                }
                                toasts.push(("a little dance on the rug".to_string(), 1.5));
                            }
                        }
                    } else if here.y > house.d() as f32 - 1.1 && (here.x - house.door().x).abs() < 0.9 {
                        go_out = true;
                    }
                }
                if turn && !shop_open {
                    if let Some(piece) = holding.as_mut() {
                        if holding_by == who {
                            piece.rot = (piece.rot + 1) % 4;
                        }
                    } else if seat[who].is_some() {
                        players[who].seated = false;
                        players[who].sleeping = false;
                        seat[who] = None;
                    } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                        let piece = house.pieces.remove(i);
                        toasts.push((format!("moving the {} - {} to place, {} to turn, or sell it at the catalogue", piece.item.name(), if who == 0 { "E" } else { "O" }, if who == 0 { "X" } else { "U" }), 3.0));
                        holding = Some(piece);
                        holding_by = who;
                    }
                }
            }
            if house.has(home::Item::WindChime) {
                chime_clock -= dt;
                if chime_clock <= 0.0 {
                    chime_clock = 2.5 + hash((time * 13.0) as i32, 4, 4) * 4.0;
                    audio.play_ch(3, &bells[(hash((time * 7.0) as i32, 2, 9) * 6.0) as usize % 6], 0.16);
                }
            }
        }

        let mut delivered: Option<Carry> = None;
        let mut delivered_by = 1usize;
        if !inside && players[0].carrying.is_some() && players[0].pos.distance(STICK_DROP) < 0.9 {
            delivered = players[0].carrying.take();
            delivered_by = 0;
        }
        if two_player && !inside {
            if players[1].carrying.is_some() && players[1].pos.distance(STICK_DROP) < 0.9 {
                delivered = players[1].carrying.take();
                delivered_by = 1;
            }
        } else if !inside {
            let cat_pos = players[0].pos;
            let dog = &mut players[1];
            let want_fire = camp.fuel < FUEL_WANT;
            let mut dir = Vec2::ZERO;
            let mut next: Option<Task> = None;
            brain.rest = (brain.rest - dt).max(0.0);
            if first_pick {
                brain.rest = brain.rest.max(0.5);
            }
            match &mut brain.task {
                Task::Idle => {
                    dog.seated = false;
                    let errand = if first_pick || (jam && camp.fuel > 40.0) {
                        None
                    } else if want_fire {
                        Some(Carry::Stick)
                    } else if brain.rest > 0.0 || fishing.active {
                        None
                    } else if inv.wood < 8 && inv.wood <= inv.stone {
                        Some(Carry::Stick)
                    } else if inv.stone < 8 {
                        Some(Carry::Stone)
                    } else if inv.wood < 8 {
                        Some(Carry::Stick)
                    } else {
                        Some(Carry::Acorn)
                    };
                    if let (Some(carry), None) = (errand, dog.carrying) {
                        let kind = if carry == Carry::Stone { Kind::Stone } else { Kind::Tree };
                        let marked = if carry == Carry::Stick { nearest_stick_tree(&world, &stick_trees, dog.pos) } else { None };
                        let found = match marked {
                            Some(m) => Some(m),
                            None => find_source(&mut world, dog.pos, kind),
                        };
                        if let Some((tile, spot)) = found {
                            next = Some(Task::Seek { spot, tile, carry });
                            brain.trip = 0.0;
                        }
                    } else if dog.carrying.is_some() {
                        next = Some(Task::Return);
                    } else if jam {
                        next = Some(Task::Sit);
                    } else {
                        let goal = if fishing.active { DOCK_END - vec2(1.0, 0.0) } else { cat_pos };
                        if dog.pos.distance(goal) > if fishing.active { 0.3 } else { 2.0 } {
                            dir = steer(&world, dog.pos, goal);
                        } else if fishing.active {
                            dog.facing = vec2(1.0, 0.0);
                        }
                    }
                }
                Task::Seek { spot, tile, carry } => {
                    dir = steer(&world, dog.pos, *spot);
                    if dog.pos.distance(*spot) < 0.3 {
                        dir = Vec2::ZERO;
                        let t = *tile;
                        dog.facing = (vec2(t.0 as f32 + 0.5, t.1 as f32 + 0.5) - dog.pos).normalize_or_zero();
                        next = Some(Task::Sniff { tile: t, timer: 1.2, carry: *carry });
                    }
                }
                Task::Sniff { tile, timer, carry } => {
                    *timer -= dt;
                    dog.sniff = 0.2;
                    if *timer <= 0.0 {
                        if *carry == Carry::Stick {
                            let t = *tile;
                            stick_trees.retain(|s| *s != t);
                        }
                        dog.carrying = Some(*carry);
                        let col = match *carry {
                            Carry::Stick => Color::from_rgba(110, 170, 80, 255),
                            Carry::Stone => Color::from_rgba(170, 170, 180, 255),
                            Carry::Acorn => ACORN_COLOR,
                        };
                        puff(&mut particles, tile.0, tile.1, col, 8);
                        next = Some(Task::Return);
                        brain.trip = 0.0;
                    }
                }
                Task::Return => {
                    dir = steer(&world, dog.pos, STICK_DROP);
                    if dog.pos.distance(STICK_DROP) < 0.45 {
                        delivered = dog.carrying.take();
                        brain.rest = if want_fire { 0.0 } else if delivered == Some(Carry::Acorn) { 14.0 } else { 5.0 };
                        next = Some(Task::Idle);
                    }
                }
                Task::Sit => {
                    if !jam || camp.fuel < 40.0 {
                        dog.seated = false;
                        next = Some(Task::Idle);
                    } else if !dog.seated {
                        dir = steer(&world, dog.pos, DOG_SEAT);
                        if dog.pos.distance(DOG_SEAT) < 0.45 {
                            dir = Vec2::ZERO;
                            dog.place_at(DOG_SEAT);
                            dog.seated = true;
                            dog.facing = vec2(-1.0, 0.0);
                        }
                    }
                }
            }
            let target = match &brain.task {
                Task::Seek { spot, .. } => Some(*spot),
                Task::Return => Some(STICK_DROP),
                _ => None,
            };
            let moved = dog.step(dir, &world, dt);
            if dir != Vec2::ZERO && moved < 0.3 * dt {
                brain.stuck += dt;
            } else {
                brain.stuck = 0.0;
            }
            brain.trip += dt;
            if let Some(goal) = target {
                if brain.stuck > 1.2 {
                    let hop = dog.pos + (goal - dog.pos).normalize_or_zero() * 1.5;
                    let spot = rescue(&world, hop);
                    dog.place_at(spot);
                    brain.stuck = 0.0;
                }
                if brain.trip > 25.0 {
                    let spot = rescue(&world, goal);
                    dog.place_at(spot);
                    brain.trip = 0.0;
                }
            } else if brain.stuck > 2.0 {
                let spot = rescue(&world, dog.pos + vec2(0.0, 1.0));
                dog.place_at(spot);
                brain.stuck = 0.0;
            }
            if let Some(t) = next {
                brain.task = t;
            }
        }

        if inside && !two_player && players[1].sleeping && !players[0].sleeping {
            players[1].sleeping = false;
            players[1].seated = false;
            players[1].perch = 0.0;
            let h = &house;
            let spot = h.free_spot_near(home::local(players[1].pos));
            players[1].place_at(home::world(spot));
        }
        if inside && !two_player && !players[1].sleeping {
            let h = &house;
            wander_clock -= dt;
                let goal = if seat[0].is_some() || dj[0] { players[0].pos + vec2(0.9, 0.4) } else { wander_to };
                let to_goal = goal - players[1].pos;
                let dir = if to_goal.length() > 0.35 { to_goal.normalize() } else { Vec2::ZERO };
                let moved = players[1].step_with(dir, dt, |p| h.walkable(home::local(p), CAT_RADIUS), |p| home::world(h.free_spot_near(home::local(p))));
                if wander_clock <= 0.0 || (dir != Vec2::ZERO && moved < 0.2 * dt) {
                    wander_clock = 3.0 + hash((time * 11.0) as i32, 6, 1) * 4.0;
                    wander_to = home::world(h.free_spot(time * 0.731 + 0.17));
                }
                if dir == Vec2::ZERO && (seat[0].is_some() || dj[0]) {
                    players[1].facing = (players[0].pos - players[1].pos).normalize_or_zero();
                }
        }
        for p in players.iter_mut() {
            p.lift = if inside { house.height_at(home::local(p.pos)) } else { 0.0 };
        }
        if go_in && !inside {
            inside = true;
            fishing.active = false;
            cooking = false;
            for p in players.iter_mut() {
                p.seated = false;
                p.sleeping = false;
            }
            seat = [None, None];
            dj = [false, false];
            shop_open = false;
            let door = house.door();
            players[0].place_at(home::world(door - vec2(0.0, 0.45)));
            players[0].facing = vec2(0.0, -1.0);
            players[1].place_at(home::world(door - vec2(1.0, 0.35)));
            players[1].facing = vec2(0.0, -1.0);
            wander_clock = 2.0;
            audio.play_ch(3, &chop, 0.3);
            toasts.push(("home sweet home".to_string(), 2.5));
        }
        if go_out && inside {
            if holding.is_some() {
                players[holding_by].place_at(home::world(vec2(3.5, 4.0)));
                toasts.push(("put your furniture down first".to_string(), 2.0));
            } else {
                inside = false;
                seat = [None, None];
                for p in players.iter_mut() {
                    p.seated = false;
                    p.sleeping = false;
                }
                dj = [false, false];
                shop_open = false;
                players[0].place_at(DOOR_OUT + vec2(0.0, 0.25));
                players[0].facing = vec2(0.0, 1.0);
                let spot = rescue(&world, DOOR_OUT + vec2(1.0, 0.4));
                players[1].place_at(spot);
                brain = DogBrain::new();
                audio.play_ch(3, &chop, 0.3);
            }
        }

        let friend = cast_list[players[1].character].noun;
        let bringer = if delivered_by == 0 { "you".to_string() } else if two_player { "player 2".to_string() } else { format!("the {}", friend) };
        let bringer_voice = players[delivered_by].character;
        let bringer_ch = delivered_by;
        match delivered {
            Some(Carry::Stick) if camp.fuel < FUEL_WANT => {
                camp.fuel = (camp.fuel + STICK_FUEL).min(240.0);
                camp.pile = (camp.fuel / STICK_FUEL).ceil() as u32;
                audio.play_ch(bringer_ch, &voices[bringer_voice][2], 0.7);
                players[delivered_by].voice_timer = 0.4;
                if !camp.lit {
                    camp.lit = true;
                    toasts.push((format!("{} brought a stick - the fire is lit!", bringer), 3.5));
                    for k in 0..30 {
                        let a = k as f32 / 30.0 * 2.0 * PI;
                        spawn(&mut particles, fire_center() + vec3(0.0, 0.3, 0.0), vec3(a.cos() * 1.5, 2.0 + hash(k, 1, 3) * 1.5, a.sin() * 1.5), 0.9, Color::from_rgba(255, 190, 80, 255), 0.04, 0.6);
                    }
                } else {
                    toasts.push((format!("{} brought a stick for the fire", bringer), 2.5));
                }
            }
            Some(Carry::Stick) => {
                inv.wood += 1;
                audio.play_ch(bringer_ch, &voices[bringer_voice][3], 0.6);
                players[delivered_by].voice_timer = 0.4;
                toasts.push((format!("{} brought wood for building", bringer), 2.5));
            }
            Some(Carry::Acorn) => {
                inv.acorns += 2;
                audio.play_ch(1, &voices[players[1].character][1], 0.6);
                players[1].voice_timer = 0.4;
                toasts.push((format!("the {} brought you 2 acorns", friend), 2.5));
            }
            Some(Carry::Stone) => {
                inv.stone += 1;
                audio.play_ch(bringer_ch, &voices[bringer_voice][4], 0.6);
                players[delivered_by].voice_timer = 0.4;
                toasts.push((format!("{} brought a stone for building", bringer), 2.5));
            }
            None => {}
        }

        if let Some((who, x, z)) = build_request {
            let blocked = players.iter().any(|p| touches_cat(x, z, p.pos));
            match build(&mut world, &mut inv, tools[who], x, z, blocked) {
                Ok(Some((kind, stack))) => {
                    let i = ((x + z + stack as i32).rem_euclid(6)) as usize;
                    match kind {
                        Kind::WoodWall => audio.play_ch(3, &marimba[i], 0.8),
                        Kind::StoneWall => audio.play_ch(3, &bells[i], 0.8),
                        _ => audio.play_ch(3, &chop, 0.7),
                    }
                    puff(&mut particles, x, z, Color::from_rgba(230, 210, 170, 255), 6);
                    players[who].voice_timer = 0.2;
                }
                Ok(None) => {}
                Err(msg) => toasts.push((msg.to_string(), 2.5)),
            }
        }

        if sampler.open && !inside {
            sampler.open = false;
        }
        if sampler.open {
            let o = sampler.owner;
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
            let (mut cx, mut cy) = sampler.cursor;
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
            sampler.cursor = (cx, cy);
            if act {
                sampler.grid[cy][cx] = !sampler.grid[cy][cx];
                save_clock = 99.0;
            }
            if close {
                sampler.open = false;
            }
            if is_key_pressed(KeyCode::Space) && sampler.recording.is_none() {
                if sampler.pads[cy].is_some() {
                    sampler.pads[cy] = None;
                    toasts.push((format!("the {} row is back to the built-in sound", SAMPLER_ROWS[cy]), 2.0));
                } else {
                    if microphone.is_none() {
                        microphone = mic::start();
                    }
                    if let Some(m) = &microphone {
                        if let Ok(mut det) = m.detector.lock() {
                            det.capture_len = (det.rate * 1.5) as usize;
                            det.capture = Some(Vec::new());
                        }
                        sampler.recording = Some((cy, 0.0));
                    } else {
                        toasts.push(("no microphone - the built-in sounds still work".to_string(), 2.5));
                    }
                }
            }
        }
        if let Some((row, waited)) = sampler.recording {
            let waited = waited + dt;
            let mut finished: Option<Option<Vec<f32>>> = None;
            if let Some(m) = &microphone {
                if let Ok(mut det) = m.detector.lock() {
                    let full = det.capture.as_ref().map(|c| c.len() >= det.capture_len).unwrap_or(true);
                    if full || waited > 6.0 {
                        let raw = det.capture.take().unwrap_or_default();
                        finished = Some(tidy_recording(&raw, det.rate));
                    }
                }
            } else {
                finished = Some(None);
            }
            sampler.recording = Some((row, waited));
            if let Some(result) = finished {
                sampler.recording = None;
                match result {
                    Some(samples) => {
                        sampler.pads[row] = Some(audio.load(samples));
                        toasts.push((format!("the {} row now plays your own sound", SAMPLER_ROWS[row]), 2.5));
                    }
                    None => toasts.push(("didn't hear anything - try again a bit louder".to_string(), 2.5)),
                }
            }
        }
        if let Some((who, i)) = ddr_request.take() {
            if !ddr.on[who] && i < house.pieces.len() {
                let piece = house.pieces[i];
                let front = rotate_y(vec3(0.0, 0.0, 0.95), piece.yaw());
                let side = if two_player { if who == 0 { -0.22 } else { 0.22 } } else { 0.0 };
                let across = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
                let spot = piece.center() + vec2(front.x + across.x, front.z + across.z);
                players[who].pos = home::world(spot);
                players[who].facing = vec2(-front.x, -front.z).normalize_or_zero();
                if !ddr.on[0] && !ddr.on[1] {
                    ddr.arrows.clear();
                }
                ddr.on[who] = true;
                ddr.score[who] = 0;
                ddr.combo[who] = 0;
                if microphone.is_none() {
                    microphone = mic::start();
                }
                if microphone.is_some() && !listening {
                    listening = true;
                    locked = false;
                    bpm_history.clear();
                    mic_message = None;
                }
                if !looper.beat && microphone.is_none() {
                    looper.beat = true;
                }
                music_mode = true;
                audio.play_ch(3, &bells[4], 0.4);
                let keys = if who == 0 { "arrow keys or WASD" } else { "J K I L" };
                toasts.push((format!("dance! step with {keys} when the arrows reach the top - {} to stop", if who == 0 { "X" } else { "U" }), 3.5));
            }
        }
        if ddr.on[0] || ddr.on[1] {
            if !inside {
                ddr.on = [false; 2];
            }
            let step_s = looper.step_seconds();
            if !looper.beat && time - ddr.last_mic > 4.0 && looper.notes.is_empty() {
                looper.beat = true;
            }
            if listening {
                if let Some(m) = &microphone {
                    let hits: Vec<(usize, f32)> = m.detector.lock().map(|mut d| d.hits.drain(..).collect()).unwrap_or_default();
                    for (band, _) in hits {
                        ddr.last_mic = time;
                        let p = looper.position;
                        let g = (p / 2.0).round() * 2.0;
                        let target = time + (g - p) * step_s + 16.0 * step_s;
                        ddr.add(DDR_LANE_OF_BAND[band], target, step_s * 2.0);
                    }
                }
            }
            for who in 0..2 {
                if !ddr.on[who] {
                    continue;
                }
                let keys: [bool; 4] = if who == 0 {
                    [
                        is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
                        is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
                        is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
                        is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
                    ]
                } else {
                    [is_key_pressed(KeyCode::J), is_key_pressed(KeyCode::K), is_key_pressed(KeyCode::I), is_key_pressed(KeyCode::L)]
                };
                let quit = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
                if quit {
                    ddr.on[who] = false;
                    toasts.push((format!("player {} scored {}", who + 1, ddr.score[who]), 2.5));
                    continue;
                }
                for lane in 0..4 {
                    if !keys[lane] {
                        continue;
                    }
                    ddr.flash[who][lane] = 1.0;
                    let best = ddr.arrows.iter().enumerate().filter(|(_, a)| a.lane == lane && !a.hit[who] && !a.missed[who] && (a.at - time).abs() < 0.18).min_by(|a, b| (a.1.at - time).abs().partial_cmp(&(b.1.at - time).abs()).unwrap()).map(|(n, a)| (n, (a.at - time).abs()));
                    if let Some((n, off)) = best {
                        ddr.arrows[n].hit[who] = true;
                        ddr.combo[who] += 1;
                        let (word, col, points) = if off < 0.05 {
                            ("perfect", Color::from_rgba(255, 226, 120, 255), 300)
                        } else if off < 0.1 {
                            ("great", Color::from_rgba(140, 230, 170, 255), 200)
                        } else {
                            ("good", Color::from_rgba(170, 210, 255, 255), 100)
                        };
                        ddr.score[who] += points + ddr.combo[who].min(50) * 2;
                        ddr.judge[who] = Some((word, col, 0.6));
                        quest_event(&mut quests, QuestKind::Dance, 1, &mut inv, &mut toasts);
                        players[who].voice_timer = 0.15;
                        audio.play_ch(who, &voices[players[who].character][lane + 1], 0.18);
                    }
                }
                for a in ddr.arrows.iter_mut() {
                    if !a.hit[who] && !a.missed[who] && time - a.at > 0.18 {
                        a.missed[who] = true;
                        ddr.combo[who] = 0;
                        ddr.judge[who] = Some(("miss", Color::from_rgba(255, 140, 140, 255), 0.6));
                    }
                }
            }
            ddr.arrows.retain(|a| time - a.at < 0.5);
        } else {
            ddr.arrows.clear();
        }
        for who in 0..2 {
            for lane in 0..4 {
                ddr.flash[who][lane] = (ddr.flash[who][lane] - dt * 5.0).max(0.0);
            }
            if let Some(j) = ddr.judge[who].as_mut() {
                j.2 -= dt;
            }
            if ddr.judge[who].map(|j| j.2 <= 0.0).unwrap_or(false) {
                ddr.judge[who] = None;
            }
        }
        {
            let mic_bands = if listening { microphone.as_ref().and_then(|m| m.detector.lock().ok().map(|d| d.bands)) } else { None };
            let beat_frac = (looper.position / 4.0).fract();
            let pulse = if looper.beat { (1.0 - beat_frac).powi(3) } else { 0.0 };
            let glow: f32 = players.iter().map(|p| p.key_glow.iter().copied().fold(0.0, f32::max)).fold(0.0, f32::max);
            let bands = mic_bands.unwrap_or([pulse, pulse * 0.4 + glow * 0.6, glow * 0.7, if looper.beat { ((looper.position * 0.5).fract() < 0.3) as u8 as f32 * 0.6 } else { glow * 0.4 }]);
            let mut pads = [0.0f32; 4];
            for lane in 0..4 {
                pads[lane] = ddr.flash[0][lane].max(ddr.flash[1][lane]);
            }
            VIS.with(|v| v.set(Vis { bands, pulse: if mic_bands.is_some() { bands[0] } else { pulse }, pads, seq: sampler.step }));
        }
        if fishing.active && fish_card <= 0.0 {
            let fish_window = 0.22 * fishing.tight / looper.step_seconds();
            let perfect_window = 0.09 * fishing.tight / looper.step_seconds();
            for who in 0..(if two_player { 2 } else { 1 }) {
                if !fish_press[who] {
                    continue;
                }
                let d = looper.position - fishing.cue_at;
                if d.abs() <= fish_window && !fishing.hit_by[who] && !fishing.early[who] {
                    fishing.hit_by[who] = true;
                    fishing.dip = 1.0;
                    audio.play_ch(3, &plink, 0.7);
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * 2.0 * PI;
                        spawn(&mut particles, BOBBER + vec3(0.0, 0.05, 0.0), vec3(a.cos() * 0.8, 1.6, a.sin() * 0.8), 0.6, Color::from_rgba(200, 230, 255, 255), 0.025, 0.5);
                    }
                    let both = fishing.hit_by[0] && fishing.hit_by[1];
                    let (word, col) = if both {
                        ("together!", Color::from_rgba(255, 150, 190, 255))
                    } else if d.abs() <= perfect_window {
                        fishing.perfects += 1;
                        ("perfect!", Color::from_rgba(255, 226, 120, 255))
                    } else {
                        ("nice!", Color::from_rgba(170, 230, 255, 255))
                    };
                    fish_say = Some((word.to_string(), col, 1.0));
                } else if d < -fish_window && d > -4.0 && !fishing.hit_by[who] {
                    fishing.early[who] = true;
                    fish_say = Some(("too early - wait for the ring".to_string(), Color::from_rgba(255, 190, 150, 255), 1.2));
                }
            }
            if looper.position > fishing.cue_at + fish_window {
                let hit = fishing.hit_by[0] || fishing.hit_by[1];
                let both = fishing.hit_by[0] && fishing.hit_by[1];
                let friend_on_dock = !two_player && players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8;
                let helped = !hit && !fishing.lesson && friend_on_dock && hash((time * 29.0) as i32, 4, 8) < 0.5;
                if !hit && fishing.lesson {
                    let key = if fishing.caster == 0 { "E" } else { "O" };
                    fish_say = Some((format!("almost! press {key} as the ring touches the bobber"), Color::from_rgba(255, 210, 160, 255), 1.6));
                } else if hit || helped {
                    fishing.hits += if both { 2 } else { 1 };
                    if helped {
                        fishing.dip = 1.0;
                        audio.play_ch(1, &voices[players[1].character][2], 0.5);
                        players[1].voice_timer = 0.4;
                        let line = format!("the {} tugged the line for you!", cast_list[players[1].character].noun);
                        fish_say = Some((line, Color::from_rgba(200, 240, 170, 255), 1.6));
                    }
                    if both {
                        for k in 0..3 {
                            hearts.push((vec3(players[k % 2].pos.x, 1.0 + k as f32 * 0.12, players[k % 2].pos.y), 1.4));
                        }
                    }
                } else {
                    fishing.tries = fishing.tries.saturating_sub(1);
                    audio.play_ch(3, &splash, 0.35);
                    if fishing.tries > 0 {
                        fish_say = Some((format!("missed - {} {} left", fishing.tries, if fishing.tries == 1 { "try" } else { "tries" }), Color::from_rgba(255, 190, 150, 255), 1.4));
                    }
                }
                if fishing.hits >= 3 {
                    let roll = hash((time * 100.0) as i32, 7, 7);
                    let roll2 = hash((time * 61.0) as i32, 3, 13);
                    let luck = (fishing.perfects as f32 / 3.0).min(1.0) + if fishing.hit_by[0] && fishing.hit_by[1] { 0.3 } else { 0.0 };
                    let species = if fishing.lesson { fish::pick(roll * 0.5, roll2, 0.0) } else { fish::pick(roll, roll2, luck) };
                    let kind = fish::kind(species);
                    let is_new = dex_counts[species] == 0;
                    dex_counts[species] += 1;
                    fish_pop = Some((species, 3.0, is_new, false));
                    inv.fish[kind] += 1;
                    quest_event(&mut quests, QuestKind::Fish, 1, &mut inv, &mut toasts);
                    inv.acorns += fish::acorns(species);
                    fishing.active = false;
                    fishing.jump = Some((kind, 0.0));
                    audio.play_ch(3, &splash, 0.6);
                    let singer = if fishing.hit_by[1] { 1 } else { fishing.caster };
                    audio.play_ch(singer, &voices[players[singer].character][5], 0.7);
                    players[singer].voice_timer = 0.5;
                    let who = if fishing.hit_by[0] && fishing.hit_by[1] { "you both caught" } else if fishing.hit_by[1] || (fishing.caster == 1 && !fishing.hit_by[0]) { "player 2 caught" } else { "caught" };
                    toasts.push((format!("{who} a {}! +{} acorns", fish::SPECIES[species], fish::acorns(species)), 3.0));
                    if fishing.lesson {
                        fish_lesson = true;
                        save_clock = 99.0;
                        fish_say = Some(("your first fish!".to_string(), Color::from_rgba(255, 226, 120, 255), 2.0));
                    }
                    if friend_on_dock && hash((time * 37.0) as i32, 3, 11) < 0.55 {
                        let bonus = fish::pick(hash((time * 53.0) as i32, 5, 2) * 0.8, hash((time * 71.0) as i32, 2, 9), 0.0);
                        dex_counts[bonus] += 1;
                        inv.fish[fish::kind(bonus)] += 1;
                        inv.acorns += 2;
                        audio.play_ch(1, &voices[players[1].character][4], 0.6);
                        players[1].voice_timer = 0.5;
                        toasts.push((format!("the {} caught a {} too! +2 acorns", cast_list[players[1].character].noun, fish::SPECIES[bonus]), 3.0));
                    }
                } else if fishing.tries == 0 {
                    fishing.active = false;
                    audio.play_ch(3, &splash, 0.5);
                    toasts.push(("the fish got away - cast again!".to_string(), 2.5));
                } else {
                    let roll = hash((time * 47.0) as i32, 9, 3);
                    let gap = if fishing.lesson { FISH_CUE_GAP } else if roll < 0.25 { FISH_CUE_GAP - 2.0 } else if roll > 0.8 { FISH_CUE_GAP + 2.0 } else { FISH_CUE_GAP };
                    fishing.cue_at += gap;
                    fishing.hit_by = [false; 2];
                    fishing.early = [false; 2];
                    if !fishing.lesson {
                        fishing.tight = 0.72 + 0.2 * hash((time * 43.0) as i32, 1, 8);
                    }
                }
            }
        }
        fish_card = (fish_card - dt).max(0.0);
        cook_card = (cook_card - dt).max(0.0);
        gather_card = (gather_card - dt).max(0.0);
        if let Some(say) = fish_say.as_mut() {
            say.2 -= dt;
        }
        if fish_say.as_ref().map(|s| s.2 <= 0.0).unwrap_or(false) {
            fish_say = None;
        }
        fishing.dip = (fishing.dip - dt * 4.0).max(0.0);
        if let Some((kind, t)) = fishing.jump {
            fishing.jump = if t + dt > 1.0 { None } else { Some((kind, t + dt)) };
        }

        let was_jam = jam;
        jam = !inside && (players[0].seated || (two_player && players[1].seated));
        if jam && !was_jam {
            music_mode = true;
            if first_steps == 2 {
                first_steps = 3;
                toasts.push(("play together to earn your first star".to_string(), 3.0));
            }
        }

        if camp.lit {
            camp.fuel -= dt;
            camp.pile = (camp.fuel / STICK_FUEL).ceil().max(0.0) as u32;
            if camp.fuel <= 0.0 {
                camp.fuel = 0.0;
                camp.lit = false;
                cooking = false;
                toasts.push(("the fire went out".to_string(), 3.0));
                for p in players.iter_mut() {
                    p.seated = false;
                }
            }
            crackle_clock -= dt;
            if crackle_clock <= 0.0 {
                crackle_clock = 0.6 + hash((time * 10.0) as i32, 3, 9) * 1.2;
                let dist = players[0].pos.distance(FIRE_SPOT);
                let vol = (1.0 - dist / 7.0).max(0.0) * 0.35;
                if vol > 0.01 {
                    audio.play_ch(3, &crackle, vol);
                }
            }
            if hash((time * 60.0) as i32, 5, 1) > 0.7 {
                let c = fire_center();
                spawn(&mut particles, c + vec3(hash((time * 97.0) as i32, 1, 2) * 0.4 - 0.2, 0.4, 0.1), vec3(hash((time * 31.0) as i32, 2, 2) * 0.6 - 0.3, 1.6, 0.0), 0.9, Color::from_rgba(255, 180, 70, 255), 0.022, 0.05);
            }
        }
        if cooking && hash((time * 40.0) as i32, 9, 2) > 0.8 {
            let c = fire_center();
            spawn(&mut particles, c + vec3(hash((time * 53.0) as i32, 1, 3) * 0.3 - 0.15, 0.62, 0.06), vec3(0.0, 0.5, 0.0), 1.6, Color::from_rgba(240, 240, 245, 255), 0.05, -0.03);
        }
        if hash((time * 30.0) as i32, 8, 1) > 0.85 {
            let (_, z0, x1, _) = COTTAGE;
            spawn(&mut particles, vec3(x1 as f32 - 0.6, 2.7, z0 as f32 + 0.6), vec3(0.15, 0.5, 0.0), 2.6, Color::from_rgba(220, 220, 228, 255), 0.06, -0.02);
        }
        let real_night = update_sky().night * 0.95;
        {
            let sk = sky();
            if quests.day != sk.day {
                quests.day = sk.day;
                quests.list = daily_quests(sk.day, house.has(home::Item::Plant), house.has(home::Item::Arcade));
                let mut restored = false;
                if let Some((day, list)) = saved_quests.take() {
                    if day == sk.day {
                        for (q, (kind, target, progress, done)) in quests.list.iter_mut().zip(list.into_iter()) {
                            if q.kind as u8 == kind && q.target == target {
                                q.progress = progress;
                                q.done = done;
                                restored = true;
                            }
                        }
                    }
                }
                if !restored {
                    toasts.push(("3 new daily quests - see them in your bag (M)".to_string(), 3.5));
                }
            }
            for piece in house.pieces.iter_mut().filter(|p| p.item == home::Item::Plant) {
                if piece.day == 0 {
                    piece.day = sk.day as i32;
                } else if sk.day as i32 - piece.day >= 2 && piece.state != home::WITHERED {
                    piece.state = home::WITHERED;
                }
            }
            let mut batch = 1;
            if sk.day != shroom_day {
                shroom_day = sk.day;
                shroom_today = mushroom::today(sk.day);
                shrooms.retain(|(_, sp)| shroom_today.contains(sp));
                shroom_clock = 0.0;
                batch = 6;
            }
            shroom_clock -= dt;
            if shroom_clock <= 0.0 {
                shroom_clock = if sk.weather == 2 { 7.0 } else { 14.0 };
                for round in 0..batch {
                if shrooms.len() < 10 && !shroom_today.is_empty() {
                    let seed = (time * 97.0) as i32 + round * 131;
                    for attempt in 0..30 {
                        let tx = (hash(seed, attempt, 31) * MAP as f32) as i32;
                        let tz = (hash(seed, attempt, 32) * MAP as f32) as i32;
                        if world.get(tx, tz).map(|t| t.kind == Kind::Tree).unwrap_or(false) {
                            let dx = (hash(seed, attempt, 33) * 3.0) as i32 - 1;
                            let dz = (hash(seed, attempt, 34) * 3.0) as i32 - 1;
                            let (gx, gz) = (tx + dx, tz + dz);
                            let ok = world.get(gx, gz).map(|t| matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Tree)).unwrap_or(false);
                            let at = vec2(gx as f32 + 0.2 + hash(seed, attempt, 35) * 0.6, gz as f32 + 0.2 + hash(seed, attempt, 36) * 0.6);
                            let trunk_clear = (dx != 0 || dz != 0) || at.distance(vec2(tx as f32 + 0.5, tz as f32 + 0.5)) > 0.3;
                            if ok && trunk_clear && shrooms.iter().all(|(o, _)| o.distance(at) > 1.0) {
                                let species = mushroom::pick_from(&shroom_today, hash(seed, attempt, 37));
                                shrooms.push((at, species));
                                break;
                            }
                        }
                    }
                }
                }
            }
            SHROOMS.with(|c| *c.borrow_mut() = shrooms.clone());
        }
        let dusk_target = real_night.max(if jam { 0.45 } else if camp.lit { 0.12 } else { 0.0 });
        dusk += (dusk_target - dusk) * (dt * 0.4).min(1.0);
        if jam && !two_player {
            let c = players[0].cam;
            players[0].cam = c + (vec2(8.5, 17.0) - c) * (dt * 2.0).min(1.0);
        }

        let active = if two_player { 2 } else { 1 };
        let dog_jamming = jam && !two_player && players[1].seated;
        for who in 0..active {
            let keys = if who == 0 { NOTE_KEYS } else { P2_NOTE_KEYS };
            for (i, key) in keys.iter().enumerate() {
                if !is_key_pressed(*key) {
                    continue;
                }
                last_note_at = time;
                if who == 0 && first_steps == 1 && music_mode && !cooking && !(inside && dj[0]) {
                    first_steps = 2;
                    toasts.push(("your first note! find the campfire next".to_string(), 3.0));
                }
                if inside && dj[who] {
                    let id = 16 + who * 6 + i;
                    if looper.press(id) {
                        play_pad(who, i, 0.85, players[1].character);
                    }
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.2;
                    continue;
                }
                if inside && matches!(seat[who], Some(home::Item::Piano) | Some(home::Item::Drums)) {
                    if seat[who] == Some(home::Item::Piano) {
                        audio.play_ch(who, &piano[i], 0.8);
                    } else {
                        let drum = [&kick, &clap, &hat, &lid, &chop, &sizzle][i];
                        audio.play_ch(2, drum, 0.85);
                    }
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.3;
                    players[1 - who].voice_timer = players[1 - who].voice_timer.max(0.15);
                    continue;
                }
                if cooking && players[who].seated && i < 4 {
                    let id = if who == 0 { 12 + i } else { 28 + i };
                    let play_now = looper.press(id);
                    if play_now {
                        audio.play_ch(2, perc[i], 0.8);
                    }
                    let f = looper.position.fract();
                    if f < 0.3 || f > 0.7 {
                        cook_progress += 1;
                    }
                    players[who].key_glow[i] = 1.0;
                    let c = fire_center() + vec3(0.0, 0.6, 0.08);
                    let col = [Color::from_rgba(255, 170, 80, 255), Color::from_rgba(220, 220, 230, 255), Color::from_rgba(120, 200, 90, 255), Color::from_rgba(255, 220, 120, 255)][i];
                    for k in 0..6 {
                        let a = (k as f32 / 6.0 + time) * 2.0 * PI;
                        spawn(&mut particles, c, vec3(a.cos() * 0.7, 1.2, a.sin() * 0.3), 0.5, col, 0.025, 0.5);
                    }
                    if cook_progress >= COOK_GOAL {
                        cook_progress = 0;
                        let fish_n = inv.fish[0] + inv.fish[1] + inv.fish[2];
                        let mush_n = inv.mush[0] + inv.mush[1] + inv.mush[2];
                        let take = |stock: &mut [u32; 3], n: u32| {
                            for _ in 0..n {
                                if let Some(k) = (0..3).find(|k| stock[*k] > 0) {
                                    stock[k] -= 1;
                                }
                            }
                        };
                        let dish = if inv.mush[2] > 0 {
                            inv.mush[2] -= 1;
                            7
                        } else if inv.fish[2] > 0 {
                            inv.fish[2] -= 1;
                            3
                        } else if fish_n >= 1 && mush_n >= 1 {
                            take(&mut inv.fish, 1);
                            take(&mut inv.mush, 1);
                            6
                        } else if mush_n >= 2 {
                            take(&mut inv.mush, 2);
                            5
                        } else if fish_n >= 2 {
                            take(&mut inv.fish, 2);
                            2
                        } else if mush_n == 1 {
                            take(&mut inv.mush, 1);
                            4
                        } else if fish_n == 1 {
                            take(&mut inv.fish, 1);
                            1
                        } else {
                            0
                        };
                        let new = !dishes.contains(&dish);
                        if new {
                            dishes.push(dish);
                        }
                        let earned = if new { 10 } else { 5 };
                        inv.acorns += earned;
                        quest_event(&mut quests, QuestKind::Cook, 1, &mut inv, &mut toasts);
                        toasts.push((
                            if new { format!("new dish: {}! +{} acorns", DISHES[dish], earned) } else { format!("cooked {} again - yum, +{} acorns", DISHES[dish], earned) },
                            3.5,
                        ));
                        audio.play_ch(1, &voices[players[1].character][0], 0.6);
                        audio.play_ch(0, &voices[players[0].character][4], 0.6);
                        players[1].voice_timer = 0.5;
                        for k in 0..6 {
                            let who_pos = if k % 2 == 0 { players[0].pos } else { players[1].pos };
                            hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.2, 1.0 + k as f32 * 0.1, who_pos.y + 0.05), 1.6));
                        }
                    }
                    continue;
                }
                let id = who * 6 + i;
                let play_now = if music_mode { looper.press(id) } else { true };
                quest_event(&mut quests, QuestKind::Sing, 1, &mut inv, &mut toasts);
                if !music_mode {
                    let (pos, facing) = (players[who].pos, players[who].facing);
                    let (tx, tz) = players[who].ahead();
                    cast(i, &mut world, pos, facing, &mut particles);
                    if i == 0 && (tx, tz) == CAMPFIRE && camp.fuel > 0.0 && !camp.lit {
                        camp.lit = true;
                        toasts.push(("ember! the fire is lit".to_string(), 2.5));
                    }
                    if i == 2 && pos.distance(FIRE_SPOT) < 2.5 && camp.lit {
                        camp.lit = false;
                        cooking = false;
                        toasts.push(("the rain put the fire out".to_string(), 2.5));
                    }
                }
                if play_now {
                    let sound = &voices[players[who].character][i];
                    audio.play_ch(who, sound, 0.8);
                    players[who].key_glow[i] = 1.0;
                    players[who].voice_timer = 0.4;
                    if who == 0 && dog_jamming {
                        cat_phrase_notes += 1;
                        recent_cat.push(i);
                        if recent_cat.len() > 8 {
                            recent_cat.remove(0);
                        }
                        if !partner_soloing && world.rand() < 0.6 {
                            echoes.push((looper.position + 8.0, i));
                        }
                    }
                }
            }
        }

        for id in looper.tick(get_frame_time()) {
            if id >= 28 && id < 32 {
                audio.play_ch(2, perc[id - 28], 0.7);
                continue;
            }
            if id >= 16 {
                play_pad((id - 16) / 6, (id - 16) % 6, 0.7, players[1].character);
                continue;
            }
            if id >= 12 {
                audio.play_ch(2, perc[(id - 12).min(3)], 0.7);
                continue;
            }
            let (who, i) = (id / 6, id % 6);
            let sound = &voices[players[who].character][i];
            audio.play_ch(who, sound, 0.75);
            players[who].key_glow[i] = 1.0;
            players[who].voice_timer = 0.3;
        }
        let position = looper.position;
        let friend_voice = players[1].character;
        echoes.retain(|&(at, i)| {
            if position >= at {
                audio.play_ch(1, &voices[friend_voice][i], 0.45);
                players[1].voice_timer = 0.3;
                false
            } else {
                position > at - 64.0
            }
        });
        let step = looper.current_step();
        if looper.last_step != Some(step) {
            let crossed: Vec<usize> = match looper.last_step {
                Some(prev) => {
                    let mut v = Vec::new();
                    let mut k = (prev + 1) % LOOP_STEPS;
                    while k != step && v.len() < LOOP_STEPS {
                        v.push(k);
                        k = (k + 1) % LOOP_STEPS;
                    }
                    v.push(step);
                    v
                }
                None => vec![step],
            };
            looper.last_step = Some(step);
            sampler.step = step % 16;
            if inside && house.has(home::Item::Sampler) {
                for row in 0..4 {
                    if sampler.grid[row][step % 16] {
                        if let Some(pad) = &sampler.pads[row] {
                            audio.play_private(pad, 0.8);
                        } else {
                            match row {
                                0 => audio.play_ch(2, &kick, 0.8),
                                1 => audio.play_ch(2, &clap, 0.55),
                                2 => audio.play_ch(2, &hat, 0.45),
                                _ => audio.play_ch(2, &voices[players[sampler.owner].character][(step / 4) % 6], 0.45),
                            }
                        }
                    }
                }
            }
            if (ddr.on[0] || ddr.on[1]) && time - ddr.last_mic > 3.0 && step % 2 == 0 {
                let step_s = looper.step_seconds();
                let frac = (looper.position - step as f32).rem_euclid(LOOP_STEPS as f32);
                let target = time + (16.0 - frac) * step_s;
                let slot_len = step_s * 2.0;
                if looper.beat {
                    let style = &BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)];
                    if velocity(style.kick, step).is_some() {
                        ddr.add(0, target, slot_len);
                    }
                    if velocity(style.clap, step).is_some() {
                        ddr.add(1, target, slot_len);
                    }
                    if velocity(style.open, step).is_some() {
                        ddr.add(3, target, slot_len);
                    } else if velocity(style.hat, step).is_some() && step % 4 == 2 {
                        ddr.add(2, target, slot_len);
                    }
                }
                let notes: Vec<usize> = looper.notes.iter().filter(|(at, _)| (at.round() as usize) % LOOP_STEPS == (step + 16) % LOOP_STEPS).map(|(_, id)| *id).collect();
                for id in notes {
                    ddr.add(note_slot(id) % 4, target, slot_len);
                }
            }
            if inside && house.record_on && house.has(home::Item::RecordPlayer) && playback.is_none() {
                if crossed.contains(&0) || record_tune.is_empty() {
                    let tunes = if songbook.is_empty() { vec![vec![(0, 0), (4, 2), (8, 4), (12, 3), (16, 2), (20, 1), (24, 0)]] } else { songbook.clone() };
                    record_pick = (record_pick + 1) % tunes.len();
                    record_tune = tunes[record_pick].clone();
                }
                for &(at, note) in &record_tune {
                    if crossed.contains(&at) {
                        audio.play_ch(1, &voices[players[1].character][note], 0.3);
                    }
                }
            }
            if looper.beat {
                let style = &BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)];
                if let Some(v) = velocity(style.kick, step) {
                    let sub = if house.switched_on(home::Item::Subwoofer) { 1.4 } else { 1.0 };
                    audio.play_ch(2, &kick, (0.75 * v * sub).min(1.0));
                }
                if let Some(v) = velocity(style.clap, step) {
                    audio.play_ch(2, &clap, 0.5 * v);
                }
                if let Some(v) = velocity(style.hat, step) {
                    audio.play_ch(2, &hat, 0.45 * v);
                }
                if let Some(v) = velocity(style.open, step) {
                    audio.play_ch(2, &open_hat, 0.4 * v);
                }
            } else if (looper.recording || fishing.active) && step % 4 == 0 {
                let accent = if step % 16 == 0 { 0.5 } else { 0.25 };
                audio.play_ch(2, &hat, accent);
            }
            if dog_jamming && crossed.contains(&0) && cat_phrase_notes >= 3 {
                inv.acorns += 1;
                toasts.push(("nice jam: +1 acorn".to_string(), 1.6));
            }
            if dog_jamming && crossed.contains(&0) {
                let chance = if partner_solo_count >= 2 {
                    0.15
                } else if cat_phrase_notes >= 2 {
                    0.75
                } else if partner_soloing {
                    0.3
                } else {
                    0.45
                };
                partner_soloing = world.rand() < chance;
                partner_solo_count = if partner_soloing { partner_solo_count + 1 } else { 0 };
                partner_phrase.clear();
                if partner_soloing {
                    let partner_mood: &str;
                    let pick = |world: &mut World, n: usize| (world.rand() * n as f32) as usize % n.max(1);
                    let longest = (2 + partner_xp as usize / 2).min(8);
                    let roll = world.rand();
                    let fresh = |world: &mut World, recent: &[usize]| {
                        let mut phrase = Vec::new();
                        let mut at = if world.rand() < 0.5 { 0 } else { 2 };
                        let length = 1 + (world.rand() * longest as f32) as usize;
                        let mut note = recent.last().copied().unwrap_or((world.rand() * 6.0) as usize % 6) as i32;
                        for k in 0..length {
                            if at >= LOOP_STEPS - 1 {
                                break;
                            }
                            let borrow = !recent.is_empty() && world.rand() < 0.35;
                            note = if borrow {
                                recent[k % recent.len()] as i32
                            } else {
                                let moves = [-2, -1, -1, 0, 1, 1, 2];
                                (note + moves[(world.rand() * 7.0) as usize % 7]).clamp(0, 5)
                            };
                            phrase.push((at, note as usize));
                            let gaps = [2, 2, 3, 4, 4, 6, 1];
                            at += gaps[(world.rand() * 7.0) as usize % 7];
                        }
                        phrase
                    };
                    if songbook.is_empty() || (roll < 0.3 && songbook.len() < 8) {
                        partner_phrase = fresh(&mut world, &recent_cat);
                        songbook.push(partner_phrase.clone());
                        partner_mood = if songbook.len() == 1 { "a first little tune" } else { "a new tune" };
                    } else if roll < 0.7 {
                        let k = pick(&mut world, songbook.len());
                        let mut tune = songbook[k].clone();
                        let change = world.rand();
                        if change < 0.4 && tune.len() < longest {
                            let last = tune.last().copied().unwrap_or((0, 2));
                            let gaps = [2, 3, 4];
                            let at = last.0 + gaps[pick(&mut world, 3)];
                            if at < LOOP_STEPS - 1 {
                                let moves = [-1, 1, 2, -2];
                                tune.push((at, (last.1 as i32 + moves[pick(&mut world, 4)]).clamp(0, 5) as usize));
                            }
                        } else if change < 0.75 && !tune.is_empty() {
                            let j = pick(&mut world, tune.len());
                            let moves = [-1, 1];
                            tune[j].1 = (tune[j].1 as i32 + moves[pick(&mut world, 2)]).clamp(0, 5) as usize;
                        } else {
                            let shift = if world.rand() < 0.5 { -1 } else { 1 };
                            for n in tune.iter_mut() {
                                n.1 = (n.1 as i32 + shift).clamp(0, 5) as usize;
                            }
                        }
                        songbook[k] = tune.clone();
                        partner_phrase = tune;
                        partner_mood = "growing a tune";
                    } else {
                        let k = pick(&mut world, songbook.len());
                        partner_phrase = songbook[k].clone();
                        partner_mood = "its favourite tune";
                    }
                    partner_xp += 1;
                    toasts.push((format!("the {} plays {}", cast_list[players[1].character].noun, partner_mood), 2.0));
                }
                cat_phrase_notes = 0;
            }
            if !dog_jamming {
                partner_soloing = false;
                partner_phrase.clear();
            }
            if dog_jamming && partner_soloing {
                for &(at, note) in &partner_phrase {
                    if crossed.contains(&at) {
                        audio.play_ch(1, &voices[players[1].character][note], 0.62);
                        players[1].voice_timer = 0.3;
                        players[1].key_glow[note] = 1.0;
                    }
                }
                if step % 4 == 2 && !looper.beat && !cooking {
                    audio.play_ch(2, &hat, 0.16);
                }
            } else if dog_jamming {
                let chord = [0, 4, 3, 1][(step / 8) % 4];
                if step % 8 == 0 {
                    audio.play_ch(1, &voices[players[1].character][chord], 0.42);
                    players[1].voice_timer = 0.25;
                } else if step % 16 == 11 {
                    audio.play_ch(1, &voices[players[1].character][chord], 0.26);
                }
                if step % 4 == 2 && !looper.beat && !cooking {
                    audio.play_ch(2, &hat, 0.16);
                }
            }
        }

        let together = !inside && if two_player { players[0].seated && players[1].seated } else { dog_jamming };
        let playing = together && time - last_note_at < 8.0;
        match session {
            None => {
                if playing {
                    session = Some(0.0);
                    session_stars = 0;
                    audio.start_jam();
                }
            }
            Some(t) => {
                if playing {
                    let t = t + dt;
                    session = Some(t);
                    let stars = stars_for(t);
                    if stars > session_stars {
                        session_stars = stars;
                        quest_event(&mut quests, QuestKind::Star, 1, &mut inv, &mut toasts);
                        let label = ["", "30 seconds together - 1 star!", "1 minute together - 2 stars!", "3 minutes together - 3 stars!!"][stars as usize];
                        toasts.push((label.to_string(), 3.0));
                        for k in 0..3 {
                            audio.play_ch(3, &bells[(k * 2 + stars as usize) % 6], 0.35);
                        }
                        for k in 0..(4 * stars as usize) {
                            let who_pos = if k % 2 == 0 { players[0].pos } else { players[1].pos };
                            hearts.push((vec3(who_pos.x + (k as f32 * 0.37).sin() * 0.25, 1.0 + k as f32 * 0.08, who_pos.y + 0.05), 1.8));
                        }
                    }
                } else {
                    let events = audio.stop_jam();
                    if t >= 30.0 {
                        diary.push(DiaryEntry {
                            when: miniquad::date::now(),
                            seconds: t as u32,
                            stars: stars_for(t),
                            who: (players[0].character, players[1].character),
                            events,
                        });
                        toasts.push((format!("the jam was written in the diary - {} star{}", stars_for(t), if stars_for(t) == 1 { "" } else { "s" }), 3.5));
                    }
                    session = None;
                    session_stars = 0;
                }
            }
        }
        if first_steps == 3 && session_stars > 0 {
            first_steps = 4;
            toasts.push(("your first jam! explore the house and make more music".to_string(), 4.0));
        }
        if let Some((idx, start, next)) = playback {
            if !inside || idx >= diary.len() {
                playback = None;
            } else {
                let events = &diary[idx].events;
                let t = (get_time() - start) as f32;
                let mut k = next;
                while k < events.len() && events[k].0 <= t {
                    audio.play_id(events[k].1, events[k].2 * 0.85);
                    k += 1;
                }
                if k >= events.len() && t > events.last().map_or(0.0, |e| e.0) + 2.0 {
                    playback = None;
                    toasts.push(("the record ends".to_string(), 2.0));
                } else {
                    playback = Some((idx, start, k));
                }
            }
        }
        stick_trees.retain(|&(x, z)| world.get(x, z).map(|t| t.kind == Kind::Tree && t.fire <= 0.0).unwrap_or(false));
        stick_clock -= dt;
        if stick_clock <= 0.0 {
            stick_clock = if stick_trees.len() < 4 { 0.5 } else { 12.0 + world.rand() * 8.0 };
            if stick_trees.len() < 6 {
                grow_stick(&mut world, &mut stick_trees);
            }
        }
        tick_clock += dt;
        while tick_clock >= TICK {
            tick_clock -= TICK;
            world.tick(&mut particles);
        }
        world.wind_time = (world.wind_time - dt).max(0.0);

        for z in 0..MAP as i32 {
            for x in 0..MAP as i32 {
                let t = world.get(x, z).unwrap();
                if t.fire > 0.0 && hash(x, z, (time * 20.0) as u32) > 0.8 {
                    let drift = if world.wind_time > 0.0 { world.wind * 1.2 } else { Vec2::ZERO };
                    spawn(
                        &mut particles,
                        vec3(x as f32 + 0.3 + hash(z, x, (time * 30.0) as u32) * 0.4, 0.9, z as f32 + 0.5),
                        vec3(drift.x, 1.4, drift.y),
                        0.8,
                        Color::from_rgba(255, 190, 80, 255),
                        0.035,
                        0.0,
                    );
                }
            }
        }
        for p in particles.iter_mut() {
            p.vel.y -= p.gravity * dt * 9.8;
            p.pos += p.vel * dt;
            p.life -= dt;
        }
        particles.retain(|p| p.life > 0.0);
        for h in hearts.iter_mut() {
            h.0.y += dt * 0.5;
            h.1 -= dt;
        }
        hearts.retain(|h| h.1 > 0.0);
        for t in toasts.iter_mut() {
            t.1 -= dt;
        }
        toasts.retain(|t| t.1 > 0.0);
        if toasts.len() > 3 {
            toasts.remove(0);
        }

        let cat = &players[0];
        let ahead = cat.ahead();
        let can_build_here = !inside && !cat.seated && !fishing.active && !(camp.lit && near_fire(cat)) && !at_dock_end(cat) && buildable(&world, ahead.0, ahead.1);
        let ring = if fish_card > 0.0 { 1.0 } else { ((fishing.cue_at - looper.position) / FISH_CUE_GAP).clamp(0.0, 1.0) };
        let extras = Extras {
            fishing: fishing.active,
            ring,
            in_window: fish_card <= 0.0 && (looper.position - fishing.cue_at).abs() <= 0.22 * fishing.tight / looper.step_seconds(),
            dip: fishing.dip,
            jump: fishing.jump,
            cat: players[fishing.caster].pos,
            pot: cooking,
            hearts: hearts.clone(),
            ghost: if can_build_here { Some((ahead.0, ahead.1, tools[0])) } else { None },
            sticks: stick_trees.iter().filter_map(|t| stick_spot(&world, *t)).collect(),
            friend_rod: if fishing.active && !two_player && players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8 { Some(players[1].pos) } else { None },
        };

        let dance = if looper.beat || dog_jamming { looper.beat_pulse() } else { 0.0 };
        if low_w() as u32 != view_w || low_h() as u32 != view_h {
            view_w = low_w() as u32;
            view_h = low_h() as u32;
            full_view = make_view(view_w, view_h);
        }
        let half = (low_w() / 2.0).floor();
        set_camera(&Camera2D {
            render_target: Some(full_view.clone()),
            zoom: vec2(2.0 / view_w as f32, -2.0 / view_h as f32),
            target: vec2(view_w as f32 / 2.0, view_h as f32 / 2.0),
            ..Default::default()
        });
        clear_background(FOG);
        let view = View {
            world: &world,
            players: &players,
            particles: &particles,
            camp: &camp,
            time,
            dance,
            dusk,
            extras: &extras,
        };
        let ghost = holding.map(|p| (p, ahead_local(&players[holding_by]) != home::CATALOGUE && house.can_place(&p, &[home::local(players[1 - holding_by].pos)])));
        if inside {
            let pov_for = |who: usize| if dj[who] { booth_pov(&house, if who == 0 { -1.0 } else { 1.0 }) } else { None };
            let beat_on = looper.beat || !now_playing.is_empty();
            let pov = if dj[0] ^ dj[1] { if dj[0] { pov_for(0) } else { pov_for(1) } } else { None };
            let hide = if dj[0] ^ dj[1] { if dj[0] { Some(0) } else { Some(1) } } else { None };
            render_home(&full_view, 0.0, low_w(), low_h(), &view, &house, ghost, pov, hide, beat_on);
        } else if two_player {
            render_view(&full_view, 0.0, half, low_h(), players[0].pos, players[0].cam, &view, true);
            render_view(&full_view, half, half, low_h(), players[1].pos, players[1].cam, &view, true);
        } else {
            render_view(&full_view, 0.0, low_w(), low_h(), players[0].pos, players[0].cam, &view, false);
        }
        if picking {
            render_pick_preview(&pick_view, &world, &cast_list, [players[0].character, players[1].character], pick_row, time);
        }
        if shop_open && shop_tab == 0 {
            render_shop_preview(
                &catalogue_view,
                0.0,
                0.0,
                catalogue_view.texture.width(),
                catalogue_view.texture.height(),
                home::row(shop_row),
                &house,
                time,
                looper.beat || !now_playing.is_empty(),
            );
        }

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &full_view.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        if two_player && !inside {
            ui_rect(half - 0.5, 0.0, 1.0, low_h(), Color::new(0.1, 0.08, 0.2, 0.9));
        }

        let ink = Color::from_rgba(40, 30, 60, 255);
        let touch_ui = radio::touch_ui() && !two_player;
        let portrait = low_h() > low_w();
        let edge = if touch_ui && !portrait { (low_w() * 0.235).round() } else { 0.0 };
        let right_edge = low_w() - edge;
        let bar_y = if touch_ui && portrait { 58.0 } else if touch_ui { low_h() - 2.0 } else { low_h() - 32.0 };
        let grid_x;
        let grid_w;
        if touch_ui {
            grid_w = 296f32.min(low_w() - 24.0);
            grid_x = (low_w() - grid_w) / 2.0;
        } else if two_player {
            for who in 0..2 {
                let box_w = 25.0;
                let start_x = who as f32 * half + (half - box_w * 6.0) / 2.0;
                let labels = if who == 0 { P1_KEY_LABELS } else { P2_KEY_LABELS };
                for i in 0..6 {
                    let x = start_x + i as f32 * box_w;
                    let glow = players[who].key_glow[i];
                    let mut bg = note_color(i);
                    bg.a = 0.55 + glow * 0.45;
                    let lift = glow * 3.0;
                    ui_rect(x + 1.0, bar_y - lift, box_w - 2.0, 28.0, bg);
                    ui_text(labels[i], x + 4.0, bar_y + 12.0 - lift, 12.0, ink);
                    let name = if inside && dj[who] { &DJ_PADS[who][i][..DJ_PADS[who][i].len().min(4)] } else if cooking && players[who].seated && i < 4 { &PERC_NAMES[i][..3] } else if inside && seat[who] == Some(home::Item::Piano) { "piano" } else if inside && seat[who] == Some(home::Item::Drums) { "drum" } else { NOTE_NAMES[i] };
                    ui_text(name, x + 4.0, bar_y + 24.0 - lift, 12.0, ink);
                }
            }
            grid_x = 12.0;
            grid_w = low_w() - 24.0;
        } else {
            let box_w = 50.0;
            let start_x = low_w() / 2.0 - box_w * 3.0;
            for i in 0..6 {
                let x = start_x + i as f32 * box_w;
                let glow = players[0].key_glow[i];
                let mut bg = note_color(i);
                bg.a = 0.55 + glow * 0.45;
                let lift = glow * 3.0;
                ui_rect(x + 2.0, bar_y - lift, box_w - 4.0, 28.0, bg);
                ui_text(&format!("{} {}", i + 1, NOTE_NAMES[i]), x + 6.0, bar_y + 12.0 - lift, 12.0, ink);
                let label = if inside && dj[0] {
                    DJ_PADS[0][i]
                } else if cooking && i < 4 {
                    PERC_NAMES[i]
                } else if music_mode {
                    voice_word(cast_list[players[0].character].voice)
                } else {
                    SPELL_NAMES[i]
                };
                ui_text(label, x + 6.0, bar_y + 24.0 - lift, 12.0, ink);
            }
            grid_x = start_x + 2.0;
            grid_w = box_w * 6.0 - 4.0;
        }

        let show_grid = music_mode || looper.recording || !looper.notes.is_empty();
        let lanes: Vec<(Option<usize>, f32, f32)> = if two_player && !touch_ui {
            (0..2).map(|who| (Some(who), who as f32 * half + (half - 150.0) / 2.0 + 1.0, 148.0)).collect()
        } else {
            vec![(None, grid_x, grid_w)]
        };
        let grid_y = bar_y - 18.0;
        if show_grid {
            for &(lane, lx, lw) in &lanes {
                let cell = lw / LOOP_STEPS as f32;
                ui_rect(lx, grid_y, lw, 14.0, Color::new(0.1, 0.08, 0.2, 0.5));
                for k in (0..LOOP_STEPS).step_by(4) {
                    let a = if k % 16 == 0 { 0.6 } else { 0.25 };
                    ui_rect(lx + k as f32 * cell, grid_y, 0.34, 14.0, Color::new(1.0, 1.0, 1.0, a));
                }
                ui_rect(lx + step as f32 * cell, grid_y, cell, 14.0, Color::new(1.0, 1.0, 1.0, 0.3));
                for &(at, id) in &looper.notes {
                    if lane.map(|who| note_owner(id) != who).unwrap_or(false) {
                        continue;
                    }
                    let slot = note_slot(id).min(5);
                    let nx = lx + at * cell + 0.3;
                    let ny = grid_y + 1.0 + (5 - slot) as f32 * 2.0;
                    let mut color = note_color(slot);
                    if lane.is_none() && note_owner(id) == 1 {
                        color = Color::new(color.r * 0.75, color.g * 0.75, color.b * 0.75, 1.0);
                    }
                    ui_rect(nx, ny, (cell - 0.6).max(1.5), 1.7, color);
                }
            }
        }

        let status_y = if show_grid { grid_y - 4.0 } else { bar_y - 4.0 };
        let mut status_x = grid_x;
        if looper.recording {
            if (time * 2.0).fract() < 0.6 {
                ui_rect(status_x, status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
            }
            ui_text("REC", status_x + 9.0, status_y, 12.0, WHITE);
            status_x += 34.0;
        }
        if listening {
            let dot = if locked {
                Color::from_rgba(90, 230, 120, 255)
            } else {
                let blink = 0.35 + 0.65 * (time * 4.0).sin().abs();
                Color::new(1.0, 0.75, 0.25, blink)
            };
            ui_rect(status_x + 1.0, status_y - 7.0, 6.0, 6.0, dot);
            let text = if locked { format!("synced {:.0} bpm", looper.bpm) } else { "syncing...".to_string() };
            ui_text(&text, status_x + 12.0, status_y, 12.0, if locked { Color::from_rgba(150, 255, 170, 255) } else { WHITE });
        } else if looper.beat {
            ui_text(&format!("beat {:.0} bpm", looper.bpm), status_x, status_y, 12.0, WHITE);
        }
        if let Some(msg) = mic_message {
            mic_message_clock += get_frame_time();
            if mic_message_clock > 3.0 {
                mic_message = None;
            } else {
                ui_rect(grid_x - 1.0, status_y - 21.0, 7.0, 7.0, Color::from_rgba(255, 90, 90, 255));
                ui_text(msg, grid_x + 9.0, status_y - 14.0, 12.0, Color::from_rgba(255, 210, 160, 255));
            }
        } else {
            mic_message_clock = 0.0;
        }
        if let Some(t) = session {
            let label = format!("together {}:{:02}", t as u32 / 60, t as u32 % 60);
            let tw = ui_measure(&label, 11.0) + 36.0;
            let sx = grid_x + (grid_w - tw) / 2.0;
            ui_text(&label, sx, status_y, 11.0, Color::from_rgba(255, 236, 200, 255));
            for k in 0..3u8 {
                let col = if k < session_stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.22) };
                ui_star(sx + ui_measure(&label, 11.0) + 4.0 + k as f32 * 11.0, status_y - 8.0, 8.0, col);
            }
        }
        let mut right_x = grid_x + grid_w;
        if ringing {
            let glow = 0.75 + 0.25 * (time * 4.0).sin();
            right_x -= ui_measure("~echo~", 12.0);
            ui_text("~echo~", right_x, status_y, 12.0, Color::new(0.75, 0.95, 1.0, glow));
            right_x -= 10.0;
        }
        if let Some(t) = audio.tape_seconds() {
            let label = format!("tape {}:{:02}", t as u32 / 60, t as u32 % 60);
            right_x -= ui_measure(&label, 12.0);
            ui_text(&label, right_x, status_y, 12.0, Color::from_rgba(255, 216, 216, 255));
            if (time * 2.0).fract() < 0.6 {
                ui_rect(right_x - 9.0, status_y - 7.0, 6.0, 6.0, Color::from_rgba(255, 70, 80, 255));
            }
        }

        ui_rect(0.0, 0.0, low_w(), 17.0, Color::new(0.1, 0.08, 0.2, 0.4));
        let mode = if cooking { "cooking".to_string() } else if music_mode { format!("music  q:{}", looper.quantise.label()) } else { "spells".to_string() };
        if two_player {
            ui_text(cast_list[players[0].character].name, 6.0, 12.0, 12.0, Color::from_rgba(255, 226, 190, 255));
            ui_text(cast_list[players[1].character].name, half + 6.0, 12.0, 12.0, Color::from_rgba(240, 206, 140, 255));
            ui_text(&mode, 76.0, 12.0, 12.0, WHITE);
        } else {
            ui_text(&mode, edge + 6.0, 12.0, 12.0, WHITE);
            if !now_playing.is_empty() && right_edge - 144.0 - edge - ui_measure(&mode, 12.0) > 40.0 {
                let left = edge + 6.0 + ui_measure(&mode, 12.0) + 12.0;
                let room = right_edge - 118.0 - 8.0 - left;
                let line = ui_ticker(&format!("~ {now_playing}"), 12.0, room, time);
                ui_text(&line, left, 12.0, 12.0, Color::from_rgba(255, 214, 110, 255));
            }
        }
        let (meter_x, meter_y) = if two_player { (6.0, 27.0) } else { (right_edge - 118.0, 12.0) };
        ui_text("fire", meter_x, meter_y, 12.0, Color::from_rgba(255, 200, 140, 255));
        ui_rect(meter_x + 24.0, meter_y - 7.0, 40.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.35));
        let fill = (camp.fuel / 180.0).min(1.0);
        let fire_col = if camp.lit { Color::from_rgba(255, 150, 60, 255) } else { Color::from_rgba(120, 110, 110, 255) };
        ui_rect(meter_x + 24.0, meter_y - 7.0, 40.0 * fill, 6.0, fire_col);
        {
            let hx = right_edge - 46.0;
            ui_rect(hx - 2.0, 2.0, 12.0, 13.0, Color::from_rgba(255, 214, 110, 255));
            ui_text("H", hx + 1.0, 12.0, 12.0, Color::from_rgba(36, 26, 71, 255));
            ui_text("help", hx + 14.0, 12.0, 12.0, WHITE);
        }
        {
            let sk = sky();
            let temp = sk.temp.map(|t| format!(" {:.0}C", t)).unwrap_or_default();
            let line = format!("{:02}:{:02}  {}  {}{}", sk.hour as u32, ((sk.hour.fract()) * 60.0) as u32, SEASONS[sk.season], WEATHERS[sk.weather], temp);
            let wx = if two_player { half - ui_measure(&line, 9.0) - 6.0 } else { right_edge - ui_measure(&line, 9.0) - 4.0 };
            ui_text(&line, wx, 38.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        if ring_flash > 0.0 {
            let msg = if ringing { "echo on  (G)" } else { "echo off  (G)" };
            let a = ring_flash.min(1.0);
            let w = ui_measure(msg, 12.0) + 12.0;
            ui_rect((low_w() - w) / 2.0, 26.0, w, 16.0, Color::new(0.1, 0.08, 0.2, 0.7 * a));
            ui_text(msg, (low_w() - w) / 2.0 + 6.0, 38.0, 12.0, Color::new(0.8, 0.95, 1.0, a));
        }

        let fish_total: u32 = inv.fish.iter().sum();
        let items = [
            (Color::from_rgba(176, 124, 76, 255), format!("{}", inv.wood)),
            (Color::from_rgba(168, 170, 184, 255), format!("{}", inv.stone)),
            (Color::from_rgba(140, 190, 210, 255), format!("{}", fish_total)),
            (Color::from_rgba(255, 140, 160, 255), format!("{}/{}", dishes.len(), DISHES.len())),
            (ACORN_COLOR, format!("{}", inv.acorns)),
        ];
        let mut ix = right_edge - 6.0 - items.len() as f32 * 24.0;
        for (col, text) in items.iter() {
            ui_rect(ix, 21.0, 6.0, 6.0, *col);
            ui_text(text, ix + 8.0, 27.0, 11.0, WHITE);
            ix += 24.0;
        }

        let inside_hint = if inside {
            let ahead = ahead_local(&players[0]);
            let here = home::local(players[0].pos);
            Some(if shop_open || dj[0] {
                String::new()
            } else if let Some(piece) = holding {
                if ahead == home::CATALOGUE {
                    format!("E: sell the {} for {} acorns", piece.item.name(), piece.item.price() / 2)
                } else {
                    format!("E: place the {} - X: turn it - M: put in bag", piece.item.name())
                }
            } else if let Some(item) = seat[0] {
                match item {
                    home::Item::Piano | home::Item::Drums => format!("1-6: play the {} - move to get up", item.name()),
                    home::Item::Bed => "zzz - move to get up".to_string(),
                    _ => format!("sitting on the {} - move to get up", item.name()),
                }
            } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
                format!("E: catalogue & diary ({} acorns)", inv.acorns)
            } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                let item = house.pieces[i].item;
                match item {
                    home::Item::Piano | home::Item::Drums => format!("E: play the {} - X: move it", item.name()),
                    home::Item::RecordPlayer => format!("E: record {} - X: move it", if house.record_on { "off" } else { "on" }),
                    home::Item::DjBooth => "E: get on the decks - X: move it".to_string(),
                    home::Item::WindChime => "E: ring the chime - X: move it".to_string(),
                    home::Item::Lamp | home::Item::RedLight => format!("E: light {} - X: move it", if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Speaker | home::Item::Subwoofer => format!("E: {} {} - X: move it", item.name(), if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Bed => "E: sleep (saves the game) - X: move it".to_string(),
                    home::Item::Sofa | home::Item::ConcreteBench => format!("E: sit on the {} - X: move it", item.name()),
                    home::Item::Plant => {
                        let piece = &house.pieces[i];
                        if piece.state == home::WITHERED {
                            "withered - E: water it to bring it back".to_string()
                        } else if piece.day == sky().day as i32 {
                            "watered today - X: move it".to_string()
                        } else {
                            "thirsty - E: water the plant - X: move it".to_string()
                        }
                    }
                    home::Item::Bookshelf => "E: read a book - X: move it".to_string(),
                    home::Item::Table => "E: serve a dish - X: move it".to_string(),
                    home::Item::SteelPillar => "E: knock on it - X: move it".to_string(),
                    home::Item::MirrorBall => format!("E: mirror ball {} - X: move it", if house.pieces[i].state == 0 { "off" } else { "on" }),
                    home::Item::Tv => format!("E: next channel ({}) - X: move it", if house.pieces[i].state == 0 { "off" } else { TV_CHANNELS[house.pieces[i].state as usize - 1] }),
                    home::Item::Arcade => "E: dance! - X: move it".to_string(),
                    home::Item::Sampler => "E: open the sampler - X: move it".to_string(),
                    _ => format!("X: move the {}", item.name()),
                }
            } else if here.y > house.d() as f32 - 1.1 && (here.x - house.door().x).abs() < 0.9 {
                "walk out the door to go outside".to_string()
            } else {
                String::new()
            })
        } else {
            None
        };
        let carried_name = |c: Option<Carry>| match c {
            Some(Carry::Stick) => "stick",
            Some(Carry::Stone) => "stone",
            Some(Carry::Acorn) => "acorn",
            None => "",
        };
        let hint: String = if let Some(h) = inside_hint {
            h
        } else if players[0].carrying.is_some() && !players[0].seated {
            format!("bring the {} to the logs by the fire", carried_name(players[0].carrying))
        } else if two_player && players[1].carrying.is_some() {
            format!("player 2: bring the {} to the logs by the fire", carried_name(players[1].carrying))
        } else if fishing.active {
            let coop = two_player && players[0].pos.distance(DOCK_END) < 2.2 && players[1].pos.distance(DOCK_END) < 2.2;
            let who = if coop { "E and O" } else if fishing.caster == 1 { "O" } else { "E" };
            if fish_card > 0.0 {
                String::new()
            } else if fishing.lesson {
                format!("{} when the ring touches the bobber - catch {}/3", who, fishing.hits.min(3))
            } else {
                format!("{} when the ring closes - catch {}/3 - tries {}", who, fishing.hits.min(3), "o".repeat(fishing.tries as usize))
            }
        } else if cooking {
            format!("cook on the beat {}/{} - {}: done", cook_progress, COOK_GOAL, if two_player { "E / O" } else { "E" })
        } else if jam {
            if dog_jamming && partner_soloing {
                format!("the {} knows {} tune{} - answer back!", cast_list[players[1].character].noun, songbook.len(), if songbook.len() == 1 { "" } else { "s" })
            } else if dog_jamming || (two_player && players[1].seated) {
                "playing together by the fire - E / O: cook, move to stand up".to_string()
            } else {
                format!("playing by the fire - E: cook, the {} is on the way", cast_list[players[1].character].noun)
            }
        } else if players[0].pos.distance(DOOR_OUT) < 0.9 && !players[0].seated {
            if two_player { "E / O: go inside".to_string() } else { "E: go inside".to_string() }
        } else if camp.lit && near_fire(&players[0]) && !players[0].seated {
            "E: sit by the fire".to_string()
        } else if at_dock_end(&players[0]) {
            if two_player { "E / O: cast your fishing line".to_string() } else { "E: cast your fishing line".to_string() }
        } else if can_build_here {
            format!("E: build {}  X: change", tools[0].name())
        } else if !camp.lit && camp.fuel <= 0.0 {
            if two_player { "find a tree with a stick at its foot - E or O picks it up".to_string() } else { format!("the {} is looking for sticks - or grab one yourself with E", cast_list[players[1].character].noun) }
        } else {
            String::new()
        };
        if two_player && inside && !picking && !shop_open {
            let ahead = ahead_local(&players[1]);
            let here = home::local(players[1].pos);
            let mut p2_hint = if dj[1] {
                "O: leave DJ booth".to_string()
            } else if let Some(piece) = holding {
                if holding_by == 1 {
                    format!("O: place {} - U: turn - Bksp: bag", piece.item.name())
                } else {
                    String::new()
                }
            } else if let Some(item) = seat[1] {
                if item == home::Item::Bed { "O: get out of bed".to_string() } else { format!("O: get up from {}", item.name()) }
            } else if here.distance(vec2(0.5, 0.5)) < 1.4 {
                "O: catalogue and diary".to_string()
            } else if let Some(i) = house.piece_at(ahead.0, ahead.1) {
                format!("O: use {} - U: move", house.pieces[i].item.name())
            } else {
                String::new()
            };
            while ui_measure(&p2_hint, 11.0) > half - 12.0 && p2_hint.len() > 7 {
                p2_hint.pop();
            }
            if !p2_hint.is_empty() {
                let w = ui_measure(&p2_hint, 11.0);
                let x = half + (half - w) * 0.5;
                let y = if show_grid { grid_y - 31.0 } else { bar_y - 26.0 };
                ui_rect(x - 4.0, y - 9.0, w + 8.0, 13.0, Color::new(0.1, 0.08, 0.2, 0.68));
                ui_text(&p2_hint, x, y, 11.0, Color::from_rgba(255, 236, 200, 255));
            }
        }
        if !hint.is_empty() && !picking {
            let w = ui_measure(&hint, 12.0);
            let hy = if touch_ui && portrait { (low_h() * 0.56).round() } else if show_grid { grid_y - 16.0 } else { bar_y - 10.0 };
            ui_rect((low_w() - w) / 2.0 - 4.0, hy - 9.0, w + 8.0, 12.0, Color::new(0.1, 0.08, 0.2, 0.5));
            ui_text(&hint, (low_w() - w) / 2.0, hy, 12.0, Color::from_rgba(255, 236, 200, 255));
        }
        for (k, (text, life)) in toasts.iter().enumerate() {
            let a = life.min(1.0);
            let toast_y = if touch_ui && portrait { 70.0 } else { 40.0 };
            ui_text(text, edge + 8.0, toast_y + k as f32 * 11.0, 12.0, Color::new(1.0, 0.95, 0.85, a));
        }
        if show_inventory && !picking && !shop_open && !show_help {
            let lines = [
                format!("acorns  {}", inv.acorns),
                format!("wood    {}   stone  {}", inv.wood, inv.stone),
                format!("fish    {}   mushrooms  {}", inv.fish[0] + inv.fish[1] + inv.fish[2], inv.mush[0] + inv.mush[1] + inv.mush[2]),
                format!("fishdex {}/{}   shrooms {}/{}", dex_counts.iter().filter(|n| **n > 0).count(), fish::COUNT, shroom_counts.iter().filter(|n| **n > 0).count(), mushroom::COUNT),
                format!("dishes  {}/{}   tunes  {}", dishes.len(), DISHES.len(), songbook.len()),
            ];
            let w = 210f32.min(low_w() - 16.0);
            let shown_items = 5usize;
            let furn_rows = if stored.is_empty() { 1 } else { stored.len().min(shown_items) };
            let h = 30.0 + lines.len() as f32 * 11.0 + 16.0 + quests.list.len() as f32 * 11.0 + 16.0 + furn_rows as f32 * 11.0 + 18.0;
            let x = 8.0;
            let y = if touch_ui && portrait { 86.0 } else { 22.0 };
            let h = h.min(low_h() - y - 4.0);
            ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.92));
            ui_text(if two_player { if bag_owner == 0 { "bag - player 1" } else { "bag - player 2" } } else { "bag" }, x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            ui_rect(x + 8.0, y + 17.0, w - 16.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
            let mut ty = y + 29.0;
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::from_rgba(255, 236, 200, 255) } else { WHITE };
                ui_text(line, x + 8.0, ty, 10.0, col);
                ty += 11.0;
            }
            ty += 5.0;
            ui_text("today's quests", x + 8.0, ty, 10.0, Color::from_rgba(150, 215, 255, 255));
            ty += 11.0;
            for q in &quests.list {
                let mark = if q.done { "[x]" } else { "[ ]" };
                let line = format!("{} {}  {}/{}", mark, q.label(), q.progress, q.target);
                let reward = format!("+{}", q.reward());
                let col = if q.done { Color::new(0.6, 0.9, 0.6, 0.9) } else { WHITE };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.0) > w - 40.0 && shown.len() > 6 {
                    shown.pop();
                }
                ui_text(&shown, x + 8.0, ty, 9.0, col);
                ui_text(&reward, x + w - 8.0 - ui_measure(&reward, 9.0), ty, 9.0, ACORN_COLOR);
                ty += 11.0;
            }
            ty += 5.0;
            ui_text(&format!("furniture in the bag ({})", stored.len()), x + 8.0, ty, 10.0, Color::from_rgba(255, 190, 210, 255));
            ty += 11.0;
            if stored.is_empty() {
                ui_text(if inside { "empty - M while carrying a piece puts it here" } else { "empty" }, x + 8.0, ty, 9.0, Color::new(1.0, 1.0, 1.0, 0.5));
            } else {
                let first = bag_sel.saturating_sub(shown_items / 2).min(stored.len().saturating_sub(shown_items));
                for r in first..(first + shown_items).min(stored.len()) {
                    if inside && r == bag_sel {
                        ui_rect(x + 5.0, ty - 8.0, w - 10.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.14));
                    }
                    ui_text(stored[r].name(), x + 10.0, ty, 9.0, WHITE);
                    ty += 11.0;
                }
            }
            let bag_hint = if !inside {
                if touch_ui { "bag to close" } else { "M to close" }
            } else if bag_clear_armed > 0.0 {
                "X again: pack ALL furniture away"
            } else if bag_owner == 0 {
                "W/S choose  E take out  X pack all  M close"
            } else {
                "I/K choose  O take out  U pack all  Bksp close"
            };
            ui_text(bag_hint, x + (w - ui_measure(bag_hint, 9.0)) / 2.0, y + h - 6.0, 9.0, if bag_clear_armed > 0.0 { Color::from_rgba(255, 150, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) });
        }

        for who in 0..2 {
            if !(inside && dj[who]) || (who == 1 && !two_player) {
                continue;
            }
            let style = BEAT_STYLES[beat_style.min(BEAT_STYLES.len() - 1)].name;
            let title = format!("{}  {:.0} bpm  {}{}", if dj[0] && dj[1] { "b2b" } else { "on the decks" }, looper.bpm, if looper.beat { style } else { "beat off" }, if looper.recording { "  rec" } else { "" });
            let area = if two_player { half } else { low_w() };
            let left = if two_player && who == 1 { half } else { 0.0 };
            let hint = if who == 0 { "W/S tempo  A/D style  R rec  X out" } else { "7 8 9 0 - = pads  O out" };
            let track = if now_playing.is_empty() { String::new() } else { format!("now playing: {now_playing}") };
            let two_decks = live_decks.len() >= 2;
            let widest = ui_measure(&title, 12.0).max(ui_measure(hint, 9.0)).max(ui_measure(&track, 10.0));
            let w = if two_decks { (area - 8.0).min(400.0) } else { (widest + 16.0).max(150.0).min(area - 8.0) };
            let col_w = (w - 24.0) / 2.0;
            let columns: Vec<Vec<String>> = if two_decks {
                live_decks.iter().take(2).map(|(_, name)| ui_wrap(name, 10.0, col_w).into_iter().take(4).collect()).collect()
            } else {
                Vec::new()
            };
            let lines = if two_decks || track.is_empty() { Vec::new() } else { ui_wrap(&track, 10.0, w - 16.0) };
            let body = if two_decks { 13.0 + columns.iter().map(|c| c.len()).max().unwrap_or(1) as f32 * 12.0 + 4.0 } else { lines.len() as f32 * 12.0 };
            let w = if live_next.is_empty() { w } else { w.max(220.0f32.min(area - 8.0)) };
            let heard_line = live_next.iter().find(|t| t.starts_with('=')).map(|t| t[1..].to_string()).filter(|t| two_decks || !track.contains(t.as_str()));
            let picks: Vec<String> = live_next.iter().filter(|t| !t.starts_with('=')).take(4).map(|t| {
                let mut t = format!("+ {}", t);
                while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                    t.pop();
                }
                t
            }).collect();
            let heard_line = heard_line.map(|t| {
                let mut t = format!("heard: {}", t);
                while ui_measure(&t, 9.0) > w - 16.0 && t.len() > 6 {
                    t.pop();
                }
                t
            });
            let heard_h = if heard_line.is_some() { 13.0 } else { 0.0 };
            let extra = heard_h + if picks.is_empty() { 0.0 } else { 14.0 + picks.len() as f32 * 11.0 };
            let h = 30.0 + body + extra;
            let x = left + (area - w) / 2.0;
            let y = if low_h() > low_w() { 84.0 } else if two_player { 64.0 } else { 52.0 };
            ui_rect(x, y, w, h, Color::new(0.06, 0.04, 0.1, 0.78));
            ui_text(&title, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 90, 100, 255));
            for (n, line) in lines.iter().enumerate() {
                ui_text(line, x + 8.0, y + 25.0 + n as f32 * 12.0, 10.0, Color::from_rgba(255, 214, 110, 255));
            }
            let deck_colors = [Color::from_rgba(220, 60, 70, 255), Color::from_rgba(240, 200, 80, 255)];
            for (c, column) in columns.iter().enumerate() {
                let cx = x + 8.0 + c as f32 * (col_w + 8.0);
                ui_rect(cx, y + 17.0, 2.0, body - 4.0, deck_colors[c]);
                ui_text(&format!("deck {}", live_decks[c].0), cx + 6.0, y + 26.0, 9.0, deck_colors[c]);
                for (n, line) in column.iter().enumerate() {
                    ui_text(line, cx + 6.0, y + 38.0 + n as f32 * 12.0, 10.0, WHITE);
                }
            }
            let lines = if two_decks { vec![String::new(); ((body - 4.0) / 12.0).round() as usize] } else { lines };
            let mut shown = hint.to_string();
            while ui_measure(&shown, 9.0) > w - 14.0 && shown.len() > 4 {
                shown.pop();
            }
            ui_text(&shown, x + 8.0, y + 25.0 + lines.len() as f32 * 12.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.75));
            if let Some(line) = &heard_line {
                ui_text(line, x + 8.0, y + 30.0 + body + 6.0, 9.0, Color::from_rgba(150, 200, 255, 255));
            }
            if !picks.is_empty() {
                let top = y + 30.0 + body + 6.0 + heard_h;
                ui_text("goes well with", x + 8.0, top, 9.0, Color::from_rgba(120, 220, 160, 255));
                for (n, pick) in picks.iter().enumerate() {
                    ui_text(pick, x + 8.0, top + 12.0 + n as f32 * 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.9));
                }
            }
        }
        if fishing.active && fish_card <= 0.0 && fishing.lesson && (looper.position - fishing.cue_at).abs() <= 0.22 / looper.step_seconds() {
            let pop = 1.0 + 0.15 * (time * 16.0).sin();
            let size = 22.0 * pop;
            let word = if fishing.caster == 1 { "O!" } else { "E!" };
            let cx = if two_player { half * (fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
            ui_text(word, cx - ui_measure(word, size) / 2.0, low_h() * 0.38, size, Color::from_rgba(255, 226, 120, 255));
        }
        if let Some((i, left)) = fish_pop.map(|(i, l, _, _)| (i, l)) {
            let is_new = fish_pop.map(|f| f.2).unwrap_or(false);
            let shroom = fish_pop.map(|f| f.3).unwrap_or(false);
            let a = (left.min(0.4) / 0.4).min(1.0) * ((3.0 - left) * 6.0).min(1.0);
            let w = 150.0;
            let cx = if two_player { half / 2.0 } else { low_w() / 2.0 };
            let x = (cx - w / 2.0).round();
            let y = if low_h() > low_w() { 120.0 } else { 46.0 };
            ui_rect(x, y, w, 48.0, Color::new(0.08, 0.12, 0.22, 0.9 * a));
            let rarity_now = if shroom { mushroom::rarity(i) } else { fish::rarity(i) };
            let rc = fish::RARITY_COLORS[rarity_now];
            ui_rect(x, y, w, 2.0, Color::new(rc.r, rc.g, rc.b, a));
            if a > 0.5 {
                if shroom {
                    draw_fish_art(&shroom_art[i], x + 8.0, y + 7.0, 2.8, true);
                } else {
                    draw_fish_art(&fish_art[i], x + 6.0, y + 8.0, 2.2, true);
                }
            }
            let tx = x + 50.0;
            ui_text(&if shroom { mushroom::code(i) } else { fish::code(i) }, tx, y + 14.0, 9.0, Color::new(0.75, 0.85, 1.0, a));
            if is_new {
                ui_rect(tx + 26.0, y + 6.0, 26.0, 10.0, Color::new(1.0, 0.45, 0.55, a));
                ui_text("new!", tx + 29.0, y + 14.0, 9.0, Color::new(1.0, 1.0, 1.0, a));
            }
            let mut name = if shroom { mushroom::SPECIES[i] } else { fish::SPECIES[i] }.to_string();
            while ui_measure(&name, 11.0) > w - 56.0 && name.len() > 3 {
                name.pop();
            }
            ui_text(&name, tx, y + 28.0, 11.0, Color::new(1.0, 1.0, 1.0, a));
            let r = rarity_now;
            let c = fish::RARITY_COLORS[r];
            ui_text(fish::RARITY_NAMES[r], tx, y + 41.0, 9.0, Color::new(c.r, c.g, c.b, a));
        }
        if let Some(pop) = fish_pop.as_mut() {
            pop.1 -= get_frame_time();
            if pop.1 <= 0.0 {
                fish_pop = None;
            }
        }
        for who in 0..2 {
            if let Some((id, t)) = emoji_choose[who] {
                let (sx, sy, cell) = emoji_strip(who, two_player);
                ui_rect(sx - 4.0, sy - 4.0, cell * emoji::COUNT as f32 + 8.0, cell + 8.0, Color::new(0.97, 0.94, 0.87, 0.92));
                for k in 0..emoji::COUNT {
                    let ex = sx + k as f32 * cell;
                    if k == id {
                        ui_rect(ex - 1.0, sy - 2.0, cell, cell + 2.0, Color::from_rgba(255, 214, 110, 255));
                    }
                    draw_emoji_ui(k, ex + 1.5, sy + 0.5, 1.15, 1.0);
                }
                let fill = (t / 1.1).min(1.0);
                ui_rect(sx, sy + cell + 1.5, cell * emoji::COUNT as f32 * fill, 1.5, Color::from_rgba(255, 214, 110, 255));
            }
        }
        let small_pop = fish_say.as_ref().map(|(w, _, _)| matches!(w.as_str(), "perfect!" | "nice!" | "together!")).unwrap_or(false);
        if let (true, Some((word, col, left))) = (small_pop, fish_say.as_ref()) {
            let view = if two_player { fishing.caster.min(1) } else { 0 };
            let (bx, by) = BOBBER_AT.with(|cell| cell.get()[view]);
            let age = 1.0 - left.clamp(0.0, 1.0);
            let a = (left / 0.35).min(1.0);
            let pop = 1.0 + 0.45 * (1.0 - (age * 6.0).min(1.0));
            let size = 9.0 * pop;
            let w = ui_measure(word, size);
            let y = by - 4.0 - age * 10.0;
            ui_text(word, bx - w / 2.0, y, size, Color::new(col.r, col.g, col.b, a));
            if word == "perfect!" {
                for k in 0..3 {
                    let ang = age * 5.0 + k as f32 * 2.1;
                    ui_star(bx + ang.cos() * (w * 0.5 + 5.0), y - 3.0 + ang.sin() * 5.0, 2.5, Color::new(1.0, 0.95, 0.6, a));
                }
            }
        } else if let Some((word, col, left)) = fish_say.as_ref() {
            let a = left.min(0.4) / 0.4;
            let rise = (1.0 - left.min(1.0)) * 6.0;
            let cx = if two_player { half * (fishing.caster as f32 + 0.5) } else { low_w() / 2.0 };
            let size = if word.len() < 12 { 16.0 } else { 11.0 };
            let w = ui_measure(word, size);
            ui_rect(cx - w / 2.0 - 6.0, low_h() * 0.3 - rise - size, w + 12.0, size + 6.0, Color::new(0.08, 0.06, 0.16, 0.7 * a));
            ui_text(word, cx - w / 2.0, low_h() * 0.3 - rise, size, Color::new(col.r, col.g, col.b, a));
        }
        if fish_card > 0.0 {
            let a = (fish_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(0.45, 0.75, 1.0, a));
            let friend = cast_list[players[1].character].noun;
            ui_text("fishing on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let cx = x + 34.0;
            let cy = y + 62.0;
            let t = (time * 0.5).fract();
            let r = 6.0 + (1.0 - t) * 18.0;
            let ring_col = if t > 0.85 { Color::new(1.0, 0.86, 0.3, a) } else { Color::new(0.95, 0.97, 1.0, a) };
            for k in 0..20 {
                let ang = k as f32 / 20.0 * 2.0 * PI;
                ui_rect(cx + ang.cos() * r - 1.0, cy + ang.sin() * r * 0.6 - 1.0, 2.0, 2.0, ring_col);
            }
            ui_rect(cx - 3.0, cy - 5.0, 6.0, 4.0, Color::new(0.9, 0.24, 0.24, a));
            ui_rect(cx - 3.0, cy - 1.0, 6.0, 3.0, Color::new(0.98, 0.98, 0.95, a));
            if t > 0.85 {
                ui_text(if fishing.caster == 1 { "O!" } else { "E!" }, cx - 6.0, cy - 22.0, 12.0, Color::new(1.0, 0.86, 0.3, a));
            }
            let lines = [
                if two_player { "the river whispers:".to_string() } else { format!("the {} whispers:", friend) },
                "watch the ring around the bobber.".to_string(),
                format!("when it shrinks onto it, press {}.", if fishing.caster == 0 { "E" } else { "O" }),
                "three good ones and the fish is yours!".to_string(),
                if two_player { "player 2 can press O too - together is better".to_string() } else { format!("miss? you get 3 tries, and the {} helps", friend) },
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.75, 0.9, 0.7, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 78.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 68.0, y + 38.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if fishing.caster == 0 { "E: got it" } else { "O: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if cook_card > 0.0 {
            let a = (cook_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(1.0, 0.62, 0.34, a));
            ui_text("cooking on the beat", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let lines = [
                "sit by the fire, then press E or O again".to_string(),
                "to put the pot on.".to_string(),
                "tap the first four notes on the beat.".to_string(),
                "fish makes bigger dishes, and new dishes".to_string(),
                "earn more acorns.".to_string(),
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if two_player { "E / O: got it" } else { "E: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if gather_card > 0.0 {
            let a = (gather_card.min(0.4) / 0.4).min(1.0);
            let w = 250f32.min(low_w() - 16.0);
            let h = 128.0;
            let x = ((low_w() - w) / 2.0).round();
            let y = ((low_h() - h) / 2.0 - 20.0).round().max(24.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.22, 0.94 * a));
            ui_rect(x, y, w, 3.0, Color::new(0.5, 0.88, 0.52, a));
            ui_text("gathering around the woods", x + 10.0, y + 18.0, 14.0, Color::new(1.0, 0.89, 0.6, a));
            let lines = [
                "face a tree or stone and press E or O.".to_string(),
                "sparkly trees give sticks to carry.".to_string(),
                "other trees shake loose acorns.".to_string(),
                "stones can be picked up too.".to_string(),
                "bring your finds back to the fire.".to_string(),
            ];
            for (n, line) in lines.iter().enumerate() {
                let col = if n == 0 { Color::new(0.9, 1.0, 0.8, a) } else { Color::new(1.0, 1.0, 1.0, 0.9 * a) };
                let mut shown = line.clone();
                while ui_measure(&shown, 9.5) > w - 20.0 && shown.len() > 4 {
                    shown.pop();
                }
                ui_text(&shown, x + 10.0, y + 42.0 + n as f32 * 14.0, 9.5, col);
            }
            let go = if two_player { "E / O: got it" } else { "E: got it" };
            ui_text(go, x + w - ui_measure(go, 10.0) - 10.0, y + h - 9.0, 10.0, Color::new(1.0, 0.89, 0.6, a));
        }
        if shop_open {
            let rows = home::shop_rows();
            let w = 330f32.min(low_w() - 10.0);
            let x = ((low_w() - w) / 2.0).round();
            let y = if low_h() > low_w() { (low_h() * 0.1).round() } else { ((low_h() - 214.0) / 2.0 - 4.0).round().max(18.0) };
            let h = 214.0f32.min(low_h() - y - 6.0);
            ui_rect(x, y, w, h, Color::new(0.12, 0.09, 0.2, 0.94));
            let tab_col = |on: bool| if on { Color::from_rgba(255, 226, 150, 255) } else { Color::new(1.0, 1.0, 1.0, 0.45) };
            ui_text("catalogue", x + 8.0, y + 16.0, 14.0, tab_col(shop_tab == 0));
            let diary_x = x + 14.0 + ui_measure("catalogue", 14.0);
            ui_text("diary", diary_x, y + 16.0, 14.0, tab_col(shop_tab == 1));
            let fish_x = diary_x + ui_measure("diary", 14.0) + 6.0;
            ui_text("fishdex", fish_x, y + 16.0, 11.0, tab_col(false));
            ui_text("mushrooms", fish_x + ui_measure("fishdex", 11.0) + 5.0, y + 16.0, 11.0, tab_col(false));
            ui_rect(if shop_tab == 0 { x + 8.0 } else { diary_x }, y + 19.0, ui_measure(if shop_tab == 0 { "catalogue" } else { "diary" }, 14.0), 1.0, Color::from_rgba(255, 226, 150, 255));
            let wallet = format!("{}", inv.acorns);
            ui_rect(x + w - 14.0 - ui_measure(&wallet, 11.0), y + 8.0, 6.0, 6.0, ACORN_COLOR);
            ui_text(&wallet, x + w - 6.0 - ui_measure(&wallet, 11.0), y + 15.0, 11.0, WHITE);
            if shop_tab == 0 {
                let row = home::row(shop_row);
                let (name, price, extra) = shop_row_info(row, &house);
                let preview_x = x + 10.0;
                let preview_y = y + 28.0;
                let preview_w = w - 20.0;
                let preview_h = (preview_w * 150.0 / 400.0).min(118.0).round();
                let (preview_kx, preview_ky) = ui_scale();
                draw_texture_ex(
                    &catalogue_view.texture,
                    preview_x * preview_kx,
                    preview_y * preview_ky,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(preview_w * preview_kx, preview_h * preview_ky)),
                        flip_y: true,
                        ..Default::default()
                    },
                );
                ui_rect(preview_x, preview_y, preview_w, preview_h, Color::new(1.0, 1.0, 1.0, 0.06));
                let left_key = if shop_owner == 1 { "J" } else { "A" };
                let right_key = if shop_owner == 1 { "L" } else { "D" };
                let index = format!("{} / {}", shop_row + 1, rows);
                ui_text(&format!("{} <", left_key), preview_x + 6.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text(&format!("> {}", right_key), preview_x + preview_w - 20.0, preview_y + preview_h / 2.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text(&index, preview_x + preview_w - ui_measure(&index, 9.0) - 6.0, preview_y + 11.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
                let title_y = preview_y + preview_h + 16.0;
                ui_text(&name, x + (w - ui_measure(&name, 15.0)) / 2.0, title_y, 15.0, Color::from_rgba(255, 240, 200, 255));
                let price_col = price.split_whitespace().next().and_then(|v| v.parse::<u32>().ok()).map(|p| if p <= inv.acorns { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) }).unwrap_or(Color::new(1.0, 1.0, 1.0, 0.75));
                ui_text(&price, x + (w - ui_measure(&price, 11.0)) / 2.0, title_y + 13.0, 11.0, price_col);
                let about = if extra.is_empty() { "a little room upgrade for your musical home" } else { &extra };
                let lines = ui_wrap(about, 10.0, w - 26.0);
                for (n, line) in lines.iter().take(2).enumerate() {
                    ui_text(line, x + (w - ui_measure(line, 10.0)) / 2.0, title_y + 28.0 + n as f32 * 11.0, 10.0, Color::new(0.88, 0.94, 1.0, 0.82));
                }
            } else if diary.is_empty() {
                ui_text("no jams yet - play together by the fire", x + 10.0, y + 46.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
                ui_text("30s = 1 star, 1 min = 2, 3 min = 3", x + 10.0, y + 62.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.5));
            } else {
                let shown = 8usize;
                let row_h = 15.0;
                let total: u32 = diary.iter().map(|e| e.stars as u32).sum();
                let n = diary.len();
                let first = diary_row.saturating_sub(shown / 2).min(n.saturating_sub(shown));
                for r in first..(first + shown).min(n) {
                    let entry = &diary[n - 1 - r];
                    let ry = y + 30.0 + (r - first) as f32 * row_h;
                    if r == diary_row {
                        ui_rect(x + 4.0, ry - 2.0, w - 8.0, row_h - 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
                    }
                    let mins = format!("{}:{:02}", entry.seconds / 60, entry.seconds % 60);
                    let who = format!("{} + {}", cast_list[entry.who.0.min(cast_list.len() - 1)].noun, cast_list[entry.who.1.min(cast_list.len() - 1)].noun);
                    ui_text(&date_label(entry.when), x + 10.0, ry + 9.0, 11.0, WHITE);
                    ui_text(&who, x + 56.0, ry + 9.0, 10.0, Color::new(0.85, 0.92, 1.0, 0.85));
                    ui_text(&mins, x + w - 52.0 - ui_measure(&mins, 10.0), ry + 9.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.7));
                    for k in 0..3u8 {
                        let col = if k < entry.stars { Color::from_rgba(255, 214, 90, 255) } else { Color::new(1.0, 1.0, 1.0, 0.18) };
                        ui_star(x + w - 42.0 + k as f32 * 11.0, ry + 1.0, 8.0, col);
                    }
                }
                let line = format!("{} jams, {} stars", n, total);
                ui_text(&line, x + w - 8.0 - ui_measure(&line, 9.0), y + 26.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.55));
            }
            let help = if shop_owner == 1 {
                if shop_tab == 0 { "I/K tabs - J/L browse - O buy - U close" } else { "I/K tabs - J/L choose - O play - U close" }
            } else if shop_tab == 0 {
                "W/S tabs - A/D browse - E buy - X close"
            } else {
                "W/S tabs - A/D choose - E play - X close"
            };
            ui_text(help, x + (w - ui_measure(help, 10.0)) / 2.0, y + h - 6.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
        }

        if !picking && !show_help && first_steps > 0 && first_steps < 4 {
            let (heading, message) = match first_steps {
                1 => ("your first song - 1/3", "press any note 1-6 to sing"),
                2 if camp.lit => ("find the fire - 2/3", "walk to the campfire and press E to sit"),
                2 => ("find the fire - 2/3", "explore while your friend brings firewood"),
                _ => ("make a memory - 3/3", "play music together to earn a star"),
            };
            let w = (low_w() - 16.0).min(250.0);
            let x = (low_w() - w) / 2.0;
            let y = if touch_ui && portrait { 27.0 } else { 21.0 };
            ui_rect(x, y, w, 30.0, Color::new(0.1, 0.08, 0.2, 0.83));
            ui_text(heading, x + 8.0, y + 12.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            let line = ui_wrap(message, 10.0, w - 16.0);
            if let Some(first) = line.first() {
                ui_text(first, x + 8.0, y + 24.0, 10.0, WHITE);
            }
        }

        if picking {
            let w = 230.0f32.min(low_w() - 10.0);
            let preview_h = (w - 12.0) * 130.0 / 300.0;
            let top = if first_pick { 24.0 } else { 30.0 };
            let x = (low_w() - w) / 2.0;
            let h = preview_h + if first_pick { 100.0 } else { 76.0 };
            ui_rect(x, top, w, h, Color::new(0.1, 0.08, 0.2, 0.94));
            let mut y = top + 16.0;
            if first_pick {
                let title = "melody quack";
                let tw = ui_measure(title, 22.0);
                ui_text(title, (low_w() - tw) / 2.0, y + 6.0, 22.0, Color::from_rgba(255, 226, 150, 255));
                y += 24.0;
            }
            {
                let (kx, ky) = ui_scale();
                let px = x + 6.0;
                let py = y - 8.0;
                draw_texture_ex(&pick_view.texture, px * kx, py * ky, WHITE, DrawTextureParams { dest_size: Some(vec2((w - 12.0) * kx, preview_h * ky)), flip_y: true, ..Default::default() });
                y += preview_h + 6.0;
            }
            let labels = if two_player { ["player 1", "player 2"] } else { ["you", "friend"] };
            for row in 0..2 {
                let chosen = row == pick_row;
                let col = if chosen { Color::from_rgba(255, 236, 170, 255) } else { Color::new(1.0, 1.0, 1.0, 0.6) };
                if chosen {
                    ui_rect(x + 6.0, y - 9.0, w - 12.0, 12.0, Color::new(1.0, 1.0, 1.0, 0.1));
                }
                ui_text(labels[row], x + 12.0, y, 12.0, col);
                let name = format!("<  {}  >", cast_list[players[row].character].name);
                ui_text(&name, x + 80.0, y, 12.0, col);
                y += 14.0;
            }
            ui_text("W/S row   A/D change   E start", x + 12.0, y + 6.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }

        if sleep_fade > 0.0 {
            sleep_fade -= get_frame_time();
            let a = (1.0 - (sleep_fade - 1.5).abs() / 1.5).clamp(0.0, 1.0);
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.04, 0.03, 0.1, a * 0.92));
            let z = if (time * 2.0) as i32 % 2 == 0 { "z z z" } else { "z Z z" };
            ui_text(z, low_w() / 2.0 - ui_measure(z, 16.0) / 2.0, low_h() / 2.0, 16.0, Color::new(1.0, 1.0, 1.0, a));
            if a > 0.6 {
                let line = "sweet dreams - game saved";
                ui_text(line, low_w() / 2.0 - ui_measure(line, 11.0) / 2.0, low_h() / 2.0 + 18.0, 11.0, Color::new(1.0, 1.0, 1.0, a * 0.8));
            }
        }
        if sampler.open {
            let w = (low_w() - 16.0).min(330.0);
            let x = ((low_w() - w) / 2.0).round();
            let cell = ((w - 60.0) / 16.0).floor();
            let h = 34.0 + cell * 4.0 + 34.0;
            let y = low_h() - h - 44.0;
            ui_rect(x, y, w, h, Color::new(0.08, 0.06, 0.16, 0.94));
            ui_text("sampler", x + 8.0, y + 13.0, 12.0, Color::from_rgba(255, 226, 150, 255));
            let bpm = format!("{:.0} bpm", looper.bpm);
            ui_text(&bpm, x + w - 8.0 - ui_measure(&bpm, 10.0), y + 13.0, 10.0, Color::new(1.0, 1.0, 1.0, 0.6));
            let gx = x + 52.0;
            let gy = y + 22.0;
            for row in 0..4 {
                let col = note_color(row);
                let label = SAMPLER_ROWS[row];
                ui_text(label, x + 8.0, gy + row as f32 * cell + cell * 0.7, 9.0, col);
                if sampler.pads[row].is_some() {
                    ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 120, 150, 255));
                    ui_text("mic", x + 35.0, gy + row as f32 * cell + cell * 0.68, 6.0, WHITE);
                }
                if matches!(sampler.recording, Some((r, _)) if r == row) && (time * 4.0).fract() < 0.6 {
                    ui_rect(x + 34.0, gy + row as f32 * cell + 2.0, 14.0, cell - 4.0, Color::from_rgba(255, 60, 70, 255));
                }
                for k in 0..16 {
                    let cx = gx + k as f32 * cell;
                    let cy = gy + row as f32 * cell;
                    let on = sampler.grid[row][k];
                    let beat = k % 4 == 0;
                    let base = if on { col } else if beat { Color::new(1.0, 1.0, 1.0, 0.16) } else { Color::new(1.0, 1.0, 1.0, 0.08) };
                    ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, base);
                    if k == sampler.step {
                        ui_rect(cx + 1.0, cy + 1.0, cell - 2.0, cell - 2.0, Color::new(1.0, 1.0, 1.0, if on { 0.45 } else { 0.12 }));
                    }
                    if (k, row) == sampler.cursor {
                        let c = Color::from_rgba(255, 226, 150, 255);
                        ui_rect(cx, cy, cell, 1.0, c);
                        ui_rect(cx, cy + cell - 1.0, cell, 1.0, c);
                        ui_rect(cx, cy, 1.0, cell, c);
                        ui_rect(cx + cell - 1.0, cy, 1.0, cell, c);
                    }
                }
            }
            let hint = if sampler.recording.is_some() {
                "recording - make a sound into your mic"
            } else if sampler.owner == 0 {
                "arrows/WASD move  E step on/off  Space record your sound  X close"
            } else {
                "IJKL move  O step on/off  Space record your sound  U close"
            };
            ui_text(hint, x + (w - ui_measure(hint, 9.0)) / 2.0, y + h - 18.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.8));
            let note = "record only your own voice or sounds - recordings stay on this device and are left out of tapes";
            let mut shown = note.to_string();
            while ui_measure(&shown, 7.0) > w - 12.0 && shown.len() > 10 {
                shown.pop();
            }
            ui_text(&shown, x + (w - ui_measure(&shown, 7.0)) / 2.0, y + h - 6.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.5));
        }
        if ddr.on[0] || ddr.on[1] {
            let active: Vec<usize> = (0..2).filter(|w| ddr.on[*w]).collect();
            for (slot, &who) in active.iter().enumerate() {
                let cx = if active.len() == 2 { low_w() * (0.17 + slot as f32 * 0.66) } else { low_w() * 0.78 };
                let lane_w = 22.0;
                let x0 = cx - lane_w * 2.0;
                let top = 34.0;
                ui_rect(x0 - 4.0, top - 8.0, lane_w * 4.0 + 8.0, low_h() - top - 36.0, Color::new(0.04, 0.03, 0.1, 0.55));
                for lane in 0..4 {
                    let lx = x0 + lane as f32 * lane_w + 2.0;
                    let f = ddr.flash[who][lane];
                    let ring: Vec<(usize, usize, Color)> = ddr_art[lane].iter().map(|(x, y, _)| (*x, *y, Color::new(0.7 + 0.3 * f, 0.7 + 0.3 * f, 0.75 + 0.25 * f, 0.35 + 0.65 * f))).collect();
                    draw_pixel_grid(&ring, lx, top, 2.0, |c| c);
                }
                for a in ddr.arrows.iter().filter(|a| !a.hit[who]) {
                    let y = top + (a.at - time) * 75.0;
                    if y < top - 20.0 || y > low_h() - 40.0 {
                        continue;
                    }
                    let fade = if a.missed[who] { 0.35 } else { 1.0 };
                    draw_pixel_grid(&ddr_art[a.lane], x0 + a.lane as f32 * lane_w + 2.0, y, 2.0, |c| Color::new(c.r, c.g, c.b, fade));
                }
                let score = format!("{}", ddr.score[who]);
                ui_text(&score, cx - ui_measure(&score, 11.0) / 2.0, top - 12.0, 11.0, Color::from_rgba(255, 226, 150, 255));
                if let Some((word, col, left)) = ddr.judge[who] {
                    let size = 13.0 + 3.0 * (left - 0.4).max(0.0) * 5.0;
                    ui_text(word, cx - ui_measure(word, size) / 2.0, top + 38.0, size, Color::new(col.r, col.g, col.b, (left / 0.3).min(1.0)));
                }
                if ddr.combo[who] >= 4 {
                    let combo = format!("{} combo", ddr.combo[who]);
                    ui_text(&combo, cx - ui_measure(&combo, 10.0) / 2.0, top + 52.0, 10.0, WHITE);
                }
            }
            let source = if time - ddr.last_mic < 3.0 { "arrows from the music on your mic" } else if microphone.is_some() { "listening... arrows follow the game beat for now" } else { "no mic - arrows follow the game beat and your loop" };
            ui_text(source, (low_w() - ui_measure(source, 9.0)) / 2.0, low_h() - 30.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        if show_help {
            let pages: [(&str, &[(&str, &str)]); 3] = [
                ("basics", &[
                    ("WASD", "move  (player 2: IJKL)"),
                    ("E", "act  (player 2: O)"),
                    ("M", "bag, quests, furniture  (player 2: Backspace)"),
                    ("Z", "emoji  (player 2: .)"),
                    ("P", "1 or 2 players"),
                    ("V", "choose characters"),
                    ("Esc", "pause, mixer, fishdex"),
                    ("H / Esc", "close this help"),
                ]),
                ("music", &[
                    ("1 - 6", "sing  (player 2: 7 8 9 0 - =)"),
                    ("Tab", "spells / music"),
                    ("R / C", "record / clear  (player 2: Y)"),
                    ("Q", "quantise"),
                    ("B", "beat on / off"),
                    ("F", "follow the mic tempo"),
                    ("G", "echo on / off"),
                    ("T", "tape: record a .wav"),
                    ("shift+C", "friend forgets all tunes"),
                ]),
                ("world", &[
                    ("E at fire", "sit, again: cook"),
                    ("E at dock", "cast and fish"),
                    ("E by trees", "stick, mushroom, acorns"),
                    ("E in yard", "build walls and floors"),
                    ("X", "change build tool"),
                    ("E at door", "go inside"),
                    ("E / X inside", "use / move furniture"),
                    ("N twice", "fresh world"),
                ]),
            ];
            let (x, y, w, h) = help_box();
            ui_rect(0.0, 0.0, low_w(), low_h(), Color::new(0.03, 0.02, 0.08, 0.55));
            ui_rect(x, y, w, h, Color::new(0.1, 0.08, 0.2, 0.97));
            ui_rect(x, y, w, 3.0, Color::from_rgba(255, 214, 110, 255));
            ui_text("help", x + 10.0, y + 15.0, 13.0, Color::from_rgba(255, 226, 150, 255));
            let tab_w = (w - 20.0) / 3.0;
            for (k, (name, _)) in pages.iter().enumerate() {
                let tx = x + 10.0 + k as f32 * tab_w;
                let on = k == help_page;
                ui_rect(tx + 1.0, y + 20.0, tab_w - 2.0, 16.0, if on { Color::from_rgba(255, 214, 110, 255) } else { Color::new(1.0, 1.0, 1.0, 0.1) });
                ui_text(name, tx + (tab_w - ui_measure(name, 11.0)) / 2.0, y + 32.0, 11.0, if on { Color::from_rgba(36, 26, 71, 255) } else { WHITE });
            }
            let rows = pages[help_page].1;
            let key_w = rows.iter().map(|(k, _)| ui_measure(k, 11.0)).fold(0.0, f32::max) + 10.0;
            for (n, (key, text)) in rows.iter().enumerate() {
                let ry = y + 44.0 + n as f32 * 16.0;
                ui_rect(x + 10.0, ry, ui_measure(key, 11.0) + 8.0, 13.0, Color::new(1.0, 0.84, 0.43, 0.2));
                ui_text(key, x + 14.0, ry + 10.0, 11.0, Color::from_rgba(255, 226, 150, 255));
                let mut t = text.to_string();
                while ui_measure(&t, 11.0) > w - key_w - 24.0 && t.len() > 4 {
                    t.pop();
                }
                ui_text(&t, x + 16.0 + key_w, ry + 10.0, 11.0, WHITE);
            }
            let foot = if radio::touch_ui() { "tap a tab - tap outside to close" } else { "left/right: page   H or Esc: close" };
            ui_text(foot, x + (w - ui_measure(foot, 9.0)) / 2.0, y + h - 6.0, 9.0, Color::new(1.0, 1.0, 1.0, 0.6));
        }

        next_frame().await
    }
}
