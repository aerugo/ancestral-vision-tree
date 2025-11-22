//! Visual regression tests
//!
//! These tests verify that the rendering produces expected visual characteristics
//! based on the input family data.

use super::metrics::VisualMetrics;

/// Test thresholds for visual metrics
pub struct VisualCriteria {
    pub min_brightness: f32,
    pub max_brightness: f32,
    pub min_bloom: f32,
    pub min_saturation: f32,
    pub max_dark_pixels: f32,
    pub min_contrast: f32,
}

impl Default for VisualCriteria {
    fn default() -> Self {
        Self {
            min_brightness: 0.05,
            max_brightness: 0.8,
            min_bloom: 0.01,
            min_saturation: 0.1,
            max_dark_pixels: 0.95,
            min_contrast: 2.0,
        }
    }
}

/// Check if visual metrics meet the given criteria
pub fn check_visual_criteria(metrics: &VisualMetrics, criteria: &VisualCriteria) -> Vec<String> {
    let mut failures = Vec::new();

    if metrics.avg_brightness < criteria.min_brightness {
        failures.push(format!(
            "Brightness {:.2}% below minimum {:.2}%",
            metrics.avg_brightness * 100.0,
            criteria.min_brightness * 100.0
        ));
    }

    if metrics.avg_brightness > criteria.max_brightness {
        failures.push(format!(
            "Brightness {:.2}% above maximum {:.2}%",
            metrics.avg_brightness * 100.0,
            criteria.max_brightness * 100.0
        ));
    }

    if metrics.bloom_coverage < criteria.min_bloom {
        failures.push(format!(
            "Bloom coverage {:.2}% below minimum {:.2}%",
            metrics.bloom_coverage * 100.0,
            criteria.min_bloom * 100.0
        ));
    }

    if metrics.avg_saturation < criteria.min_saturation {
        failures.push(format!(
            "Saturation {:.2}% below minimum {:.2}%",
            metrics.avg_saturation * 100.0,
            criteria.min_saturation * 100.0
        ));
    }

    if metrics.dark_pixels > criteria.max_dark_pixels {
        failures.push(format!(
            "Dark pixels {:.2}% above maximum {:.2}%",
            metrics.dark_pixels * 100.0,
            criteria.max_dark_pixels * 100.0
        ));
    }

    if metrics.contrast_ratio < criteria.min_contrast {
        failures.push(format!(
            "Contrast ratio {:.2}x below minimum {:.2}x",
            metrics.contrast_ratio,
            criteria.min_contrast
        ));
    }

    failures
}

