#![allow(dead_code)]

extern crate rand;
extern crate tcod;

mod entity;
use entity::Object;
mod rect;
use rect::Rect;

use std::cmp;
use tcod::map::{Map as FovMap, FovAlgorithm};
use rand::Rng;
use tcod::input::Key;
use tcod::console::*;
use tcod::Color;
use tcod::colors;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50 };

const MAX_ROOM_MONSTERS: i32 = 3;

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile { blocked: false, block_sight: false, explored: false, }
    }

    pub fn wall() -> Self {
        Tile { blocked: true, block_sight: true, explored: false, }
    }
}


struct Map {
    tiles: Vec<Vec<Tile>>,
    rooms: Vec<Rect>,
}

impl Map {
    pub fn new() -> Self {
        let mut map = Map {
            tiles: vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize],
            rooms: vec![],
        };
        for _ in 0..MAX_ROOMS {
            let width = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            let height = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            let left = rand::thread_rng().gen_range(0, MAP_WIDTH - width);
            let top = rand::thread_rng().gen_range(0, MAP_HEIGHT - height);

            let new_room = Rect::new(left, top, width, height);
            let fits = map.rooms.iter().all(|other_room| !new_room.intersects_with(other_room));
            if fits {
                map.create_room(new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    // TODO: try map.rooms.last().unwrap()
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rand::random() {
                        // first move horizontally, then vertically
                        map.create_h_tunnel(prev_x, new_x, prev_y);
                        map.create_v_tunnel(new_x, prev_y, new_y);
                    } else {
                        // first move vertically, then horizontally
                        map.create_v_tunnel(prev_x, prev_y, new_y);
                        map.create_h_tunnel(prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);
            }
        }
        map
    }

    fn create_room(&mut self, room: Rect) {
        for j in 0 .. room.height {
            for i in 0 .. room.width {
                self.tiles[(room.left + i) as usize][(room.top + j) as usize] = Tile::empty();
            }
        }
    }

    fn create_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let left = cmp::min(x1, x2);
        let width = 1 + (x1 - x2).abs();
        self.create_room(Rect::new(left, y, width, 1));
    }

    fn create_v_tunnel(&mut self, x: i32, y1: i32, y2: i32) {
        let top = cmp::min(y1, y2);
        let height = 1 + (y1 - y2).abs();
        self.create_room(Rect::new(x, top, 1, height));
    }
}



fn place_objects(room: Rect, objects: &mut Vec<Object>) {
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        let pos = (
            rand::thread_rng().gen_range(room.left + 1, room.right),
            rand::thread_rng().gen_range(room.top + 1, room.bottom),
        );
        let mut monster = if rand::random::<f32>() < 0.8 {
            Object::new(pos, 'o', colors::DESATURATED_GREEN)
        } else {
            Object::new(pos, 'T', colors::DARKER_GREEN)
        };
        objects.push(monster);
    }
}



fn render_all(root: &mut Root, console: &mut Offscreen, objects: &[Object], map: &mut Map, fov_map: &FovMap) {
    root.clear();
    
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = fov_map.is_in_fov(x, y);
            let wall = map.tiles[x as usize][y as usize].block_sight;
            let explored = &mut map.tiles[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                let color = match (visible, wall) {
                    (false, false) => COLOR_DARK_GROUND,
                    (false, true) => COLOR_DARK_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                    (true, true) => COLOR_LIGHT_WALL,
                };
                console.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }
    
    for object in objects {
        if fov_map.is_in_fov(object.position.0, object.position.1) {
            object.draw(console);
        }
    }

    blit(console, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), root, (0, 0), 1.0, 1.0);
}


fn handle_keys(root: &mut Root, player: &mut Object, key: Key) -> bool {
    use tcod::input::KeyCode::*;
    match key {
        // TODO: don't allow move if it's into a wall
        Key { code: Up, .. } => player.move_by(0, -1),
        Key { code: Down, .. } => player.move_by(0, 1),
        Key { code: Left, .. } => player.move_by(-1, 0),
        Key { code: Right, .. } => player.move_by(1, 0),
        Key { code: Char, printable: 'q', .. } => return true,
        Key { code: Escape, .. } => return true,
        Key { code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        },
        _ => {},
    }
    false
}

fn main() {
    let mut root = Root::initializer()
        .font("../arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .renderer(Renderer::GLSL)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();
    let mut console = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    
    tcod::system::set_fps(LIMIT_FPS);

    let mut map = Map::new();
    let player = Object::new(map.rooms[0].center(), '@', colors::WHITE);
    let mut previous_player_position = (-1, -1);
    let mut objects = [player];

    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x, y,
                        !map.tiles[x as usize][y as usize].block_sight,
                        !map.tiles[x as usize][y as usize].blocked);
        }
    }
    
    while !root.window_closed() {
        console.clear();
        if previous_player_position != objects[0].position {
            let player = &objects[0];
            fov_map.compute_fov(player.position.0, player.position.1,
                                TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        }

        render_all(&mut root, &mut console, &objects, &mut map, &fov_map);
        root.flush();
        
        let key = root.wait_for_keypress(true);
        previous_player_position = objects[0].position;
        if handle_keys(&mut root, &mut objects[0], key) {
            break;
        }

    }
}
