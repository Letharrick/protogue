use components::displacement::Displacement;
use components::position::Position;
use components::velocity::Velocity;
use legion::systems::CommandBuffer;
use legion::Entity;
use legion::*;
use std::collections::LinkedList;

#[system(for_each)]
pub fn velocity(
    commands: &mut CommandBuffer,
    entity: &Entity,
    _position: &Position,
    velocity: &Velocity,
) {
    let mut path = LinkedList::new();
    path.push_back(velocity.direction.as_unit_vector() * velocity.magnitude);

    commands.add_component(*entity, Displacement { path })
}
