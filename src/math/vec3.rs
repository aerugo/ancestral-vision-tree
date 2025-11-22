use std::ops::{Add, Sub, Mul, Neg};
use serde::{Serialize, Deserialize};

/// 3D vector for positions, directions, and colors
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const RIGHT: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    /// Convert to array for WebGL
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// Distance to another point
    pub fn distance(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }

    /// Create a perpendicular vector (useful for making coordinate frames)
    pub fn perpendicular(&self) -> Self {
        let n = self.normalize();
        if n.y.abs() < 0.9 {
            n.cross(&Vec3::UP).normalize()
        } else {
            n.cross(&Vec3::RIGHT).normalize()
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_creation() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!((v.length() - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
        assert!((n.x - 0.6).abs() < 0.0001);
        assert!((n.y - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert!((a.dot(&b)).abs() < 0.0001);

        let c = Vec3::new(1.0, 2.0, 3.0);
        let d = Vec3::new(4.0, 5.0, 6.0);
        assert!((c.dot(&d) - 32.0).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_cross() {
        let a = Vec3::RIGHT;  // (1, 0, 0)
        let b = Vec3::UP;     // (0, 1, 0)
        let c = a.cross(&b);  // RIGHT x UP = FORWARD (0, 0, 1)
        assert!((c.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_lerp() {
        let a = Vec3::ZERO;
        let b = Vec3::new(10.0, 20.0, 30.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 0.0001);
        assert!((mid.y - 10.0).abs() < 0.0001);
        assert!((mid.z - 15.0).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_ops() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let sum = a + b;
        assert_eq!(sum.x, 5.0);

        let diff = b - a;
        assert_eq!(diff.x, 3.0);

        let scaled = a * 2.0;
        assert_eq!(scaled.x, 2.0);

        let neg = -a;
        assert_eq!(neg.x, -1.0);
    }
}
