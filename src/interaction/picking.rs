use crate::math::{Vec3, Mat4};
use crate::mesh::generator::BranchMeshInfo;

/// Information about a ray-branch intersection
#[derive(Debug, Clone)]
pub struct HitInfo {
    pub person_id: String,
    pub distance: f32,
    pub hit_point: Vec3,
}

/// Ray-based picking for selecting branches
pub struct RayPicker {
    /// Cached branch bounds for efficient picking
    branch_bounds: Vec<BranchMeshInfo>,
}

impl RayPicker {
    pub fn new() -> Self {
        Self {
            branch_bounds: Vec::new(),
        }
    }

    /// Set branch bounds for picking
    pub fn set_branches(&mut self, branches: Vec<BranchMeshInfo>) {
        self.branch_bounds = branches;
    }

    /// Cast a ray from screen coordinates and find the closest hit
    pub fn pick(
        &self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
        view: &Mat4,
        projection: &Mat4,
        camera_pos: Vec3,
    ) -> Option<HitInfo> {
        // Convert screen to normalized device coordinates
        let ndc_x = (2.0 * screen_x / screen_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / screen_height);

        // Calculate ray direction
        let ray_dir = self.screen_to_world_ray(ndc_x, ndc_y, view, projection);

        // Test against all branches
        let mut closest: Option<HitInfo> = None;
        let mut min_dist = f32::MAX;

        for branch in &self.branch_bounds {
            if let Some(dist) = self.ray_sphere_intersect(
                camera_pos,
                ray_dir,
                branch.bounds_center,
                branch.bounds_radius,
            ) {
                if dist < min_dist {
                    min_dist = dist;
                    closest = Some(HitInfo {
                        person_id: branch.person_id.clone(),
                        distance: dist,
                        hit_point: camera_pos + ray_dir.scale(dist),
                    });
                }
            }
        }

        closest
    }

    /// Convert screen coordinates to world ray direction
    fn screen_to_world_ray(&self, ndc_x: f32, ndc_y: f32, view: &Mat4, projection: &Mat4) -> Vec3 {
        // Inverse projection and view matrices
        let inv_proj = invert_perspective(projection);
        let inv_view = invert_view(view);

        // Create ray in clip space
        let ray_clip = Vec3::new(ndc_x, ndc_y, -1.0);

        // To view space
        let ray_view = inv_proj.transform_point(ray_clip);
        let ray_view = Vec3::new(ray_view.x, ray_view.y, -1.0);

        // To world space
        let ray_world = inv_view.transform_direction(ray_view);

        ray_world.normalize()
    }

    /// Ray-sphere intersection test
    fn ray_sphere_intersect(
        &self,
        ray_origin: Vec3,
        ray_dir: Vec3,
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Option<f32> {
        let oc = ray_origin - sphere_center;

        let a = ray_dir.dot(&ray_dir);
        let b = 2.0 * oc.dot(&ray_dir);
        let c = oc.dot(&oc) - sphere_radius * sphere_radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);

        if t > 0.0 {
            Some(t)
        } else {
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
            if t2 > 0.0 {
                Some(t2)
            } else {
                None
            }
        }
    }
}

impl Default for RayPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Approximate inverse of a perspective matrix
fn invert_perspective(m: &Mat4) -> Mat4 {
    // For a standard perspective matrix, we can compute inverse directly
    let a = m.data[0];
    let b = m.data[5];
    let c = m.data[10];
    let d = m.data[14];
    let e = m.data[11];

    let mut inv = Mat4::identity();
    inv.data[0] = 1.0 / a;
    inv.data[5] = 1.0 / b;
    inv.data[10] = 0.0;
    inv.data[11] = 1.0 / d;
    inv.data[14] = 1.0 / e;
    inv.data[15] = -c / (d * e);

    inv
}

/// Approximate inverse of a view matrix (rotation + translation)
fn invert_view(m: &Mat4) -> Mat4 {
    // For orthonormal view matrix, inverse is transpose of rotation + negated translation
    let mut inv = Mat4::identity();

    // Transpose rotation part
    inv.data[0] = m.data[0];
    inv.data[1] = m.data[4];
    inv.data[2] = m.data[8];

    inv.data[4] = m.data[1];
    inv.data[5] = m.data[5];
    inv.data[6] = m.data[9];

    inv.data[8] = m.data[2];
    inv.data[9] = m.data[6];
    inv.data[10] = m.data[10];

    // Inverse translation
    let tx = m.data[12];
    let ty = m.data[13];
    let tz = m.data[14];

    inv.data[12] = -(inv.data[0] * tx + inv.data[4] * ty + inv.data[8] * tz);
    inv.data[13] = -(inv.data[1] * tx + inv.data[5] * ty + inv.data[9] * tz);
    inv.data[14] = -(inv.data[2] * tx + inv.data[6] * ty + inv.data[10] * tz);

    inv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker_creation() {
        let picker = RayPicker::new();
        assert!(picker.branch_bounds.is_empty());
    }

    #[test]
    fn test_set_branches() {
        let mut picker = RayPicker::new();
        let branches = vec![
            BranchMeshInfo {
                person_id: "test".to_string(),
                vertex_start: 0,
                vertex_count: 10,
                index_start: 0,
                index_count: 30,
                bounds_center: Vec3::new(0.0, 2.0, 0.0),
                bounds_radius: 1.0,
            },
        ];
        picker.set_branches(branches);
        assert_eq!(picker.branch_bounds.len(), 1);
    }

    #[test]
    fn test_ray_sphere_hit() {
        let picker = RayPicker::new();

        // Ray pointing directly at sphere
        let result = picker.ray_sphere_intersect(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::ZERO,
            1.0,
        );

        assert!(result.is_some());
        let dist = result.unwrap();
        assert!((dist - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_ray_sphere_miss() {
        let picker = RayPicker::new();

        // Ray pointing away from sphere
        let result = picker.ray_sphere_intersect(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::ZERO,
            1.0,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_ray_sphere_offset() {
        let picker = RayPicker::new();

        // Ray that misses by going to the side
        let result = picker.ray_sphere_intersect(
            Vec3::new(5.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::ZERO,
            1.0,
        );

        assert!(result.is_none());
    }
}
