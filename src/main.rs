extern crate bracket_lib;
extern crate lazy_static;
extern crate legion;
extern crate ndarray;
extern crate num_traits;
extern crate object_derive;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate shred;

use bracket_lib::prelude::{main_loop, BTermBuilder};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use entities::floor::Floor;
use entities::wall::Wall;

mod components;
mod entities;
mod game;
mod map;
mod systems;
mod vector;

#[macro_use]
mod interface;
mod location;

const WALLS_FILE_PATH: &str = "assets/walls.json";
const FLOORS_FILE_PATH: &str = "assets/floors.json";
const OBJECTS_FILE_PATH: &str = "assets/objects.json";

pub const WINDOW_TITLE: &str = "Protogue";
pub const WINDOW_DIMENSIONS: (i32, i32) = (100, 50);
pub const WINDOW_CENTER: (i32, i32) = (WINDOW_DIMENSIONS.0 / 2 - 1, WINDOW_DIMENSIONS.1 / 2 - 1);
pub const WINDOW_FULLSCREEN: bool = false;

pub const GUI_WIDTH: i32 = WINDOW_DIMENSIONS.0 / 4;
pub const MAP_DIMENSIONS: (i32, i32) = (250, 250);

// Create game objects (Walls, floors, items, etc)
lazy_static! {
    static ref WALLS: HashMap<String, Wall> = serde_json::from_reader(BufReader::new(
        File::open(Path::new(WALLS_FILE_PATH)).expect("Failed to load walls")
    ))
    .unwrap();
    static ref FLOORS: HashMap<String, Floor> = serde_json::from_reader(BufReader::new(
        File::open(Path::new(FLOORS_FILE_PATH)).expect("Failed to load floors")
    ))
    .unwrap();
    static ref ITEMS: HashMap<String, entities::item::Item> = serde_json::from_reader(
        BufReader::new(File::open(Path::new(OBJECTS_FILE_PATH)).expect("Failed to load items"))
    )
    .unwrap();
    static ref CREATURES: HashMap<String, entities::creature::Creature> = serde_json::from_reader(
        BufReader::new(File::open(Path::new(OBJECTS_FILE_PATH)).expect("Failed to load creatures"))
    )
    .unwrap();
}

fn main() {
    // Initialize terminal and game
    let ctx = BTermBuilder::simple(WINDOW_DIMENSIONS.0, WINDOW_DIMENSIONS.1)
        .unwrap()
        .with_tile_dimensions(16, 16)
        .with_title("Roguelike")
        .build()
        .unwrap();
    let g = game::Game::new(MAP_DIMENSIONS);

    // Run game
    main_loop(ctx, g);
}
