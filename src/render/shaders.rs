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

/// Fragment shader for bioluminescent tree
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

// Convert HSV to RGB
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Improved noise function for organic variation
float hash(vec3 p) {
    return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453);
}

float noise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);

    float n = hash(i) * (1.0 - f.x) * (1.0 - f.y) * (1.0 - f.z)
            + hash(i + vec3(1.0, 0.0, 0.0)) * f.x * (1.0 - f.y) * (1.0 - f.z)
            + hash(i + vec3(0.0, 1.0, 0.0)) * (1.0 - f.x) * f.y * (1.0 - f.z)
            + hash(i + vec3(1.0, 1.0, 0.0)) * f.x * f.y * (1.0 - f.z)
            + hash(i + vec3(0.0, 0.0, 1.0)) * (1.0 - f.x) * (1.0 - f.y) * f.z
            + hash(i + vec3(1.0, 0.0, 1.0)) * f.x * (1.0 - f.y) * f.z
            + hash(i + vec3(0.0, 1.0, 1.0)) * (1.0 - f.x) * f.y * f.z
            + hash(i + vec3(1.0, 1.0, 1.0)) * f.x * f.y * f.z;
    return n;
}

// Fractal brownian motion for organic patterns
float fbm(vec3 p) {
    float value = 0.0;
    float amplitude = 0.5;
    for (int i = 0; i < 4; i++) {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(u_camera_pos - v_world_position);

    // Height-based gradient: warm at base (red/orange), cool at tips (cyan/green)
    float height_factor = clamp(v_world_position.y / 10.0, 0.0, 1.0);

    // Base hue transitions: red (0.0) -> orange (0.08) -> yellow (0.15) -> green (0.33) -> cyan (0.5)
    float base_hue = mix(0.02, 0.45, height_factor); // Red to cyan gradient
    float personal_hue = (v_hue / 360.0) * 0.2; // Person's hue contributes 20%
    float hue = fract(base_hue + personal_hue);

    // Saturation and value based on luminance
    float saturation = 0.7 + v_luminance * 0.25;
    float value = 0.25 + v_luminance * 0.6;
    vec3 base_color = hsv2rgb(vec3(hue, saturation, value));

    // Ambient light
    vec3 ambient = u_ambient_strength * base_color;

    // Enhanced Fresnel effect for edge glow (bioluminescence)
    float fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 4.0);
    vec3 glow_color = hsv2rgb(vec3(fract(hue + 0.08), 0.9, 1.0));
    vec3 edge_glow = fresnel * glow_color * v_glow * 3.0;

    // Energy veins - pulsing patterns that flow upward
    float vein_flow = u_time * 1.5 - v_world_position.y * 0.8;
    float vein_pattern = sin(vein_flow + v_uv.x * 20.0) * 0.5 + 0.5;
    vein_pattern *= sin(vein_flow * 0.7 + v_uv.y * 15.0) * 0.5 + 0.5;
    float veins = pow(vein_pattern, 3.0) * v_luminance;
    vec3 vein_color = hsv2rgb(vec3(fract(hue + 0.15), 0.95, 1.0));
    vec3 energy_veins = vein_color * veins * 0.6;

    // Inner bioluminescence - multi-frequency pulsing
    float pulse1 = sin(u_time * 2.0 + v_world_position.y * 2.0) * 0.5 + 0.5;
    float pulse2 = sin(u_time * 3.3 + v_world_position.y * 1.5 + 1.0) * 0.5 + 0.5;
    float pulse3 = sin(u_time * 0.7 + v_world_position.y * 3.0 + 2.0) * 0.5 + 0.5;
    float combined_pulse = (pulse1 + pulse2 * 0.5 + pulse3 * 0.25) / 1.75;
    float inner_glow = v_luminance * (0.4 + combined_pulse * 0.6);
    vec3 bio_color = hsv2rgb(vec3(fract(hue + 0.05), 0.85, 1.0));
    vec3 bioluminescence = bio_color * inner_glow * 0.7;

    // Subsurface scattering approximation - enhanced
    float sss = max(dot(-normal, view_dir), 0.0) * 0.4 * v_luminance;
    vec3 sss_color = hsv2rgb(vec3(fract(hue - 0.05), 0.7, 1.0));
    vec3 subsurface = sss_color * sss;

    // Organic bark texture using fbm
    float bark = fbm(v_position * 5.0 + vec3(0.0, u_time * 0.05, 0.0)) * 0.15;
    float bark_detail = noise(v_position * 20.0) * 0.08;

    // Core glow - strongest at center of branches (based on luminance)
    float core_intensity = v_luminance * v_luminance * 0.5;
    vec3 core_color = hsv2rgb(vec3(fract(hue + 0.1), 0.6, 1.0));
    vec3 core_glow = core_color * core_intensity;

    // Combine all lighting
    vec3 final_color = ambient + edge_glow + energy_veins + bioluminescence + subsurface + core_glow;
    final_color *= (1.0 + bark + bark_detail);

    // Ethereal atmosphere with height-based fog
    float atmosphere = exp(-length(v_world_position) * 0.08) * 0.15;
    float height_fog = exp(-v_world_position.y * 0.15) * 0.1;
    vec3 fog_color = hsv2rgb(vec3(0.55, 0.3, 0.2)); // Soft teal fog
    final_color += fog_color * (atmosphere + height_fog);

    // Magical sparkle effect on high-luminance areas
    float sparkle = noise(v_position * 50.0 + u_time * 5.0);
    sparkle = pow(sparkle, 20.0) * v_luminance * 2.0;
    final_color += vec3(1.0) * sparkle;

    // HDR tone mapping (ACES approximation)
    final_color = final_color * (2.51 * final_color + 0.03) / (final_color * (2.43 * final_color + 0.59) + 0.14);

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
