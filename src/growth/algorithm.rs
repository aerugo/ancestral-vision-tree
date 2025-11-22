use crate::data::{FamilyTree, Person, VisualParams};
use std::f32::consts::PI;
use crate::math::Vec3;

/// Parameters controlling tree growth appearance
#[derive(Debug, Clone, Copy)]
pub struct GrowthParams {
    /// Base height for trunk/initial segment
    pub base_height: f32,
    /// Height reduction per generation (multiplier)
    pub height_decay: f32,
    /// Base radius for trunk
    pub base_radius: f32,
    /// Radius reduction per generation (multiplier)
    pub radius_decay: f32,
    /// Angle spread for binary splits (radians)
    pub branch_spread: f32,
    /// Random variation in angles
    pub angle_variance: f32,
    /// Curvature for organic feel (0.0 to 1.0)
    pub curvature: f32,
    /// Vertical tendency (0.0 = horizontal, 1.0 = vertical)
    pub verticality: f32,
    /// Gravitropism - tendency for branches to curve upward (0.0 to 1.0)
    pub gravitropism: f32,
    /// Whether to generate secondary branches (twigs)
    pub generate_twigs: bool,
    /// Number of twigs per branch segment (0-4)
    pub twigs_per_branch: usize,
    /// Minimum generation before twigs appear
    pub twig_min_generation: usize,
    /// Pitch variance - how much branches can tilt up/down (radians)
    pub pitch_variance: f32,
}

impl Default for GrowthParams {
    fn default() -> Self {
        Self {
            base_height: 5.0,           // Taller trunk
            height_decay: 0.68,         // Slower decay for longer branches
            base_radius: 0.5,           // Thicker trunk
            radius_decay: 0.62,         // Slower taper
            branch_spread: std::f32::consts::PI / 3.0, // 60 degrees - wider spread
            angle_variance: 0.35,       // More natural variation
            curvature: 0.45,            // More organic curves
            verticality: 0.25,          // Less vertical, more natural spread
            gravitropism: 0.15,         // Slight upward curve tendency
            generate_twigs: true,       // Add secondary branches
            twigs_per_branch: 2,        // Moderate twig count
            twig_min_generation: 1,     // Twigs after trunk
            pitch_variance: 0.4,        // Allow up/down variation
        }
    }
}

/// A node in the grown tree structure
#[derive(Debug, Clone)]
pub struct BranchNode {
    /// Person associated with this branch
    pub person_id: String,
    /// Visual parameters derived from person
    pub visual: VisualParams,
    /// Start position of branch segment
    pub start: Vec3,
    /// End position of branch segment
    pub end: Vec3,
    /// Direction at start
    pub start_direction: Vec3,
    /// Direction at end
    pub end_direction: Vec3,
    /// Radius at start
    pub start_radius: f32,
    /// Radius at end
    pub end_radius: f32,
    /// Generation (depth from root, 0 = trunk)
    pub generation: usize,
    /// Child branch nodes
    pub children: Vec<BranchNode>,
}

impl BranchNode {
    /// Get all nodes in pre-order (self first, then children)
    pub fn iter_preorder(&self) -> impl Iterator<Item = &BranchNode> {
        PreorderNodeIter { stack: vec![self] }
    }

    /// Total number of nodes in subtree
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

struct PreorderNodeIter<'a> {
    stack: Vec<&'a BranchNode>,
}

impl<'a> Iterator for PreorderNodeIter<'a> {
    type Item = &'a BranchNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}

/// Tree growth algorithm
pub struct TreeGrowth {
    pub params: GrowthParams,
    seed: u32,
}

impl TreeGrowth {
    pub fn new(params: GrowthParams) -> Self {
        Self { params, seed: 42 }
    }

    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Grow a tree structure from a family tree
    pub fn grow(&self, family: &FamilyTree) -> Option<BranchNode> {
        let root = family.root()?;
        Some(self.grow_branch(family, root, Vec3::ZERO, Vec3::UP, 0))
    }

