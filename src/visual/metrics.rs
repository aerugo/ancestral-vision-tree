//! Visual metrics calculation for automated visual testing
//!
//! These metrics allow programmatic verification of visual effects
//! like bloom, glow intensity, and color distribution.

use wasm_bindgen::prelude::*;

/// Visual metrics computed from rendered frame
#[derive(Debug, Clone, Default)]
pub struct VisualMetrics {
    /// Average brightness (0-1)
    pub avg_brightness: f32,
    /// Maximum brightness found
    pub max_brightness: f32,
    /// Percentage of pixels above bloom threshold (0.7)
    pub bloom_coverage: f32,
    /// Color distribution statistics
    pub color_distribution: ColorDistribution,
    /// Contrast ratio (max/min non-black brightness)
    pub contrast_ratio: f32,
    /// Average saturation (0-1)
    pub avg_saturation: f32,
    /// Dominant hue (0-360 degrees)
    pub dominant_hue: f32,
    /// Percentage of very dark pixels (<0.05 brightness)
    pub dark_pixels: f32,
    /// Percentage of very bright pixels (>0.9 brightness)
    pub bright_pixels: f32,
}

/// Color distribution across hue spectrum
#[derive(Debug, Clone, Default)]
pub struct ColorDistribution {
    /// Histogram of hues (12 bins, 30 degrees each)
    pub hue_histogram: [f32; 12],
    /// Most common hue bin (0-11)
    pub peak_hue_bin: usize,
    /// Distribution variance
    pub variance: f32,
}

/// Analyze raw pixel data (RGBA format, 4 bytes per pixel)
/// Returns comprehensive visual metrics for validation
pub fn analyze_pixels(pixels: &[u8], width: u32, height: u32) -> VisualMetrics {
    let pixel_count = (width * height) as usize;
    if pixels.len() < pixel_count * 4 {
        return VisualMetrics::default();
    }

    let mut total_brightness = 0.0f64;
    let mut max_brightness = 0.0f32;
    let mut min_non_black_brightness = 1.0f32;
    let mut bloom_pixels = 0u32;
    let mut dark_pixels = 0u32;
    let mut bright_pixels = 0u32;
    let mut total_saturation = 0.0f64;
    let mut hue_histogram = [0u32; 12];

    for i in 0..pixel_count {
        let r = pixels[i * 4] as f32 / 255.0;
        let g = pixels[i * 4 + 1] as f32 / 255.0;
        let b = pixels[i * 4 + 2] as f32 / 255.0;

        // Calculate brightness (luminance)
        let brightness = 0.299 * r + 0.587 * g + 0.114 * b;
        total_brightness += brightness as f64;

        if brightness > max_brightness {
            max_brightness = brightness;
        }
        if brightness > 0.01 && brightness < min_non_black_brightness {
            min_non_black_brightness = brightness;
        }

        // Bloom threshold (HDR-like bright areas)
        if brightness > 0.7 {
            bloom_pixels += 1;
        }
        if brightness < 0.05 {
            dark_pixels += 1;
        }
        if brightness > 0.9 {
            bright_pixels += 1;
        }

        // Calculate HSV for color analysis
        let (h, s, _v) = rgb_to_hsv(r, g, b);
        total_saturation += s as f64;

        // Bin hue into 12 segments (30 degrees each)
        if s > 0.1 && brightness > 0.05 {
            // Only count colored, visible pixels
            let hue_bin = ((h / 30.0).floor() as usize) % 12;
            hue_histogram[hue_bin] += 1;
        }
    }

    let avg_brightness = (total_brightness / pixel_count as f64) as f32;
    let avg_saturation = (total_saturation / pixel_count as f64) as f32;
    let bloom_coverage = bloom_pixels as f32 / pixel_count as f32;
    let dark_coverage = dark_pixels as f32 / pixel_count as f32;
    let bright_coverage = bright_pixels as f32 / pixel_count as f32;

    // Contrast ratio
    let contrast_ratio = if min_non_black_brightness > 0.001 {
        max_brightness / min_non_black_brightness
    } else {
        max_brightness / 0.001
    };

    // Analyze color distribution
    let total_colored: u32 = hue_histogram.iter().sum();
    let mut normalized_histogram = [0.0f32; 12];
    let mut peak_bin = 0;
    let mut peak_value = 0.0f32;

    for (i, &count) in hue_histogram.iter().enumerate() {
        normalized_histogram[i] = if total_colored > 0 {
            count as f32 / total_colored as f32
        } else {
            0.0
        };
        if normalized_histogram[i] > peak_value {
            peak_value = normalized_histogram[i];
            peak_bin = i;
        }
    }

    // Calculate variance in hue distribution
    let mean = 1.0 / 12.0;
    let variance = normalized_histogram
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>()
        / 12.0;

    let dominant_hue = peak_bin as f32 * 30.0 + 15.0; // Center of bin

    VisualMetrics {
        avg_brightness,
        max_brightness,
        bloom_coverage,
        color_distribution: ColorDistribution {
            hue_histogram: normalized_histogram,
            peak_hue_bin: peak_bin,
            variance,
        },
        contrast_ratio,
        avg_saturation,
        dominant_hue,
        dark_pixels: dark_coverage,
        bright_pixels: bright_coverage,
    }
}

