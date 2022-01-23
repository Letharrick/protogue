use serde::Deserialize;
use std::sync::{Arc, RwLock};

use components::glyph::*;
use components::position::*;

use components::barrier::*;
use components::opaque::*;

use map::{Map, Object};
use vector::Vector;

use components::description::Description;
use components::light::Light;
use legion::systems::CommandBuffer;
use object_derive::ObjectBase;

#[derive(ObjectBase, Clone, Debug, Deserialize)]
pub struct Wall {
    pub barrier: Barrier,
    pub glyph: Glyph,
    pub description: Description,
    pub opaque: Option<Opaque>,
    pub light: Option<Light>,
}