    fn grow_branch(
        &self,
        family: &FamilyTree,
        person: &Person,
        start: Vec3,
        direction: Vec3,
        generation: usize,
    ) -> BranchNode {
        let visual = person.visual_params();
        let params = &self.params;

        // Calculate segment length and radius based on generation and visual params
        let gen_factor = params.height_decay.powi(generation as i32);
        let length = params.base_height * gen_factor * (0.8 + 0.4 * visual.branch_thickness);
        let start_radius = params.base_radius * gen_factor * visual.branch_thickness;
        let end_radius = start_radius * params.radius_decay;

        // Multiple hash values for different random variations
        let hash1 = self.hash_string(&person.id);
        let hash2 = self.hash_string(&format!("{}pitch", person.id));

        // Yaw variation (horizontal angle)
        let yaw_var = (hash1 as f32 / u32::MAX as f32 - 0.5) * params.angle_variance;
        // Pitch variation (vertical angle) - branches can tilt up or down
        let pitch_var = (hash2 as f32 / u32::MAX as f32 - 0.5) * params.pitch_variance;

        // Apply gravitropism - branches tend to curve slightly upward over their length
        let gravitropic_lift = params.gravitropism * (1.0 - gen_factor);

        // Start direction remains as passed in
        // End direction has variations applied
        let mut end_direction = direction;

        // Apply yaw rotation (around up axis)
        end_direction = self.rotate_around_up(end_direction, yaw_var);

        // Apply pitch rotation (tilt up/down)
        end_direction = self.rotate_pitch(end_direction, pitch_var);

        // Apply gravitropism (slight upward tendency)
        end_direction = self.blend_direction(end_direction, Vec3::UP, gravitropic_lift);

        // Blend toward vertical based on verticality parameter (but less than before)
        if generation == 0 {
            // Trunk should be more vertical
            end_direction = self.blend_direction(end_direction, Vec3::UP, params.verticality + 0.4);
        } else {
            end_direction = self.blend_direction(end_direction, Vec3::UP, params.verticality * 0.5);
        }

        end_direction = end_direction.normalize();

        // Calculate end position
        let end = start + end_direction.scale(length);

        // Grow children from family tree data
        let children_data = family.children_of(&person.id);
        let mut children = self.grow_children(family, &children_data, end, end_direction, generation);

        // Generate decorative twigs if enabled and past minimum generation
        if params.generate_twigs && generation >= params.twig_min_generation && children_data.is_empty() {
            let twigs = self.generate_twigs(&person.id, start, end, direction, end_direction,
                                           start_radius, end_radius, generation);
            children.extend(twigs);
        }

        BranchNode {
            person_id: person.id.clone(),
            visual,
            start,
            end,
            start_direction: direction,
            end_direction,
            start_radius,
            end_radius,
            generation,
            children,
        }
    }

