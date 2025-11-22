use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlVertexArrayObject,
    WebGlTexture, WebGlFramebuffer, WebGlUniformLocation,
};
use crate::math::{Vec3, Mat4};
use crate::mesh::Mesh;
use super::webgl::WebGLContext;
use super::shaders::*;

/// Cached uniform locations for tree shader
struct TreeUniforms {
    model: Option<WebGlUniformLocation>,
    view: Option<WebGlUniformLocation>,
    projection: Option<WebGlUniformLocation>,
    time: Option<WebGlUniformLocation>,
    camera_pos: Option<WebGlUniformLocation>,
    base_color: Option<WebGlUniformLocation>,
    ambient_strength: Option<WebGlUniformLocation>,
}

/// Cached uniform locations for particle shader
struct ParticleUniforms {
    view: Option<WebGlUniformLocation>,
    projection: Option<WebGlUniformLocation>,
    time: Option<WebGlUniformLocation>,
}

/// Cached uniform locations for post-processing
struct PostUniforms {
    texture: Option<WebGlUniformLocation>,
    threshold: Option<WebGlUniformLocation>,
    direction: Option<WebGlUniformLocation>,
    scene: Option<WebGlUniformLocation>,
    bloom: Option<WebGlUniformLocation>,
    bloom_strength: Option<WebGlUniformLocation>,
    vignette_strength: Option<WebGlUniformLocation>,
}

/// Complete render pipeline for the tree visualization
pub struct RenderPipeline {
    ctx: WebGLContext,

    // Shaders
    tree_program: WebGlProgram,
    particle_program: WebGlProgram,
    bloom_extract_program: WebGlProgram,
    blur_program: WebGlProgram,
    composite_program: WebGlProgram,

    // Uniform locations
    tree_uniforms: TreeUniforms,
    particle_uniforms: ParticleUniforms,
    post_uniforms: PostUniforms,

    // Tree mesh data
    tree_vao: Option<WebGlVertexArrayObject>,
    tree_vertex_buffer: Option<WebGlBuffer>,
    tree_index_buffer: Option<WebGlBuffer>,
    tree_index_count: i32,

    // Particle data
    particle_vao: Option<WebGlVertexArrayObject>,
    particle_buffer: Option<WebGlBuffer>,
    particle_count: i32,

    // Framebuffers for post-processing
    scene_texture: Option<WebGlTexture>,
    scene_fbo: Option<WebGlFramebuffer>,
    bloom_textures: [Option<WebGlTexture>; 2],
    bloom_fbos: [Option<WebGlFramebuffer>; 2],

    // Dimensions
    width: i32,
    height: i32,

    // Camera state
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub fov: f32,
}

impl RenderPipeline {
    pub fn new(gl: WebGl2RenderingContext, width: i32, height: i32) -> Result<Self, String> {
        let ctx = WebGLContext::new(gl);

        // Compile shaders
        let tree_program = ctx.create_program(TREE_VERTEX_SHADER, TREE_FRAGMENT_SHADER)?;
        let particle_program = ctx.create_program(PARTICLE_VERTEX_SHADER, PARTICLE_FRAGMENT_SHADER)?;
        let bloom_extract_program = ctx.create_program(FULLSCREEN_VERTEX_SHADER, BLOOM_EXTRACT_SHADER)?;
        let blur_program = ctx.create_program(FULLSCREEN_VERTEX_SHADER, BLUR_SHADER)?;
        let composite_program = ctx.create_program(FULLSCREEN_VERTEX_SHADER, COMPOSITE_SHADER)?;

        // Get uniform locations
        let tree_uniforms = TreeUniforms {
            model: ctx.get_uniform_location(&tree_program, "u_model"),
            view: ctx.get_uniform_location(&tree_program, "u_view"),
            projection: ctx.get_uniform_location(&tree_program, "u_projection"),
            time: ctx.get_uniform_location(&tree_program, "u_time"),
            camera_pos: ctx.get_uniform_location(&tree_program, "u_camera_pos"),
            base_color: ctx.get_uniform_location(&tree_program, "u_base_color"),
            ambient_strength: ctx.get_uniform_location(&tree_program, "u_ambient_strength"),
        };

        let particle_uniforms = ParticleUniforms {
            view: ctx.get_uniform_location(&particle_program, "u_view"),
            projection: ctx.get_uniform_location(&particle_program, "u_projection"),
            time: ctx.get_uniform_location(&particle_program, "u_time"),
        };

        let post_uniforms = PostUniforms {
            texture: ctx.get_uniform_location(&blur_program, "u_texture"),
            threshold: ctx.get_uniform_location(&bloom_extract_program, "u_threshold"),
            direction: ctx.get_uniform_location(&blur_program, "u_direction"),
            scene: ctx.get_uniform_location(&composite_program, "u_scene"),
            bloom: ctx.get_uniform_location(&composite_program, "u_bloom"),
            bloom_strength: ctx.get_uniform_location(&composite_program, "u_bloom_strength"),
            vignette_strength: ctx.get_uniform_location(&composite_program, "u_vignette_strength"),
        };

        let mut pipeline = Self {
            ctx,
            tree_program,
            particle_program,
            bloom_extract_program,
            blur_program,
            composite_program,
            tree_uniforms,
            particle_uniforms,
            post_uniforms,
            tree_vao: None,
            tree_vertex_buffer: None,
            tree_index_buffer: None,
            tree_index_count: 0,
            particle_vao: None,
            particle_buffer: None,
            particle_count: 0,
            scene_texture: None,
            scene_fbo: None,
            bloom_textures: [None, None],
            bloom_fbos: [None, None],
            width,
            height,
            camera_position: Vec3::new(0.0, 4.0, 10.0),
            camera_target: Vec3::new(0.0, 3.0, 0.0),
            fov: std::f32::consts::FRAC_PI_4,
        };

        pipeline.create_framebuffers()?;

        Ok(pipeline)
    }

