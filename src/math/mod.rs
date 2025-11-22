pub mod vec3;
pub mod matrix;
pub mod spline;

pub use vec3::Vec3;
pub use matrix::Mat4;
pub use spline::{CatmullRomSpline, evaluate_catmull_rom, generate_branch_curve};
