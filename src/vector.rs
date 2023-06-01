use std::ops;
// TODO: generics

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vector2D {
    pub x: i32,
    pub y: i32,
}

impl Vector2D {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Vector2D {
    pub fn max(&self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl std::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
