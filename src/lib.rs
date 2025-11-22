use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, HtmlCanvasElement};

pub mod data;
pub mod growth;
pub mod math;
pub mod mesh;
pub mod particles;
pub mod render;
pub mod interaction;
pub mod visual;
pub mod animation;

// Re-export visual analyzer for JavaScript
pub use visual::metrics::VisualAnalyzer;

use data::FamilyTree;
use growth::{TreeGrowth, GrowthParams, BranchNode};
use mesh::generator::{MeshParams, TrackedMeshGenerator};
use particles::{FireflySystem, OrbSystem};
use render::RenderPipeline;
use interaction::RayPicker;
use math::{Vec3, Mat4};
use animation::GrowthAnimation;

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Main engine state exposed to JavaScript
#[wasm_bindgen]
pub struct AncestralVisionTree {
    pipeline: RenderPipeline,
    fireflies: FireflySystem,
    orbs: OrbSystem,
    picker: RayPicker,
    family_tree: Option<FamilyTree>,
    /// Stored tree structure for animation
    tree_structure: Option<BranchNode>,
    /// Growth animation controller
    growth_animation: GrowthAnimation,
    time: f32,
    width: i32,
    height: i32,
    // Camera orbit controls
    camera_distance: f32,
    camera_angle_x: f32,
    camera_angle_y: f32,
    camera_target: Vec3,
    // Hover state
    hovered_person_id: Option<String>,
}

#[wasm_bindgen]
impl AncestralVisionTree {
    /// Create a new engine instance
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<AncestralVisionTree, JsValue> {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        let gl = canvas
            .get_context("webgl2")?
            .ok_or("Failed to get WebGL2 context")?
            .dyn_into::<WebGl2RenderingContext>()?;

        let pipeline = RenderPipeline::new(gl, width, height)
            .map_err(|e| JsValue::from_str(&e))?;

        let fireflies = FireflySystem::new(150);
        let orbs = OrbSystem::new(50); // Fewer orbs, larger and more prominent
        let picker = RayPicker::new();

        Ok(Self {
            pipeline,
            fireflies,
            orbs,
            picker,
            family_tree: None,
            tree_structure: None,
            growth_animation: GrowthAnimation::instant(), // Default to fully grown
            time: 0.0,
            width,
            height,
            camera_distance: 12.0,
            camera_angle_x: 0.3,
            camera_angle_y: 0.0,
            camera_target: Vec3::new(0.0, 3.5, 0.0),
            hovered_person_id: None,
        })
    }

    /// Load family tree from YAML string
    #[wasm_bindgen]
    pub fn load_family(&mut self, yaml: &str) -> Result<(), JsValue> {
        self.load_family_with_animation(yaml, false)
    }

    /// Load family tree with optional growth animation
    #[wasm_bindgen]
    pub fn load_family_animated(&mut self, yaml: &str, duration: f32) -> Result<(), JsValue> {
        self.load_family_internal(yaml, true, duration)
    }

    fn load_family_with_animation(&mut self, yaml: &str, animated: bool) -> Result<(), JsValue> {
        self.load_family_internal(yaml, animated, 5.0)
    }

    fn load_family_internal(&mut self, yaml: &str, animated: bool, duration: f32) -> Result<(), JsValue> {
        let family = FamilyTree::from_yaml(yaml)
            .map_err(|e| JsValue::from_str(&e))?;

        // Generate tree structure
        let growth = TreeGrowth::new(GrowthParams::default());
        let tree = growth.grow(&family)
            .ok_or_else(|| JsValue::from_str("Failed to grow tree"))?;

        // Initialize animation
        if animated {
            self.growth_animation = GrowthAnimation::new(duration);
            self.growth_animation.init_from_tree(&tree);
            self.growth_animation.start();
        } else {
            self.growth_animation = GrowthAnimation::instant();
        }

        // Generate mesh with tracking for picking
        let mesh_params = MeshParams::default();
        let generator = TrackedMeshGenerator::new(mesh_params);
        let (mesh, branch_infos) = generator.generate_tree_tracked(&tree);

        // Upload to GPU
        self.pipeline.upload_tree_mesh(&mesh)
            .map_err(|e| JsValue::from_str(&e))?;

        // Set up picking
        self.picker.set_branches(branch_infos);

        // Configure particle systems based on tree
        self.fireflies.configure_from_tree(&tree);
        self.orbs.configure_from_tree(&tree);

        // Initial particle upload
        let particle_data = self.fireflies.get_particle_data();
        if !particle_data.is_empty() {
            // Pre-allocate with some initial particles
            let mut initial_data = vec![0.0f32; 150 * 8];
            for (i, &v) in particle_data.iter().enumerate() {
                if i < initial_data.len() {
                    initial_data[i] = v;
                }
            }
            self.pipeline.upload_particles(&initial_data)
                .map_err(|e| JsValue::from_str(&e))?;
        }

        // Store tree structure for animation updates
        self.tree_structure = Some(tree);
        self.family_tree = Some(family);

        Ok(())
    }

