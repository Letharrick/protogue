use components::position::Position;

use components::actions::grab::Grab;
use components::description::Description;
use components::equipment::Equipment;
use components::glyph::Glyph;
use components::weight::Weight;

use interface::Label;
use interface::List;
use label;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use map::{Map, Object};
use std::sync::{Arc, RwLock};

#[system(for_each)]
#[read_component(Position)]
#[read_component(Weight)]
#[read_component(Glyph)]
#[read_component(Description)]
pub fn grab(
    commands: &mut CommandBuffer,
    subworld: &mut SubWorld,
    #[resource] map: &Arc<RwLock<Map>>,
    #[resource] log: &Arc<RwLock<List>>,
    entity: &Entity,
    grab: &Grab,
    equipment: &mut Equipment,
) {
    let (left, _right) = subworld.split::<(&Position, &Weight, &Glyph, &Description)>();
    let mut log = log.write().unwrap();
    let mut map = map.write().unwrap();

    let entry = left.entry_ref(*entity).unwrap();
    let position = entry.get_component::<Position>().unwrap();
    let grab_entry = left.entry_ref(grab.entity).unwrap();
    let grab_glyph = grab_entry.get_component::<Glyph>().unwrap();
    let grab_position = grab_entry.get_component::<Position>().unwrap();
    let grab_description = grab_entry.get_component::<Description>().unwrap();

    let mut label = label!["You "];

    match grab_entry.get_component::<Weight>() {
        Ok(_grab_item) => {
            // Remove item from map
            let obj_stack = &mut map[grab_position.vector];
            let entity_index = obj_stack
                .iter()
                .position(|obj| obj.entity == grab.entity)
                .unwrap();
            obj_stack.remove(entity_index);
            commands.remove_component::<Position>(grab.entity);

            if let Some(held_entity) = equipment.held {
                let held_entry = left.entry_ref(held_entity).unwrap();
                let held_glyph = held_entry.get_component::<Glyph>().unwrap();
                let held_description = held_entry.get_component::<Description>().unwrap();

                // Place held item on map
                map[position.vector].insert(1, Object::new(held_entity, false, false));
                commands.add_component::<Position>(held_entity, *position);

                // Log putting down currently held item
                label += label![
                    "put down the ",
                    (held_description.name.as_str(), held_glyph.colour),
                    " and "
                ];
            }

            equipment.held = Some(grab.entity);

            label += label![
                "pick up the ",
                (grab_description.name.as_str(), grab_glyph.colour)
            ];
            log.add(label);
        }

        Err(_) => {
            if equipment.held.is_none() {
                log.add(
                    label
                        + label![
                            "touch the ",
                            (grab_description.name.as_str(), grab_glyph.colour)
                        ],
                );
            } else {
                log.add(label!["Your hands are full"]);
            }
        }
    }

    commands.remove_component::<Grab>(*entity);
}