/// Convert RGB (0-1) to HSV (hue: 0-360, saturation: 0-1, value: 0-1)
fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;

    let s = if max > 0.0 { delta / max } else { 0.0 };

    let h = if delta < 0.0001 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}

/// WASM-bindgen wrapper for analyzing pixels from JavaScript
#[wasm_bindgen]
pub struct VisualAnalyzer;

#[wasm_bindgen]
impl VisualAnalyzer {
    /// Analyze pixel data and return JSON metrics
    #[wasm_bindgen]
    pub fn analyze(pixels: &[u8], width: u32, height: u32) -> String {
        let metrics = analyze_pixels(pixels, width, height);
        format!(
            r#"{{
  "avgBrightness": {:.4},
  "maxBrightness": {:.4},
  "bloomCoverage": {:.4},
  "contrastRatio": {:.4},
  "avgSaturation": {:.4},
  "dominantHue": {:.1},
  "darkPixels": {:.4},
  "brightPixels": {:.4},
  "hueHistogram": [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}],
  "peakHueBin": {},
  "hueVariance": {:.6}
}}"#,
            metrics.avg_brightness,
            metrics.max_brightness,
            metrics.bloom_coverage,
            metrics.contrast_ratio,
            metrics.avg_saturation,
            metrics.dominant_hue,
            metrics.dark_pixels,
            metrics.bright_pixels,
            metrics.color_distribution.hue_histogram[0],
            metrics.color_distribution.hue_histogram[1],
            metrics.color_distribution.hue_histogram[2],
            metrics.color_distribution.hue_histogram[3],
            metrics.color_distribution.hue_histogram[4],
            metrics.color_distribution.hue_histogram[5],
            metrics.color_distribution.hue_histogram[6],
            metrics.color_distribution.hue_histogram[7],
            metrics.color_distribution.hue_histogram[8],
            metrics.color_distribution.hue_histogram[9],
            metrics.color_distribution.hue_histogram[10],
            metrics.color_distribution.hue_histogram[11],
            metrics.color_distribution.peak_hue_bin,
            metrics.color_distribution.variance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_black_image() {
        let pixels = vec![0u8; 100 * 100 * 4];
        let metrics = analyze_pixels(&pixels, 100, 100);

        assert_eq!(metrics.avg_brightness, 0.0);
        assert_eq!(metrics.max_brightness, 0.0);
        assert_eq!(metrics.bloom_coverage, 0.0);
        assert_eq!(metrics.dark_pixels, 1.0); // All pixels are dark
    }

    #[test]
    fn test_analyze_white_image() {
        let mut pixels = vec![0u8; 100 * 100 * 4];
        for i in 0..(100 * 100) {
            pixels[i * 4] = 255;     // R
            pixels[i * 4 + 1] = 255; // G
            pixels[i * 4 + 2] = 255; // B
            pixels[i * 4 + 3] = 255; // A
        }
        let metrics = analyze_pixels(&pixels, 100, 100);

        assert!((metrics.avg_brightness - 1.0).abs() < 0.001);
        assert!(metrics.bloom_coverage > 0.99);
        assert!(metrics.bright_pixels > 0.99);
    }

    #[test]
    fn test_analyze_red_image() {
        let mut pixels = vec![0u8; 100 * 100 * 4];
        for i in 0..(100 * 100) {
            pixels[i * 4] = 255;     // R
            pixels[i * 4 + 1] = 0;   // G
            pixels[i * 4 + 2] = 0;   // B
            pixels[i * 4 + 3] = 255; // A
        }
        let metrics = analyze_pixels(&pixels, 100, 100);

        // Red should have high saturation
        assert!(metrics.avg_saturation > 0.9);
        // Dominant hue should be around 0/360 (red)
        assert!(metrics.dominant_hue < 30.0 || metrics.dominant_hue > 330.0);
    }

    #[test]
    fn test_analyze_cyan_image() {
        let mut pixels = vec![0u8; 100 * 100 * 4];
        for i in 0..(100 * 100) {
            pixels[i * 4] = 0;       // R
            pixels[i * 4 + 1] = 255; // G
            pixels[i * 4 + 2] = 255; // B
            pixels[i * 4 + 3] = 255; // A
        }
        let metrics = analyze_pixels(&pixels, 100, 100);

        // Cyan hue should be around 180 degrees
        assert!(metrics.dominant_hue > 150.0 && metrics.dominant_hue < 210.0);
    }

    #[test]
    fn test_rgb_to_hsv() {
        // Red
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 1.0 || (h - 360.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Green
        let (h, s, v) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - 120.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Blue
        let (h, s, v) = rgb_to_hsv(0.0, 0.0, 1.0);
        assert!((h - 240.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // White (no saturation)
        let (_, s, _) = rgb_to_hsv(1.0, 1.0, 1.0);
        assert!(s < 0.01);
    }

    #[test]
    fn test_bloom_detection() {
        // Half bright, half dark image
        let mut pixels = vec![0u8; 100 * 100 * 4];
        for i in 0..5000 {
            // First half: bright
            pixels[i * 4] = 200;
            pixels[i * 4 + 1] = 200;
            pixels[i * 4 + 2] = 200;
            pixels[i * 4 + 3] = 255;
        }
        for i in 5000..10000 {
            // Second half: dark
            pixels[i * 4] = 50;
            pixels[i * 4 + 1] = 50;
            pixels[i * 4 + 2] = 50;
            pixels[i * 4 + 3] = 255;
        }

        let metrics = analyze_pixels(&pixels, 100, 100);

        // Roughly half should be above bloom threshold
        assert!(metrics.bloom_coverage > 0.4 && metrics.bloom_coverage < 0.6);
    }

    #[test]
    fn test_contrast_ratio() {
        // High contrast image
        let mut pixels = vec![0u8; 4 * 4 * 4];
        // One bright pixel
        pixels[0] = 255;
        pixels[1] = 255;
        pixels[2] = 255;
        pixels[3] = 255;
        // Rest dim but not black
        for i in 1..16 {
            pixels[i * 4] = 25;
            pixels[i * 4 + 1] = 25;
            pixels[i * 4 + 2] = 25;
            pixels[i * 4 + 3] = 255;
        }

        let metrics = analyze_pixels(&pixels, 4, 4);

        // Should have significant contrast
        assert!(metrics.contrast_ratio > 5.0);
    }
}
