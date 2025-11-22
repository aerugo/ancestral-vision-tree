//! Animation module for tree growth and visual effects
//!
//! Handles the animated growth from sapling to full tree,
//! with smooth easing and per-generation timing.

mod growth_animation;
mod easing;

pub use growth_animation::{GrowthAnimation, BranchAnimState};
pub use easing::{Easing, ease};
