use crate::data::{FamilyTree, Person, VisualParams};
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
}

impl Default for GrowthParams {
    fn default() -> Self {
        Self {
            base_height: 3.0,
            height_decay: 0.75,
            base_radius: 0.3,
            radius_decay: 0.7,
            branch_spread: std::f32::consts::PI / 4.0, // 45 degrees
            angle_variance: 0.1,
            curvature: 0.3,
            verticality: 0.6,
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

        // Add slight random variation for organic feel
        let hash = self.hash_string(&person.id);
        let angle_var = (hash as f32 / u32::MAX as f32 - 0.5) * params.angle_variance;

        // Adjust direction with some upward bias
        let end_direction = self.blend_direction(direction, Vec3::UP, params.verticality);
        let end_direction = self.rotate_slightly(end_direction, angle_var);

        // Calculate end position
        let end = start + end_direction.scale(length);

        // Grow children
        let children_data = family.children_of(&person.id);
        let children = self.grow_children(family, &children_data, end, end_direction, generation);

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

        children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let direction = if n == 1 {
                    // Single child continues mostly straight with slight deviation
                    let hash = self.hash_string(&child.id);
                    let deviation = (hash as f32 / u32::MAX as f32 - 0.5) * spread * 0.3;
                    self.rotate_around_up(parent_direction, deviation)
                } else {
                    // Multiple children: spread them out
                    let angle = if n == 2 {
                        // Binary split: left and right
                        if i == 0 { -spread } else { spread }
                    } else {
                        // Multiple: evenly spaced
                        let t = i as f32 / (n - 1) as f32;
                        spread * (t * 2.0 - 1.0)
                    };
                    self.rotate_around_up(parent_direction, angle)
                };

                self.grow_branch(family, child, parent_end, direction.normalize(), next_gen)
            })
            .collect()
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

    /// Rotate a direction slightly
    fn rotate_slightly(&self, dir: Vec3, angle: f32) -> Vec3 {
        let perp = dir.perpendicular();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        (dir.scale(cos_a) + perp.scale(sin_a)).normalize()
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
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        let left = &tree.children[0];
        let right = &tree.children[1];

        // Children should branch to different positions (spread apart)
        let left_offset = left.end - left.start;
        let right_offset = right.end - right.start;
        // Their x or z components should differ due to branching
        let diff = (left_offset.x - right_offset.x).abs() + (left_offset.z - right_offset.z).abs();
        assert!(diff > 0.01, "Children should branch differently, diff={}", diff);
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
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        assert_eq!(tree.count(), 3);
    }

    #[test]
    fn test_preorder_iteration() {
        let family = FamilyTree::from_yaml(TEST_YAML).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
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
