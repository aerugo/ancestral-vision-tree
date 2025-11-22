use crate::math::Vec3;

/// A vertex with position, normal, UV, and custom attributes
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
    /// Glow intensity for this vertex (for shader)
    pub glow: f32,
    /// Luminance for bioluminescence
    pub luminance: f32,
    /// Hue shift for color variation
    pub hue: f32,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position,
            normal,
            uv: [0.0, 0.0],
            glow: 0.3,
            luminance: 0.3,
            hue: 0.0,
        }
    }

    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.uv = [u, v];
        self
    }

    pub fn with_visual(mut self, glow: f32, luminance: f32, hue: f32) -> Self {
        self.glow = glow;
        self.luminance = luminance;
        self.hue = hue;
        self
    }

    /// Convert to flat array for WebGL buffer
    /// Layout: position(3) + normal(3) + uv(2) + glow(1) + luminance(1) + hue(1) = 11 floats
    pub fn to_array(&self) -> [f32; 11] {
        [
            self.position.x, self.position.y, self.position.z,
            self.normal.x, self.normal.y, self.normal.z,
            self.uv[0], self.uv[1],
            self.glow, self.luminance, self.hue,
        ]
    }
}

/// A mesh composed of vertices and triangle indices
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// Bounding sphere for picking
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add vertices and return the starting index
    pub fn add_vertices(&mut self, verts: impl IntoIterator<Item = Vertex>) -> u32 {
        let start = self.vertices.len() as u32;
        self.vertices.extend(verts);
        start
    }

    /// Add a triangle (indices are relative to the mesh's vertex buffer)
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    /// Add a quad as two triangles (CCW winding)
    pub fn add_quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        // First triangle: a, b, c
        self.add_triangle(a, b, c);
        // Second triangle: a, c, d
        self.add_triangle(a, c, d);
    }

    /// Merge another mesh into this one
    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertices.len() as u32;
        self.vertices.extend(other.vertices.iter().cloned());
        for idx in &other.indices {
            self.indices.push(idx + offset);
        }
    }

    /// Calculate bounding sphere
    pub fn calculate_bounds(&mut self) {
        if self.vertices.is_empty() {
            self.bounds_center = Vec3::ZERO;
            self.bounds_radius = 0.0;
            return;
        }

        // Find centroid
        let mut center = Vec3::ZERO;
        for v in &self.vertices {
            center = center + v.position;
        }
        center = center.scale(1.0 / self.vertices.len() as f32);

        // Find max distance from centroid
        let mut max_dist = 0.0f32;
        for v in &self.vertices {
            let dist = v.position.distance(&center);
            max_dist = max_dist.max(dist);
        }

        self.bounds_center = center;
        self.bounds_radius = max_dist;
    }

    /// Get vertex buffer data as flat f32 array
    pub fn vertex_data(&self) -> Vec<f32> {
        self.vertices
            .iter()
            .flat_map(|v| v.to_array())
            .collect()
    }

    /// Get index data
    pub fn index_data(&self) -> &[u32] {
        &self.indices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Create a ring of vertices at a given position/direction/radius
pub fn create_ring(
    center: Vec3,
    direction: Vec3,
    radius: f32,
    segments: usize,
    v_coord: f32,
    glow: f32,
    luminance: f32,
    hue: f32,
) -> Vec<Vertex> {
    let tangent = direction.perpendicular();
    let bitangent = direction.cross(&tangent).normalize();

    (0..segments)
        .map(|i| {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let offset = tangent.scale(cos_a * radius) + bitangent.scale(sin_a * radius);
            let position = center + offset;
            let normal = offset.normalize();
            let u = i as f32 / segments as f32;

            Vertex::new(position, normal)
                .with_uv(u, v_coord)
                .with_visual(glow, luminance, hue)
        })
        .collect()
}

/// Connect two rings with triangles
pub fn connect_rings(mesh: &mut Mesh, ring1_start: u32, ring2_start: u32, segments: usize) {
    for i in 0..segments {
        let i_next = (i + 1) % segments;

        let a = ring1_start + i as u32;
        let b = ring1_start + i_next as u32;
        let c = ring2_start + i_next as u32;
        let d = ring2_start + i as u32;

        mesh.add_quad(a, d, c, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_to_array() {
        let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0), Vec3::UP)
            .with_uv(0.5, 0.5)
            .with_visual(0.8, 0.6, 120.0);

        let arr = v.to_array();
        assert_eq!(arr.len(), 11);
        assert_eq!(arr[0], 1.0); // position.x
        assert_eq!(arr[4], 1.0); // normal.y (UP)
        assert_eq!(arr[6], 0.5); // uv.u
        assert_eq!(arr[8], 0.8); // glow
    }

    #[test]
    fn test_mesh_add_vertices() {
        let mut mesh = Mesh::new();
        let verts = vec![
            Vertex::new(Vec3::ZERO, Vec3::UP),
            Vertex::new(Vec3::RIGHT, Vec3::UP),
            Vertex::new(Vec3::UP, Vec3::UP),
        ];
        let start = mesh.add_vertices(verts);
        assert_eq!(start, 0);
        assert_eq!(mesh.vertex_count(), 3);
    }

    #[test]
    fn test_mesh_add_triangle() {
        let mut mesh = Mesh::new();
        mesh.add_vertices(vec![
            Vertex::new(Vec3::ZERO, Vec3::UP),
            Vertex::new(Vec3::RIGHT, Vec3::UP),
            Vertex::new(Vec3::UP, Vec3::UP),
        ]);
        mesh.add_triangle(0, 1, 2);
        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_mesh_merge() {
        let mut mesh1 = Mesh::new();
        mesh1.add_vertices(vec![Vertex::new(Vec3::ZERO, Vec3::UP)]);
        mesh1.add_triangle(0, 0, 0);

        let mut mesh2 = Mesh::new();
        mesh2.add_vertices(vec![Vertex::new(Vec3::UP, Vec3::UP)]);
        mesh2.add_triangle(0, 0, 0);

        mesh1.merge(&mesh2);
        assert_eq!(mesh1.vertex_count(), 2);
        assert_eq!(mesh1.indices[3], 1); // Index offset applied
    }

    #[test]
    fn test_create_ring() {
        let ring = create_ring(Vec3::ZERO, Vec3::UP, 1.0, 8, 0.0, 0.5, 0.5, 0.0);
        assert_eq!(ring.len(), 8);

        // All vertices should be at distance 1 from center in XZ plane
        for v in &ring {
            let dist = (v.position.x.powi(2) + v.position.z.powi(2)).sqrt();
            assert!((dist - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_connect_rings() {
        let mut mesh = Mesh::new();
        let ring1 = create_ring(Vec3::ZERO, Vec3::UP, 1.0, 4, 0.0, 0.5, 0.5, 0.0);
        let ring2 = create_ring(Vec3::UP, Vec3::UP, 0.8, 4, 1.0, 0.5, 0.5, 0.0);

        let start1 = mesh.add_vertices(ring1);
        let start2 = mesh.add_vertices(ring2);
        connect_rings(&mut mesh, start1, start2, 4);

        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 8); // 4 quads = 8 triangles
    }

    #[test]
    fn test_calculate_bounds() {
        let mut mesh = Mesh::new();
        mesh.add_vertices(vec![
            Vertex::new(Vec3::new(-1.0, 0.0, 0.0), Vec3::UP),
            Vertex::new(Vec3::new(1.0, 0.0, 0.0), Vec3::UP),
            Vertex::new(Vec3::new(0.0, 2.0, 0.0), Vec3::UP),
        ]);
        mesh.calculate_bounds();

        assert!(mesh.bounds_radius > 0.0);
    }

    #[test]
    fn test_vertex_data_flat() {
        let mut mesh = Mesh::new();
        mesh.add_vertices(vec![
            Vertex::new(Vec3::ZERO, Vec3::UP),
            Vertex::new(Vec3::RIGHT, Vec3::UP),
        ]);

        let data = mesh.vertex_data();
        assert_eq!(data.len(), 22); // 2 vertices * 11 floats
    }
}
