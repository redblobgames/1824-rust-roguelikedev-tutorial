#![allow(dead_code)]

extern crate rand;
extern crate tcod;

mod entity;
use entity::*;
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

const PLAYER: usize = 0;
const MAX_ROOM_MONSTERS: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

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


fn distance(here: (i32, i32), there: (i32, i32)) -> f32 {
    let dx = here.0 - there.0;
    let dy = here.1 - there.1;
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
}


fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        let pos = (
            rand::thread_rng().gen_range(room.left + 1, room.right),
            rand::thread_rng().gen_range(room.top + 1, room.bottom),
        );
        if !is_blocked(pos, map, objects) {
            let mut monster = if rand::random::<f32>() < 0.8 {
                let mut orc = Object::new("Orc", 'o', colors::DESATURATED_GREEN, pos);
                orc.fighter = Some(Fighter{max_hp: 10, hp: 10, defense: 0, power: 3, on_death: DeathCallback::Monster});
                orc.ai = Some(Ai);
                orc
            } else {
                let mut troll = Object::new("Troll", 'T', colors::DARKER_GREEN, pos);
                troll.fighter = Some(Fighter{max_hp: 16, hp: 16, defense: 1, power: 4, on_death: DeathCallback::Monster});
                troll.ai = Some(Ai);
                troll
            };
            objects.push(monster);
        }
    }
}


fn is_blocked(pos: (i32, i32), map: &Map, objects: &[Object]) -> bool {
    if map.tiles[pos.0 as usize][pos.1 as usize].blocked {
        return true;
    }

    return objects.iter().any(|object| {
        object.blocks && object.position == pos
    });
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

    let to_draw: Vec<_> = objects.iter().filter(|o| fov_map.is_in_fov(o.position.0, o.position.1)).collect();
    let layer_under: Vec<_> = to_draw.iter().filter(|o| !o.blocks).collect();
    let layer_over: Vec<_> = to_draw.iter().filter(|o| o.blocks).collect();
    for object in &layer_under {
        object.draw(console);
    }
    for object in &layer_over {
        object.draw(console);
    }

    blit(console, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), root, (0, 0), 1.0, 1.0);

/// (copied from roguelike tutorial without trying to understand it --amitp)
/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}


fn move_towards(id: usize, target: (i32, i32), map: &Map, objects: &mut [Object]) {
    let d = distance(target, objects[id].position);
    let dx = ((target.0 - objects[id].position.0) as f32 / d).round() as i32;
    let dy = ((target.1 - objects[id].position.1) as f32 / d).round() as i32;
    let new_pos = (objects[id].position.0 + dx,
                   objects[id].position.1 + dy);
    
    if !is_blocked(new_pos, map, objects) {
        objects[id].move_to(new_pos);
    }
}


fn move_or_attack(id: usize, delta: (i32, i32), map: &Map, objects: &mut[Object]) -> PlayerAction {
    let new_pos = (objects[id].position.0 + delta.0,
                   objects[id].position.1 + delta.1);
    let target_id = objects.iter().position(|object| {
        object.fighter.is_some() && object.position == new_pos
    });

    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target);
            PlayerAction::TookTurn
        }
        None => {
            if !is_blocked(new_pos, map, objects) {
                objects[id].move_to(new_pos);
                PlayerAction::TookTurn
            } else {
                PlayerAction::DidntTakeTurn
            }
        }
    }
}


fn ai_take_turn(monster_id: usize, map: &Map, objects: &mut [Object], fov_map: &FovMap) {
    let pos = objects[monster_id].position;
    if fov_map.is_in_fov(pos.0, pos.1) {
        if distance(objects[monster_id].position, objects[PLAYER].position) >= 2.0 {
            move_towards(monster_id, objects[PLAYER].position, map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            let (monster, player) = mut_two(monster_id, PLAYER, objects);
            monster.attack(player);
        }
    }
}


fn handle_keys(root: &mut Root, map: &Map, objects: &mut[Object], key: Key) -> PlayerAction {
    use PlayerAction::*;
    use tcod::input::KeyCode::*;
    // TODO: handle player being dead
    return match key {
        Key { code: Up, .. } => move_or_attack(PLAYER, (0, -1), map, objects),
        Key { code: Down, .. } => move_or_attack(PLAYER, (0, 1), map, objects),
        Key { code: Left, .. } => move_or_attack(PLAYER, (-1, 0), map, objects),
        Key { code: Right, .. } => move_or_attack(PLAYER, (1, 0), map, objects),
        Key { code: Char, printable: 'q', .. } => Exit,
        Key { code: Escape, .. } => Exit,
        Key { code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        },
        _ => DidntTakeTurn,
    }
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
    let mut player = Object::new("Player", '@', colors::WHITE, map.rooms[0].center());
    player.fighter = Some(Fighter{max_hp: 30, hp: 30, defense: 2, power: 5, on_death: DeathCallback::Player});
    let mut previous_player_position = (-1, -1);
    let mut objects:Vec<Object> = vec![player];

    for room in map.rooms.iter() {
        place_objects(*room, &map, &mut objects);
    }
    
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
        if previous_player_position != objects[PLAYER].position {
            let player = &objects[PLAYER];
            fov_map.compute_fov(player.position.0, player.position.1,
                                TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        }

        render_all(&mut root, &mut console, &objects, &mut map, &fov_map);
        root.flush();
        
        let key = root.wait_for_keypress(true);
        previous_player_position = objects[PLAYER].position;
        let player_action = handle_keys(&mut root, &map, &mut objects, key);
        if player_action == PlayerAction::Exit {
            break;
        }
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, &map, &mut objects, &fov_map);
                }
            }
        }
    }
}
