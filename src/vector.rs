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

impl Vector2D {
    pub fn max(&self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2d_add() {
        let v1 = Vector2D::new(1, 2);
        let v2 = Vector2D::new(3, 4);
        let v3 = v1 + v2;
        assert_eq!(v3, Vector2D::new(4, 6));
    }

    #[test]
    fn test_vector2d_sub() {
        let v1 = Vector2D::new(1, 2);
        let v2 = Vector2D::new(3, 4);
        let v3 = v1 - v2;
        assert_eq!(v3, Vector2D::new(-2, -2));
    }

    #[test]
    fn test_vector2d_max() {
        let v1 = Vector2D::new(1, 2);
        let v2 = Vector2D::new(3, 4);
        let v3 = v1.max(v2);
        assert_eq!(v3, Vector2D::new(3, 4));
    }
}
