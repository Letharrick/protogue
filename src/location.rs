use bracket_lib::prelude::RandomNumberGenerator;
use components::position::Position;
use map::{Map, Object};

use components::barrier::Barrier;

use entities::floor::Floor;
use entities::wall::Wall;
use legion::systems::CommandBuffer;
use legion::*;
use std::sync::{Arc, RwLock};

pub struct Dungeon<'a> {
    world: &'a mut World,
    map: Arc<RwLock<Map>>,
    position: (i32, i32),
    dimensions: (i32, i32),
    floor: &'a Floor,
    wall: &'a Wall,
}

impl<'a> Dungeon<'a> {
    pub fn new(
        world: &'a mut World,
        map: Arc<RwLock<Map>>,
        position: (i32, i32),
        dimensions: (i32, i32),
        floor: &'a Floor,
        wall: &'a Wall,
    ) -> Self {
        Self {
            world,
            map,
            position,
            dimensions,
            floor,
            wall,
        }
    }

    /*
    pub fn generate_rooms(
        self,
        rooms: u32
    ) {
        for i in 0..rooms {
            self.rooms.push(
                Room::new(
                    self.world,
                    self.map.clone(),
                    self.position.into(),
                    self.dimensions, self.floor,
                    self.wall
                )
            );
        }
    }
     */

    pub fn generate_caves(self, enhancement_passes: u32) {
        let mut commands = CommandBuffer::new(self.world);
        let mut rng = RandomNumberGenerator::new();

        // Randomly spawn walls
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if rng.rand::<f32>() < 0.35 {
                    self.wall.spawn(&mut commands, self.map.clone(), (x, y));
                } else {
                    self.floor.spawn(&mut commands, self.map.clone(), (x, y));
                }
            }
        }

        commands.flush(self.world);

        // Repeatedly enhance tha map, making smoother caves and removing random walls
        for _i in 0..enhancement_passes {
            let mut commands = CommandBuffer::new(self.world);

            for (position, barrier) in <(&Position, Option<&Barrier>)>::query()
                .iter(self.world)
                .map(|(p, w)| (*p, w.cloned()))
                .collect::<Vec<_>>()
            {
                let wall_neighbours: Vec<Object> = self
                    .map
                    .read()
                    .unwrap()
                    .neighbours(position.vector)
                    .into_iter()
                    .fold(Vec::new(), |mut v: Vec<Object>, stack| {
                        v.extend(stack.iter().cloned());
                        v
                    })
                    .into_iter()
                    .filter(|obj| {
                        self.world
                            .entry_ref(obj.entity)
                            .unwrap()
                            .get_component::<Barrier>()
                            .is_ok()
                    })
                    .collect();

                let become_wall = !barrier.is_some() && wall_neighbours.len() > 4;
                let delete_wall = barrier.is_some() && (wall_neighbours.len() < 2);

                if become_wall || delete_wall {
                    self.world.remove(
                        self.map.write().unwrap()[position.vector]
                            .pop()
                            .unwrap()
                            .entity,
                    );

                    if become_wall {
                        self.wall
                            .spawn(&mut commands, self.map.clone(), position.vector);
                    } else {
                        self.floor
                            .spawn(&mut commands, self.map.clone(), position.vector);
                    }
                }

                commands.flush(self.world);
            }
        }
    }
}

pub struct Room {
    position: (i32, i32),
    dimensions: (i32, i32),
}

impl Room {
    fn new(
        world: &mut World,
        map: Arc<RwLock<Map>>,
        position: (i32, i32),
        dimensions: (i32, i32),
        floor: &Floor,
        wall: &Wall,
    ) -> Self {
        let mut commands = CommandBuffer::new(world);

        // Flooring
        for y in (position.1 + 1)..dimensions.1 {
            for x in (position.0 + 1)..dimensions.0 {
                floor.spawn(&mut commands, map.clone(), (x, y));
            }
        }

        // Top and bottom walls
        for x in (position.0 + 1)..dimensions.0 {
            wall.spawn(&mut commands, map.clone(), (x, position.1));
            wall.spawn(&mut commands, map.clone(), (x, dimensions.1));
        }

        // Left and right walls
        for y in position.1..dimensions.1 {
            wall.spawn(&mut commands, map.clone(), (position.0, y));
            wall.spawn(&mut commands, map.clone(), (dimensions.0, y));
        }

        commands.flush(world);

        Self {
            position,
            dimensions,
        }
    }
}