    fn create_framebuffers(&mut self) -> Result<(), String> {
        // Scene framebuffer
        let scene_tex = self.ctx.create_texture(self.width, self.height, WebGl2RenderingContext::RGBA)?;
        let scene_fbo = self.ctx.create_framebuffer(&scene_tex)?;
        self.scene_texture = Some(scene_tex);
        self.scene_fbo = Some(scene_fbo);

        // Bloom framebuffers (at half resolution)
        let bloom_width = self.width / 2;
        let bloom_height = self.height / 2;

        for i in 0..2 {
            let tex = self.ctx.create_texture(bloom_width, bloom_height, WebGl2RenderingContext::RGBA)?;
            let fbo = self.ctx.create_framebuffer(&tex)?;
            self.bloom_textures[i] = Some(tex);
            self.bloom_fbos[i] = Some(fbo);
        }

        Ok(())
    }

    /// Upload tree mesh to GPU
    pub fn upload_tree_mesh(&mut self, mesh: &Mesh) -> Result<(), String> {
        let gl = &self.ctx.gl;

        // Create VAO
        let vao = self.ctx.create_vao()?;
        gl.bind_vertex_array(Some(&vao));

        // Upload vertex data
        let vertex_data = mesh.vertex_data();
        let vertex_buffer = self.ctx.create_buffer_f32(&vertex_data, WebGl2RenderingContext::STATIC_DRAW)?;

        // Upload index data
        let index_data = mesh.index_data();
        let index_buffer = self.ctx.create_index_buffer(index_data, WebGl2RenderingContext::STATIC_DRAW)?;

        // Set up vertex attributes
        // Layout: position(3) + normal(3) + uv(2) + glow(1) + luminance(1) + hue(1) = 11 floats
        let stride = 11 * 4; // 11 floats * 4 bytes

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        // Position (location 0)
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, stride, 0);

