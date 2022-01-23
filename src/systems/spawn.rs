use components::position::Position;

use bracket_lib::prelude::RandomNumberGenerator;
use map::Map;
use std::sync::{Arc, RwLock};

use components::spawn::{Spawn, SpawnType};
use legion::systems::CommandBuffer;
use legion::*;
use {CREATURES, FLOORS, ITEMS, WALLS};

#[system(for_each)]
pub fn spawn(
    commands: &mut CommandBuffer,
    #[resource] map: &mut Arc<RwLock<Map>>,
    #[resource] rng: &mut RandomNumberGenerator,
    entity: &Entity,
    position: &Position,
    spawn: &Spawn,
) {
    let choice = rng
        .random_slice_entry(&spawn.choices)
        .expect("No objects to spawn");

    if rng.rand::<f32>() < choice.probability {
        match choice.ty {
            SpawnType::Floor => &FLOORS[&choice.name].spawn(commands, map.clone(), position.vector),
            SpawnType::Wall => &WALLS[&choice.name].spawn(commands, map.clone(), position.vector),
            SpawnType::Item => &ITEMS[&choice.name].spawn(commands, map.clone(), position.vector),
            SpawnType::Creature => {
                &CREATURES[&choice.name].spawn(commands, map.clone(), position.vector)
            }
        };
    }

    commands.remove_component::<Spawn>(*entity);
}
