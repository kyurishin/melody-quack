//! Building tools.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tool {
    WoodWall,
    StoneWall,
    Floor,
    Remove,
}

impl Tool {
    pub(crate) fn next(self) -> Tool {
        match self {
            Tool::WoodWall => Tool::StoneWall,
            Tool::StoneWall => Tool::Floor,
            Tool::Floor => Tool::Remove,
            Tool::Remove => Tool::WoodWall,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Tool::WoodWall => "wood wall",
            Tool::StoneWall => "stone wall",
            Tool::Floor => "floor",
            Tool::Remove => "remove",
        }
    }
}

pub(crate) fn buildable(world: &World, x: i32, z: i32) -> bool {
    if in_forest(x, z) || x <= 0 || z <= 0 || x >= MAP as i32 - 1 || z >= MAP as i32 - 1 || SEAT_TILES.contains(&(x, z)) {
        return false;
    }
    world
        .get(x, z)
        .map(|t| matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash | Kind::Floor | Kind::WoodWall | Kind::StoneWall) && t.fire <= 0.0)
        .unwrap_or(false)
}

pub(crate) fn build(world: &mut World, inv: &mut Inventory, tool: Tool, x: i32, z: i32, blocked: bool) -> Result<Option<(Kind, u8)>, &'static str> {
    if !buildable(world, x, z) {
        return Ok(None);
    }
    let t = world.get_mut(x, z).unwrap();
    match tool {
        Tool::WoodWall | Tool::StoneWall => {
            let (kind, stock) = if tool == Tool::WoodWall { (Kind::WoodWall, &mut inv.wood) } else { (Kind::StoneWall, &mut inv.stone) };
            if *stock == 0 {
                return Err(if tool == Tool::WoodWall { "need wood - your friend can fetch some" } else { "need stone - your friend can fetch some" });
            }
            if t.kind == kind && t.stack < 3 {
                t.stack += 1;
            } else if matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash | Kind::Floor) && !blocked {
                t.kind = kind;
                t.stack = 1;
            } else {
                return Ok(None);
            }
            *stock -= 1;
            Ok(Some((kind, t.stack)))
        }
        Tool::Floor => {
            if inv.wood == 0 {
                return Err("need wood - your friend can fetch some");
            }
            if matches!(t.kind, Kind::Grass | Kind::Flowers | Kind::Ash) {
                t.kind = Kind::Floor;
                inv.wood -= 1;
                Ok(Some((Kind::Floor, 0)))
            } else {
                Ok(None)
            }
        }
        Tool::Remove => match t.kind {
            Kind::WoodWall | Kind::StoneWall => {
                let kind = t.kind;
                if kind == Kind::WoodWall {
                    inv.wood += 1;
                } else {
                    inv.stone += 1;
                }
                t.stack -= 1;
                if t.stack == 0 {
                    t.kind = Kind::Grass;
                }
                Ok(Some((kind, t.stack)))
            }
            Kind::Floor => {
                t.kind = Kind::Grass;
                inv.wood += 1;
                Ok(Some((Kind::Floor, 0)))
            }
            _ => Ok(None),
        },
    }
}
