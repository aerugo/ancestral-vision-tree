/// Vertex shader for the tree
pub const TREE_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in float a_glow;
layout(location = 4) in float a_luminance;
layout(location = 5) in float a_hue;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform float u_time;

out vec3 v_position;
out vec3 v_normal;
out vec3 v_world_position;
out vec2 v_uv;
out float v_glow;
out float v_luminance;
out float v_hue;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);

    // Subtle breathing animation
    float breath = sin(u_time * 0.5 + a_position.y * 0.5) * 0.02 * a_luminance;
    world_pos.xyz += a_normal * breath;

    v_world_position = world_pos.xyz;
    v_position = a_position;
    v_normal = mat3(u_model) * a_normal;
    v_uv = a_uv;
    v_glow = a_glow;
    v_luminance = a_luminance;
    v_hue = a_hue;

    gl_Position = u_projection * u_view * world_pos;
}
"#;

/// Fragment shader for natural tree bark
pub const TREE_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec3 v_position;
in vec3 v_normal;
in vec3 v_world_position;
in vec2 v_uv;
in float v_glow;
in float v_luminance;
in float v_hue;

uniform vec3 u_camera_pos;
uniform float u_time;
uniform vec3 u_base_color;
uniform float u_ambient_strength;

out vec4 fragColor;

// Noise functions for bark texture
float hash(vec3 p) {
    p = fract(p * vec3(443.897, 441.423, 437.195));
    p += dot(p, p.yxz + 19.19);
    return fract((p.x + p.y) * p.z);
}

float noise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(mix(hash(i + vec3(0,0,0)), hash(i + vec3(1,0,0)), f.x),
            mix(hash(i + vec3(0,1,0)), hash(i + vec3(1,1,0)), f.x), f.y),
        mix(mix(hash(i + vec3(0,0,1)), hash(i + vec3(1,0,1)), f.x),
            mix(hash(i + vec3(0,1,1)), hash(i + vec3(1,1,1)), f.x), f.y),
        f.z
    );
}

