use components::barrier::Barrier;
use components::collision::Collision;
use components::displacement::Displacement;
use components::position::Position;
use components::velocity::Velocity;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use map::Map;
use std::sync::{Arc, RwLock};

#[system(for_each)]
#[read_component(Barrier)]
pub fn displacement(
    commands: &mut CommandBuffer,
    subworld: &mut SubWorld,
    entity: &Entity,
    position: &mut Position,
    displacement: &mut Displacement,
    _collision: Option<&Collision>,
    #[resource] map: &Arc<RwLock<Map>>,
) {
    let mut map = map.write().unwrap();
    let (mut left, _right) = subworld.split::<&Barrier>();

    if displacement.path.is_empty() {
        commands.remove_component::<Displacement>(*entity);
    } else {
        let new_position = position.vector + displacement.path.pop_front().unwrap();
        let objects_at_new_pos = &map[new_position];

        let top_obj_entry = left.entry_mut(objects_at_new_pos[0].entity).unwrap();

        match top_obj_entry.get_component::<Barrier>() {
            Ok(_barrier) => {
                commands.remove_component::<Displacement>(*entity);
                commands.remove_component::<Velocity>(*entity);
            }
            Err(_) => map.move_object_by_entity(*entity, position, new_position),
        }
    }
}
