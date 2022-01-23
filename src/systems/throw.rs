use components::actions::throw::Throw;
use components::description::Description;
use components::equipment::Equipment;

use components::position::Position;
use components::velocity::Velocity;
use components::weight::Weight;

use interface::List;

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use map::{Map, Object};
use std::sync::{Arc, RwLock};

#[system(for_each)]
#[read_component(Weight)]
#[read_component(Description)]
pub fn throw(
    commands: &mut CommandBuffer,
    subworld: &mut SubWorld,
    #[resource] map: &Arc<RwLock<Map>>,
    #[resource] log: &Arc<RwLock<List>>,
    entity: &Entity,
    position: &Position,
    throw: &Throw,
    equipment: &mut Equipment,
) {
    let (mut left, _right) = subworld.split::<(&Weight, &Description)>();
    let _log = log.write().unwrap();
    let mut map = map.write().unwrap();

    if let Some(held_entity) = equipment.held {
        let new_position = position.vector;
        let held_entry = left.entry_mut(held_entity).unwrap();
        let _held_description = held_entry.get_component::<Description>().unwrap();

        commands.add_component(
            held_entity,
            Position {
                vector: new_position,
            },
        );
        map[new_position].push(Object::from_entity(held_entity, &left));

        commands.add_component(
            held_entity,
            Velocity {
                direction: throw.direction,
                magnitude: 1,
            },
        );

        equipment.held = None;
    }

    commands.remove_component::<Throw>(*entity);
}
