//! Visual evaluation framework for AI-assisted development
//!
//! This module provides tools for programmatic evaluation of visual output,
//! enabling TDD-based development of shader and rendering effects.

pub mod metrics;

pub use metrics::{VisualMetrics, analyze_pixels, ColorDistribution, VisualAnalyzer};
