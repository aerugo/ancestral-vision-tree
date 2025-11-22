//! Tree growth animation system
//!
//! Animates the tree from sapling to full tree with:
//! - Sequential branch emergence by generation
//! - Smooth radius and length interpolation
//! - Organic easing curves

use std::collections::HashMap;
use super::easing::{Easing, ease};
use crate::growth::BranchNode;

/// Overall growth animation state
#[derive(Debug, Clone)]
pub struct GrowthAnimation {
    /// Overall progress (0.0 = seed, 1.0 = fully grown)
    pub progress: f32,
    /// Animation duration in seconds
    pub duration: f32,
    /// Current elapsed time
    pub elapsed: f32,
    /// Whether animation is playing
    pub playing: bool,
    /// Whether animation is complete
    pub complete: bool,
    /// Easing function for overall growth
    pub easing: Easing,
    /// Per-generation delay (stagger effect)
    pub generation_delay: f32,
    /// Maximum generation in tree
    pub max_generation: usize,
    /// Per-branch animation states
    branch_states: HashMap<String, BranchAnimState>,
}

/// Animation state for a single branch
#[derive(Debug, Clone, Copy)]
pub struct BranchAnimState {
    /// Branch visibility (0.0 = hidden, 1.0 = fully visible)
    pub visibility: f32,
    /// Length scale (0.0 = no length, 1.0 = full length)
    pub length_scale: f32,
    /// Radius scale (0.0 = no thickness, 1.0 = full thickness)
    pub radius_scale: f32,
    /// Glow intensity multiplier
    pub glow_scale: f32,
    /// Branch generation (depth from root)
    pub generation: usize,
    /// Local progress for this branch (0.0 to 1.0)
    pub local_progress: f32,
}

impl Default for BranchAnimState {
    fn default() -> Self {
        Self {
            visibility: 0.0,
            length_scale: 0.0,
            radius_scale: 0.0,
            glow_scale: 0.0,
            generation: 0,
            local_progress: 0.0,
        }
    }
}

impl BranchAnimState {
    /// Create a fully-grown branch state
    pub fn full() -> Self {
        Self {
            visibility: 1.0,
            length_scale: 1.0,
            radius_scale: 1.0,
            glow_scale: 1.0,
            generation: 0,
            local_progress: 1.0,
        }
    }
}

impl Default for GrowthAnimation {
    fn default() -> Self {
        Self {
            progress: 0.0,
            duration: 5.0, // 5 second growth animation
            elapsed: 0.0,
            playing: false,
            complete: false,
            easing: Easing::Organic,
            generation_delay: 0.15, // 15% delay between generations
            max_generation: 0,
            branch_states: HashMap::new(),
        }
    }
}

impl GrowthAnimation {
    /// Create a new growth animation with specified duration
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    /// Create animation that's already complete (instant growth)
    pub fn instant() -> Self {
        Self {
            progress: 1.0,
            complete: true,
            ..Default::default()
        }
    }

    /// Initialize branch states from tree structure
    pub fn init_from_tree(&mut self, root: &BranchNode) {
        self.branch_states.clear();
        self.max_generation = 0;
        self.collect_branches(root);
    }

    fn collect_branches(&mut self, node: &BranchNode) {
        self.max_generation = self.max_generation.max(node.generation);

        self.branch_states.insert(
            node.person_id.clone(),
            BranchAnimState {
                generation: node.generation,
                ..Default::default()
            },
        );

        for child in &node.children {
            self.collect_branches(child);
        }
    }

    /// Start the growth animation
    pub fn start(&mut self) {
        self.elapsed = 0.0;
        self.progress = 0.0;
        self.playing = true;
        self.complete = false;
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.progress = 0.0;
        self.playing = false;
        self.complete = false;
        for state in self.branch_states.values_mut() {
            state.visibility = 0.0;
            state.length_scale = 0.0;
            state.radius_scale = 0.0;
            state.glow_scale = 0.0;
            state.local_progress = 0.0;
        }
    }

