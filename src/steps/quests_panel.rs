#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;
use steps::venues::{button, hit};

pub(crate) fn open(g: &mut Game) {
    g.venue.overlay = venues::Overlay::Quests;
    g.venue.opened_at = get_time();
}

fn gift(x: f32, y: f32, s: f32, open: bool) {
    let box_c = Color::from_rgba(255, 214, 110, 255);
    let ribbon = ui::skin::PINK;
    ui_round(x, y + s * 0.35, s, s * 0.65, 2.0, box_c);
    ui_rect(x - 1.0, y + s * 0.25, s + 2.0, s * 0.22, Color::from_rgba(255, 190, 80, 255));
    ui_rect(x + s * 0.42, y + s * 0.25, s * 0.16, s * 0.75, ribbon);
    if !open {
        ui_circle(x + s * 0.32, y + s * 0.18, s * 0.14, ribbon);
        ui_circle(x + s * 0.68, y + s * 0.18, s * 0.14, ribbon);
    } else {
        ui_rect(x + s * 0.2, y - s * 0.15, s * 0.16, s * 0.3, ui::skin::GOLD);
        ui_rect(x + s * 0.64, y - s * 0.2, s * 0.16, s * 0.3, ui::skin::GOLD);
    }
}

fn check(x: f32, y: f32, done: bool) {
    if done {
        ui_round(x, y, 11.0, 11.0, 2.5, Color::from_rgba(255, 214, 110, 255));
        for k in 0..3 {
            ui_rect(x + 2.0 + k as f32, y + 5.0 + k as f32, 1.4, 1.4, ui::skin::INK);
        }
        for k in 0..5 {
            ui_rect(x + 5.0 + k as f32, y + 7.0 - k as f32, 1.4, 1.4, ui::skin::INK);
        }
    } else {
        ui_round_frame(x, y, 11.0, 11.0, 2.5, Color::new(0.0, 0.0, 0.0, 0.15), ui::skin::PINK_EDGE);
    }
}

fn row(x: f32, y: f32, w: f32, label: &str, reward: u32, done: bool, on: bool) {
    ui_round(x, y, w, 17.0, 4.0, if on { Color::new(1.0, 1.0, 1.0, 0.1) } else { Color::new(1.0, 1.0, 1.0, 0.04) });
    check(x + 5.0, y + 3.0, done);
    let reward_s = format!("+{reward}");
    let rw = ui_measure_as(ui::Face::Data, &reward_s, 9.0);
    ui_text_fit(label, x + 21.0, y + 12.0, 8.0, w - 46.0 - rw, if done { ui::skin::MUTED } else { ui::skin::CREAM });
    ui_acorn(x + w - rw - 14.0, y + 8.5, 1.0);
    ui_text_as(ui::Face::Data, &reward_s, x + w - rw - 6.0, y + 12.5, 9.0, ui::skin::GOLD);
}

