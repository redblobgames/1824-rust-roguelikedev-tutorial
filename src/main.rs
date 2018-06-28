#![allow(dead_code)]

extern crate rand;
extern crate tcod;

mod entity;
use entity::Object;

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


#[derive(Clone, Copy, Debug)]
struct Rect {
    left: i32, right: i32, width: i32,
    top: i32, bottom: i32, height: i32,
    // TODO: how do I mark these as 'const'?
}

impl Rect {
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Self {
        Rect {
            left: left, right: left + width, width: width,
            top: top, bottom: top + height, height: height,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        (self.left + self.width/2,
         self.top + self.height/2)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.left <= other.right) && (other.left <= self.right)
            && (self.top <= other.bottom) && (other.top <= self.bottom)
    }
}

type Map = Vec<Vec<Tile>>;

fn make_map() -> (Map, (i32, i32)) {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let width = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let height = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let left = rand::thread_rng().gen_range(0, MAP_WIDTH - width);
        let top = rand::thread_rng().gen_range(0, MAP_HEIGHT - height);

        let new_room = Rect::new(left, top, width, height);
        let fits = rooms.iter().all(|other_room| !new_room.intersects_with(other_room));
        if fits {
            create_room(new_room, &mut map);
            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(new_x, prev_y, new_y, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_x, prev_y, new_y, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            rooms.push(new_room);
        }
    }
    
    (map, rooms[0].center())
}

fn create_room(room: Rect, map: &mut Map) {
    for j in 0 .. room.height {
        for i in 0 .. room.width {
            map[(room.left + i) as usize][(room.top + j) as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    let left = cmp::min(x1, x2);
    let width = 1 + (x1 - x2).abs();
    create_room(Rect::new(left, y, width, 1), map);
}

fn create_v_tunnel(x: i32, y1: i32, y2: i32, map: &mut Map) {
    let top = cmp::min(y1, y2);
    let height = 1 + (y1 - y2).abs();
    create_room(Rect::new(x, top, 1, height), map);
}


fn render_all(root: &mut Root, console: &mut Offscreen, objects: &[Object], map: &mut Map, fov_map: &FovMap) {
    root.clear();
    
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = fov_map.is_in_fov(x, y);
            let wall = map[x as usize][y as usize].block_sight;
            let explored = &mut map[x as usize][y as usize].explored;
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

    let (mut map, player_position) = make_map();
    let mut previous_player_position = (-1, -1);
    let player = Object::new(player_position, '@', colors::WHITE);
    let npc = Object::new((54, 27), 'R', colors::YELLOW);
    let mut objects = [player, npc];

    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x, y,
                        !map[x as usize][y as usize].block_sight,
                        !map[x as usize][y as usize].blocked);
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