    /// Jump to fully grown state
    pub fn complete_instantly(&mut self) {
        self.progress = 1.0;
        self.playing = false;
        self.complete = true;
        for state in self.branch_states.values_mut() {
            state.visibility = 1.0;
            state.length_scale = 1.0;
            state.radius_scale = 1.0;
            state.glow_scale = 1.0;
            state.local_progress = 1.0;
        }
    }

    /// Update animation state
    pub fn update(&mut self, dt: f32) {
        if !self.playing || self.complete {
            return;
        }

        self.elapsed += dt;
        self.progress = (self.elapsed / self.duration).min(1.0);

        if self.progress >= 1.0 {
            self.playing = false;
            self.complete = true;
            self.progress = 1.0;
        }

        // Update per-branch states
        self.update_branch_states();
    }

    fn update_branch_states(&mut self) {
        let total_gens = self.max_generation + 1;
        let gen_window = 1.0 / total_gens as f32;

        for state in self.branch_states.values_mut() {
            // Calculate when this branch should start growing
            // Earlier generations start earlier
            let gen_start = state.generation as f32 * self.generation_delay;
            let gen_end = gen_start + (1.0 - self.generation_delay * self.max_generation as f32);

            // Calculate local progress for this branch
            let local_t = if self.progress <= gen_start {
                0.0
            } else if self.progress >= gen_end {
                1.0
            } else {
                (self.progress - gen_start) / (gen_end - gen_start)
            };

            // Apply easing
            let eased = ease(local_t, self.easing);
            state.local_progress = eased;

            // Different aspects animate at slightly different rates for organic feel
            state.visibility = eased;
            state.length_scale = ease(local_t * 1.1, Easing::EaseOut).min(1.0);
            state.radius_scale = ease(local_t * 0.9, Easing::EaseInOut).min(1.0);

            // Glow comes in after the branch is mostly grown
            let glow_t = ((local_t - 0.5) * 2.0).max(0.0);
            state.glow_scale = ease(glow_t, Easing::EaseIn);
        }
    }

    /// Get animation state for a specific branch
    pub fn get_branch_state(&self, person_id: &str) -> BranchAnimState {
        if self.complete {
            return BranchAnimState::full();
        }
        self.branch_states
            .get(person_id)
            .copied()
            .unwrap_or_default()
    }

    /// Get overall progress
    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Check if animation is currently playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Set animation to a specific progress value (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.elapsed = self.progress * self.duration;
        self.complete = self.progress >= 1.0;
        self.playing = false;
        self.update_branch_states();
    }
}

/// Apply growth animation to a branch node, returning scaled values
pub struct AnimatedBranch {
    pub length_scale: f32,
    pub radius_scale: f32,
    pub glow_scale: f32,
    pub visible: bool,
}

