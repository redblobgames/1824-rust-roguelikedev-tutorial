use tcod::Color;
use tcod::console::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ai;

pub struct Object {
    pub name: String,
    pub glyph: char,
    pub color: Color,
    pub position: (i32, i32),
    pub blocks: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
}

impl Object {
    pub fn new(name: &str, glyph: char, color: Color, position: (i32, i32)) -> Self {
        Object {
            name: name.into(),
            glyph: glyph,
            color: color,
            position: position,
            blocks: true,
            alive: true,
            fighter: None,
            ai: None,
        }
    }

    pub fn move_to(&mut self, pos: (i32, i32)) {
        println!("move_to {} moves from {},{} to {},{}", self.name, self.position.0, self.position.1, pos.0, pos.1);
        self.position = pos;
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.position.0, self.position.1, self.glyph, BackgroundFlag::None);
    }
}