    /// Update and render a frame
    #[wasm_bindgen]
    pub fn render(&mut self, dt: f32) {
        self.time += dt;

        // Update growth animation
        self.growth_animation.update(dt);

        // Pass animation progress to pipeline for shader-based animation
        self.pipeline.set_growth_progress(self.growth_animation.get_progress());

        // Update particle systems (scale activity with growth)
        let growth_scale = self.growth_animation.get_progress();

        // Update fireflies
        self.fireflies.set_activity_scale(growth_scale);
        self.fireflies.update(dt, self.time);

        // Update orbs (attracted to high-luminance branches)
        self.orbs.set_activity_scale(growth_scale);
        self.orbs.update(dt, self.time);

        // Combine particle data from both systems
        let mut particle_data = self.fireflies.get_particle_data();
        particle_data.extend(self.orbs.get_particle_data());

        if !particle_data.is_empty() {
            self.pipeline.update_particles(&particle_data);
        }

        // Update camera position from orbit angles
        let cos_x = self.camera_angle_x.cos();
        let sin_x = self.camera_angle_x.sin();
        let cos_y = self.camera_angle_y.cos();
        let sin_y = self.camera_angle_y.sin();

        self.pipeline.camera_position = Vec3::new(
            self.camera_target.x + self.camera_distance * cos_x * sin_y,
            self.camera_target.y + self.camera_distance * sin_x,
            self.camera_target.z + self.camera_distance * cos_x * cos_y,
        );
        self.pipeline.camera_target = self.camera_target;

        // Render
        self.pipeline.render(self.time);
    }

    /// Resize the canvas
    #[wasm_bindgen]
    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), JsValue> {
        self.width = width;
        self.height = height;
        self.pipeline.resize(width, height)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Handle mouse move for hover detection
    #[wasm_bindgen]
    pub fn on_mouse_move(&mut self, x: f32, y: f32) -> Option<String> {
        let aspect = self.width as f32 / self.height as f32;
        let projection = Mat4::perspective(self.pipeline.fov, aspect, 0.1, 100.0);
        let view = Mat4::look_at(
            self.pipeline.camera_position,
            self.pipeline.camera_target,
            Vec3::UP,
        );

        if let Some(hit) = self.picker.pick(
            x,
            y,
            self.width as f32,
            self.height as f32,
            &view,
            &projection,
            self.pipeline.camera_position,
        ) {
            self.hovered_person_id = Some(hit.person_id.clone());
            Some(hit.person_id)
        } else {
            self.hovered_person_id = None;
            None
        }
    }

    /// Get person info by ID (returns JSON string)
    #[wasm_bindgen]
    pub fn get_person_info(&self, id: &str) -> Option<String> {
        self.family_tree.as_ref().and_then(|tree| {
            tree.get(id).map(|person| {
                format!(
                    r#"{{"id":"{}","name":"{}","biography":"{}","lifespan":"{}"}}"#,
                    escape_json(&person.id),
                    escape_json(&person.name),
                    escape_json(&person.biography),
                    escape_json(&person.lifespan_string())
                )
            })
        })
    }

    /// Orbit camera
    #[wasm_bindgen]
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.camera_angle_y += delta_x * 0.01;
        self.camera_angle_x = (self.camera_angle_x + delta_y * 0.01)
            .clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
    }

    /// Zoom camera
    #[wasm_bindgen]
    pub fn zoom(&mut self, delta: f32) {
        self.camera_distance = (self.camera_distance + delta * 0.5).clamp(3.0, 30.0);
    }

    /// Pan camera target
    #[wasm_bindgen]
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        // Pan in camera-relative space
        let right = Vec3::new(
            self.camera_angle_y.cos(),
            0.0,
            -self.camera_angle_y.sin(),
        );
        let up = Vec3::UP;

        self.camera_target = self.camera_target
            + right.scale(-delta_x * 0.01)
            + up.scale(delta_y * 0.01);
    }

    /// Get current hovered person ID
    #[wasm_bindgen]
    pub fn get_hovered_person(&self) -> Option<String> {
        self.hovered_person_id.clone()
    }

    // === Animation Controls ===

    /// Start the growth animation
    #[wasm_bindgen]
    pub fn start_growth_animation(&mut self) {
        self.growth_animation.start();
    }

    /// Reset and restart the growth animation
    #[wasm_bindgen]
    pub fn reset_growth_animation(&mut self) {
        self.growth_animation.reset();
        self.growth_animation.start();
    }

    /// Complete the growth instantly
    #[wasm_bindgen]
    pub fn complete_growth(&mut self) {
        self.growth_animation.complete_instantly();
    }

    /// Set growth progress manually (0.0 to 1.0)
    #[wasm_bindgen]
    pub fn set_growth_progress(&mut self, progress: f32) {
        self.growth_animation.set_progress(progress);
    }

    /// Get current growth progress (0.0 to 1.0)
    #[wasm_bindgen]
    pub fn get_growth_progress(&self) -> f32 {
        self.growth_animation.get_progress()
    }

    /// Check if growth animation is complete
    #[wasm_bindgen]
    pub fn is_growth_complete(&self) -> bool {
        self.growth_animation.is_complete()
    }

    /// Check if growth animation is playing
    #[wasm_bindgen]
    pub fn is_growth_playing(&self) -> bool {
        self.growth_animation.is_playing()
    }
}

/// Escape special characters for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json(r#"say "hi""#), r#"say \"hi\""#);
    }
}
