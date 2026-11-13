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

#[cfg(not(target_arch = "wasm32"))]
#[path = "native/net.rs"]
mod net;

#[cfg(target_arch = "wasm32")]
#[path = "web/net.rs"]
mod net;

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
mod state;
mod frame_state;
mod steps;
mod venues;

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
use state::*;
use frame_state::*;

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
    let view_w = low_w() as u32;
    let view_h = low_h() as u32;
    let full_view = make_view(view_w, view_h);
    let catalogue_view = make_view(400, 150);
    let pick_view = make_view(300, 130);

    let mut world = World::new();
    let particles: Vec<Particle> = Vec::new();
    let mut players = vec![Player::new(0, cast_list[0].body, vec2(6.5, 16.5)), Player::new(4, cast_list[4].body, vec2(7.5, 16.5))];
    let mut picking = true;
    let mut first_pick = true;
    let mut first_steps = 0u8;
    let pick_row = 0usize;
    let brain = DogBrain::new();
    let mut camp = Camp { fuel: 0.0, lit: false, pile: 0 };
    let mut inv = Inventory { wood: 2, stone: 0, fish: [0; 3], mush: [0; 3], acorns: 0 };
    let tools = [Tool::WoodWall, Tool::StoneWall];
    let fishing = Fishing { active: false, caster: 0, cue_at: 0.0, hits: 0, tries: FISH_TRIES, hit_by: [false; 2], early: [false; 2], lesson: false, perfects: 0, dip: 0.0, jump: None, tight: 1.0 };
    let mut fish_lesson = false;
    let mut cook_lesson = false;
    let mut gather_lesson = false;
    let fish_card = 0.0_f32;
    let cook_card = 0.0_f32;
    let gather_card = 0.0_f32;
    let fish_say: Option<(String, Color, f32)> = None;
    let mut dex_counts: Vec<u32> = vec![0; fish::COUNT];
    let mut shroom_counts: Vec<u32> = vec![0; mushroom::COUNT];
    let shroom_art: Vec<Vec<(usize, usize, Color)>> = (0..mushroom::COUNT).map(mushroom::art).collect();
    let shrooms: Vec<(Vec2, usize)> = Vec::new();
    let shroom_clock = 3.0f32;
    let shroom_day = i64::MIN;
    let shroom_today: Vec<usize> = Vec::new();
    let dex_mode = 0usize;
    let dex_from_shop = false;
    let fish_art: Vec<Vec<(usize, usize, Color)>> = (0..fish::COUNT).map(fish::art).collect();
    let fish_pop: Option<(usize, f32, bool, bool)> = None;
    let dex_open = false;
    let dex_query = String::new();
    let dex_sel = 0usize;
    let dex_drag: Option<(f32, usize)> = None;
    let emoji_choose: [Option<(usize, f32)>; 2] = [None, None];
    let emoji_last = [0usize, 0usize];
    let emoji_reply: Option<(usize, f32)> = None;
    let cooking = false;
    let mut cook_progress = 0u32;
    let mut dishes: Vec<usize> = Vec::new();
    let hearts: Vec<(Vec3, f32)> = Vec::new();
    let jam = false;
    let dusk = update_sky().night * 0.95;
    let echoes: Vec<(f32, usize)> = Vec::new();
    let partner_phrase: Vec<(usize, usize)> = Vec::new();
    let partner_soloing = false;
    let partner_solo_count = 0u32;
    let mut songbook: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut partner_xp = 0u32;
    let cat_phrase_notes = 0u32;
    let recent_cat: Vec<usize> = Vec::new();
    let crackle_clock = 0.0_f32;
    let mut toasts: Vec<(String, f32)> = Vec::new();
    let two_player = false;
    let mut ringing = false;
    let ring_flash = 0.0_f32;
    let show_help = false;
    let help_page = 0usize;
    let show_inventory = false;
    let bag_owner = 0usize;
    let bag_sel = 0usize;
    let bag_clear_armed = 0.0f32;
    let mut stored: Vec<home::Item> = Vec::new();
    let tick_clock = 0.0_f32;
    let mut looper = Looper::new();
    let music_mode = true;
    let microphone: Option<mic::Mic> = None;
    let listening = false;
    let listen_clock = 0.0_f32;
    let bpm_history: Vec<f32> = Vec::new();
    let locked = false;
    let mic_message: Option<&str> = None;
    let save_clock = 0.0_f32;
    let sleep_fade = 0.0_f32;
    let shelf_open = false;
    let shelf_sel = 0usize;
    let open_book: Option<usize> = None;
    let book_spread = 0usize;
    let page_turn = 0.0f32;
    let turn_dir = 1.0f32;
    let painted: Option<(usize, usize)> = None;
    let spines_painted = false;
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
    let mic_message_clock = 0.0_f32;
    let tree_rest: std::collections::HashMap<(i32, i32), f32> = std::collections::HashMap::new();
    let stick_trees: Vec<(i32, i32)> = Vec::new();
    let mut diary: Vec<DiaryEntry> = Vec::new();
    let session: Option<f32> = None;
    let session_stars = 0u8;
    let last_note_at = -100.0_f32;
    let shop_tab = 0usize;
    let diary_row = 0usize;
    let playback: Option<(usize, f64, usize)> = None;
    let dj = [false, false];
    let ddr = Ddr { on: [false; 2], arrows: Vec::new(), score: [0; 2], combo: [0; 2], judge: [None; 2], flash: [[0.0; 4]; 2], last_mic: -99.0 };
    let ddr_request: Option<(usize, usize)> = None;
    let quests = Quests { day: i64::MIN, list: Vec::new() };
    let mut sampler = Sampler { open: false, owner: 0, grid: [[false; 16]; 4], cursor: (0, 0), pads: [None, None, None, None], recording: None, step: 0 };
    let mut saved_quests: Option<(i64, Vec<(u8, u32, u32, bool)>)> = None;
    let ddr_art: Vec<Vec<(usize, usize, Color)>> = (0..4).map(ddr_arrow_pixels).collect();
    let beat_style = 3usize;
    let stick_clock = 0.0_f32;
    let mut house = home::Home::new();
    let inside = false;
    let holding: Option<home::Piece> = None;
    let holding_by = 0usize;
    let shop_open = false;
    let shop_owner = 0usize;
    let shop_row = 0usize;
    let seat: [Option<home::Item>; 2] = [None, None];
    let chime_clock = 3.0_f32;
    let record_tune: Vec<(usize, usize)> = Vec::new();
    let record_pick = 0usize;
    let wander_clock = 0.0_f32;
    let wander_to = home::world(home::Home::new().door());
    let paused = false;
    let restart_now = false;
    let restart_confirm = 0.0_f32;
    let mut mix = [1.0_f32; 5];
    let mix_drag: Option<usize> = None;
    let first_click_done = false;
    let radio_clock = 0.0_f32;
    let last_report = String::new();
    let now_playing = String::new();
    let live_decks: Vec<(String, String)> = Vec::new();
    let live_next: Vec<String> = Vec::new();
    let radio_bpm = 0.0_f32;
    let fresh_confirm = 0.0_f32;
    let mut saved_name = String::new();
    let mut saved_dj = (String::new(), false);
    let mut saved_tour = 0u8;
    let mut fresh_key: Option<(String, String)> = None;
    let mut saved_session = String::new();
    let mut saved_companion = true;
    let loaded = store::read().and_then(|t| decode_progress(&t, cast_list.len())).filter(|s| {
        if s.fresh {
            fresh_key = Some((s.dj_key.clone(), s.session.clone()));
        }
        !s.fresh
    });
    if let Some((key, session)) = fresh_key {
        saved_dj = (key, false);
        saved_session = session;
    }
    if let Some(saved) = loaded {
        saved_name = saved.name.clone();
        saved_dj = (saved.dj_key.clone(), saved.welcome);
        saved_tour = saved.tour;
        saved_session = saved.session.clone();
        saved_companion = !saved.companion_off;
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

    venues::stamp(&mut world);
    let mut g = Game {
        venue: venues::VenueState::new(saved_name, saved_dj.0, saved_dj.1),
        audio,
        cast_list,
        voices,
        marimba,
        bells,
        piano,
        open_hat,
        tom,
        rim,
        bass,
        kick,
        clap,
        hat,
        crackle,
        lid,
        chop,
        sizzle,
        plink,
        splash,
        view_w,
        view_h,
        full_view,
        catalogue_view,
        pick_view,
        world,
        particles,
        players,
        picking,
        first_pick,
        first_steps,
        pick_row,
        brain,
        camp,
        inv,
        tools,
        fishing,
        fish_lesson,
        cook_lesson,
        gather_lesson,
        fish_card,
        cook_card,
        gather_card,
        fish_say,
        dex_counts,
        shroom_counts,
        shroom_art,
        shrooms,
        shroom_clock,
        shroom_day,
        shroom_today,
        dex_mode,
        dex_from_shop,
        fish_art,
        fish_pop,
        dex_open,
        dex_query,
        dex_sel,
        dex_drag,
        emoji_choose,
        emoji_last,
        emoji_reply,
        cooking,
        cook_progress,
        dishes,
        hearts,
        jam,
        dusk,
        echoes,
        partner_phrase,
        partner_soloing,
        partner_solo_count,
        songbook,
        partner_xp,
        cat_phrase_notes,
        recent_cat,
        crackle_clock,
        toasts,
        two_player,
        ringing,
        ring_flash,
        show_help,
        help_page,
        show_inventory,
        bag_owner,
        bag_sel,
        bag_clear_armed,
        stored,
        tick_clock,
        looper,
        music_mode,
        microphone,
        listening,
        listen_clock,
        bpm_history,
        locked,
        mic_message,
        save_clock,
        sleep_fade,
        shelf_open,
        shelf_sel,
        open_book,
        book_spread,
        page_turn,
        turn_dir,
        painted,
        spines_painted,
        page_rt,
        spine_rt,
        mic_message_clock,
        tree_rest,
        stick_trees,
        diary,
        session,
        session_stars,
        last_note_at,
        shop_tab,
        diary_row,
        playback,
        dj,
        ddr,
        ddr_request,
        quests,
        sampler,
        saved_quests,
        ddr_art,
        beat_style,
        stick_clock,
        house,
        inside,
        holding,
        holding_by,
        shop_open,
        shop_owner,
        shop_row,
        seat,
        chime_clock,
        record_tune,
        record_pick,
        wander_clock,
        wander_to,
        paused,
        restart_now,
        companion: saved_companion,
        restart_confirm,
        mix,
        mix_drag,
        first_click_done,
        radio_clock,
        last_report,
        now_playing,
        live_decks,
        live_next,
        radio_bpm,
        fresh_confirm,
    };
    g.venue.tour = saved_tour;
    g.venue.session = saved_session;
    loop {
        let mut fr = Frame::default();
        steps::input::begin_frame(&mut g, &mut fr);
        if steps::venues::frame(&mut g, &mut fr) {
            next_frame().await;
            continue;
        }
        if g.shelf_open {
            steps::screens::bookshelf(&mut g, &mut fr);
            next_frame().await;
            continue;
        }
        if g.dex_open {
            steps::screens::fishdex(&mut g, &mut fr);
            next_frame().await;
            continue;
        }
        steps::menus::help_and_escape(&mut g, &mut fr);
        if g.paused {
            steps::screens::pause_menu(&mut g, &mut fr);
            next_frame().await;
            continue;
        }
        steps::menus::hotkeys(&mut g, &mut fr);
        steps::menus::bag(&mut g, &mut fr);
        steps::menus::tape_and_fresh_start(&mut g, &mut fr);
        steps::radio_step::report_and_radio(&mut g, &mut fr);
        steps::saving::autosave(&mut g, &mut fr);
        if fr.quitting || g.restart_now {
            if let Some(mix) = g.audio.stop_tape() {
                store::save_jam(&wav_bytes(&mix));
            }
            if g.restart_now {
                g.restart_now = false;
                store::restart();
            } else {
                break;
            }
        }
        steps::audio_step::echo_and_quantise(&mut g, &mut fr);
        steps::picker::character_picker(&mut g, &mut fr);
        steps::audio_step::tool_keys(&mut g, &mut fr);
        steps::audio_step::listening(&mut g, &mut fr);
        steps::movement::read_movement(&mut g, &mut fr);
        steps::movement::outdoors(&mut g, &mut fr);
        steps::indoors::indoors(&mut g, &mut fr);
        steps::friend::friend_and_deliveries(&mut g, &mut fr);
        steps::movement::doors(&mut g, &mut fr);
        steps::friend::receive_delivery(&mut g, &mut fr);
        steps::movement::build_walls(&mut g, &mut fr);
        steps::sampler_step::sampler(&mut g, &mut fr);
        steps::arcade::dance_arcade(&mut g, &mut fr);
        steps::audio_step::visual_bands(&mut g, &mut fr);
        steps::fishing_step::fishing(&mut g, &mut fr);
        steps::camp_step::jam_and_fire(&mut g, &mut fr);
        steps::day::day_and_mushrooms(&mut g, &mut fr);
        steps::notes::play_notes(&mut g, &mut fr);
        steps::notes::on_step(&mut g, &mut fr);
        steps::notes::session_and_playback(&mut g, &mut fr);
        steps::world_step::world_tick(&mut g, &mut fr);
        steps::draw_scene::draw_scene(&mut g, &mut fr);
        steps::hud::draw_hud(&mut g, &mut fr);
        steps::hud::draw_bag(&mut g, &mut fr);
        steps::popups::draw_popups(&mut g, &mut fr);
        steps::shop::draw_shop(&mut g, &mut fr);
        steps::overlays::draw_overlays(&mut g, &mut fr);
        steps::venues::mail_icon(&mut g, &mut fr);
        next_frame().await
    }
}