        // Normal (location 1)
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, stride, 12);

        // UV (location 2)
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 2, WebGl2RenderingContext::FLOAT, false, stride, 24);

        // Glow (location 3)
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_with_i32(3, 1, WebGl2RenderingContext::FLOAT, false, stride, 32);

        // Luminance (location 4)
        gl.enable_vertex_attrib_array(4);
        gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::FLOAT, false, stride, 36);

        // Hue (location 5)
        gl.enable_vertex_attrib_array(5);
        gl.vertex_attrib_pointer_with_i32(5, 1, WebGl2RenderingContext::FLOAT, false, stride, 40);

        gl.bind_vertex_array(None);

        self.tree_vao = Some(vao);
        self.tree_vertex_buffer = Some(vertex_buffer);
        self.tree_index_buffer = Some(index_buffer);
        self.tree_index_count = index_data.len() as i32;

        Ok(())
    }

    /// Upload particle data to GPU
    /// Format: position(3) + size(1) + alpha(1) + color(3) = 8 floats per particle
    pub fn upload_particles(&mut self, data: &[f32]) -> Result<(), String> {
        let gl = &self.ctx.gl;

        let vao = self.ctx.create_vao()?;
        gl.bind_vertex_array(Some(&vao));

        let buffer = self.ctx.create_buffer_f32(data, WebGl2RenderingContext::DYNAMIC_DRAW)?;

        let stride = 8 * 4;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Position
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, stride, 0);

        // Size
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 1, WebGl2RenderingContext::FLOAT, false, stride, 12);

        // Alpha
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 1, WebGl2RenderingContext::FLOAT, false, stride, 16);

        // Color
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_with_i32(3, 3, WebGl2RenderingContext::FLOAT, false, stride, 20);

        gl.bind_vertex_array(None);

        self.particle_vao = Some(vao);
        self.particle_buffer = Some(buffer);
        self.particle_count = (data.len() / 8) as i32;

        Ok(())
    }

    /// Update particle buffer data
    pub fn update_particles(&mut self, data: &[f32]) {
        if let Some(ref buffer) = self.particle_buffer {
            let gl = &self.ctx.gl;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
            unsafe {
                let array = js_sys::Float32Array::view(data);
                gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &array,
                );
            }
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
            self.particle_count = (data.len() / 8) as i32;
        }
    }

    /// Render a frame
    pub fn render(&self, time: f32) {
        let gl = &self.ctx.gl;

        // Calculate matrices
        let aspect = self.width as f32 / self.height as f32;
        let projection = Mat4::perspective(self.fov, aspect, 0.1, 100.0);
        let view = Mat4::look_at(self.camera_position, self.camera_target, Vec3::UP);
        let model = Mat4::identity();

        // === Pass 1: Render scene to framebuffer ===
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, self.scene_fbo.as_ref());
        self.ctx.viewport(0, 0, self.width, self.height);
        self.ctx.clear(0.02, 0.03, 0.05, 1.0);
        self.ctx.enable_depth_test();

        // Render tree
        if self.tree_vao.is_some() {
            gl.use_program(Some(&self.tree_program));

            self.ctx.uniform_matrix4fv(self.tree_uniforms.model.as_ref(), model.as_slice());
            self.ctx.uniform_matrix4fv(self.tree_uniforms.view.as_ref(), view.as_slice());
            self.ctx.uniform_matrix4fv(self.tree_uniforms.projection.as_ref(), projection.as_slice());
            self.ctx.uniform_1f(self.tree_uniforms.time.as_ref(), time);
            self.ctx.uniform_3f(
                self.tree_uniforms.camera_pos.as_ref(),
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
            );
            self.ctx.uniform_3f(self.tree_uniforms.base_color.as_ref(), 0.2, 0.8, 0.6);
            self.ctx.uniform_1f(self.tree_uniforms.ambient_strength.as_ref(), 0.3);

            gl.bind_vertex_array(self.tree_vao.as_ref());
            gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                self.tree_index_count,
                WebGl2RenderingContext::UNSIGNED_INT,
                0,
            );
        }

        // Render particles
        if self.particle_vao.is_some() && self.particle_count > 0 {
            gl.use_program(Some(&self.particle_program));
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            self.ctx.enable_additive_blending();

            self.ctx.uniform_matrix4fv(self.particle_uniforms.view.as_ref(), view.as_slice());
            self.ctx.uniform_matrix4fv(self.particle_uniforms.projection.as_ref(), projection.as_slice());
            self.ctx.uniform_1f(self.particle_uniforms.time.as_ref(), time);

            gl.bind_vertex_array(self.particle_vao.as_ref());
            gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, self.particle_count);
        }

        // === Pass 2: Extract bloom ===
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, self.bloom_fbos[0].as_ref());
        self.ctx.viewport(0, 0, self.width / 2, self.height / 2);
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.disable(WebGl2RenderingContext::BLEND);

        gl.use_program(Some(&self.bloom_extract_program));
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.scene_texture.as_ref());
        self.ctx.uniform_1i(self.post_uniforms.texture.as_ref(), 0);
        self.ctx.uniform_1f(self.post_uniforms.threshold.as_ref(), 0.5);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);

        // === Pass 3: Blur horizontally ===
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, self.bloom_fbos[1].as_ref());
        gl.use_program(Some(&self.blur_program));
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.bloom_textures[0].as_ref());
        self.ctx.uniform_2f(self.post_uniforms.direction.as_ref(), 1.0, 0.0);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);

        // === Pass 4: Blur vertically ===
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, self.bloom_fbos[0].as_ref());
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.bloom_textures[1].as_ref());
        self.ctx.uniform_2f(self.post_uniforms.direction.as_ref(), 0.0, 1.0);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);

        // === Pass 5: Composite ===
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        self.ctx.viewport(0, 0, self.width, self.height);

        gl.use_program(Some(&self.composite_program));

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.scene_texture.as_ref());
        self.ctx.uniform_1i(self.post_uniforms.scene.as_ref(), 0);

        gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.bloom_textures[0].as_ref());
        self.ctx.uniform_1i(self.post_uniforms.bloom.as_ref(), 1);

        self.ctx.uniform_1f(self.post_uniforms.bloom_strength.as_ref(), 0.8);
        self.ctx.uniform_1f(self.post_uniforms.vignette_strength.as_ref(), 0.4);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
    }

    /// Resize the render pipeline
    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        self.create_framebuffers()
    }
}
