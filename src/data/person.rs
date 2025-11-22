use serde::{Deserialize, Serialize};

/// A person in the family tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub biography: String,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    #[serde(default)]
    pub children: Vec<String>,
}

impl Person {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            biography: String::new(),
            birth_year: None,
            death_year: None,
            children: Vec::new(),
        }
    }

    pub fn with_biography(mut self, bio: &str) -> Self {
        self.biography = bio.to_string();
        self
    }

    pub fn with_children(mut self, children: Vec<&str>) -> Self {
        self.children = children.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_years(mut self, birth: Option<i32>, death: Option<i32>) -> Self {
        self.birth_year = birth;
        self.death_year = death;
        self
    }

    /// Calculate biography influence (0.0 to 1.0) based on length
    pub fn biography_influence(&self) -> f32 {
        let len = self.biography.len();
        // Short bio (< 50 chars): low influence
        // Medium bio (50-500 chars): medium influence
        // Long bio (> 500 chars): high influence
        // Sigmoid-like curve that saturates at ~1000 chars
        let normalized = (len as f32 / 500.0).min(2.0);
        1.0 - (-normalized * 2.0).exp()
    }

    /// Generate visual parameters based on person's data
    pub fn visual_params(&self) -> VisualParams {
        let influence = self.biography_influence();

        VisualParams {
            glow_intensity: 0.2 + influence * 0.8,
            color_vibrancy: 0.3 + influence * 0.7,
            branch_thickness: 0.5 + influence * 0.5,
            luminance: 0.1 + influence * 0.9,
            hue_shift: (self.id.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)) % 360) as f32,
        }
    }

    /// Lifespan as string for display
    pub fn lifespan_string(&self) -> String {
        match (self.birth_year, self.death_year) {
            (Some(b), Some(d)) => format!("{} - {}", b, d),
            (Some(b), None) => format!("{} - present", b),
            (None, Some(d)) => format!("? - {}", d),
            (None, None) => String::new(),
        }
    }
}

/// Visual parameters derived from person data
#[derive(Debug, Clone, Copy)]
pub struct VisualParams {
    /// Glow intensity (0.0 to 1.0)
    pub glow_intensity: f32,
    /// Color saturation boost (0.0 to 1.0)
    pub color_vibrancy: f32,
    /// Relative branch thickness multiplier
    pub branch_thickness: f32,
    /// Bioluminescence strength (0.0 to 1.0)
    pub luminance: f32,
    /// Hue rotation in degrees (0 to 360)
    pub hue_shift: f32,
}

impl Default for VisualParams {
    fn default() -> Self {
        Self {
            glow_intensity: 0.3,
            color_vibrancy: 0.5,
            branch_thickness: 0.7,
            luminance: 0.3,
            hue_shift: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation() {
        let person = Person::new("test-id", "Test Name");
        assert_eq!(person.id, "test-id");
        assert_eq!(person.name, "Test Name");
        assert!(person.children.is_empty());
    }

    #[test]
    fn test_person_builder() {
        let person = Person::new("alice", "Alice Smith")
            .with_biography("A wonderful person who lived a full life.")
            .with_children(vec!["bob", "carol"])
            .with_years(Some(1950), Some(2020));

        assert_eq!(person.children.len(), 2);
        assert_eq!(person.birth_year, Some(1950));
        assert!(!person.biography.is_empty());
    }

    #[test]
    fn test_biography_influence_empty() {
        let person = Person::new("test", "Test");
        let influence = person.biography_influence();
        assert!(influence < 0.1);
    }

    #[test]
    fn test_biography_influence_short() {
        let person = Person::new("test", "Test")
            .with_biography("Short bio.");
        let influence = person.biography_influence();
        assert!(influence > 0.0 && influence < 0.3);
    }

    #[test]
    fn test_biography_influence_long() {
        let long_bio = "A".repeat(1000);
        let person = Person::new("test", "Test")
            .with_biography(&long_bio);
        let influence = person.biography_influence();
        assert!(influence > 0.9);
    }

    #[test]
    fn test_visual_params_vary_with_bio() {
        let short_bio_person = Person::new("a", "A").with_biography("Hi");
        let long_bio_person = Person::new("b", "B").with_biography(&"X".repeat(800));

        let short_params = short_bio_person.visual_params();
        let long_params = long_bio_person.visual_params();

        assert!(long_params.glow_intensity > short_params.glow_intensity);
        assert!(long_params.luminance > short_params.luminance);
        assert!(long_params.branch_thickness > short_params.branch_thickness);
    }

    #[test]
    fn test_lifespan_string() {
        let p1 = Person::new("a", "A").with_years(Some(1900), Some(1980));
        assert_eq!(p1.lifespan_string(), "1900 - 1980");

        let p2 = Person::new("b", "B").with_years(Some(1990), None);
        assert_eq!(p2.lifespan_string(), "1990 - present");

        let p3 = Person::new("c", "C");
        assert_eq!(p3.lifespan_string(), "");
    }
}
