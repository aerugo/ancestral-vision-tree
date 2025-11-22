use crate::math::Vec3;
use crate::growth::BranchNode;

/// A single firefly particle
#[derive(Debug, Clone)]
struct Firefly {
    position: Vec3,
    velocity: Vec3,
    phase: f32,        // Phase offset for flicker
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    color: Vec3,
}

impl Firefly {
    fn new(position: Vec3, seed: u32) -> Self {
        let phase = (seed as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let size = 8.0 + (seed % 100) as f32 * 0.1;
        let lifetime = 2.0 + (seed % 50) as f32 * 0.1;

        // Vary color from greenish to cyan
        let hue = 0.3 + (seed % 1000) as f32 * 0.0002; // 0.3 to 0.5
        let color = hsv_to_rgb(hue, 0.6, 1.0);

        Self {
            position,
            velocity: Vec3::ZERO,
            phase,
            size,
            lifetime,
            max_lifetime: lifetime,
            color,
        }
    }

    fn alpha(&self) -> f32 {
        // Fade in and out
        let t = self.lifetime / self.max_lifetime;
        let fade_in = (t * 3.0).min(1.0);
        let fade_out = ((1.0 - t) * 3.0).min(1.0);
        fade_in * fade_out * 0.8
    }
}

/// System managing multiple firefly particles
pub struct FireflySystem {
    fireflies: Vec<Firefly>,
    max_fireflies: usize,
    spawn_rate: f32,
    spawn_accumulator: f32,
    /// Bounds for spawning (derived from tree)
    bounds_min: Vec3,
    bounds_max: Vec3,
    /// High-luminance positions (attract fireflies)
    attractors: Vec<(Vec3, f32)>, // (position, strength)
    seed: u32,
}

impl FireflySystem {
    pub fn new(max_fireflies: usize) -> Self {
        Self {
            fireflies: Vec::with_capacity(max_fireflies),
            max_fireflies,
            spawn_rate: 10.0,
            spawn_accumulator: 0.0,
            bounds_min: Vec3::new(-3.0, 0.0, -3.0),
            bounds_max: Vec3::new(3.0, 8.0, 3.0),
            attractors: Vec::new(),
            seed: 42,
        }
    }

    /// Configure bounds and attractors from tree
    pub fn configure_from_tree(&mut self, root: &BranchNode) {
        self.attractors.clear();

        // Find bounds and collect high-luminance positions
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for node in root.iter_preorder() {
            // Update bounds
            min.x = min.x.min(node.start.x).min(node.end.x);
            min.y = min.y.min(node.start.y).min(node.end.y);
            min.z = min.z.min(node.start.z).min(node.end.z);
            max.x = max.x.max(node.start.x).max(node.end.x);
            max.y = max.y.max(node.start.y).max(node.end.y);
            max.z = max.z.max(node.start.z).max(node.end.z);

            // Add attractor at branch midpoint with strength based on luminance
            if node.visual.luminance > 0.5 {
                let mid = node.start.lerp(&node.end, 0.5);
                self.attractors.push((mid, node.visual.luminance));
            }
        }

        // Expand bounds slightly
        let margin = Vec3::new(2.0, 1.0, 2.0);
        self.bounds_min = min - margin;
        self.bounds_max = max + margin;
    }

    /// Update the particle system
    pub fn update(&mut self, dt: f32, time: f32) {
        // Spawn new fireflies
        self.spawn_accumulator += dt * self.spawn_rate;
        while self.spawn_accumulator >= 1.0 && self.fireflies.len() < self.max_fireflies {
            self.spawn_firefly();
            self.spawn_accumulator -= 1.0;
        }

        // Update existing fireflies
        for firefly in &mut self.fireflies {
            // Update lifetime
            firefly.lifetime -= dt;

            // Calculate wandering velocity using Perlin-like noise
            let noise_x = simplex_noise(firefly.position.x * 0.5, time * 0.3 + firefly.phase);
            let noise_y = simplex_noise(firefly.position.y * 0.5, time * 0.2 + firefly.phase + 100.0);
            let noise_z = simplex_noise(firefly.position.z * 0.5, time * 0.25 + firefly.phase + 200.0);

            let wander = Vec3::new(noise_x, noise_y * 0.5, noise_z);

            // Attraction to high-luminance branches
            let mut attraction = Vec3::ZERO;
            for (pos, strength) in &self.attractors {
                let to_attractor = *pos - firefly.position;
                let dist = to_attractor.length();
                if dist > 0.5 && dist < 5.0 {
                    attraction = attraction + to_attractor.normalize().scale(*strength * 0.3 / dist);
                }
            }

            // Update velocity with damping
            firefly.velocity = firefly.velocity.scale(0.95) + wander.scale(0.5) + attraction.scale(dt);

            // Clamp velocity
            let speed = firefly.velocity.length();
            if speed > 2.0 {
                firefly.velocity = firefly.velocity.scale(2.0 / speed);
            }

            // Update position
            firefly.position = firefly.position + firefly.velocity.scale(dt);

            // Soft boundary constraints
            firefly.position.x = soft_clamp(firefly.position.x, self.bounds_min.x, self.bounds_max.x);
            firefly.position.y = soft_clamp(firefly.position.y, self.bounds_min.y, self.bounds_max.y);
            firefly.position.z = soft_clamp(firefly.position.z, self.bounds_min.z, self.bounds_max.z);
        }

        // Remove dead fireflies
        self.fireflies.retain(|f| f.lifetime > 0.0);
    }

