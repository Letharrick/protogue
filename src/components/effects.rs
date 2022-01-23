use components::glyph::Colour;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Effect {
    Stain { colour: Colour },
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Effects {
    pub touch: Vec<Effect>,
    pub consume: Vec<Effect>,
}
