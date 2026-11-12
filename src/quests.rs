//! Daily quests.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum QuestKind {
    Fish,
    Mushroom,
    Cook,
    Water,
    Sticks,
    Shake,
    Sing,
    Dance,
    Star,
    Emoji,
}

pub(crate) const QUEST_KINDS: [QuestKind; 10] = [QuestKind::Fish, QuestKind::Mushroom, QuestKind::Cook, QuestKind::Water, QuestKind::Sticks, QuestKind::Shake, QuestKind::Sing, QuestKind::Dance, QuestKind::Star, QuestKind::Emoji];

#[derive(Clone, Copy)]
pub(crate) struct Quest {
    pub(crate) kind: QuestKind,
    pub(crate) target: u32,
    pub(crate) progress: u32,
    pub(crate) done: bool,
}

impl Quest {
    pub(crate) fn reward(&self) -> u32 {
        match self.kind {
            QuestKind::Fish => 12 * self.target,
            QuestKind::Mushroom => 5 * self.target,
            QuestKind::Cook => 15 * self.target,
            QuestKind::Water => 10,
            QuestKind::Sticks => 6 * self.target,
            QuestKind::Shake => 4 * self.target,
            QuestKind::Sing => 10,
            QuestKind::Dance => 20,
            QuestKind::Star => 20,
            QuestKind::Emoji => 3 * self.target,
        }
    }

    pub(crate) fn label(&self) -> String {
        let n = self.target;
        let s = if n == 1 { "" } else { "s" };
        match self.kind {
            QuestKind::Fish => format!("catch {n} fish"),
            QuestKind::Mushroom => format!("pick {n} mushroom{s}"),
            QuestKind::Cook => format!("cook {n} dish{}", if n == 1 { "" } else { "es" }),
            QuestKind::Water => "water a plant".to_string(),
            QuestKind::Sticks => format!("pick up {n} stick{s}"),
            QuestKind::Shake => format!("shake acorns from {n} tree{s}"),
            QuestKind::Sing => format!("sing {n} notes"),
            QuestKind::Dance => format!("land {n} steps on the dance arcade"),
            QuestKind::Star => "earn a star playing together".to_string(),
            QuestKind::Emoji => format!("send {n} emoji{s}"),
        }
    }
}

pub(crate) struct Quests {
    pub(crate) day: i64,
    pub(crate) list: Vec<Quest>,
}

pub(crate) fn daily_quests(day: i64, has_plant: bool, has_arcade: bool) -> Vec<Quest> {
    let mut out: Vec<Quest> = Vec::new();
    let mut salt = 0;
    while out.len() < 3 && salt < 60 {
        let kind = QUEST_KINDS[(hash(day as i32, salt, 501) * QUEST_KINDS.len() as f32) as usize % QUEST_KINDS.len()];
        salt += 1;
        if out.iter().any(|q| q.kind == kind) || (kind == QuestKind::Water && !has_plant) || (kind == QuestKind::Dance && !has_arcade) {
            continue;
        }
        let roll = hash(day as i32, salt, 502);
        let target = match kind {
            QuestKind::Fish => 1 + (roll * 3.0) as u32,
            QuestKind::Mushroom => 2 + (roll * 4.0) as u32,
            QuestKind::Cook => 1 + (roll * 2.0) as u32,
            QuestKind::Sticks => 1 + (roll * 3.0) as u32,
            QuestKind::Shake => 2 + (roll * 3.0) as u32,
            QuestKind::Sing => 20 + (roll * 4.0) as u32 * 10,
            QuestKind::Dance => 20 + (roll * 3.0) as u32 * 10,
            QuestKind::Emoji => 2 + (roll * 3.0) as u32,
            _ => 1,
        };
        out.push(Quest { kind, target, progress: 0, done: false });
    }
    out
}

pub(crate) fn quest_event(quests: &mut Quests, kind: QuestKind, amount: u32, inv: &mut Inventory, toasts: &mut Vec<(String, f32)>) {
    for q in quests.list.iter_mut() {
        if q.kind == kind && !q.done {
            q.progress = (q.progress + amount).min(q.target);
            if q.progress >= q.target {
                q.done = true;
                inv.acorns += q.reward();
                toasts.push((format!("daily quest done: {} - +{} acorns", q.label(), q.reward()), 3.5));
            }
        }
    }
}
