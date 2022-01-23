use components::description::*;
use components::glyph::*;
use components::position::*;
use components::spawn::*;
use components::tile::*;
use legion::systems::CommandBuffer;

use map::{Map, Object};
use object_derive::ObjectBase;
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use vector::Vector;

#[derive(ObjectBase, Clone, Debug, Deserialize)]
pub struct Floor {
    pub tile: Tile,
    pub glyph: Glyph,
    pub description: Description,
    pub spawn: Option<Spawn>,
}
