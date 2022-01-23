use std::sync::{Arc, RwLock};

use components::position::Position;
use WINDOW_DIMENSIONS;
use {systems, WINDOW_CENTER};

use bracket_lib::prelude::{
    field_of_view, main_loop, string_to_cp437, to_cp437, Algorithm2D, BTerm, BTermBuilder, BaseMap,
    GameState, Input, Point, RandomNumberGenerator, RgbaLerp, VirtualKeyCode, BLACK,
    BLANCHED_ALMOND, RGB, RGBA, WHITE_SMOKE,
};
use map::{Map, Object};

use components::collision::Collision;
use components::glyph::Glyph;

use components::displacement::Displacement;
use components::memory::Memory;

use components::description::Description;
use components::direction::Direction;
use components::meta::camera_focus::CameraFocus;
use components::tile::Tile;
use interface::{Element, List};
use legion::*;
use location::Dungeon;
use std::ops::Deref;
use GUI_WIDTH;
use {FLOORS, WALLS};

use components::actions::grab::Grab;
use components::equipment::Equipment;
use components::storage::Storage;

use components::actions::throw::Throw;
use components::illumination::Illumination;
use components::light::Light;
use components::meta::intention::{Intent, Intention};

use interface::Label;
use label;
use legion::systems::CommandBuffer;

use vector::Vector;

pub const EVENT_LOG_PANE_SIZE: (i32, i32) = (WINDOW_DIMENSIONS.0, WINDOW_DIMENSIONS.1 / 4);
pub const EVENT_LOG_PANE_POSITION: (i32, i32) = (0, WINDOW_DIMENSIONS.1 - EVENT_LOG_PANE_SIZE.1);
pub const INVENTORY_PANE_SIZE: (i32, i32) = (WINDOW_DIMENSIONS.0 / 4, WINDOW_DIMENSIONS.1);
pub const INVENTORY_PANE_POSITION: (i32, i32) = (WINDOW_DIMENSIONS.0 - INVENTORY_PANE_SIZE.0, 0);

pub struct Game {
    pub player: Entity,
    pub map: Arc<RwLock<Map>>,
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
    pub inventory_pane: List,
    pub log_pane: Arc<RwLock<List>>,
}

