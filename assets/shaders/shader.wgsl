// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
    view_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(
    model_vertex: VertexInput,
) -> VertexOutput {
    let model_matrix = model;
    
    var out: VertexOutput;
    out.tex_coords = model_vertex.tex_coords;
    out.world_normal = normalize((model_matrix * vec4<f32>(model_vertex.normal, 0.0)).xyz);
    out.world_position = (model_matrix * vec4<f32>(model_vertex.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

// Fragment shader

struct Material {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
};

@group(1) @binding(1)
var<uniform> material: Material;

@group(1) @binding(2)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(3)
var s_diffuse: sampler;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
};

@group(2) @binding(0)
var<uniform> light: Light;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Базовый цвет из текстуры или униформ
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb * material.base_color.rgb;
    
    // Направление к источнику света
    let light_dir = normalize(light.position - in.world_position);
    
    // Базовое освещение (рассеянное)
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength * light.intensity;
    
    // Направление к камере
    let view_dir = normalize(camera.view_position - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    
    // Бликовое освещение
    let specular_strength = 0.5;
    let shininess = 32.0;
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular_color = light.color * spec * specular_strength * light.intensity;
    
    // Окружающее освещение
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;
    
    // Итоговый цвет
    let result = (ambient_color + diffuse_color + specular_color) * object_color;
    
    return vec4<f32>(result, material.base_color.a);
}

// Базовый шейдер для отрисовки без освещения (на случай, если нужно)
@fragment
fn fs_unlit(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * material.base_color;
} 