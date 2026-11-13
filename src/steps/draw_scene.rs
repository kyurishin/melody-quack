#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn draw_scene(g: &mut Game, fr: &mut Frame) {
    let cat = &g.players[0];
    fr.ahead = cat.ahead();
    fr.can_build_here = !g.inside && !cat.seated && !g.fishing.active && !(g.camp.lit && near_fire(cat)) && !at_dock_end(cat) && buildable(&g.world, fr.ahead.0, fr.ahead.1);
    fr.ring = if g.fish_card > 0.0 { 1.0 } else { ((g.fishing.cue_at - g.looper.position) / FISH_CUE_GAP).clamp(0.0, 1.0) };
    fr.extras = Extras {
        fishing: g.fishing.active,
        ring: fr.ring,
        in_window: g.fish_card <= 0.0 && (g.looper.position - g.fishing.cue_at).abs() <= 0.22 * g.fishing.tight / g.looper.step_seconds(),
        dip: g.fishing.dip,
        jump: g.fishing.jump,
        cat: g.players[g.fishing.caster].pos,
        pot: g.cooking,
        hearts: g.hearts.clone(),
        ghost: if fr.can_build_here { Some((fr.ahead.0, fr.ahead.1, g.tools[0])) } else { None },
        sticks: g.stick_trees.iter().filter_map(|t| stick_spot(&g.world, *t)).collect(),
        friend_rod: if g.fishing.active && !g.two_player && g.players[1].pos.distance(DOCK_END - vec2(1.0, 0.0)) < 0.8 { Some(g.players[1].pos) } else { None },
    };

    fr.dance = if g.looper.beat || fr.dog_jamming { g.looper.beat_pulse() } else { 0.0 };
    if low_w() as u32 != g.view_w || low_h() as u32 != g.view_h {
        g.view_w = low_w() as u32;
        g.view_h = low_h() as u32;
        g.full_view = make_view(g.view_w, g.view_h);
    }
    fr.half = (low_w() / 2.0).floor();
    set_camera(&Camera2D {
        render_target: Some(g.full_view.clone()),
        zoom: vec2(2.0 / g.view_w as f32, -2.0 / g.view_h as f32),
        target: vec2(g.view_w as f32 / 2.0, g.view_h as f32 / 2.0),
        ..Default::default()
    });
    clear_background(FOG);
    let view = View {
        world: &g.world,
        players: g.shown_players(),
        particles: &g.particles,
        camp: &g.camp,
        time: fr.time,
        dance: fr.dance,
        dusk: g.dusk,
        extras: &fr.extras,
    };
    fr.ghost = g.holding.map(|p| (p, ahead_local(&g.players[g.holding_by]) != home::CATALOGUE && g.house.can_place(&p, &[home::local(g.players[1 - g.holding_by].pos)])));
    if g.inside {
        let pov_for = |who: usize| if g.dj[who] { booth_pov(&g.house, if who == 0 { -1.0 } else { 1.0 }) } else { None };
        let beat_on = g.looper.beat || !g.now_playing.is_empty();
        let pov = if g.dj[0] ^ g.dj[1] { if g.dj[0] { pov_for(0) } else { pov_for(1) } } else { None };
        let hide = if g.dj[0] ^ g.dj[1] { if g.dj[0] { Some(0) } else { Some(1) } } else { None };
        render_home(&g.full_view, 0.0, low_w(), low_h(), &view, &g.house, fr.ghost, pov, hide, beat_on);
    } else if g.two_player {
        render_view(&g.full_view, 0.0, fr.half, low_h(), g.players[0].pos, g.players[0].cam, &view, true);
        render_view(&g.full_view, fr.half, fr.half, low_h(), g.players[1].pos, g.players[1].cam, &view, true);
    } else {
        render_view(&g.full_view, 0.0, low_w(), low_h(), g.players[0].pos, g.players[0].cam, &view, false);
    }
    if g.picking {
        render_pick_preview(&g.pick_view, &g.world, &g.cast_list, [g.players[0].character, g.players[1].character], g.pick_row, fr.time);
    }
    if g.shop_open && g.shop_tab == 0 {
        render_shop_preview(
            &g.catalogue_view,
            0.0,
            0.0,
            g.catalogue_view.texture.width(),
            g.catalogue_view.texture.height(),
            home::row(g.shop_row),
            &g.house,
            fr.time,
            g.looper.beat || !g.now_playing.is_empty(),
        );
    }

    set_default_camera();
    clear_background(BLACK);
    draw_texture_ex(
        &g.full_view.texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            flip_y: true,
            ..Default::default()
        },
    );
    if g.two_player && !g.inside {
        ui_rect(fr.half - 0.5, 0.0, 1.0, low_h(), Color::new(0.1, 0.08, 0.2, 0.9));
    }
}
