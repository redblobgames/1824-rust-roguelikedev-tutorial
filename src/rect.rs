#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub left: i32, pub right: i32, pub width: i32,
    pub top: i32, pub bottom: i32, pub height: i32,
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
