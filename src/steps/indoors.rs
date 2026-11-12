#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

/// Everything a player can do at home: furniture, shop, instruments.
pub(crate) fn indoors(g: &mut Game, fr: &mut Frame) {
    if g.inside && !g.picking {
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            if g.seat[who].is_some() && !g.players[who].seated {
                g.seat[who] = None;
            }
        }
        if let Some(piece) = g.holding.as_mut() {
            let ahead = ahead_local(&g.players[g.holding_by]);
            piece.x = ahead.0;
            piece.z = ahead.1;
        }
        let rows = home::shop_rows();
        let shop_left = if g.shop_owner == 0 { is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) } else { is_key_pressed(KeyCode::J) };
        let shop_right = if g.shop_owner == 0 { is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) } else { is_key_pressed(KeyCode::L) };
        let shop_up = if g.shop_owner == 0 { is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) } else { is_key_pressed(KeyCode::I) };
        let shop_down = if g.shop_owner == 0 { is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) } else { is_key_pressed(KeyCode::K) };
        let shop_act = if g.shop_owner == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
        let shop_turn = if g.shop_owner == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
        let ahead = ahead_local(&g.players[g.shop_owner]);
        let was_shop = g.shop_open;
        let was_dj = g.dj[0];
        if g.dj[0] && !was_shop {
            if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                g.looper.bpm = (g.looper.bpm + 2.0).min(170.0);
            }
            if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                g.looper.bpm = (g.looper.bpm - 2.0).max(80.0);
            }
            let n = BEAT_STYLES.len();
            let mut changed = false;
            if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                g.beat_style = (g.beat_style + n - 1) % n;
                changed = true;
            }
            if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                g.beat_style = (g.beat_style + 1) % n;
                changed = true;
            }
            if changed {
                g.looper.beat = BEAT_STYLES[g.beat_style].name != "off";
            }
            if is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Escape) {
                g.dj[0] = false;
            }
        } else if was_shop {
            if shop_up {
                g.shop_tab = g.shop_tab.saturating_sub(1);
            }
            if shop_down {
                g.shop_tab = (g.shop_tab + 1).min(3);
            }
            if g.shop_tab >= 2 {
                g.dex_mode = g.shop_tab - 2;
                g.shop_tab = 1;
                g.shop_open = false;
                g.dex_open = true;
                g.dex_from_shop = true;
                g.dex_query.clear();
                g.dex_sel = 0;
                while get_char_pressed().is_some() {}
            }
            if g.shop_tab == 0 {
                if shop_left {
                    g.shop_row = (g.shop_row + rows - 1) % rows;
                }
                if shop_right {
                    g.shop_row = (g.shop_row + 1) % rows;
                }
            } else if !g.diary.is_empty() {
                let n = g.diary.len();
                if shop_left {
                    g.diary_row = (g.diary_row + n - 1) % n;
                }
                if shop_right {
                    g.diary_row = (g.diary_row + 1) % n;
                }
                g.diary_row = g.diary_row.min(n - 1);
                if shop_act && !fr.picker_closed {
                    let idx = n - 1 - g.diary_row;
                    if !g.house.has(home::Item::RecordPlayer) {
                        g.toasts.push(("buy a record player to listen to your jams".to_string(), 2.5));
                    } else if g.diary[idx].events.is_empty() {
                        g.toasts.push(("this page has no recording left".to_string(), 2.0));
                    } else {
                        g.playback = Some((idx, get_time(), 0));
                        g.house.record_on = true;
                        g.shop_open = false;
                        g.toasts.push((format!("the record player plays your jam from {}", date_label(g.diary[idx].when)), 3.0));
                    }
                }
            }
            if shop_turn || is_key_pressed(KeyCode::Escape) {
                g.shop_open = false;
            }
            if shop_act && !fr.picker_closed && g.shop_tab == 0 {
                match home::row(g.shop_row) {
                    home::Row::Upgrade => match g.house.next_upgrade() {
                        None => g.toasts.push(("your home is fully built".to_string(), 2.0)),
                        Some((name, _, price)) => {
                            if g.inv.acorns < price {
                                g.toasts.push((format!("the {} costs {} acorns", name, price), 2.0));
                            } else {
                                g.inv.acorns -= price;
                                let mut way = g.house.in_the_way();
                                way.sort_unstable_by(|a, b| b.cmp(a));
                                let moved = way.len();
                                for i in way {
                                    g.stored.push(g.house.pieces.remove(i).item);
                                }
                                if moved > 0 {
                                    g.toasts.push((format!("{} piece{} in the way went into your bag (M)", moved, if moved == 1 { "" } else { "s" }), 4.0));
                                }
                                g.house.size += 1;
                                g.shop_open = false;
                                g.save_clock = 99.0;
                                for k in 0..6 {
                                    g.audio.play_ch(3, &g.bells[k], 0.25);
                                }
                                let msg = if g.house.size > home::LOFT_SIZE { "the penthouse grows - your loft upstairs is twice as deep now".to_string() } else if g.house.loft() { "penthouse! take the stairs at the back right up to your loft".to_string() } else { format!("the walls slide out - your room is now {} x {}", g.house.w(), g.house.d()) };
                                g.toasts.push((msg, 4.0));
                            }
                        }
                    },
                    home::Row::Item(item) => {
                        if g.inv.acorns >= item.price() {
                            g.inv.acorns -= item.price();
                            g.holding = Some(home::Piece { item, x: ahead.0, z: ahead.1, rot: 0, state: 0, day: 0 });
                            g.holding_by = g.shop_owner;
                            g.shop_open = false;
                            g.audio.play_ch(3, &g.plink, 0.6);
                            g.toasts.push((format!("bought a {} - {} to place it, {} to turn it", item.name(), if g.shop_owner == 0 { "E" } else { "O" }, if g.shop_owner == 0 { "X" } else { "U" }), 3.0));
                        } else {
                            g.toasts.push((format!("a {} costs {} acorns - you have {}", item.name(), item.price(), g.inv.acorns), 2.5));
                        }
                    }
                    home::Row::Wall(i) | home::Row::Floor(i) => {
                        let is_wall = matches!(home::row(g.shop_row), home::Row::Wall(_));
                        let bit = if is_wall { home::wall_bit(i) } else { home::floor_bit(i) };
                        let owned = g.house.owned_styles & bit != 0;
                        if owned || g.inv.acorns >= home::STYLE_PRICE {
                            if !owned {
                                g.inv.acorns -= home::STYLE_PRICE;
                                g.house.owned_styles |= bit;
                            }
                            if is_wall {
                                g.house.wall = i as u8;
                            } else {
                                g.house.floor = i as u8;
                            }
                            g.audio.play_ch(3, &g.bells[i % 6], 0.5);
                        } else {
                            g.toasts.push((format!("that costs {} acorns", home::STYLE_PRICE), 2.0));
                        }
                    }
                    home::Row::Set => {
                        let added = g.house.techno_club(&[home::local(g.players[0].pos), home::local(g.players[1].pos)]);
                        g.looper.beat = true;
                        g.looper.bpm = 132.0;
                        g.shop_open = false;
                        g.audio.play_ch(2, &g.kick, 0.9);
                        g.toasts.push((format!("techno club mode: concrete, red light, 132 bpm ({} pieces fit)", added), 3.5));
                    }
                }
            }
        }
        for who in 0..(if g.two_player { 2 } else { 1 }) {
            let act = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };
            let turn = if who == 0 { is_key_pressed(KeyCode::X) } else { is_key_pressed(KeyCode::U) };
            if was_shop || g.shop_open || (who == 0 && was_dj) {
                continue;
            }
            if g.dj[who] {
                if who == 1 && act {
                    g.dj[who] = false;
                }
                continue;
            }
            if g.ddr.on[who] || (fr.bag_busy && g.bag_owner == who) || (g.sampler.open && g.sampler.owner == who) {
                continue;
            }
            let ahead = ahead_local(&g.players[who]);
            let here = home::local(g.players[who].pos);
            let near_catalogue = here.distance(vec2(0.5, 0.5)) < 1.4;
            if act && !fr.picker_closed {
                if let Some(piece) = g.holding {
                    if g.holding_by == who {
                        if ahead == home::CATALOGUE {
                            let back = piece.item.price() / 2;
                            g.inv.acorns += back;
                            g.holding = None;
                            g.toasts.push((format!("sold the {} back for {} acorns", piece.item.name(), back), 2.5));
                        } else if g.house.can_place(&piece, &[home::local(g.players[1 - who].pos)]) {
                            g.house.pieces.push(piece);
                            g.holding = None;
                            g.audio.play_ch(3, &g.marimba[(piece.x + piece.z).rem_euclid(6) as usize], 0.7);
                            if piece.item == home::Item::Speaker {
                                g.toasts.push(("the speaker makes your echo bigger (G for echo)".to_string(), 3.0));
                            }
                        } else {
                            g.toasts.push(("it doesn't fit there".to_string(), 1.8));
                        }
                    }
                } else if g.seat[who].is_some() {
                    g.players[who].seated = false;
                    g.players[who].sleeping = false;
                    g.seat[who] = None;
                } else if near_catalogue {
                    g.shop_open = true;
                    g.shop_owner = who;
                    g.audio.play_ch(3, &g.plink, 0.4);
                } else if let Some(i) = g.house.piece_at(ahead.0, ahead.1) {
                    let item = g.house.pieces[i].item;
                    match item {
                        home::Item::Piano | home::Item::Drums => {
                            g.seat[who] = Some(item);
                            g.players[who].seated = true;
                            g.toasts.push((format!("playing the {} - {} to play, move to get up", item.name(), if who == 0 { "1-6" } else { "7-=" }), 3.0));
                        }
                        home::Item::DjBooth => {
                            g.dj[who] = true;
                            g.music_mode = true;
                            if !g.looper.beat {
                                g.looper.beat = true;
                                g.looper.bpm = 132.0;
                                g.beat_style = 0;
                            }
                        }
                        home::Item::RecordPlayer => {
                            g.playback = None;
                            g.house.record_on = !g.house.record_on;
                            g.toasts.push((if g.house.record_on { "the record spins".to_string() } else { "the record stops".to_string() }, 2.0));
                        }
                        home::Item::WindChime => {
                            for k in 0..3 {
                                g.audio.play_ch(3, &g.bells[(k * 2 + (fr.time as usize)) % 6], 0.3);
                            }
                        }
                        home::Item::Bed => {
                            let piece = g.house.pieces[i];
                            g.seat[who] = Some(item);
                            g.players[who].seated = true;
                            g.players[who].sleeping = true;
                            g.players[who].yaw = std::f32::consts::PI;
                            let front = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                            g.players[who].facing = vec2(front.x, front.z);
                            g.players[who].sleep_yaw = piece.yaw();
                            g.players[who].perch = 0.06;
                            let side = if who == 0 { -0.42 } else { 0.42 };
                            let offset = rotate_y(vec3(side, 0.0, 0.0), piece.yaw());
                            g.players[who].place_at(home::world(piece.center() + vec2(offset.x, offset.z)));
                            if who == 0 && !g.two_player {
                                let friend_side = rotate_y(vec3(0.42, 0.0, 0.0), piece.yaw());
                                let friend = &mut g.players[1];
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
                                g.hearts.push((vec3(g.players[who].pos.x + k as f32 * 0.15 - 0.15, 1.32 + k as f32 * 0.12, g.players[who].pos.y), 1.4));
                            }
                            g.sleep_fade = 3.0;
                            g.save_clock = 99.0;
                        }
                        home::Item::Sofa | home::Item::ConcreteBench => {
                            let piece = g.house.pieces[i];
                            let out = rotate_y(vec3(0.0, 0.0, 1.0), piece.yaw());
                            g.seat[who] = Some(item);
                            g.players[who].seated = true;
                            g.players[who].perch = if item == home::Item::Sofa { 0.2 } else { 0.24 };
                            let (w, _) = piece.item.size();
                            let side = if w > 1 { if who == 0 { -0.45 } else { 0.45 } } else { 0.0 };
                            let along = rotate_y(vec3(side, 0.0, 0.05), piece.yaw());
                            g.players[who].place_at(home::world(piece.center() + vec2(along.x, along.z)));
                            g.players[who].facing = vec2(out.x, out.z);
                            g.toasts.push((format!("sitting on the {} - move to get up", item.name()), 2.0));
                        }
                        home::Item::Lamp | home::Item::RedLight | home::Item::Speaker | home::Item::Subwoofer => {
                            let piece = &mut g.house.pieces[i];
                            piece.state = if piece.state == 0 { 1 } else { 0 };
                            let on = piece.state == 0;
                            g.audio.play_ch(3, &g.plink, 0.45);
                            let msg = match item {
                                home::Item::Speaker => if on { "speaker on - bigger echo" } else { "speaker off" },
                                home::Item::Subwoofer => if on { "subwoofer on - heavier kick" } else { "subwoofer off" },
                                _ => if on { "light on" } else { "light off" },
                            };
                            g.toasts.push((msg.to_string(), 1.5));
                        }
                        home::Item::Plant => {
                            let piece = &mut g.house.pieces[i];
                            let revived = piece.state == home::WITHERED;
                            if revived {
                                piece.state = 0;
                            }
                            let first_today = piece.day != sky().day as i32;
                            piece.day = sky().day as i32;
                            if !revived && piece.state < 3 && first_today {
                                piece.state += 1;
                            }
                            g.audio.play_ch(3, &g.bells[(piece.state as usize + 1) % 6], 0.4);
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
                            quest_event(&mut g.quests, QuestKind::Water, 1, &mut g.inv, &mut g.toasts);
                            g.toasts.push((msg.to_string(), 2.0));
                        }
                        home::Item::Bookshelf => {
                            g.audio.play_ch(3, &g.plink, 0.3);
                            g.shelf_open = true;
                            g.open_book = None;
                            g.page_turn = 0.0;
                        }
                        home::Item::Table => {
                            let piece = &mut g.house.pieces[i];
                            let mut known: Vec<usize> = g.dishes.clone();
                            known.sort();
                            if known.is_empty() {
                                g.toasts.push(("nothing to serve yet - cook by the fire first".to_string(), 2.5));
                            } else {
                                let current = piece.state as usize;
                                let next = known.iter().find(|d| **d + 1 > current).map(|d| *d + 1).unwrap_or(0);
                                piece.state = next as u8;
                                g.audio.play_ch(3, &g.plink, 0.4);
                                if next == 0 {
                                    g.toasts.push(("you clear the table".to_string(), 1.5));
                                } else {
                                    g.toasts.push((format!("you serve {} - a friend comes to eat", DISHES[next - 1]), 2.5));
                                    for k in 0..3 {
                                        g.hearts.push((vec3(g.players[1 - who].pos.x + k as f32 * 0.15 - 0.15, 1.0 + k as f32 * 0.12, g.players[1 - who].pos.y), 1.4));
                                    }
                                }
                            }
                        }
                        home::Item::SteelPillar => {
                            g.audio.play_ch(3, &g.bells[0], 0.6);
                            g.audio.play_ch(3, &g.bells[3], 0.25);
                        }
                        home::Item::MirrorBall => {
                            let piece = &mut g.house.pieces[i];
                            piece.state = if piece.state == 0 { 1 } else { 0 };
                            g.audio.play_ch(3, &g.bells[5], 0.35);
                            g.toasts.push((if piece.state == 0 { "the mirror ball spins".to_string() } else { "the mirror ball rests".to_string() }, 1.5));
                        }
                        home::Item::Tv => {
                            let piece = &mut g.house.pieces[i];
                            piece.state = (piece.state + 1) % 6;
                            g.audio.play_ch(3, &g.plink, 0.35);
                            let msg = if piece.state == 0 { "tv off".to_string() } else { format!("channel {}: {}", piece.state, TV_CHANNELS[piece.state as usize - 1]) };
                            g.toasts.push((msg, 1.6));
                        }
                        home::Item::Arcade => {
                            g.ddr_request = Some((who, i));
                        }
                        home::Item::Sampler => {
                            g.sampler.open = true;
                            g.sampler.owner = who;
                            g.audio.play_ch(3, &g.plink, 0.35);
                        }
                        home::Item::Rug => {
                            for k in 0..4 {
                                g.hearts.push((vec3(g.players[who].pos.x + k as f32 * 0.12 - 0.18, 0.9 + k as f32 * 0.1, g.players[who].pos.y), 1.2));
                            }
                            g.toasts.push(("a little dance on the rug".to_string(), 1.5));
                        }
                    }
                } else if here.y > g.house.d() as f32 - 1.1 && (here.x - g.house.door().x).abs() < 0.9 {
                    fr.go_out = true;
                }
            }
            if turn && !g.shop_open {
                if let Some(piece) = g.holding.as_mut() {
                    if g.holding_by == who {
                        piece.rot = (piece.rot + 1) % 4;
                    }
                } else if g.seat[who].is_some() {
                    g.players[who].seated = false;
                    g.players[who].sleeping = false;
                    g.seat[who] = None;
                } else if let Some(i) = g.house.piece_at(ahead.0, ahead.1) {
                    let piece = g.house.pieces.remove(i);
                    g.toasts.push((format!("moving the {} - {} to place, {} to turn, or sell it at the catalogue", piece.item.name(), if who == 0 { "E" } else { "O" }, if who == 0 { "X" } else { "U" }), 3.0));
                    g.holding = Some(piece);
                    g.holding_by = who;
                }
            }
        }
        if g.house.has(home::Item::WindChime) {
            g.chime_clock -= fr.dt;
            if g.chime_clock <= 0.0 {
                g.chime_clock = 2.5 + hash((fr.time * 13.0) as i32, 4, 4) * 4.0;
                g.audio.play_ch(3, &g.bells[(hash((fr.time * 7.0) as i32, 2, 9) * 6.0) as usize % 6], 0.16);
            }
        }
    }
}
