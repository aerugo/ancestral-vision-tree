//! Visual evaluation framework for AI-assisted development
//!
//! This module provides tools for programmatic evaluation of visual output,
//! enabling TDD-based development of shader and rendering effects.

pub mod metrics;
pub mod tests;

pub use metrics::{VisualMetrics, analyze_pixels, ColorDistribution, VisualAnalyzer};
pub use tests::{VisualCriteria, check_visual_criteria, generate_visual_report};
