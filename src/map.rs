use bracket_lib::prelude::*;

use components::position::Position;

use vector::Vector;

use components::barrier::Barrier;
use components::direction::Direction;
use components::opaque::Opaque;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use entities::floor::Floor;
use entities::wall::Wall;
use std::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};

// A generic object on the map (used for computing FOV and collision detection)
#[derive(Debug, Clone)]
pub struct Object {
    pub entity: Entity,
    pub block_fov: bool,
    pub block_movement: bool,
}

impl Object {
    pub fn new(entity: Entity, block_fov: bool, block_movement: bool) -> Self {
        Self {
            entity,
            block_fov,
            block_movement,
        }
    }

    pub fn from_entity(entity: Entity, subworld: &SubWorld) -> Self {
        let entry = subworld.entry_ref(entity).unwrap();

        Self {
            entity,
            block_fov: entry.get_component::<Opaque>().is_ok(),
            block_movement: entry.get_component::<Barrier>().is_ok(),
        }
    }
}

type ObjectStack = Vec<Object>;
pub type Map = MapBase<ObjectStack>;

#[derive(Debug)]
pub struct MapBase<C> {
    stacks: Vec<C>,
    size: Vector<i32>,
}

impl<C: Clone + Default + AsRef<ObjectStack>> MapBase<C> {
    pub fn new(size: Vector<i32>) -> Self {
        Self {
            stacks: vec![C::default(); (size.0 * size.1) as usize],
            size,
        }
    }
}

impl<C: AsRef<ObjectStack>> MapBase<C> {
    pub fn get_objects(&self, position: Vector<i32>) -> Option<&C> {
        let index = self.point2d_to_index(position.into());

        self.stacks.get(index)
    }

    pub fn get_objects_at_entity(&self, entity: Entity) -> Option<&C> {
        for obj_stack in self.stacks.iter() {
            if obj_stack
                .as_ref()
                .iter()
                .find(|obj| obj.entity == entity)
                .is_some()
            {
                return Some(obj_stack);
            }
        }

        None
    }

    pub fn region(&self, position: Vector<i32>, size: Vector<i32>) -> MapBase<&C> {
        let mut region = Vec::<&C>::default();

        for y in position.1..position.1 + size.1 {
            for x in position.0..position.0 + size.0 {
                let index = self.point2d_to_index((x, y).into());

                region.push(self.stacks.get(index).expect("Region out of bounds"))
            }
        }

        MapBase {
            stacks: region,
            size,
        }
    }

    pub fn neighbours(&self, position: Vector<i32>) -> Vec<&C> {
        let mut neighbouring_obj_stacks = Vec::new();

        for direction in Direction::all().iter() {
            if let Some(obj_stack) = self.get_objects(position + direction.as_unit_vector()) {
                neighbouring_obj_stacks.push(obj_stack);
            }
        }

        neighbouring_obj_stacks
    }
}

impl<C: AsRef<ObjectStack> + AsMut<ObjectStack>> MapBase<C> {
    pub fn get_objects_mut(&mut self, position: Vector<i32>) -> Option<&mut C> {
        let index = self.point2d_to_index(position.into());

        self.stacks.get_mut(index)
    }

    pub fn get_objects_at_entity_mut(&mut self, entity: Entity) -> Option<&mut C> {
        for obj_stack in self.stacks.iter_mut() {
            if obj_stack.as_ref().iter().any(|obj| obj.entity == entity) {
                return Some(obj_stack);
            }
        }

        None
    }

    pub fn region_mut(&mut self, position: Vector<i32>, size: Vector<i32>) -> MapBase<&mut C> {
        let mut stacks_iter_mut = self.stacks.iter_mut();
        let mut region = Vec::<&mut C>::default();

        for y in position.1..position.1 + size.1 {
            for x in position.0..position.0 + size.0 {
                let index = (x + y * self.size.0 as i32) as usize;

                let obj_stack = stacks_iter_mut
                    .by_ref()
                    .nth(index)
                    .expect("Region out of bounds");
                region.push(obj_stack)
            }
        }

        MapBase {
            stacks: region,
            size,
        }
    }

    pub fn move_object_by_entity(
        &mut self,
        entity: Entity,
        current_position: &mut Position,
        new_position: Vector<i32>,
    ) {
        let curr_obj_stack = self.get_objects_at_entity_mut(entity).unwrap().as_mut();
        let curr_obj_index = curr_obj_stack
            .iter()
            .position(|obj| obj.entity == entity)
            .unwrap();
        let object = curr_obj_stack.remove(curr_obj_index);

        current_position.vector = new_position; // Mutate the component

        let new_obj_stack = self[new_position].as_mut();
        if curr_obj_index < new_obj_stack.len() {
            new_obj_stack.insert(curr_obj_index, object);
        } else {
            new_obj_stack.push(object);
        }
    }
}

