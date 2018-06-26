extern crate rand;
extern crate tcod;

use std::cmp;
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
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile { blocked: false, block_sight: false }
    }

    pub fn wall() -> Self {
        Tile { blocked: true, block_sight: true }
    }
}


#[derive(Clone, Copy, Debug)]
struct Rect {
    left: i32, right: i32, width: i32,
    top: i32, bottom: i32, height: i32,
    
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


struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color,) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}


fn render_all(root: &mut Root, console: &mut Offscreen, objects: &[Object], map: &Map) {
    root.clear();
    
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = map[x as usize][y as usize].block_sight;
            console.set_char_background(x, y, if wall { COLOR_DARK_WALL } else { COLOR_DARK_GROUND }, BackgroundFlag::Set);
        }
    }
    
    for object in objects {
        object.draw(console);
    }

    blit(console, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), root, (0, 0), 1.0, 1.0);
}


fn handle_keys(root: &mut Root, player: &mut Object, key: Key) -> bool {
    use tcod::input::KeyCode::*;
    match key {
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

    let (map, (player_x, player_y)) = make_map();
    let player = Object::new(player_x, player_y, '@', colors::WHITE);
    let npc = Object::new(54, 27, 'R', colors::YELLOW);
    let mut objects = [player, npc];
    
    while !root.window_closed() {
        console.clear();
        render_all(&mut root, &mut console, &objects, &map);
        root.flush();
        
        let key = root.wait_for_keypress(true);
        if handle_keys(&mut root, &mut objects[0], key) {
            break;
        }

    }
}