pub(crate) fn panel(g: &mut Game, fr: &mut Frame, fresh: bool) {
    let w = 250f32.min(low_w() - 12.0);
    let h = 172f32.min(low_h() - 50.0);
    let x = ((low_w() - w) / 2.0).round();
    let y = (44.0f32).max(((low_h() - h) / 2.0).round()).min(low_h() - h - 4.0);
    ui_round(x + 2.0, y + 3.0, w, h, 9.0, Color::new(0.05, 0.03, 0.12, 0.5));
    ui_round_frame(x, y, w, h, 9.0, ui::skin::PANEL_SOLID, ui::skin::EDGE_STRONG);
    ui_round(x + 8.0, y + 6.0, 13.0, 12.0, 3.0, ui::skin::PINK);
    ui_rect(x + 11.0, y + 9.0, 7.0, 1.2, ui::skin::INK);
    ui_rect(x + 11.0, y + 12.0, 7.0, 1.2, ui::skin::INK);
    ui_rect(x + 11.0, y + 15.0, 5.0, 1.2, ui::skin::INK);
    ui_text_as(ui::Face::Display, "quests", x + 26.0, y + 17.0, 11.0, ui::skin::GOLD);
    let close = button(fr, "x", x + w - 21.0, y + 5.0, 15.0, 14.0, false) && !fresh;
    let tabs = ["daily", "weekly", "event"];
    let tw = (w - 24.0) / 3.0;
    for (i, t) in tabs.iter().enumerate() {
        if button(fr, t, x + 8.0 + i as f32 * (tw + 4.0), y + 26.0, tw, 15.0, g.venue.quest_tab == i) && !fresh {
            g.venue.quest_tab = i;
            fr.click = false;
        }
    }
    if !fresh && (is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) {
        g.venue.quest_tab = (g.venue.quest_tab + 1) % 3;
    }
    if !fresh && (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) {
        g.venue.quest_tab = (g.venue.quest_tab + 2) % 3;
    }
    let ly = y + 48.0;
    let lw = w - 16.0;
    match g.venue.quest_tab {
        0 => {
            let list = g.quests.list.clone();
            ui_round(x + 6.0, ly - 3.0, w - 12.0, 3.0 * 20.0 + 4.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.12));
            for (i, q) in list.iter().enumerate() {
                let label = format!("{} ({}/{})", q.label(), q.progress, q.target);
                row(x + 8.0, ly + i as f32 * 20.0, lw, &label, q.reward(), q.done, i == 0 && !q.done);
            }
            if list.is_empty() {
                ui_text("new quests arrive every day", x + 12.0, ly + 12.0, 8.0, ui::skin::MUTED);
            }
            let done = list.iter().filter(|q| q.done).count();
            let total = list.len().max(1);
            let by = ly + 3.0 * 20.0 + 10.0;
            let bh = 34.0;
            ui_round_frame(x + 6.0, by, w - 12.0, bh, 7.0, Color::new(1.0, 1.0, 1.0, 0.06), ui::skin::EDGE_STRONG);
            let claimed = g.quests.bonus_day == g.quests.day;
            gift(x + 14.0, by + 8.0, 18.0, claimed);
            ui_text_as(ui::Face::Label, "daily bonus", x + 40.0, by + 14.0, 9.0, ui::skin::CREAM);
            let count = format!("{done}/{total}");
            ui_text_as(ui::Face::Data, &count, x + w - 60.0 - ui_measure_as(ui::Face::Data, &count, 9.0), by + 14.0, 9.0, ui::skin::CREAM);
            let bar_w = w - 112.0;
            ui_round(x + 40.0, by + 20.0, bar_w, 5.0, 2.5, Color::new(1.0, 1.0, 1.0, 0.12));
            ui_round(x + 40.0, by + 20.0, (bar_w * done as f32 / total as f32).max(5.0), 5.0, 2.5, ui::skin::PINK);
            let ready = done >= total && !list.is_empty() && !claimed;
            if ready {
                if button(fr, &format!("+{DAILY_BONUS}"), x + w - 50.0, by + 9.0, 38.0, 16.0, true) && !fresh {
                    g.quests.bonus_day = g.quests.day;
                    g.inv.acorns += DAILY_BONUS;
                    g.toasts.push((format!("daily bonus: +{DAILY_BONUS} acorns"), 3.0));
                    g.audio.play_ch(3, &g.bells[4], 0.4);
                    g.save_clock = 99.0;
                    fr.click = false;
                }
            } else {
                gift(x + w - 34.0, by + 8.0, 18.0, claimed);
            }
            let note = if claimed { "bonus collected - see you tomorrow" } else if ready { "all done! open your gift" } else { "finish all three for a bonus gift" };
            ui_text(note, x + 12.0, by + bh + 11.0, 7.0, ui::skin::MUTED);
        }
        1 => {
            let list = g.quests.weekly.clone();
            ui_round(x + 6.0, ly - 3.0, w - 12.0, 3.0 * 20.0 + 4.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.12));
            for (i, q) in list.iter().enumerate() {
                let label = format!("{} ({}/{})", q.label(), q.progress, q.target);
                row(x + 8.0, ly + i as f32 * 20.0, lw, &label, q.reward(), q.done, false);
            }
            let left = 7 - sky().day.rem_euclid(7);
            ui_text(&format!("new weekly quests in {left} day{}", if left == 1 { "" } else { "s" }), x + 12.0, ly + 3.0 * 20.0 + 16.0, 8.0, ui::skin::MUTED);
            ui_text("weekly quests pay double", x + 12.0, ly + 3.0 * 20.0 + 28.0, 7.0, ui::skin::MUTED);
        }
        _ => {
            let done = g.quests.event_day == sky().day;
            ui_round(x + 6.0, ly - 3.0, w - 12.0, 24.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.12));
            row(x + 8.0, ly, lw, "get into tonight's secret rave", EVENT_REWARD, done, !done);
            let line = match steps::venues::doors_in(g) {
                Some(ms) if ms > 0.0 => format!("doors open in {} at st. bassilica", steps::venues::clock_text(ms)),
                Some(_) => "the doors are open now. find the code at the pc bang".to_string(),
                None => "go online to see when the next rave is".to_string(),
            };
            ui_text_fit(&line, x + 12.0, ly + 36.0, 8.0, w - 24.0, ui::skin::CREAM);
            for (i, l) in ui_wrap("find tonight's code at the pc bang terminal and give it to the goose at the chapel door. one event quest a day.", 7.0, w - 24.0).iter().enumerate() {
                ui_text(l, x + 12.0, ly + 50.0 + i as f32 * 9.0, 7.0, ui::skin::MUTED);
            }
        }
    }
    if fresh {
        return;
    }
    if close || is_key_pressed(KeyCode::Escape) || (fr.click && !hit(fr, x, y, w, h)) {
        g.venue.overlay = venues::Overlay::None;
        fr.click = false;
    }
}
