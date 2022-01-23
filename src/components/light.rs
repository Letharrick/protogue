use components::glyph::Colour;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Copy, Clone)]
pub struct Light {
    pub colour: Colour,
    pub intensity: f32,
    pub radius: i32,
}
