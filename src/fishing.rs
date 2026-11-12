//! Fishing on the beat.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const DOCK_END: Vec2 = Vec2::new(15.5, 18.5);

pub(crate) const BOBBER: Vec3 = Vec3::new(16.75, 0.31, 18.5);

pub(crate) struct Fishing {
    pub(crate) active: bool,
    pub(crate) caster: usize,
    pub(crate) cue_at: f32,
    pub(crate) hits: u32,
    pub(crate) tries: u32,
    pub(crate) hit_by: [bool; 2],
    pub(crate) early: [bool; 2],
    pub(crate) lesson: bool,
    pub(crate) perfects: u32,
    pub(crate) dip: f32,
    pub(crate) jump: Option<(usize, f32)>,
    pub(crate) tight: f32,
}

pub(crate) const FISH_TRIES: u32 = 3;

pub(crate) const FISH_CUE_GAP: f32 = 8.0;
