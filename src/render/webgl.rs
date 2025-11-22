use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlUniformLocation, WebGlVertexArrayObject, WebGlTexture, WebGlFramebuffer,
};

/// Wrapper around WebGL2 context with helper methods
pub struct WebGLContext {
    pub gl: WebGl2RenderingContext,
}

impl WebGLContext {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self { gl }
    }

    /// Compile a shader from source
    pub fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let gl = &self.gl;

        let shader = gl.create_shader(shader_type)
            .ok_or("Failed to create shader")?;

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let log = gl.get_shader_info_log(&shader).unwrap_or_default();
            gl.delete_shader(Some(&shader));
            Err(format!("Shader compilation failed: {}", log))
        }
    }

    /// Create a shader program from vertex and fragment shaders
    pub fn create_program(&self, vert_src: &str, frag_src: &str) -> Result<WebGlProgram, String> {
        let gl = &self.gl;

        let vert_shader = self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_src)?;
        let frag_shader = self.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_src)?;

        let program = gl.create_program().ok_or("Failed to create program")?;

        gl.attach_shader(&program, &vert_shader);
        gl.attach_shader(&program, &frag_shader);
        gl.link_program(&program);

        // Clean up shaders (they're linked into the program now)
        gl.delete_shader(Some(&vert_shader));
        gl.delete_shader(Some(&frag_shader));

        if gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let log = gl.get_program_info_log(&program).unwrap_or_default();
            gl.delete_program(Some(&program));
            Err(format!("Program linking failed: {}", log))
        }
    }

    /// Create a buffer and upload data
    pub fn create_buffer_f32(&self, data: &[f32], usage: u32) -> Result<WebGlBuffer, String> {
        let gl = &self.gl;

        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Safety: we're creating a view into the data slice
        unsafe {
            let array = js_sys::Float32Array::view(data);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &array,
                usage,
            );
        }

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        Ok(buffer)
    }

    /// Create an index buffer
    pub fn create_index_buffer(&self, data: &[u32], usage: u32) -> Result<WebGlBuffer, String> {
        let gl = &self.gl;

        let buffer = gl.create_buffer().ok_or("Failed to create index buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let array = js_sys::Uint32Array::view(data);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &array,
                usage,
            );
        }

        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
        Ok(buffer)
    }

    /// Create a Vertex Array Object
    pub fn create_vao(&self) -> Result<WebGlVertexArrayObject, String> {
        self.gl.create_vertex_array().ok_or("Failed to create VAO".to_string())
    }

    /// Create a texture
    pub fn create_texture(&self, width: i32, height: i32, internal_format: u32) -> Result<WebGlTexture, String> {
        let gl = &self.gl;

        let texture = gl.create_texture().ok_or("Failed to create texture")?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            internal_format as i32,
            width,
            height,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        ).map_err(|e| format!("Failed to create texture: {:?}", e))?;

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        Ok(texture)
    }

    /// Create a framebuffer with a texture attachment
    pub fn create_framebuffer(&self, texture: &WebGlTexture) -> Result<WebGlFramebuffer, String> {
        let gl = &self.gl;

        let fbo = gl.create_framebuffer().ok_or("Failed to create framebuffer")?;
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fbo));

        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(texture),
            0,
        );

        let status = gl.check_framebuffer_status(WebGl2RenderingContext::FRAMEBUFFER);
        if status != WebGl2RenderingContext::FRAMEBUFFER_COMPLETE {
            return Err(format!("Framebuffer incomplete: {}", status));
        }

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        Ok(fbo)
    }

    /// Get uniform location
    pub fn get_uniform_location(&self, program: &WebGlProgram, name: &str) -> Option<WebGlUniformLocation> {
        self.gl.get_uniform_location(program, name)
    }

    /// Set float uniform
    pub fn uniform_1f(&self, location: Option<&WebGlUniformLocation>, value: f32) {
        self.gl.uniform1f(location, value);
    }

    /// Set vec2 uniform
    pub fn uniform_2f(&self, location: Option<&WebGlUniformLocation>, x: f32, y: f32) {
        self.gl.uniform2f(location, x, y);
    }

    /// Set vec3 uniform
    pub fn uniform_3f(&self, location: Option<&WebGlUniformLocation>, x: f32, y: f32, z: f32) {
        self.gl.uniform3f(location, x, y, z);
    }

    /// Set mat4 uniform
    pub fn uniform_matrix4fv(&self, location: Option<&WebGlUniformLocation>, data: &[f32; 16]) {
        self.gl.uniform_matrix4fv_with_f32_array(location, false, data);
    }

    /// Set integer uniform
    pub fn uniform_1i(&self, location: Option<&WebGlUniformLocation>, value: i32) {
        self.gl.uniform1i(location, value);
    }

    /// Clear the screen
    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
    }

    /// Enable depth testing
    pub fn enable_depth_test(&self) {
        self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
    }

    /// Enable blending
    pub fn enable_blending(&self) {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }

    /// Enable additive blending (for particles/glow)
    pub fn enable_additive_blending(&self) {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
        );
    }

    /// Set viewport
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.gl.viewport(x, y, width, height);
    }
}
