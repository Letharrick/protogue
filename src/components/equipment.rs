use legion::Entity;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Equipment {
    pub held: Option<Entity>,
    pub storage: Option<Entity>,
    pub wearables: Vec<Entity>,
}
