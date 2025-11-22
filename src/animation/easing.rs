//! Easing functions for smooth animations

/// Easing function types
#[derive(Debug, Clone, Copy, Default)]
pub enum Easing {
    /// Linear interpolation
    Linear,
    /// Smooth ease-in-out (default for organic growth)
    #[default]
    EaseInOut,
    /// Slow start, accelerate
    EaseIn,
    /// Fast start, decelerate
    EaseOut,
    /// Bounce at the end
    EaseOutBack,
    /// Organic growth - slow start, natural acceleration
    Organic,
}

/// Apply easing function to a value t in range [0, 1]
pub fn ease(t: f32, easing: Easing) -> f32 {
    let t = t.clamp(0.0, 1.0);

    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        Easing::EaseOutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        Easing::Organic => {
            // Custom organic curve: slow start, natural mid-acceleration, gentle finish
            // Inspired by plant growth patterns
            let t2 = t * t;
            let t3 = t2 * t;
            // Hermite interpolation with organic feel
            (3.0 * t2 - 2.0 * t3) * (1.0 + 0.3 * (1.0 - t).sin())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_bounds() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::Organic,
        ] {
            assert!((ease(0.0, easing) - 0.0).abs() < 0.01, "Easing {:?} should start near 0", easing);
            assert!((ease(1.0, easing) - 1.0).abs() < 0.1, "Easing {:?} should end near 1", easing);
        }
    }

    #[test]
    fn test_ease_monotonic() {
        for easing in [Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut] {
            let mut prev = 0.0;
            for i in 0..=100 {
                let t = i as f32 / 100.0;
                let v = ease(t, easing);
                assert!(v >= prev - 0.001, "Easing {:?} should be monotonic", easing);
                prev = v;
            }
        }
    }

    #[test]
    fn test_ease_in_out_symmetric() {
        let v1 = ease(0.25, Easing::EaseInOut);
        let v2 = ease(0.75, Easing::EaseInOut);
        assert!((v1 + v2 - 1.0).abs() < 0.01, "EaseInOut should be symmetric");
    }

    #[test]
    fn test_ease_clamps_input() {
        assert_eq!(ease(-0.5, Easing::Linear), 0.0);
        assert_eq!(ease(1.5, Easing::Linear), 1.0);
    }
}