    /// Generate decorative twigs along a branch segment
    fn generate_twigs(
        &self,
        parent_id: &str,
        branch_start: Vec3,
        branch_end: Vec3,
        start_dir: Vec3,
        end_dir: Vec3,
        start_radius: f32,
        end_radius: f32,
        generation: usize,
    ) -> Vec<BranchNode> {
        let params = &self.params;
        let mut twigs = Vec::new();

        let num_twigs = params.twigs_per_branch.min(4);
        if num_twigs == 0 {
            return twigs;
        }

        for i in 0..num_twigs {
            // Position along the branch (avoid very start and very end)
            let hash = self.hash_string(&format!("{}twig{}", parent_id, i));
            let t = 0.3 + (hash as f32 / u32::MAX as f32) * 0.5; // 30% to 80% along branch

            let twig_start = branch_start.lerp(&branch_end, t);
            let local_dir = start_dir.lerp(&end_dir, t).normalize();
            let local_radius = start_radius + (end_radius - start_radius) * t;

            // Twig direction - perpendicular to branch with some randomness
            let hash2 = self.hash_string(&format!("{}twig{}dir", parent_id, i));
            let hash3 = self.hash_string(&format!("{}twig{}pitch", parent_id, i));

            // Rotate around the branch axis for radial placement
            let radial_angle = (hash2 as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let pitch_angle = (hash3 as f32 / u32::MAX as f32 - 0.3) * std::f32::consts::PI * 0.4; // Slight upward bias

            let perp = local_dir.perpendicular();
            let twig_dir = self.rotate_around_axis(perp, local_dir, radial_angle);
            let twig_dir = self.blend_direction(twig_dir, Vec3::UP, 0.2 + pitch_angle.abs() * 0.3);
            let twig_dir = twig_dir.normalize();

            // Twig properties - much smaller than main branches
            let twig_length = params.base_height * params.height_decay.powi((generation + 2) as i32) * 0.5;
            let twig_radius = local_radius * 0.3;
            let twig_end = twig_start + twig_dir.scale(twig_length);

            // Create visual params for twig (dimmer than main branches)
            let visual = VisualParams {
                glow_intensity: 0.15,
                color_vibrancy: 0.4,
                branch_thickness: 0.3,
                luminance: 0.2,
                hue_shift: (hash as f32 / u32::MAX as f32) * 360.0,
            };

            twigs.push(BranchNode {
                person_id: format!("{}__twig{}", parent_id, i),
                visual,
                start: twig_start,
                end: twig_end,
                start_direction: twig_dir,
                end_direction: self.blend_direction(twig_dir, Vec3::UP, params.gravitropism),
                start_radius: twig_radius,
                end_radius: twig_radius * 0.4,
                generation: generation + 2,
                children: vec![],
            });
        }

        twigs
    }

    fn grow_children(
        &self,
        family: &FamilyTree,
        children: &[&Person],
        parent_end: Vec3,
        parent_direction: Vec3,
        parent_generation: usize,
    ) -> Vec<BranchNode> {
        let n = children.len();
        if n == 0 {
            return Vec::new();
        }

        let spread = self.params.branch_spread;
        let next_gen = parent_generation + 1;

        // Get perpendicular axes to spread branches in a plane perpendicular to parent
        let (perp1, perp2) = self.get_perpendicular_frame(parent_direction);

        children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                // Get deterministic random values for this child
                let hash1 = self.hash_string(&child.id);
                let hash2 = self.hash_string(&format!("{}spread", child.id));
                let hash3 = self.hash_string(&format!("{}elev", child.id));

                // Random variations
                let spread_var = (hash2 as f32 / u32::MAX as f32 - 0.5) * self.params.angle_variance;
                let elev_var = (hash3 as f32 / u32::MAX as f32 - 0.5) * self.params.pitch_variance * 0.3;

                let direction = if n == 1 {
                    // Single child continues mostly straight with slight deviation
                    let deviation = (hash1 as f32 / u32::MAX as f32 - 0.5) * spread * 0.3;
                    // Tilt slightly away from parent direction
                    let offset = perp1.scale(deviation.sin()) + perp2.scale(elev_var.sin() * 0.5);
                    (parent_direction + offset.scale(0.3)).normalize()
                } else if n == 2 {
                    // Binary split: spread outward from parent direction in opposite directions
                    let angle = if i == 0 {
                        -spread + spread_var
                    } else {
                        spread + spread_var
                    };

                    // Create direction by tilting away from parent in the perpendicular plane
                    let cos_spread = angle.cos();
                    let sin_spread = angle.sin().abs(); // Use absolute value for outward spread

                    // Offset perpendicular to parent direction
                    let radial_offset = if i == 0 { perp1 } else { perp1.scale(-1.0) };
                    let tilt = radial_offset.scale(sin_spread) + perp2.scale(elev_var * 0.5);

                    // Combine: mostly continue parent direction, but spread outward
                    (parent_direction.scale(cos_spread) + tilt).normalize()
                } else {
                    // Multiple children: distribute radially around parent direction
                    let golden_angle = PI * (3.0 - (5.0_f32).sqrt()); // ~137.5 degrees
                    let radial_angle = i as f32 * golden_angle + spread_var;

                    // Calculate offset in the perpendicular plane
                    let sin_spread = spread.sin();
                    let cos_spread = spread.cos();

                    let radial_offset = perp1.scale(radial_angle.cos()) + perp2.scale(radial_angle.sin());
                    let tilt = radial_offset.scale(sin_spread);

                    // Add elevation variation
                    let elev_offset = parent_direction.scale(elev_var * 0.3);

                    (parent_direction.scale(cos_spread) + tilt + elev_offset).normalize()
                };

                self.grow_branch(family, child, parent_end, direction.normalize(), next_gen)
            })
            .collect()
    }

    /// Get two perpendicular vectors to the given direction
    fn get_perpendicular_frame(&self, dir: Vec3) -> (Vec3, Vec3) {
        let dir = dir.normalize();

        // Choose a vector that's not parallel to dir
        let helper = if dir.y.abs() < 0.9 {
            Vec3::UP
        } else {
            Vec3::RIGHT
        };

        let perp1 = dir.cross(&helper).normalize();
        let perp2 = dir.cross(&perp1).normalize();

        (perp1, perp2)
    }