    fn spawn_firefly(&mut self) {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);

        // Random position within bounds
        let t_x = (self.seed % 10000) as f32 / 10000.0;
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let t_y = (self.seed % 10000) as f32 / 10000.0;
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let t_z = (self.seed % 10000) as f32 / 10000.0;

        let position = Vec3::new(
            lerp(self.bounds_min.x, self.bounds_max.x, t_x),
            lerp(self.bounds_min.y, self.bounds_max.y, t_y),
            lerp(self.bounds_min.z, self.bounds_max.z, t_z),
        );

        self.fireflies.push(Firefly::new(position, self.seed));
    }

    /// Get particle data for GPU upload
    /// Format: position(3) + size(1) + alpha(1) + color(3) = 8 floats per particle
    pub fn get_particle_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.fireflies.len() * 8);

        for f in &self.fireflies {
            data.push(f.position.x);
            data.push(f.position.y);
            data.push(f.position.z);
            data.push(f.size);
            data.push(f.alpha());
            data.push(f.color.x);
            data.push(f.color.y);
            data.push(f.color.z);
        }

        data
    }

    pub fn count(&self) -> usize {
        self.fireflies.len()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn soft_clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min + (value - min) * 0.1
    } else if value > max {
        max + (value - max) * 0.1
    } else {
        value
    }
}

/// Simple simplex-like noise
fn simplex_noise(x: f32, y: f32) -> f32 {
    let x = x + y * 0.5;
    let y = y + x * 0.3;

    let fx = x.fract();
    let fy = y.fract();

    let h00 = hash2d(x.floor() as i32, y.floor() as i32);
    let h10 = hash2d(x.floor() as i32 + 1, y.floor() as i32);
    let h01 = hash2d(x.floor() as i32, y.floor() as i32 + 1);
    let h11 = hash2d(x.floor() as i32 + 1, y.floor() as i32 + 1);

    let u = fx * fx * (3.0 - 2.0 * fx);
    let v = fy * fy * (3.0 - 2.0 * fy);

    let a = lerp(h00, h10, u);
    let b = lerp(h01, h11, u);

    lerp(a, b, v) * 2.0 - 1.0
}

fn hash2d(x: i32, y: i32) -> f32 {
    let n = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n as u32 as f32) / (u32::MAX as f32)
}

/// HSV to RGB conversion
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Vec3 {
    let h = h * 6.0;
    let i = h.floor() as i32;
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i % 6 {
        0 => Vec3::new(v, t, p),
        1 => Vec3::new(q, v, p),
        2 => Vec3::new(p, v, t),
        3 => Vec3::new(p, q, v),
        4 => Vec3::new(t, p, v),
        _ => Vec3::new(v, p, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firefly_system_creation() {
        let system = FireflySystem::new(100);
        assert_eq!(system.count(), 0);
    }

    #[test]
    fn test_firefly_spawn() {
        let mut system = FireflySystem::new(100);
        system.update(1.0, 0.0);
        assert!(system.count() > 0);
    }

    #[test]
    fn test_particle_data_format() {
        let mut system = FireflySystem::new(10);
        system.update(0.5, 0.0);

        let data = system.get_particle_data();
        assert_eq!(data.len() % 8, 0);
    }

    #[test]
    fn test_firefly_lifetime() {
        let mut system = FireflySystem::new(50);
        system.spawn_rate = 100.0;

        // Spawn many fireflies
        system.update(1.0, 0.0);
        let initial_count = system.count();
        assert!(initial_count > 0);

        // Let them die
        for i in 0..100 {
            system.spawn_rate = 0.0; // Stop spawning
            system.update(0.1, i as f32 * 0.1);
        }

        assert!(system.count() < initial_count);
    }

    #[test]
    fn test_hsv_to_rgb() {
        // Red
        let red = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((red.x - 1.0).abs() < 0.01);
        assert!(red.y.abs() < 0.01);
        assert!(red.z.abs() < 0.01);

        // Green
        let green = hsv_to_rgb(1.0 / 3.0, 1.0, 1.0);
        assert!(green.x.abs() < 0.01);
        assert!((green.y - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_noise_range() {
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let y = i as f32 * 0.07;
            let n = simplex_noise(x, y);
            assert!(n >= -1.0 && n <= 1.0);
        }
    }
}
