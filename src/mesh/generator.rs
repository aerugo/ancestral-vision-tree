use crate::growth::BranchNode;
use crate::math::{Vec3, generate_branch_curve};
use super::branch::{Mesh, Vertex, create_ring, connect_rings};

/// Parameters for mesh generation
#[derive(Debug, Clone, Copy)]
pub struct MeshParams {
    /// Radial segments around each ring (more = smoother)
    pub radial_segments: usize,
    /// Length segments per branch (more = smoother curves)
    pub length_segments: usize,
    /// Amount of bark-like displacement
    pub bark_displacement: f32,
    /// Seed for procedural displacement
    pub seed: u32,
}

impl Default for MeshParams {
    fn default() -> Self {
        Self {
            radial_segments: 12,
            length_segments: 8,
            bark_displacement: 0.02,
            seed: 42,
        }
    }
}

/// Generates organic meshes from tree branch structures
pub struct MeshGenerator {
    params: MeshParams,
}

impl MeshGenerator {
    pub fn new(params: MeshParams) -> Self {
        Self { params }
    }

    /// Generate mesh for entire tree
    pub fn generate_tree(&self, root: &BranchNode) -> Mesh {
        let mut mesh = Mesh::new();
        self.generate_branch_recursive(root, &mut mesh);
        mesh.calculate_bounds();
        mesh
    }

    fn generate_branch_recursive(&self, node: &BranchNode, mesh: &mut Mesh) {
        // Generate this branch segment
        self.generate_branch_segment(node, mesh);

        // Generate children
        for child in &node.children {
            self.generate_branch_recursive(child, mesh);
        }

        // If we have children, generate a joint to smooth the transition
        // If no children, generate an organic tip for leaf branches
        if !node.children.is_empty() {
            self.generate_joint(node, mesh);
        } else {
            self.generate_organic_tip(node, mesh);
        }
    }

    /// Generate a single branch segment with smooth interpolation
    fn generate_branch_segment(&self, node: &BranchNode, mesh: &mut Mesh) {
        let params = &self.params;
        let visual = &node.visual;

        // Generate curve points along the branch
        let curve_points = generate_branch_curve(
            node.start,
            node.end,
            node.start_direction,
            node.end_direction,
            0.4, // Curvature
            params.length_segments,
        );

        // Calculate directions along the curve
        let mut directions = Vec::with_capacity(params.length_segments);
        for i in 0..params.length_segments {
            let dir = if i == 0 {
                node.start_direction
            } else if i == params.length_segments - 1 {
                node.end_direction
            } else {
                let prev = curve_points[i - 1];
                let next = curve_points[(i + 1).min(params.length_segments - 1)];
                (next - prev).normalize()
            };
            directions.push(dir);
        }

        // Create rings along the curve
        let mut ring_starts = Vec::with_capacity(params.length_segments);

        for i in 0..params.length_segments {
            let t = i as f32 / (params.length_segments - 1) as f32;

            // Interpolate radius
            let radius = lerp(node.start_radius, node.end_radius, t);

            // Add slight bark displacement
            let displaced_radius = radius + self.bark_noise(i, params.seed) * params.bark_displacement;

            // Create ring
            let ring = create_ring(
                curve_points[i],
                directions[i],
                displaced_radius,
                params.radial_segments,
                t, // v coordinate
                visual.glow_intensity,
                visual.luminance,
                visual.hue_shift,
            );

            let ring_start = mesh.add_vertices(ring);
            ring_starts.push(ring_start);
        }

        // Connect consecutive rings
        for i in 0..(params.length_segments - 1) {
            connect_rings(mesh, ring_starts[i], ring_starts[i + 1], params.radial_segments);
        }
    }

    /// Generate a smooth joint where parent meets children
    fn generate_joint(&self, parent: &BranchNode, mesh: &mut Mesh) {
        let children = &parent.children;
        if children.is_empty() {
            return;
        }

        // For now, we generate a simple dome cap at the end of the parent
        // A more sophisticated approach would blend smoothly into children
        let center = parent.end;
        let direction = parent.end_direction;
        let radius = parent.end_radius;

        // Create a small dome at the end
        let dome_segments = 3;
        let mut prev_ring_start = None;

        for i in 0..=dome_segments {
            let t = i as f32 / dome_segments as f32;
            let dome_radius = radius * (1.0 - t * 0.5).max(0.1);
            let offset = direction.scale(radius * t * 0.3);
            let ring_center = center + offset;

            let ring = create_ring(
                ring_center,
                direction,
                dome_radius,
                self.params.radial_segments,
                1.0 + t * 0.1,
                parent.visual.glow_intensity,
                parent.visual.luminance,
                parent.visual.hue_shift,
            );

            let ring_start = mesh.add_vertices(ring);

            if let Some(prev_start) = prev_ring_start {
                connect_rings(mesh, prev_start, ring_start, self.params.radial_segments);
            }

            prev_ring_start = Some(ring_start);
        }

        // Cap the top with a fan
        if let Some(last_ring) = prev_ring_start {
            let tip = center + direction.scale(radius * 0.5);
            let tip_vertex = Vertex::new(tip, direction)
                .with_uv(0.5, 1.0)
                .with_visual(
                    parent.visual.glow_intensity,
                    parent.visual.luminance,
                    parent.visual.hue_shift,
                );
            let tip_idx = mesh.add_vertices(std::iter::once(tip_vertex));

            for i in 0..self.params.radial_segments {
                let next = (i + 1) % self.params.radial_segments;
                mesh.add_triangle(
                    last_ring + i as u32,
                    last_ring + next as u32,
                    tip_idx,
                );
            }
        }
    }

