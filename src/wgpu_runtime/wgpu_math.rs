use wgpu::Extent3d;
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, Copy)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn new_dim(size: (u32, u32)) -> Self {
        Self {
            x: size.0 as i32,
            y: size.1 as i32,
        }
    }

    pub fn new_ext(ext: &Extent3d) -> Self {
        return Self {
            x: ext.width as i32,
            y: ext.height as i32,
        };
    }

    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }

    pub fn multiply(&self, multiplier: i32) -> Vec2i {
        return Self { x: self.x * multiplier, y: self.y * multiplier };
    }

    pub fn divide_single(&self, divider: i32) -> Vec2i {
        return Self { x: self.x / divider, y: self.y / divider };
    }

    pub fn divide(&self, x: i32, y: i32) -> Vec2i {
        return Self { x: self.x / x, y: self.y / y };
    }

    pub fn subtract(&self, subtract: Vec2i) -> Vec2i {
        return Self { x: self.x - subtract.x, y: self.y - subtract.y };
    }

    pub fn to_v2f(&self) -> Vec2f {
        Vec2f::new(self.x as f32, self.y as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_2i(x: i32, y: i32) -> Self {
        Self { x: x as f32, y: y as f32 }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn add(&self, x: f32, y: f32) -> Vec2f {
        return Vec2f::new(self.x + x, self.y + y);
    }

    pub fn add_i(&self, x: i32, y: i32) -> Vec2f {
        return Vec2f::new(self.x + x as f32, self.y + y as f32);
    }

    pub fn add_v2i(&self, vec: Vec2i) -> Vec2f {
        return Vec2f::new(self.x + vec.x as f32, self.y + vec.y as f32);
    }

    pub fn add_v2f(&self, vec: Vec2f) -> Vec2f {
        return Vec2f::new(self.x + vec.x, self.y + vec.y);
    }
    pub fn subtract(&self, subtraction: f32) -> Vec2f {
        return Self { x: self.x - subtraction, y: self.y - subtraction };
    }

    pub fn subtract_v2f(&self, vec: Vec2f) -> Vec2f {
        return Self { x: self.x - vec.x, y: self.y - vec.y };
    }

    pub fn multiply(&self, scalar: f32) -> Vec2f {
        return Vec2f::new(self.x * scalar, self.y * scalar);
    }

    pub fn divide_single(&self, divider: f32) -> Vec2f {
        return Self { x: self.x / divider, y: self.y / divider };
    }

    pub fn divide(&self, x: f32, y: f32) -> Vec2f {
        return Self { x: self.x / x, y: self.y / y };
    }

    pub fn floor(&self) -> Vec2f {
        return Vec2f::new(self.x.floor(), self.y.floor());
    }

    pub fn round(&self) -> Vec2f {
        return Vec2f::new(self.x.round(), self.y.round());
    }

    pub fn floor_2i(&self) -> Vec2i {
        Vec2i::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    pub fn ceil_2i(&self) -> Vec2i {
        Vec2i::new(self.x.ceil() as i32, self.y.ceil() as i32)
    }

    pub fn round_2i(&self) -> Vec2i {
        Vec2i::new(self.x.round() as i32, self.y.round() as i32)
    }
}

impl PartialEq<PhysicalSize<u32>> for Vec2i {
    fn eq(&self, other: &PhysicalSize<u32>) -> bool {
        self.x == other.width as i32 && self.y == other.height as i32
    }
}