    /// Simple deterministic hash for consistent randomness
    fn hash_string(&self, s: &str) -> u32 {
        let mut h = self.seed;
        for b in s.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u32);
        }
        h
    }

    /// Blend two directions
    fn blend_direction(&self, dir: Vec3, target: Vec3, amount: f32) -> Vec3 {
        dir.lerp(&target, amount).normalize()
    }

    /// Rotate direction around global up axis
    fn rotate_around_up(&self, dir: Vec3, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            dir.x * cos_a - dir.z * sin_a,
            dir.y,
            dir.x * sin_a + dir.z * cos_a,
        )
    }

    /// Rotate direction by pitch angle (tilt up/down)
    fn rotate_pitch(&self, dir: Vec3, angle: f32) -> Vec3 {
        // Get a horizontal axis perpendicular to dir
        let right = Vec3::UP.cross(&dir);
        if right.length() < 0.001 {
            // dir is nearly vertical, use a different axis
            return dir;
        }
        let right = right.normalize();

        // Rotate around this horizontal axis
        self.rotate_around_axis(dir, right, angle)
    }

    /// Rotate a vector around an arbitrary axis using Rodrigues' rotation formula
    fn rotate_around_axis(&self, v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let k = axis.normalize();

        // Rodrigues' rotation formula: v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_cross_v = k.cross(&v);
        let k_dot_v = k.dot(&v);

        v.scale(cos_a) + k_cross_v.scale(sin_a) + k.scale(k_dot_v * (1.0 - cos_a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::FamilyTree;

    const TEST_YAML: &str = r#"
family:
  name: "Test"
  root: "root"
people:
  - id: "root"
    name: "Root"
    biography: "The founder."
    children:
      - "left"
      - "right"
  - id: "left"
    name: "Left Child"
    biography: ""
  - id: "right"
    name: "Right Child"
    biography: "A longer biography that should make this branch more prominent."
"#;

    #[test]
    fn test_grow_basic_tree() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        assert_eq!(tree.person_id, "root");
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn test_tree_starts_at_origin() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        assert_eq!(tree.start, Vec3::ZERO);
    }

    #[test]
    fn test_tree_grows_upward() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        assert!(tree.end.y > tree.start.y);
    }

    #[test]
    fn test_children_branch_differently() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        // Disable twig generation to focus on main branch structure
        let params = GrowthParams {
            generate_twigs: false,
            ..Default::default()
        };
        let growth = TreeGrowth::new(params);
        let tree = growth.grow(&family).unwrap();

        let left = &tree.children[0];
        let right = &tree.children[1];

        // Children should branch to different positions (spread apart in 3D)
        let left_dir = (left.end - left.start).normalize();
        let right_dir = (right.end - right.start).normalize();

        // The dot product should be less than 1.0 (not parallel)
        // and the directions should be different
        let similarity = left_dir.dot(&right_dir);
        assert!(similarity < 0.99, "Children should branch differently, similarity={}", similarity);

        // Also check that they spread apart (end positions should differ)
        let end_diff = left.end.distance(&right.end);
        assert!(end_diff > 0.1, "Children end positions should differ, diff={}", end_diff);
    }

    #[test]
    fn test_visual_params_affect_radius() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        // "right" has longer bio, should have thicker branch
        let left = &tree.children[0];
        let right = &tree.children[1];

        assert!(right.start_radius > left.start_radius);
    }

    #[test]
    fn test_generation_increments() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        assert_eq!(tree.generation, 0);
        assert_eq!(tree.children[0].generation, 1);
    }

    #[test]
    fn test_node_count() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        // Disable twig generation to test core node counting
        let params = GrowthParams {
            generate_twigs: false,
            ..Default::default()
        };
        let growth = TreeGrowth::new(params);
        let tree = growth.grow(&family).unwrap();

        assert_eq!(tree.count(), 3);
    }

    #[test]
    fn test_preorder_iteration() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        // Disable twig generation to test core iteration
        let params = GrowthParams {
            generate_twigs: false,
            ..Default::default()
        };
        let growth = TreeGrowth::new(params);
        let tree = growth.grow(&family).unwrap();

        let ids: Vec<_> = tree.iter_preorder().map(|n| n.person_id.as_str()).collect();
        assert_eq!(ids[0], "root");
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth1 = TreeGrowth::new(GrowthParams::default()).with_seed(123);
        let growth2 = TreeGrowth::new(GrowthParams::default()).with_seed(123);

        let tree1 = growth1.grow(&family).unwrap();
        let tree2 = growth2.grow(&family).unwrap();

        assert_eq!(tree1.end.x, tree2.end.x);
        assert_eq!(tree1.end.y, tree2.end.y);
    }
}
