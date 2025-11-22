use super::Vec3;

/// Catmull-Rom spline for smooth organic curves through control points
#[derive(Debug, Clone)]
pub struct CatmullRomSpline {
    pub points: Vec<Vec3>,
    pub tension: f32, // 0.0 to 1.0, affects curvature
}

impl CatmullRomSpline {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            points,
            tension: 0.5, // Default tension
        }
    }

    pub fn with_tension(points: Vec<Vec3>, tension: f32) -> Self {
        Self { points, tension }
    }

    /// Evaluate spline at parameter t (0.0 to 1.0 across entire spline)
    pub fn evaluate(&self, t: f32) -> Vec3 {
        if self.points.len() < 2 {
            return self.points.first().cloned().unwrap_or(Vec3::ZERO);
        }

        let t = t.clamp(0.0, 1.0);
        let segments = self.points.len() - 1;
        let total_t = t * segments as f32;
        let segment = (total_t as usize).min(segments - 1);
        let local_t = total_t - segment as f32;

        self.evaluate_segment(segment, local_t)
    }

    /// Evaluate specific segment at local parameter t
    fn evaluate_segment(&self, segment: usize, t: f32) -> Vec3 {
        let n = self.points.len();

        // Get four control points (with endpoint handling)
        let p0 = if segment == 0 {
            self.points[0]
        } else {
            self.points[segment - 1]
        };
        let p1 = self.points[segment];
        let p2 = self.points[segment + 1];
        let p3 = if segment + 2 >= n {
            self.points[n - 1]
        } else {
            self.points[segment + 2]
        };

        evaluate_catmull_rom(p0, p1, p2, p3, t, self.tension)
    }

    /// Get tangent at parameter t
    pub fn tangent(&self, t: f32) -> Vec3 {
        let delta = 0.001;
        let t1 = (t - delta).max(0.0);
        let t2 = (t + delta).min(1.0);
        (self.evaluate(t2) - self.evaluate(t1)).normalize()
    }

    /// Sample spline at N evenly spaced points
    pub fn sample(&self, n: usize) -> Vec<Vec3> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1).max(1) as f32;
                self.evaluate(t)
            })
            .collect()
    }

    /// Total approximate length of spline
    pub fn approximate_length(&self, samples: usize) -> f32 {
        let points = self.sample(samples);
        points
            .windows(2)
            .map(|w| w[0].distance(&w[1]))
            .sum()
    }
}

/// Evaluate Catmull-Rom spline between p1 and p2
pub fn evaluate_catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32, tension: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    let s = (1.0 - tension) / 2.0;

    // Catmull-Rom basis functions
    let h1 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h2 = -2.0 * t3 + 3.0 * t2;
    let h3 = t3 - 2.0 * t2 + t;
    let h4 = t3 - t2;

    // Tangents at p1 and p2
    let m1 = (p2 - p0).scale(s);
    let m2 = (p3 - p1).scale(s);

    p1.scale(h1) + p2.scale(h2) + m1.scale(h3) + m2.scale(h4)
}

/// Hermite spline evaluation for branch curves
pub fn hermite_curve(p0: Vec3, p1: Vec3, m0: Vec3, m1: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    p0.scale(h00) + m0.scale(h10) + p1.scale(h01) + m1.scale(h11)
}

/// Generate smooth curve between two points with given directions
pub fn generate_branch_curve(
    start: Vec3,
    end: Vec3,
    start_dir: Vec3,
    end_dir: Vec3,
    curvature: f32,
    samples: usize,
) -> Vec<Vec3> {
    let length = start.distance(&end);
    let m0 = start_dir.scale(length * curvature);
    let m1 = end_dir.scale(length * curvature);

    (0..samples)
        .map(|i| {
            let t = i as f32 / (samples - 1).max(1) as f32;
            hermite_curve(start, end, m0, m1, t)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spline_endpoints() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let spline = CatmullRomSpline::new(points.clone());

        let start = spline.evaluate(0.0);
        assert!((start.x - 0.0).abs() < 0.0001);
        assert!((start.y - 0.0).abs() < 0.0001);

        let end = spline.evaluate(1.0);
        assert!((end.x - 2.0).abs() < 0.0001);
        assert!((end.y - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_spline_midpoint() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let spline = CatmullRomSpline::new(points);

        let mid = spline.evaluate(0.5);
        assert!((mid.x - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_spline_sampling() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let spline = CatmullRomSpline::new(points);

        let samples = spline.sample(11);
        assert_eq!(samples.len(), 11);
    }

    #[test]
    fn test_hermite_endpoints() {
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let p1 = Vec3::new(1.0, 1.0, 0.0);
        let m0 = Vec3::new(1.0, 0.0, 0.0);
        let m1 = Vec3::new(1.0, 0.0, 0.0);

        let start = hermite_curve(p0, p1, m0, m1, 0.0);
        assert!((start.x - 0.0).abs() < 0.0001);

        let end = hermite_curve(p0, p1, m0, m1, 1.0);
        assert!((end.x - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_branch_curve_generation() {
        let curve = generate_branch_curve(
            Vec3::ZERO,
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::UP,
            Vec3::UP,
            0.5,
            10,
        );
        assert_eq!(curve.len(), 10);
        assert!((curve[0].y - 0.0).abs() < 0.0001);
        assert!((curve[9].y - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_spline_tangent() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let spline = CatmullRomSpline::new(points);

        let tangent = spline.tangent(0.5);
        assert!((tangent.x - 1.0).abs() < 0.01);
        assert!(tangent.y.abs() < 0.01);
    }
}