/// Generate a test report for visual metrics
pub fn generate_visual_report(metrics: &VisualMetrics) -> String {
    format!(
        r#"Visual Metrics Report
=====================
Brightness:    {:.2}% (avg) / {:.2}% (max)
Bloom Coverage: {:.2}%
Saturation:    {:.2}%
Contrast Ratio: {:.2}x
Dark Pixels:   {:.2}%
Bright Pixels: {:.2}%
Dominant Hue:  {:.0}° (bin {})
Hue Variance:  {:.4}
"#,
        metrics.avg_brightness * 100.0,
        metrics.max_brightness * 100.0,
        metrics.bloom_coverage * 100.0,
        metrics.avg_saturation * 100.0,
        metrics.contrast_ratio,
        metrics.dark_pixels * 100.0,
        metrics.bright_pixels * 100.0,
        metrics.dominant_hue,
        metrics.color_distribution.peak_hue_bin,
        metrics.color_distribution.variance
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::metrics::analyze_pixels;

    fn create_test_image_dark() -> Vec<u8> {
        // 10x10 mostly dark image
        let mut pixels = vec![0u8; 100 * 4];
        // Add a few bright pixels
        for i in 0..10 {
            pixels[i * 4] = 200;
            pixels[i * 4 + 1] = 180;
            pixels[i * 4 + 2] = 160;
            pixels[i * 4 + 3] = 255;
        }
        pixels
    }

    fn create_test_image_bright() -> Vec<u8> {
        // 10x10 bright image
        let mut pixels = vec![0u8; 100 * 4];
        for i in 0..100 {
            pixels[i * 4] = 180;
            pixels[i * 4 + 1] = 200;
            pixels[i * 4 + 2] = 220;
            pixels[i * 4 + 3] = 255;
        }
        pixels
    }

    fn create_test_image_colored() -> Vec<u8> {
        // 10x10 image with cyan/teal colors (like bioluminescence)
        let mut pixels = vec![0u8; 100 * 4];
        for i in 0..100 {
            let t = i as f32 / 100.0;
            pixels[i * 4] = (50.0 + t * 50.0) as u8;     // R
            pixels[i * 4 + 1] = (150.0 + t * 50.0) as u8; // G
            pixels[i * 4 + 2] = (180.0 + t * 30.0) as u8; // B
            pixels[i * 4 + 3] = 255;
        }
        pixels
    }

    #[test]
    fn test_check_criteria_pass() {
        let metrics = VisualMetrics {
            avg_brightness: 0.15,
            max_brightness: 0.8,
            bloom_coverage: 0.05,
            avg_saturation: 0.3,
            dark_pixels: 0.6,
            bright_pixels: 0.1,
            contrast_ratio: 5.0,
            dominant_hue: 180.0,
            color_distribution: Default::default(),
        };

        let criteria = VisualCriteria::default();
        let failures = check_visual_criteria(&metrics, &criteria);

        assert!(failures.is_empty(), "Expected no failures, got: {:?}", failures);
    }

    #[test]
    fn test_check_criteria_fail_brightness() {
        let metrics = VisualMetrics {
            avg_brightness: 0.01, // Too dark
            max_brightness: 0.1,
            bloom_coverage: 0.05,
            avg_saturation: 0.3,
            dark_pixels: 0.6,
            bright_pixels: 0.1,
            contrast_ratio: 5.0,
            dominant_hue: 180.0,
            color_distribution: Default::default(),
        };

        let criteria = VisualCriteria::default();
        let failures = check_visual_criteria(&metrics, &criteria);

        assert!(!failures.is_empty());
        assert!(failures[0].contains("Brightness"));
    }

    #[test]
    fn test_analyze_dark_image() {
        let pixels = create_test_image_dark();
        let metrics = analyze_pixels(&pixels, 10, 10);

        // Mostly dark image should have low brightness
        assert!(metrics.avg_brightness < 0.2);
        assert!(metrics.dark_pixels > 0.5);
    }

    #[test]
    fn test_analyze_bright_image() {
        let pixels = create_test_image_bright();
        let metrics = analyze_pixels(&pixels, 10, 10);

        // Bright image should have high brightness
        assert!(metrics.avg_brightness > 0.5);
    }

    #[test]
    fn test_analyze_colored_image() {
        let pixels = create_test_image_colored();
        let metrics = analyze_pixels(&pixels, 10, 10);

        // Colored image should have measurable saturation
        assert!(metrics.avg_saturation > 0.1);
        // Cyan-ish hue should be around 180 degrees (bin 6)
        assert!(metrics.dominant_hue > 150.0 && metrics.dominant_hue < 210.0);
    }

    #[test]
    fn test_generate_report() {
        let metrics = VisualMetrics {
            avg_brightness: 0.15,
            max_brightness: 0.8,
            bloom_coverage: 0.05,
            avg_saturation: 0.3,
            dark_pixels: 0.6,
            bright_pixels: 0.1,
            contrast_ratio: 5.0,
            dominant_hue: 180.0,
            color_distribution: Default::default(),
        };

        let report = generate_visual_report(&metrics);

        assert!(report.contains("Visual Metrics Report"));
        assert!(report.contains("Brightness"));
        assert!(report.contains("Bloom"));
    }
}
