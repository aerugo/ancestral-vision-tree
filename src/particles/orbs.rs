//! Floating glowing orbs particle system
//!
//! Creates ethereal, bioluminescent orbs that float around the tree,
//! attracted to branches with high luminance (long biographies).

use crate::math::Vec3;
use crate::growth::BranchNode;

/// A single glowing orb particle
#[derive(Debug, Clone)]
struct Orb {
    position: Vec3,
    velocity: Vec3,
    phase: f32,        // Phase offset for pulsing
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    base_color: Vec3,
    orbit_center: Vec3, // Point to orbit around
    orbit_radius: f32,
    orbit_speed: f32,
}

impl Orb {
    fn new(position: Vec3, orbit_center: Vec3, seed: u32) -> Self {
        let phase = (seed as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let size = 15.0 + (seed % 100) as f32 * 0.15; // Larger than fireflies
        let lifetime = 4.0 + (seed % 30) as f32 * 0.1; // Longer lifetime
        let orbit_radius = 0.3 + (seed % 50) as f32 * 0.02;
        let orbit_speed = 0.5 + (seed % 100) as f32 * 0.01;

        // Vary color from warm amber to cool cyan
        let hue = 0.1 + (seed % 1000) as f32 * 0.0004; // 0.1 to 0.5
        let color = hsv_to_rgb(hue, 0.4, 1.0); // Less saturated, more ethereal

        Self {
            position,
            velocity: Vec3::ZERO,
            phase,
            size,
            lifetime,
            max_lifetime: lifetime,
            base_color: color,
            orbit_center,
            orbit_radius,
            orbit_speed,
        }
    }

    fn alpha(&self) -> f32 {
        // Smooth fade in and out
        let t = self.lifetime / self.max_lifetime;
        let fade_in = (t * 2.0).min(1.0);
        let fade_out = ((1.0 - t) * 2.0).min(1.0);
        fade_in * fade_out * 0.6 // More transparent than fireflies
    }
}

/// System managing ethereal glowing orbs
pub struct OrbSystem {
    orbs: Vec<Orb>,
    max_orbs: usize,
    spawn_rate: f32,
    spawn_accumulator: f32,
    /// High-luminance branch positions as attractors
    attractors: Vec<OrbAttractor>,
    seed: u32,
    activity_scale: f32,
}

/// An attractor point derived from high-luminance branches
#[derive(Debug, Clone)]
struct OrbAttractor {
    position: Vec3,
    luminance: f32,
    #[allow(dead_code)] // Reserved for future hover interactions
    person_id: String,
}

impl OrbSystem {
    pub fn new(max_orbs: usize) -> Self {
        Self {
            orbs: Vec::with_capacity(max_orbs),
            max_orbs,
            spawn_rate: 3.0, // Slower spawn than fireflies
            spawn_accumulator: 0.0,
            attractors: Vec::new(),
            seed: 12345,
            activity_scale: 1.0,
        }
    }

    /// Configure attractors from tree
    pub fn configure_from_tree(&mut self, root: &BranchNode) {
        self.attractors.clear();

        // Collect high-luminance positions
        for node in root.iter_preorder() {
            // Only create attractors for high-luminance branches (long biographies)
            if node.visual.luminance > 0.6 {
                let mid = node.start.lerp(&node.end, 0.5);
                self.attractors.push(OrbAttractor {
                    position: mid,
                    luminance: node.visual.luminance,
                    person_id: node.person_id.clone(),
                });
            }
        }
    }

    /// Set activity scale based on tree growth
    pub fn set_activity_scale(&mut self, scale: f32) {
        self.activity_scale = scale.clamp(0.0, 1.0);
    }

