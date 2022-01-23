use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum SpawnType {
    Floor,
    Wall,
    Item,
    Creature,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpawnDescription {
    pub ty: SpawnType,
    pub name: String,
    pub probability: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Spawn {
    pub choices: Vec<SpawnDescription>,
}
