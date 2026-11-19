#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) fn autosave(g: &mut Game, fr: &mut Frame) {
    g.save_clock += get_frame_time();
    fr.quitting = is_quit_requested();
    if g.restart_now {
        store::write("melody-quack-save 1\nfresh 1\n");
        return;
    }
    if (g.save_clock > 5.0 || fr.quitting) && !g.first_pick {
        g.save_clock = 0.0;
        let progress = Progress {
            tiles: g.world.tiles.iter().map(|t| (kind_index(t.kind), t.stack)).collect(),
            wood: g.inv.wood,
            stone: g.inv.stone,
            fish: g.inv.fish,
            fuel: g.camp.fuel,
            lit: g.camp.lit,
            pile: g.camp.pile,
            ring: g.camp.ring,
            dishes: g.dishes.clone(),
            cook_progress: g.cook_progress,
            characters: [g.players[0].character, g.players[1].character],
            spots: if g.inside { [DOOR_OUT + vec2(0.0, 0.25), DOOR_OUT + vec2(1.0, 0.4)] } else { [g.players[0].pos, g.players[1].pos] },
            echo: g.ringing,
            beat: g.looper.beat,
            bpm: g.looper.bpm,
            quantise: [Quantise::Off, Quantise::Soft, Quantise::Tight].iter().position(|q| *q == g.looper.quantise).unwrap_or(1) as u8,
            notes: g.looper.notes.clone(),
            songbook: g.songbook.clone(),
            partner_xp: g.partner_xp,
            mix: g.mix,
            acorns: g.inv.acorns + g.holding.map_or(0, |h| h.item.price() / 2),
            pieces: steps::visits::own_house(g).pieces.clone(),
            styles: { let h = steps::visits::own_house(g); (h.wall, h.floor, h.owned_styles, h.record_on) },
            diary: g.diary.clone(),
            fish_lesson: g.fish_lesson,
            cook_lesson: g.cook_lesson,
            gather_lesson: g.gather_lesson,
            room: steps::visits::own_house(g).size,
            home_code: g.venue.visits.code.clone(),
            friends: steps::chat::friends_text(g),
            gfx: (crate::glow::enabled(), crate::render::pixel_look()),
            dex: g.dex_counts.clone(),
            mush: g.inv.mush,
            shroomdex: g.shroom_counts.clone(),
            stored: g.stored.iter().map(|i| i.index()).collect(),
            name: g.venue.name.clone(),
            dj_key: g.venue.dj_key.clone(),
            welcome: false,
            tour: if g.venue.tour >= 6 { 6 } else { 0 },
            fresh: false,
            session: g.venue.session.clone(),
            companion_off: !g.venue.visits.away.as_ref().map(|a| a.companion).unwrap_or(g.companion),
            seq: g.sampler.grid.map(|row| row.iter().enumerate().fold(0u16, |m, (k, on)| if *on { m | (1 << k) } else { m })),
            weekly: if g.quests.week == i64::MIN { None } else { Some((g.quests.week, g.quests.weekly.iter().map(|q| (q.kind as u8, q.target, q.progress, q.done)).collect())) },
            quest_marks: (g.quests.bonus_day, g.quests.event_day),
            quests: if g.quests.day == i64::MIN { None } else { Some((g.quests.day, g.quests.list.iter().map(|q| (q.kind as u8, q.target, q.progress, q.done)).collect())) },
        };
        let text = encode_progress(&progress);
        store::write(&text);
        if g.venue.account.is_some() && net::state() == 2 && (g.venue.cloud_now || g.venue.cloud_clock > 30.0) && g.venue.cloud_offer.is_none() {
            let shared: String = text.lines().filter(|l| !l.starts_with("account ")).map(|l| format!("{l}\n")).collect();
            net::send(&format!("cloudput {}", venues::b64_encode(shared.as_bytes())));
            g.venue.cloud_now = false;
            g.venue.cloud_clock = 0.0;
        }
    }
}