impl Game {
    pub fn new(map_dimensions: (i32, i32)) -> Game {
        let mut world = World::default();
        let mut resources = Resources::default();
        let map = Arc::new(RwLock::new(Map::new(map_dimensions.into())));
        let rng = RandomNumberGenerator::new();

        resources.insert(map.clone());
        resources.insert(rng);

        let schedule = Schedule::builder()
            .add_system(systems::spawn::spawn_system())
            .add_system(systems::displacement::displacement_system())
            .add_system(systems::velocity::velocity_system())
            .add_system(systems::grab::grab_system())
            .add_system(systems::throw::throw_system())
            .build();

        // Interface
        let inventory_pane = List::new(INVENTORY_PANE_POSITION, INVENTORY_PANE_SIZE, 0);
        let log_pane = Arc::new(RwLock::new(List::new(
            EVENT_LOG_PANE_POSITION,
            EVENT_LOG_PANE_SIZE,
            0,
        )));
        resources.insert(log_pane.clone());

        // Map generation
        let dungeon = Dungeon::new(
            &mut world,
            map.clone(),
            (0, 0),
            (100, 100),
            &FLOORS["cave"],
            &WALLS["cave"],
        );
        dungeon.generate_caves(25);

        let player_start = resources
            .get_mut::<RandomNumberGenerator>()
            .unwrap()
            .random_slice_entry(
                <(&Tile, &Position)>::query()
                    .iter(&world)
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .unwrap()
            .1
            .vector;

        let player_colour = RGBA::from((115, 255, 115, 255)).into();
        let player = world.push((
            Intention {
                intent: Intent::Walk,
            },
            Equipment::default(),
            Collision,
            Glyph {
                character: '@',
                colour: player_colour,
            },
            CameraFocus,
            Position {
                vector: player_start,
            },
            Memory::default(),
        ));
        map.write().unwrap()[player_start].push(Object {
            entity: player,
            block_movement: false,
            block_fov: false,
        });

        Game {
            player,
            map,
            world,
            resources,
            schedule,
            inventory_pane,
            log_pane,
        }
    }

    pub fn render_interface(&mut self, ctx: &mut BTerm) {
        let player = self.world.entry_ref(self.player).unwrap();
        let player_equipment = player.get_component::<Equipment>().unwrap();

        self.inventory_pane.add(label![format!(
            "{:?}",
            player.get_component::<Intention>().unwrap().intent
        )
        .to_uppercase()]);
        self.inventory_pane.add(match player_equipment.held {
            Some(held_entity) => {
                let held_entry = self.world.entry_ref(held_entity).unwrap();

                label![
                    "Hands [",
                    *held_entry.get_component::<Glyph>().unwrap(),
                    " ",
                    *held_entry.get_component::<Description>().unwrap().name,
                    "]"
                ]
            }
            None => label!["Hands []"],
        });
        self.inventory_pane.render(ctx);
        self.inventory_pane.clear();

        if let Ok(log_pane) = self.log_pane.write() {
            log_pane.render(ctx);
        }
    }

    pub fn apply_illumination(
        &mut self,
        _commands: &mut CommandBuffer,
        positions: &[Vector<i32>],
        _ctx: &mut BTerm,
    ) {
        let map = self.map.read().unwrap();

        for position in positions {
            let objects = map
                .get_objects(Vector::from((position.0, position.1)))
                .unwrap();

            if !objects.is_empty() {
                let cell_entity = objects.iter().last().unwrap().entity;
                let cell_pos_opt;
                let cell_light_opt;

                {
                    let cell_entry = self.world.entry_ref(cell_entity).unwrap();
                    cell_pos_opt = cell_entry.get_component::<Position>().ok().cloned();
                    cell_light_opt = cell_entry.get_component::<Light>().ok().cloned();
                }

                if let Some(cell_light) = cell_light_opt {
                    let position = cell_pos_opt.unwrap();
                    let illuminated_points = field_of_view(
                        (position.vector.0, position.vector.1).into(),
                        cell_light.radius,
                        self.map.clone().read().unwrap().deref(),
                    );

                    for point in illuminated_points {
                        let point_pos: Vector<i32> = point.into();
                        let objects_at_point = map.get_objects(point_pos).unwrap();

                        if !objects_at_point.is_empty() {
                            let top_obj_entity = objects_at_point.iter().last().unwrap().entity;
                            let mut top_obj_entry = self.world.entry(top_obj_entity).unwrap();

                            let cell_dist = position.vector.distance(point_pos);
                            let light_source_info = (cell_light, cell_dist);

                            match top_obj_entry.get_component_mut::<Illumination>() {
                                Ok(illumination) => {
                                    illumination.sources.push(light_source_info);
                                }

                                Err(_) => top_obj_entry.add_component(Illumination {
                                    sources: vec![light_source_info],
                                }),
                            };
                        }
                    }
                }
            }
        }
    }

    pub fn render_world(&mut self, ctx: &mut BTerm) {
        let mut commands = CommandBuffer::new(&self.world);
        let map = self.map.clone();

        if let Ok(map) = map.read() {
            let background_bg = RGB::from((10, 10, 15));
            let mut visible_points = Vec::default();

            if let Some((_, position)) = <(&CameraFocus, &Position)>::query()
                .iter(&self.world)
                .collect::<Vec<_>>()
                .first()
            {
                let origin = Point {
                    x: position.vector.0 as i32,
                    y: position.vector.1 as i32,
                };
                visible_points.append(
                    &mut field_of_view(origin, 10, map.deref())
                        .into_iter()
                        .map(|p| Vector::from(p))
                        .collect::<Vec<_>>(),
                );

                self.apply_illumination(&mut commands, &visible_points, ctx);
            }

            if let Some((_, position, _)) = <(&CameraFocus, &Position, &Glyph)>::query()
                .iter(&self.world)
                .collect::<Vec<_>>()
                .first()
            {
                ctx.cls_bg(background_bg);

                for point in &visible_points {
                    if let Some(top_obj) = map[*point].last() {
                        let top_obj_entry = self.world.entry_ref(top_obj.entity).unwrap();
                        let top_obj_glyph = top_obj_entry
                            .get_component::<Glyph>()
                            .map_or(Glyph::default(), |g| *g);

                        let mut top_obj_rgba = top_obj_glyph
                            .colour
                            .rgba
                            .lerp((25, 25, 35, 25).into(), 0.65); // COLOUR CORRECTION AND LIGHT DIMMING

                        if let Ok(top_obj_illumination) =
                            top_obj_entry.get_component::<Illumination>()
                        {
                            let source_count = top_obj_illumination.sources.len();

                            for (light, dist) in &top_obj_illumination.sources {
                                let intensity = if *dist != 0 {
                                    light.intensity / (*dist as f32)
                                } else {
                                    1.0
                                };

                                top_obj_rgba.r +=
                                    (light.colour.rgba.r * intensity) / source_count as f32;
                                top_obj_rgba.g +=
                                    (light.colour.rgba.g * intensity) / source_count as f32;
                                top_obj_rgba.b +=
                                    (light.colour.rgba.b * intensity) / source_count as f32;
                                top_obj_rgba.a += light.colour.rgba.a * intensity;
                            }

                            commands.remove_component::<Illumination>(top_obj.entity);
                        }

                        ctx.set(
                            WINDOW_CENTER.0 as i32
                                - (position.vector.0 as i32 - point.0)
                                - (GUI_WIDTH / 2) as i32,
                            WINDOW_CENTER.1 as i32 - (position.vector.1 as i32 - point.1),
                            top_obj_rgba,
                            background_bg,
                            to_cp437(top_obj_glyph.character),
                        );
                    }
                }
            }

            let mut focused_entity = None;
            if let Some((e, _, position, _, memory)) =
                <(Entity, &CameraFocus, &Position, &Glyph, &Memory)>::query()
                    .iter(&self.world)
                    .collect::<Vec<_>>()
                    .first()
            {
                focused_entity = Some(**e);

                for point in &memory.spatial {
                    if let Some(top_obj) = map[*point].last() {
                        let top_obj_entry = self.world.entry_ref(top_obj.entity).unwrap();
                        let top_obj_glyph = top_obj_entry
                            .get_component::<Glyph>()
                            .map_or(Glyph::default(), |g| *g);

                        if !visible_points.contains(point) {
                            ctx.set(
                                WINDOW_CENTER.0 as i32
                                    - (position.vector.0 as i32 - point.0)
                                    - (GUI_WIDTH / 2) as i32,
                                WINDOW_CENTER.1 as i32 - (position.vector.1 as i32 - point.1),
                                (30, 30, 35),
                                background_bg,
                                to_cp437(top_obj_glyph.character),
                            );
                        }
                    }
                }
            }

            if let Some(focused_entity) = focused_entity {
                let mut focused_entity = self.world.entry_mut(focused_entity).unwrap();
                let memory = focused_entity.get_component_mut::<Memory>().unwrap();

                for vec in visible_points {
                    if !memory.spatial.contains(&vec) {
                        memory.spatial.push(vec);
                    }
                }
            }
        }

        commands.flush(&mut self.world);
    }

    pub fn get_input(&mut self, ctx: &mut BTerm) {
        let mut player = self.world.entry(self.player).unwrap();
        let player_position = player.get_component::<Position>().unwrap();
        let player_intention = player.get_component::<Intention>().unwrap();
        let map = self.map.write().unwrap();
        let objects_at_player_tile = &map[player_position.vector];

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Tab => player.add_component(Intention {
                    intent: Intent::Walk,
                }),
                VirtualKeyCode::G => player.add_component(Intention {
                    intent: Intent::Grab,
                }),
                VirtualKeyCode::T => player.add_component(Intention {
                    intent: Intent::Throw,
                }),

                VirtualKeyCode::W | VirtualKeyCode::A | VirtualKeyCode::S | VirtualKeyCode::D => {
                    if let Some(direction) = match key {
                        VirtualKeyCode::W => Some(Direction::North),
                        VirtualKeyCode::S => Some(Direction::South),
                        VirtualKeyCode::A => Some(Direction::West),
                        VirtualKeyCode::D => Some(Direction::East),
                        _ => None,
                    } {
                        match player_intention.intent {
                            Intent::Walk => {
                                player.add_component(Displacement::from(direction));
                            }

                            Intent::Grab => {
                                player.add_component(Grab {
                                    entity: map
                                        [player_position.vector + direction.as_unit_vector()]
                                    .last()
                                    .unwrap()
                                    .entity,
                                });
                            }

                            Intent::Throw => {
                                player.add_component(Throw { direction });
                            }

                            _ => {}
                        }
                    }
                }

                VirtualKeyCode::F => match player_intention.intent {
                    Intent::Walk | Intent::Grab => {
                        player.add_component(Grab {
                            entity: objects_at_player_tile[1].entity,
                        });
                    }
                    _ => {}
                },

                VirtualKeyCode::B => {
                    let mut label = Label::from("You ");
                    let player = self.world.entry_ref(self.player).unwrap();
                    let player_equipment = player.get_component::<Equipment>().unwrap();

                    if let Some(held_item_entity) = player_equipment.held {
                        let held_item_glyph;
                        let held_item_description;

                        {
                            let held_item_entry = self.world.entry_ref(held_item_entity).unwrap();
                            held_item_glyph = *held_item_entry.get_component::<Glyph>().unwrap();
                            held_item_description = held_item_entry
                                .get_component::<Description>()
                                .unwrap()
                                .clone();
                        }

                        label += if let Some(storage_entity) = player_equipment.storage {
                            let mut storage_entry = self.world.entry_mut(storage_entity).unwrap();
                            let storage_description = storage_entry
                                .get_component::<Description>()
                                .unwrap()
                                .clone();
                            let storage_storage =
                                storage_entry.get_component_mut::<Storage>().unwrap();

                            storage_storage.objects.push(held_item_entity);

                            label![
                                "place the ",
                                (held_item_description.name, held_item_glyph.colour),
                                " in your ",
                                storage_description.name
                            ]
                        } else {
                            label![
                                "have no place to store the ",
                                (held_item_description.name, held_item_glyph.colour)
                            ]
                        };
                    } else {
                        label += label!["are not holding anything"]
                    }

                    self.log_pane.write().unwrap().add(label);
                }

                _ => {}
            }
        }
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.schedule.execute(&mut self.world, &mut self.resources);
        self.render_world(ctx);
        self.render_interface(ctx);
        self.get_input(ctx);
    }
}