    /// Update the orb system
    pub fn update(&mut self, dt: f32, time: f32) {
        // Scale spawn rate and max by activity
        let effective_spawn_rate = self.spawn_rate * self.activity_scale;
        let effective_max = ((self.max_orbs as f32) * self.activity_scale) as usize;

        // Spawn new orbs near attractors
        self.spawn_accumulator += dt * effective_spawn_rate;
        while self.spawn_accumulator >= 1.0 && self.orbs.len() < effective_max && !self.attractors.is_empty() {
            self.spawn_orb();
            self.spawn_accumulator -= 1.0;
        }

        // Update existing orbs
        for orb in &mut self.orbs {
            orb.lifetime -= dt;

            // Orbital motion around attractor
            let orbit_angle = time * orb.orbit_speed + orb.phase;
            let orbit_offset = Vec3::new(
                orbit_angle.cos() * orb.orbit_radius,
                (orbit_angle * 1.3).sin() * orb.orbit_radius * 0.5,
                orbit_angle.sin() * orb.orbit_radius,
            );

            // Slow drift toward orbit center
            let target = orb.orbit_center + orbit_offset;
            let to_target = target - orb.position;
            orb.velocity = orb.velocity.scale(0.95) + to_target.scale(0.5);

            // Gentle upward drift
            orb.velocity.y += 0.1 * dt;

            // Clamp velocity
            let speed = orb.velocity.length();
            if speed > 1.0 {
                orb.velocity = orb.velocity.scale(1.0 / speed);
            }

            // Update position
            orb.position = orb.position + orb.velocity.scale(dt);
        }

        // Remove dead orbs
        self.orbs.retain(|o| o.lifetime > 0.0);
    }

    fn spawn_orb(&mut self) {
        if self.attractors.is_empty() {
            return;
        }

        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);

        // Choose attractor weighted by luminance
        let total_luminance: f32 = self.attractors.iter().map(|a| a.luminance).sum();
        let mut choice = (self.seed as f32 / u32::MAX as f32) * total_luminance;
        let mut chosen_attractor = &self.attractors[0];

        for attractor in &self.attractors {
            choice -= attractor.luminance;
            if choice <= 0.0 {
                chosen_attractor = attractor;
                break;
            }
        }

        // Spawn near the attractor
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let offset_x = ((self.seed % 1000) as f32 / 500.0 - 1.0) * 0.5;
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let offset_y = ((self.seed % 1000) as f32 / 500.0 - 1.0) * 0.5;
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let offset_z = ((self.seed % 1000) as f32 / 500.0 - 1.0) * 0.5;

        let position = chosen_attractor.position + Vec3::new(offset_x, offset_y, offset_z);

        self.orbs.push(Orb::new(position, chosen_attractor.position, self.seed));
    }

    /// Get particle data for GPU upload
    /// Format: position(3) + size(1) + alpha(1) + color(3) = 8 floats per orb
    pub fn get_particle_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.orbs.len() * 8);

        for orb in &self.orbs {
            // Pulsing size effect
            let pulse = (orb.phase + orb.lifetime * 2.0).sin() * 0.3 + 1.0;
            let size = orb.size * pulse;

            data.push(orb.position.x);
            data.push(orb.position.y);
            data.push(orb.position.z);
            data.push(size);
            data.push(orb.alpha());
            data.push(orb.base_color.x);
            data.push(orb.base_color.y);
            data.push(orb.base_color.z);
        }

        data
    }

    pub fn count(&self) -> usize {
        self.orbs.len()
    }
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
    fn test_orb_system_creation() {
        let system = OrbSystem::new(50);
        assert_eq!(system.count(), 0);
    }

    #[test]
    fn test_orb_system_needs_attractors() {
        let mut system = OrbSystem::new(50);
        // Without attractors, no orbs should spawn
        system.update(1.0, 0.0);
        assert_eq!(system.count(), 0);
    }

    #[test]
    fn test_particle_data_format() {
        let mut system = OrbSystem::new(10);

        // Add a dummy attractor manually for testing
        system.attractors.push(OrbAttractor {
            position: Vec3::new(0.0, 2.0, 0.0),
            luminance: 0.9,
            person_id: "test".to_string(),
        });

        system.update(1.0, 0.0);

        let data = system.get_particle_data();
        assert_eq!(data.len() % 8, 0);
    }

    #[test]
    fn test_activity_scale() {
        let mut system = OrbSystem::new(50);
        system.set_activity_scale(0.5);
        assert!((system.activity_scale - 0.5).abs() < 0.001);

        system.set_activity_scale(1.5); // Should clamp
        assert!((system.activity_scale - 1.0).abs() < 0.001);
    }
}