impl<T: Into<Vector<i32>>, C: AsRef<ObjectStack>> Index<T> for MapBase<C> {
    type Output = C;

    fn index(&self, index: T) -> &Self::Output {
        self.get_objects(index.into()).unwrap()
    }
}

impl<T: Into<Vector<i32>>, C: AsRef<ObjectStack> + AsMut<ObjectStack>> IndexMut<T> for MapBase<C> {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        self.get_objects_mut(index.into()).unwrap()
    }
}

impl<C: AsRef<ObjectStack>> BaseMap for MapBase<C> {
    fn is_opaque(&self, index: usize) -> bool {
        if let Some(items) = self.stacks.get(index) {
            items.as_ref().first().map_or(false, |obj| obj.block_fov)
        } else {
            false
        }
    }
}

impl<C: AsRef<ObjectStack>> Algorithm2D for MapBase<C> {
    fn point2d_to_index(&self, point: Point) -> usize {
        (point.x + point.y * self.size.0 as i32) as usize
    }

    fn index_to_point2d(&self, index: usize) -> Point {
        Point {
            x: (index / self.size.1 as usize) as i32,
            y: (index / self.size.0 as usize) as i32,
        }
    }

    fn dimensions(&self) -> Point {
        Point {
            x: self.size.0 as i32,
            y: self.size.1 as i32,
        }
    }
}

pub struct Generator<'a> {
    world: &'a mut World,
    map: Arc<RwLock<Map>>,
}

impl<'a> Generator<'a> {
    fn new(world: &'a mut World, map: Arc<RwLock<Map>>) -> Self {
        Self { world, map }
    }

    pub fn room(
        &mut self,
        position: Vector<i32>,
        dimensions: Vector<i32>,
        floor: &Floor,
        wall: &Wall,
    ) {
        let mut commands = CommandBuffer::new(self.world);

        // Flooring
        for y in (position.1 + 1)..dimensions.1 {
            for x in (position.0 + 1)..dimensions.0 {
                floor.spawn(&mut commands, self.map.clone(), (x, y));
            }
        }

        // Top and bottom walls
        for x in (position.0 + 1)..dimensions.0 {
            wall.spawn(&mut commands, self.map.clone(), (x, position.1));
            wall.spawn(&mut commands, self.map.clone(), (x, dimensions.1));
        }

        // Left and right walls
        for y in position.1..dimensions.1 {
            wall.spawn(&mut commands, self.map.clone(), (position.0, y));
            wall.spawn(&mut commands, self.map.clone(), (dimensions.0, y));
        }

        commands.flush(self.world);
    }

    pub fn generate_caves(&mut self, floor: &Floor, wall: &Wall, enhancement_passes: u32) {
        let mut commands = CommandBuffer::new(self.world);
        let mut rng = RandomNumberGenerator::new();
        let map = self.map.read().unwrap();
        let map_dimensions = map.dimensions();

        // Randomly spawn walls
        for y in 0..map_dimensions.y {
            for x in 0..map_dimensions.x {
                if rng.rand::<f32>() < 0.35 {
                    wall.spawn(&mut commands, self.map.clone(), (x, y));
                } else {
                    floor.spawn(&mut commands, self.map.clone(), (x, y));
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
                let wall_neighbours: ObjectStack = map
                    .neighbours(position.vector)
                    .into_iter()
                    .fold(Vec::new(), |mut v: ObjectStack, stack| {
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
                        wall.spawn(&mut commands, self.map.clone(), position.vector);
                    } else {
                        floor.spawn(&mut commands, self.map.clone(), position.vector);
                    }
                }

                commands.flush(self.world);
            }
        }
    }

    pub fn generate(self) {}
}

pub struct Populator<'a> {
    world: &'a mut World,
    map: Arc<RwLock<Map>>,
}

impl<'a> Populator<'a> {
    pub fn new(world: &'a mut World, map: Arc<RwLock<Map>>) -> Self {
        Self { world, map }
    }

    pub fn spawn_player(&self) {
        let mut rng = RandomNumberGenerator::new();

        rng.random_slice_entry(
            <(&Tile, &Position)>::query()
                .iter(self.world)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .unwrap()
        .1
        .vector;
    }

    pub fn populate(self) {}
}
