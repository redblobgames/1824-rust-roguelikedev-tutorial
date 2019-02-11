use tcod::Color;
use tcod::colors;
use tcod::console::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub on_death: DeathCallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
    Player,
    Monster,
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

    pub fn take_damage(&mut self, damage: i32) {
        if damage > 0 {
            if let Some(fighter) = self.fighter.as_mut() {
                fighter.hp -= damage;
            }
            if let Some(fighter) = self.fighter {
                if fighter.hp <= 0 {
                    self.alive = false;
                    fighter.on_death.callback(self);
                }
            }
        }
    }

    pub fn attack(&mut self, target: &mut Object) {
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            println!("{} attacks {} for {} hit points.", self.name, target.name, damage);
            target.take_damage(damage);
        } else {
            println!("{} attacks {} but it has no effect!", self.name, target.name);
        }
    }
}

fn player_death(player: &mut Object) {
    // the game ended!
    println!("You died!");

    // for added effect, transform the player into a corpse!
    player.glyph = '%';
    player.color = colors::DARK_RED;
}

fn monster_death(monster: &mut Object) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    println!("{} is dead!", monster.name);
    monster.glyph = '%';
    monster.color = colors::DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}

impl DeathCallback {
    fn callback(self, object: &mut Object) {
        use DeathCallback::*;
        let callback: fn(&mut Object) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object);
    }
}
