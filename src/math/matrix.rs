use super::Vec3;

/// 4x4 matrix for transformations (column-major for WebGL)
#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub data: [f32; 16],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity();
        m.data[12] = x;
        m.data[13] = y;
        m.data[14] = z;
        m
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity();
        m.data[0] = x;
        m.data[5] = y;
        m.data[10] = z;
        m
    }

    pub fn rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, c, s, 0.0,
                0.0, -s, c, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [
                c, 0.0, -s, 0.0,
                0.0, 1.0, 0.0, 0.0,
                s, 0.0, c, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotation_z(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [
                c, s, 0.0, 0.0,
                -s, c, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create rotation matrix to align with a direction
    pub fn look_rotation(forward: Vec3, up: Vec3) -> Self {
        let f = forward.normalize();
        let r = up.cross(&f).normalize();
        let u = f.cross(&r);

        Self {
            data: [
                r.x, u.x, f.x, 0.0,
                r.y, u.y, f.y, 0.0,
                r.z, u.z, f.z, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Perspective projection matrix
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);

        Self {
            data: [
                f / aspect, 0.0, 0.0, 0.0,
                0.0, f, 0.0, 0.0,
                0.0, 0.0, (far + near) * nf, -1.0,
                0.0, 0.0, 2.0 * far * near * nf, 0.0,
            ],
        }
    }

    /// Look-at view matrix
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let r = f.cross(&up).normalize();
        let u = r.cross(&f);

        Self {
            data: [
                r.x, u.x, -f.x, 0.0,
                r.y, u.y, -f.y, 0.0,
                r.z, u.z, -f.z, 0.0,
                -r.dot(&eye), -u.dot(&eye), f.dot(&eye), 1.0,
            ],
        }
    }

    /// Matrix multiplication
    pub fn mul(&self, other: &Mat4) -> Self {
        let mut result = [0.0f32; 16];

        for row in 0..4 {
            for col in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.data[row + k * 4] * other.data[k + col * 4];
                }
                result[row + col * 4] = sum;
            }
        }

        Self { data: result }
    }

    /// Transform a point (applies translation)
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            self.data[0] * p.x + self.data[4] * p.y + self.data[8] * p.z + self.data[12],
            self.data[1] * p.x + self.data[5] * p.y + self.data[9] * p.z + self.data[13],
            self.data[2] * p.x + self.data[6] * p.y + self.data[10] * p.z + self.data[14],
        )
    }

    /// Transform a direction (ignores translation)
    pub fn transform_direction(&self, d: Vec3) -> Vec3 {
        Vec3::new(
            self.data[0] * d.x + self.data[4] * d.y + self.data[8] * d.z,
            self.data[1] * d.x + self.data[5] * d.y + self.data[9] * d.z,
            self.data[2] * d.x + self.data[6] * d.y + self.data[10] * d.z,
        )
    }

    /// Get as slice for WebGL
    pub fn as_slice(&self) -> &[f32; 16] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Mat4::identity();
        assert_eq!(m.data[0], 1.0);
        assert_eq!(m.data[5], 1.0);
        assert_eq!(m.data[10], 1.0);
        assert_eq!(m.data[15], 1.0);
    }

    #[test]
    fn test_translation() {
        let m = Mat4::translation(1.0, 2.0, 3.0);
        let p = Vec3::ZERO;
        let result = m.transform_point(p);
        assert!((result.x - 1.0).abs() < 0.0001);
        assert!((result.y - 2.0).abs() < 0.0001);
        assert!((result.z - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_scale() {
        let m = Mat4::scale(2.0, 3.0, 4.0);
        let p = Vec3::new(1.0, 1.0, 1.0);
        let result = m.transform_point(p);
        assert!((result.x - 2.0).abs() < 0.0001);
        assert!((result.y - 3.0).abs() < 0.0001);
        assert!((result.z - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_rotation_z() {
        let m = Mat4::rotation_z(std::f32::consts::FRAC_PI_2);
        let p = Vec3::new(1.0, 0.0, 0.0);
        let result = m.transform_point(p);
        assert!((result.x).abs() < 0.0001);
        assert!((result.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_mul() {
        let t = Mat4::translation(1.0, 0.0, 0.0);
        let s = Mat4::scale(2.0, 2.0, 2.0);
        let combined = t.mul(&s);
        let p = Vec3::new(1.0, 0.0, 0.0);
        let result = combined.transform_point(p);
        assert!((result.x - 3.0).abs() < 0.0001);
    }
}