    /// Simple deterministic noise for bark texture
    fn bark_noise(&self, index: usize, seed: u32) -> f32 {
        let x = (index as u32).wrapping_mul(seed).wrapping_add(12345);
        let x = x ^ (x >> 16);
        let x = x.wrapping_mul(0x85ebca6b);
        let x = x ^ (x >> 13);
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Generate organic tapered tip for leaf branches (no children)
    fn generate_organic_tip(&self, node: &BranchNode, mesh: &mut Mesh) {
        let visual = &node.visual;
        let params = &self.params;

        let tip_length = node.end_radius * 1.5;
        let tip_segments = 4;
        let direction = node.end_direction;
        let mut prev_ring_start = None;
        let start_pos = node.end;
        let start_radius = node.end_radius;

        for i in 0..tip_segments {
            let t = i as f32 / (tip_segments - 1) as f32;
            // Organic taper using smooth curve (not linear)
            let taper = 1.0 - smooth_step(t);
            let radius = start_radius * taper * 0.5;

            let offset = direction.scale(tip_length * t);
            let ring_center = start_pos + offset;

            // Fewer segments for smaller rings
            let seg_count = (params.radial_segments as f32 * (0.5 + 0.5 * taper)) as usize;
            let seg_count = seg_count.max(4);

            let ring = create_ring(
                ring_center,
                direction,
                radius.max(0.01),
                seg_count,
                1.0 + t * 0.2,
                visual.glow_intensity * (1.0 + 0.5 * t), // Brighter at tip
                visual.luminance * (1.0 + 0.3 * t),
                visual.hue_shift,
            );

            let ring_start = mesh.add_vertices(ring);

            if let Some((prev_start, prev_seg_count)) = prev_ring_start {
                // Connect rings even if segment count differs
                connect_rings_adaptive(mesh, prev_start, ring_start, prev_seg_count, seg_count);
            }

            prev_ring_start = Some((ring_start, seg_count));
        }

        // Glowing point at the very tip
        if let Some((last_ring, seg_count)) = prev_ring_start {
            let tip = start_pos + direction.scale(tip_length);
            let tip_vertex = Vertex::new(tip, direction)
                .with_uv(0.5, 1.2)
                .with_visual(
                    visual.glow_intensity * 1.5, // Extra glow at tip
                    visual.luminance * 1.2,
                    visual.hue_shift,
                );
            let tip_idx = mesh.add_vertices(std::iter::once(tip_vertex));

            for i in 0..seg_count {
                let next = (i + 1) % seg_count;
                mesh.add_triangle(
                    last_ring + i as u32,
                    last_ring + next as u32,
                    tip_idx,
                );
            }
        }
    }
}

/// Smooth step function for organic tapering
fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Connect two rings with different segment counts
fn connect_rings_adaptive(
    mesh: &mut Mesh,
    ring1_start: u32,
    ring2_start: u32,
    seg1: usize,
    seg2: usize,
) {
    if seg1 == seg2 {
        connect_rings(mesh, ring1_start, ring2_start, seg1);
        return;
    }

    // Handle different segment counts by interpolating
    let max_seg = seg1.max(seg2);

    for i in 0..max_seg {
        // Map indices from larger ring to smaller
        let i1 = if seg1 >= seg2 {
            i % seg1
        } else {
            (i * seg1 / max_seg) % seg1
        };
        let i1_next = (i1 + 1) % seg1;

        let i2 = if seg2 >= seg1 {
            i % seg2
        } else {
            (i * seg2 / max_seg) % seg2
        };
        let i2_next = (i2 + 1) % seg2;

        let a = ring1_start + i1 as u32;
        let b = ring1_start + i1_next as u32;
        let c = ring2_start + i2_next as u32;
        let d = ring2_start + i2 as u32;

        if a != b && c != d {
            mesh.add_quad(a, d, c, b);
        } else if a != b {
            mesh.add_triangle(a, d, b);
        } else if c != d {
            mesh.add_triangle(a, d, c);
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Per-branch mesh data for picking
#[derive(Debug, Clone)]
pub struct BranchMeshInfo {
    pub person_id: String,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub index_start: u32,
    pub index_count: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}

/// Generate mesh with per-branch tracking for picking
pub struct TrackedMeshGenerator {
    generator: MeshGenerator,
}

impl TrackedMeshGenerator {
    pub fn new(params: MeshParams) -> Self {
        Self {
            generator: MeshGenerator::new(params),
        }
    }

    /// Generate mesh and return branch info for picking
    pub fn generate_tree_tracked(&self, root: &BranchNode) -> (Mesh, Vec<BranchMeshInfo>) {
        let mut mesh = Mesh::new();
        let mut branch_infos = Vec::new();

        self.generate_branch_tracked(root, &mut mesh, &mut branch_infos);
        mesh.calculate_bounds();

        (mesh, branch_infos)
    }

    fn generate_branch_tracked(
        &self,
        node: &BranchNode,
        mesh: &mut Mesh,
        infos: &mut Vec<BranchMeshInfo>,
    ) {
        let vertex_start = mesh.vertices.len() as u32;
        let index_start = mesh.indices.len() as u32;

        // Generate this branch
        self.generator.generate_branch_segment(node, mesh);

        let vertex_count = mesh.vertices.len() as u32 - vertex_start;
        let index_count = mesh.indices.len() as u32 - index_start;

        // Calculate bounds for this branch
        let center = node.start.lerp(&node.end, 0.5);
        let radius = node.start.distance(&node.end) / 2.0 + node.start_radius;

        infos.push(BranchMeshInfo {
            person_id: node.person_id.clone(),
            vertex_start,
            vertex_count,
            index_start,
            index_count,
            bounds_center: center,
            bounds_radius: radius,
        });

        // Generate children
        for child in &node.children {
            self.generate_branch_tracked(child, mesh, infos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{FamilyTree, VisualParams};
    use crate::growth::{TreeGrowth, GrowthParams};

    fn create_simple_node() -> BranchNode {
        BranchNode {
            person_id: "test".to_string(),
            visual: VisualParams::default(),
            start: Vec3::ZERO,
            end: Vec3::new(0.0, 2.0, 0.0),
            start_direction: Vec3::UP,
            end_direction: Vec3::UP,
            start_radius: 0.3,
            end_radius: 0.2,
            generation: 0,
            children: vec![],
        }
    }

    #[test]
    fn test_generate_single_branch() {
        let node = create_simple_node();
        let generator = MeshGenerator::new(MeshParams::default());
        let mesh = generator.generate_tree(&node);

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_mesh_has_correct_structure() {
        let node = create_simple_node();
        let params = MeshParams {
            radial_segments: 8,
            length_segments: 4,
            ..Default::default()
        };
        let generator = MeshGenerator::new(params);
        let mesh = generator.generate_tree(&node);

        // Should have at least 4 rings of 8 vertices each for the branch
        // Plus additional vertices for the organic tip
        assert!(mesh.vertex_count() >= 4 * 8, "Expected at least 32 vertices, got {}", mesh.vertex_count());

        // Should have at least 3 * 8 * 2 triangles for the branch connections
        // Plus additional triangles for the organic tip
        assert!(mesh.triangle_count() >= 3 * 8 * 2, "Expected at least 48 triangles, got {}", mesh.triangle_count());
    }

    #[test]
    fn test_vertex_data_layout() {
        let node = create_simple_node();
        let generator = MeshGenerator::new(MeshParams::default());
        let mesh = generator.generate_tree(&node);

        let data = mesh.vertex_data();
        assert_eq!(data.len() % 11, 0); // Each vertex is 11 floats
    }

    #[test]
    fn test_generate_full_tree() {
        let yaml = r#"
family:
  name: "Test"
  root: "root"
people:
  - id: "root"
    name: "Root"
    children: ["child"]
  - id: "child"
    name: "Child"
"#;
        let family = FamilyTree::from_yaml(yaml).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        let generator = MeshGenerator::new(MeshParams::default());
        let mesh = generator.generate_tree(&tree);

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.bounds_radius > 0.0);
    }

    #[test]
    fn test_tracked_generation() {
        let yaml = r#"
family:
  name: "Test"
  root: "root"
people:
  - id: "root"
    name: "Root"
    children: ["a", "b"]
  - id: "a"
    name: "A"
  - id: "b"
    name: "B"
"#;
        let family = FamilyTree::from_yaml(yaml).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        let generator = TrackedMeshGenerator::new(MeshParams::default());
        let (mesh, infos) = generator.generate_tree_tracked(&tree);

        assert_eq!(infos.len(), 3);
        assert_eq!(infos[0].person_id, "root");
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_branch_bounds_calculated() {
        let yaml = r#"
family:
  name: "Test"
  root: "root"
people:
  - id: "root"
    name: "Root"
"#;
        let family = FamilyTree::from_yaml(yaml).unwrap();
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family).unwrap();

        let generator = TrackedMeshGenerator::new(MeshParams::default());
        let (_, infos) = generator.generate_tree_tracked(&tree);

        assert!(infos[0].bounds_radius > 0.0);
    }
}
