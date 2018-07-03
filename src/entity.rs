use tcod::Color;
use tcod::console::*;

pub struct Object {
    pub name: String,
    pub glyph: char,
    pub color: Color,
    pub position: (i32, i32),
    pub blocks: bool,
    pub alive: bool,
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
        }
    }

    pub fn move_to(&mut self, pos: (i32, i32)) {
        self.position = pos;
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.position.0, self.position.1, self.glyph, BackgroundFlag::None);
    }
}