float fbm(vec3 p) {
    float value = 0.0;
    float amplitude = 0.5;
    for(int i = 0; i < 4; i++) {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(u_camera_pos - v_world_position);

    // Natural bark color palette - browns and grays
    vec3 dark_bark = vec3(0.15, 0.10, 0.07);   // Dark brown
    vec3 mid_bark = vec3(0.35, 0.25, 0.18);    // Medium brown
    vec3 light_bark = vec3(0.50, 0.40, 0.30);  // Light brown/tan
    vec3 highlight = vec3(0.60, 0.55, 0.45);   // Highlights

    // Generate bark texture using position
    vec3 bark_pos = v_position * 3.0;

    // Vertical bark grain
    float vertical_grain = fbm(vec3(bark_pos.x * 2.0, bark_pos.y * 0.5, bark_pos.z * 2.0));

    // Horizontal rings (subtle)
    float rings = sin(bark_pos.y * 8.0 + fbm(bark_pos * 2.0) * 2.0) * 0.5 + 0.5;
    rings = smoothstep(0.3, 0.7, rings);

    // Combine for bark pattern
    float bark_pattern = mix(vertical_grain, rings, 0.3);

    // Add some noise variation
    float detail_noise = fbm(bark_pos * 8.0) * 0.3;
    bark_pattern = bark_pattern + detail_noise;

    // Mix bark colors based on pattern
    vec3 bark_color = mix(dark_bark, mid_bark, bark_pattern);
    bark_color = mix(bark_color, light_bark, smoothstep(0.5, 0.8, bark_pattern));

    // Add variation based on height (younger bark at top is smoother)
    float height_factor = clamp(v_world_position.y / 10.0, 0.0, 1.0);
    bark_color = mix(bark_color, bark_color * 1.1, height_factor * 0.2);

    // Lighting - simple directional light from above-right
    vec3 light_dir = normalize(vec3(0.5, 1.0, 0.3));
    float ndotl = max(dot(normal, light_dir), 0.0);

    // Ambient occlusion approximation (darker in crevices)
    float ao = 0.5 + 0.5 * vertical_grain;

    // Ambient light (sky blue tint from above)
    vec3 ambient = vec3(0.4, 0.45, 0.5) * u_ambient_strength * ao;

    // Diffuse lighting
    vec3 diffuse = bark_color * ndotl * 0.8;

    // Subtle rim lighting for depth
    float rim = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    vec3 rim_light = vec3(0.6, 0.55, 0.5) * rim * 0.15;

    // Specular highlight (very subtle for bark)
    vec3 half_dir = normalize(light_dir + view_dir);
    float spec = pow(max(dot(normal, half_dir), 0.0), 16.0);
    vec3 specular = highlight * spec * 0.1;

    // Fill light from below (subtle bounce light)
    vec3 fill_dir = normalize(vec3(-0.3, -0.5, 0.2));
    float fill = max(dot(normal, -fill_dir), 0.0) * 0.15;
    vec3 fill_light = vec3(0.3, 0.25, 0.2) * fill;

    // Combine lighting
    vec3 final_color = ambient * bark_color + diffuse + rim_light + specular + fill_light;

    // Distance fog (subtle atmospheric perspective)
    float dist = length(v_world_position - u_camera_pos);
    float fog = 1.0 - exp(-dist * 0.02);
    vec3 fog_color = vec3(0.6, 0.65, 0.7);
    final_color = mix(final_color, fog_color, fog * 0.3);

    // Tone mapping
    final_color = final_color / (final_color + vec3(1.0));

    // Gamma correction
    final_color = pow(final_color, vec3(1.0 / 2.2));

    fragColor = vec4(final_color, 1.0);
}
"#;

/// Vertex shader for firefly particles
pub const PARTICLE_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;
layout(location = 1) in float a_size;
layout(location = 2) in float a_alpha;
layout(location = 3) in vec3 a_color;

uniform mat4 u_view;
uniform mat4 u_projection;
uniform float u_time;

out float v_alpha;
out vec3 v_color;

void main() {
    // Flicker effect
    float flicker = sin(u_time * 10.0 + a_position.x * 100.0) * 0.3 + 0.7;
    v_alpha = a_alpha * flicker;
    v_color = a_color;

    vec4 view_pos = u_view * vec4(a_position, 1.0);
    gl_Position = u_projection * view_pos;
    gl_PointSize = a_size * (100.0 / -view_pos.z);
}
"#;

/// Fragment shader for firefly particles
pub const PARTICLE_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;

in float v_alpha;
in vec3 v_color;

out vec4 fragColor;

void main() {
    // Circular soft particle
    vec2 coord = gl_PointCoord - vec2(0.5);
    float dist = length(coord);

    if (dist > 0.5) {
        discard;
    }

    // Soft falloff
    float alpha = v_alpha * (1.0 - dist * 2.0);
    alpha = alpha * alpha; // Quadratic falloff for softer glow

    // Glow color
    vec3 glow = v_color * (1.0 + alpha);

    fragColor = vec4(glow, alpha);
}
"#;

/// Fullscreen quad vertex shader for post-processing
pub const FULLSCREEN_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

out vec2 v_uv;

void main() {
    // Fullscreen triangle
    float x = float((gl_VertexID & 1) << 2);
    float y = float((gl_VertexID & 2) << 1);
    v_uv = vec2(x * 0.5, y * 0.5);
    gl_Position = vec4(x - 1.0, y - 1.0, 0.0, 1.0);
}
"#;

/// Bloom extraction shader
pub const BLOOM_EXTRACT_SHADER: &str = r#"#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform float u_threshold;

out vec4 fragColor;

void main() {
    vec3 color = texture(u_texture, v_uv).rgb;
    float brightness = dot(color, vec3(0.2126, 0.7152, 0.0722));

    if (brightness > u_threshold) {
        fragColor = vec4(color * (brightness - u_threshold), 1.0);
    } else {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
"#;

/// Gaussian blur shader
pub const BLUR_SHADER: &str = r#"#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform vec2 u_direction;

out vec4 fragColor;

void main() {
    vec2 tex_size = vec2(textureSize(u_texture, 0));
    vec2 texel = 1.0 / tex_size;

    // 9-tap Gaussian blur
    float weights[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

    vec3 result = texture(u_texture, v_uv).rgb * weights[0];

    for (int i = 1; i < 5; i++) {
        vec2 offset = u_direction * texel * float(i) * 2.0;
        result += texture(u_texture, v_uv + offset).rgb * weights[i];
        result += texture(u_texture, v_uv - offset).rgb * weights[i];
    }

    fragColor = vec4(result, 1.0);
}
"#;

/// Final composite shader
pub const COMPOSITE_SHADER: &str = r#"#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_scene;
uniform sampler2D u_bloom;
uniform float u_bloom_strength;
uniform float u_vignette_strength;

out vec4 fragColor;

void main() {
    vec3 scene = texture(u_scene, v_uv).rgb;
    vec3 bloom = texture(u_bloom, v_uv).rgb;

    // Add bloom
    vec3 color = scene + bloom * u_bloom_strength;

    // Vignette
    vec2 uv = v_uv - 0.5;
    float vignette = 1.0 - dot(uv, uv) * u_vignette_strength;
    color *= vignette;

    // Color grading - slightly teal shadows, warm highlights
    vec3 shadows = vec3(0.0, 0.05, 0.1);
    vec3 highlights = vec3(0.05, 0.0, 0.0);
    float luma = dot(color, vec3(0.299, 0.587, 0.114));
    color += mix(shadows, highlights, luma) * 0.5;

    fragColor = vec4(color, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaders_not_empty() {
        assert!(!TREE_VERTEX_SHADER.is_empty());
        assert!(!TREE_FRAGMENT_SHADER.is_empty());
        assert!(!PARTICLE_VERTEX_SHADER.is_empty());
        assert!(!PARTICLE_FRAGMENT_SHADER.is_empty());
    }

    #[test]
    fn test_shader_version() {
        assert!(TREE_VERTEX_SHADER.contains("#version 300 es"));
        assert!(TREE_FRAGMENT_SHADER.contains("#version 300 es"));
    }
}