impl AnimatedBranch {
    pub fn from_state(state: &BranchAnimState, threshold: f32) -> Self {
        Self {
            length_scale: state.length_scale,
            radius_scale: state.radius_scale,
            glow_scale: state.glow_scale,
            visible: state.visibility > threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::VisualParams;
    use crate::math::Vec3;

    fn create_test_tree() -> BranchNode {
        BranchNode {
            person_id: "root".to_string(),
            visual: VisualParams::default(),
            start: Vec3::ZERO,
            end: Vec3::new(0.0, 2.0, 0.0),
            start_direction: Vec3::UP,
            end_direction: Vec3::UP,
            start_radius: 0.3,
            end_radius: 0.2,
            generation: 0,
            children: vec![
                BranchNode {
                    person_id: "child1".to_string(),
                    visual: VisualParams::default(),
                    start: Vec3::new(0.0, 2.0, 0.0),
                    end: Vec3::new(1.0, 3.0, 0.0),
                    start_direction: Vec3::UP,
                    end_direction: Vec3::new(0.5, 0.5, 0.0).normalize(),
                    start_radius: 0.2,
                    end_radius: 0.15,
                    generation: 1,
                    children: vec![],
                },
                BranchNode {
                    person_id: "child2".to_string(),
                    visual: VisualParams::default(),
                    start: Vec3::new(0.0, 2.0, 0.0),
                    end: Vec3::new(-1.0, 3.0, 0.0),
                    start_direction: Vec3::UP,
                    end_direction: Vec3::new(-0.5, 0.5, 0.0).normalize(),
                    start_radius: 0.2,
                    end_radius: 0.15,
                    generation: 1,
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_animation_init() {
        let mut anim = GrowthAnimation::new(5.0);
        let tree = create_test_tree();
        anim.init_from_tree(&tree);

        assert_eq!(anim.max_generation, 1);
        assert!(anim.branch_states.contains_key("root"));
        assert!(anim.branch_states.contains_key("child1"));
        assert!(anim.branch_states.contains_key("child2"));
    }

    #[test]
    fn test_animation_start() {
        let mut anim = GrowthAnimation::new(5.0);
        anim.start();

        assert!(anim.playing);
        assert!(!anim.complete);
        assert_eq!(anim.progress, 0.0);
    }

    #[test]
    fn test_animation_progress() {
        let mut anim = GrowthAnimation::new(1.0); // 1 second duration
        let tree = create_test_tree();
        anim.init_from_tree(&tree);
        anim.start();

        // After half the duration
        anim.update(0.5);
        assert!((anim.progress - 0.5).abs() < 0.01);
        assert!(anim.playing);

        // Root should be partially grown
        let root_state = anim.get_branch_state("root");
        assert!(root_state.visibility > 0.0);
    }

    #[test]
    fn test_animation_completion() {
        let mut anim = GrowthAnimation::new(1.0);
        let tree = create_test_tree();
        anim.init_from_tree(&tree);
        anim.start();

        // Run past completion
        anim.update(1.5);

        assert!(anim.complete);
        assert!(!anim.playing);
        assert_eq!(anim.progress, 1.0);
    }

    #[test]
    fn test_instant_animation() {
        let anim = GrowthAnimation::instant();
        assert!(anim.complete);
        assert_eq!(anim.progress, 1.0);
    }

    #[test]
    fn test_branch_state_full() {
        let anim = GrowthAnimation::instant();
        let state = anim.get_branch_state("any");

        assert_eq!(state.visibility, 1.0);
        assert_eq!(state.length_scale, 1.0);
        assert_eq!(state.radius_scale, 1.0);
    }

    #[test]
    fn test_generation_stagger() {
        let mut anim = GrowthAnimation::new(1.0);
        anim.generation_delay = 0.3; // 30% delay
        let tree = create_test_tree();
        anim.init_from_tree(&tree);
        anim.start();

        // Early in animation - only root should be growing
        anim.update(0.2);
        let root_state = anim.get_branch_state("root");
        let child_state = anim.get_branch_state("child1");

        // Root should be ahead of children
        assert!(root_state.visibility > child_state.visibility);
    }

    #[test]
    fn test_reset() {
        let mut anim = GrowthAnimation::new(1.0);
        let tree = create_test_tree();
        anim.init_from_tree(&tree);
        anim.start();
        anim.update(0.5);
        anim.reset();

        assert!(!anim.playing);
        assert!(!anim.complete);
        assert_eq!(anim.progress, 0.0);

        let state = anim.get_branch_state("root");
        assert_eq!(state.visibility, 0.0);
    }

    #[test]
    fn test_set_progress() {
        let mut anim = GrowthAnimation::new(1.0);
        let tree = create_test_tree();
        anim.init_from_tree(&tree);

        anim.set_progress(0.75);

        assert!((anim.progress - 0.75).abs() < 0.01);
        assert!(!anim.playing);

        let state = anim.get_branch_state("root");
        assert!(state.visibility > 0.5);
    }
}
