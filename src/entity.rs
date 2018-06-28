use tcod::Color;
use tcod::console::*;

pub struct Object {
    pub position: (i32, i32),
    glyph: char,
    color: Color,
}

impl Object {
    pub fn new(position: (i32, i32), glyph: char, color: Color) -> Self {
        Object {
            position: position,
            glyph: glyph,
            color: color,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.position.0, self.position.1, self.glyph, BackgroundFlag::None);
    }
}


