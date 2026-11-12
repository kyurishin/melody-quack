#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) struct Inventory {
    pub(crate) wood: u32,
    pub(crate) stone: u32,
    pub(crate) fish: [u32; 3],
    pub(crate) mush: [u32; 3],
    pub(crate) acorns: u32,
}

pub(crate) const ACORN_COLOR: Color = Color::new(0.78, 0.5, 0.26, 1.0);
