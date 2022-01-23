use serde::Deserialize;
use std::sync::{Arc, RwLock};

use components::description::*;
use components::effects::*;
use components::glyph::*;
use components::position::*;
use components::weight::*;

use map::{Map, Object};
use vector::Vector;

use components::light::Light;
use legion::systems::CommandBuffer;
use object_derive::ObjectBase;

#[derive(ObjectBase, Clone, Debug, Deserialize)]
pub struct Item {
    pub glyph: Glyph,
    pub description: Description,
    pub weight: Weight,
    pub effects: Option<Effects>,
    pub light: Option<Light>,
}
