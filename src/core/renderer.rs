use crate::core::ecs::{System, Resource};
use crate::core::physics::TransformComponent;
use hecs::World;
use wgpu::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::{Window, WindowId},
};
use glam::{Mat4, Vec3};

/// Компонент рендеринга
#[derive(Clone, Copy)]
pub struct RenderComponent {
    pub mesh_id: usize,
    pub material_id: usize,
    pub visible: bool,
    pub scale: Vec3,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            mesh_id: 0,
            material_id: 0,
            visible: true,
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/// Менеджер ресурсов для рендеринга
pub struct RenderResourceManager {
    pub mesh_data: Vec<MeshData>,
    pub material_data: Vec<MaterialData>,
    pub texture_paths: Vec<String>,
}

impl RenderResourceManager {
    pub fn new() -> Self {
        Self {
            mesh_data: Vec::new(),
            material_data: Vec::new(),
            texture_paths: Vec::new(),
        }
    }

    pub fn add_mesh_data(&mut self, mesh_data: MeshData) -> usize {
        let id = self.mesh_data.len();
        self.mesh_data.push(mesh_data);
        id
    }

    pub fn add_material_data(&mut self, material_data: MaterialData) -> usize {
        let id = self.material_data.len();
        self.material_data.push(material_data);
        id
    }

    pub fn add_texture_path(&mut self, path: String) -> usize {
        let id = self.texture_paths.len();
        self.texture_paths.push(path);
        id
    }
    
    // Добавляем простой куб в качестве меша
    pub fn add_simple_cube(&mut self) -> usize {
        // Создадим вершины куба 1х1х1
        let vertices = vec![
            // Передняя грань (z+)
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 1.0] },
            
            // Задняя грань (z-)
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, -1.0] },
            
            // Верхняя грань (y+)
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            
            // Нижняя грань (y-)
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, -1.0, 0.0] },
            
            // Правая грань (x+)
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [0.0, 0.0], normal: [1.0, 0.0, 0.0] },
            
            // Левая грань (x-)
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 0.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0], normal: [-1.0, 0.0, 0.0] },
        ];
        
        // Индексы для рисования треугольников
        let indices = vec![
            0, 1, 2, 2, 3, 0,     // передняя грань
            4, 5, 6, 6, 7, 4,     // задняя грань
            8, 9, 10, 10, 11, 8,  // верхняя грань
            12, 13, 14, 14, 15, 12, // нижняя грань
            16, 17, 18, 18, 19, 16, // правая грань
            20, 21, 22, 22, 23, 20, // левая грань
        ];
        
        let mesh_data = MeshData {
            vertices,
            indices: Some(indices),
        };
        
        self.add_mesh_data(mesh_data)
    }
    
    // Создаем базовый материал с указанным цветом
    pub fn add_basic_material(&mut self, color: [f32; 4]) -> usize {
        let material_data = MaterialData {
            base_color: color,
            metallic: 0.0,
            roughness: 0.5,
            albedo_texture_path: None,
            normal_texture_path: None,
        };
        self.add_material_data(material_data)
    }
}

/// Структура меша
pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Option<Buffer>,
    pub num_vertices: u32,
    pub num_indices: u32,
}

/// Вспомогательная структура для хранения данных меша до создания буферов
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: None,
        }
    }
}

/// Данные цвета и текстуры материала
#[derive(Default, Clone)]
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub albedo_texture_path: Option<String>,
    pub normal_texture_path: Option<String>,
}

/// Структура материала
pub struct Material {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub albedo_texture: Option<usize>,
    pub normal_texture: Option<usize>,
    pub bind_group: Option<BindGroup>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            albedo_texture: None,
            normal_texture: None,
            bind_group: None,
        }
    }
}

/// Структура текстуры
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

/// Определение формата вершины
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

impl Vertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Система рендеринга
pub struct RenderSystem<'window> {
    instance: Instance,
    surface: Option<Surface<'window>>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    depth_texture: Option<Texture>,
    camera_bind_group: BindGroup,
    model_bind_group: BindGroup,
    light_bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    camera_buffer: Buffer,
    model_buffer: Buffer,
    material_buffer: Buffer,
}

impl<'window> RenderSystem<'window> {
    pub async fn new(window: &'window Window) -> Self {
        let size = window.inner_size();

        // Инициализация wgpu
        let instance = Instance::new(&InstanceDescriptor::default());
        
        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::default(),
                    trace: Trace::default(),
                },
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);

        // Создаем пайплайн для рендеринга (упрощенный для примера)
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("../../assets/shaders/shader.wgsl").into()),
        });

        // Создаем bind group layout для камеры
        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        
        // Создаем bind group layout для модели и материала
        let model_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[
                // Матрица модели
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Материал
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Текстура
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Сэмплер
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        // Создаем bind group layout для источника света
        let light_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Упрощенная настройка пайплайна
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &model_bind_group_layout,
                &light_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        // Создаем буфер для данных камеры
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Создаем буфер для матрицы модели
        let model_uniform = ModelUniform::new();
        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[model_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Создаем буфер для материала
        let material_uniform = MaterialUniform::new();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Создаем буфер для источника света
        let light_uniform = LightUniform::new();
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Создаем временную (заглушку) текстуру 1x1
        let temp_texture = device.create_texture(&TextureDescriptor {
            label: Some("Temp Texture"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        let temp_texture_view = temp_texture.create_view(&TextureViewDescriptor::default());
        
        // Создаем сэмплер
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest, // Используем Nearest для более четких текстур
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Инициализируем временную текстуру красным пикселем для большей видимости
        let red_pixel: [u8; 4] = [255, 0, 0, 255];
        queue.write_texture(
            ImageCopyTexture {
                texture: &temp_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &red_pixel,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Создаем bind group для камеры
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        
        // Создаем bind group для модели/материала (заглушка)
        let model_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: material_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&temp_texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("model_bind_group"),
        });
        
        // Создаем bind group для источника света (заглушка)
        let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });

        // Создаем трехмерный куб вместо плоского квадрата
        let vertices = [
            // Передняя грань (z+)
            Vertex { position: [-1.0, -1.0, 1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [1.0, -1.0, 1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-1.0, 1.0, 1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 1.0] },
            
            // Задняя грань (z-)
            Vertex { position: [1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-1.0, 1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, -1.0] },
            
            // Верхняя грань (y+)
            Vertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0, 1.0, 1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            
            // Нижняя грань (y-)
            Vertex { position: [-1.0, -1.0, 1.0], tex_coords: [0.0, 1.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [1.0, -1.0, 1.0], tex_coords: [1.0, 1.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [1.0, -1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, -1.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, -1.0, 0.0] },
            
            // Правая грань (x+)
            Vertex { position: [1.0, -1.0, 1.0], tex_coords: [0.0, 1.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [1.0, 0.0], normal: [1.0, 0.0, 0.0] },
            Vertex { position: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0], normal: [1.0, 0.0, 0.0] },
            
            // Левая грань (x-)
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-1.0, -1.0, 1.0], tex_coords: [1.0, 1.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-1.0, 1.0, 1.0], tex_coords: [1.0, 0.0], normal: [-1.0, 0.0, 0.0] },
            Vertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 0.0], normal: [-1.0, 0.0, 0.0] },
        ];
        
        let indices: &[u16] = &[
            0, 1, 2, 2, 3, 0,     // передняя грань
            4, 5, 6, 6, 7, 4,     // задняя грань
            8, 9, 10, 10, 11, 8,  // верхняя грань
            12, 13, 14, 14, 15, 12, // нижняя грань
            16, 17, 18, 18, 19, 16, // правая грань
            20, 21, 22, 22, 23, 20, // левая грань
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let num_indices = indices.len() as u32;

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_unlit"), // Используем упрощенный шейдер без освещения
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None, // Отключаем culling для отображения всех граней
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: Default::default(),
        });

        Self {
            instance,
            surface: Some(surface),
            device,
            queue,
            config,
            pipeline,
            depth_texture: None,
            camera_bind_group,
            model_bind_group,
            light_bind_group,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera_buffer,
            model_buffer,
            material_buffer,
        }
    }

    // Метод для пересоздания поверхности при необходимости
    pub fn recreate_surface(&mut self, window: &'window Window) {
        let surface = self.instance.create_surface(window).unwrap();
        surface.configure(&self.device, &self.config);
        self.surface = Some(surface);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, &self.config);
            }
            
            // Пересоздаем depth texture при изменении размера
            // ...
        }
    }

    // Публичный метод для рендеринга, который можно вызывать напрямую
    pub fn render(&mut self, world: &World, _delta_time: f32) {
        // Обновляем и рендерим сцену
        if let Err(e) = self.render_scene(world) {
            eprintln!("Ошибка рендеринга: {:?}", e);
        }
    }

    fn render_scene(&mut self, world: &World) -> Result<(), SurfaceError> {
    let surface = match &self.surface {
        Some(surface) => surface,
        None => return Ok(()),
    };
    
    // Получаем камеру из мира
    if let Some((_, camera)) = world.query::<&CameraComponent>().into_iter().next() {
        // Обновление матриц камеры
        let view_proj = camera.build_view_projection_matrix();
        
        // Обновление буфера униформ для камеры
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            view_position: [camera.position.x, camera.position.y, camera.position.z],
            _padding: 0.0,
        };
        
        // Обновляем буфер камеры
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform])
        );
    }
    
    // Создаем простую модельную матрицу для тестового куба
    let model_matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(5.0, 5.0, 5.0), // Большой куб для видимости
        glam::Quat::from_rotation_y(std::f32::consts::PI * 0.25), // Поворот для лучшего обзора
        Vec3::new(0.0, 5.0, 0.0), // Поднят выше для лучшей видимости
    );
    
    // Обновляем модельную матрицу
    let model_uniform = ModelUniform {
        model: model_matrix.to_cols_array_2d(),
    };
    
    self.queue.write_buffer(
        &self.model_buffer,
        0,
        bytemuck::cast_slice(&[model_uniform])
    );
    
    // Обновляем материал - яркий красный для видимости
    let material_uniform = MaterialUniform {
        base_color: [1.0, 0.0, 0.0, 1.0], // Красный
        metallic: 0.0,
        roughness: 0.5,
        ambient_occlusion: 1.0,
        _padding: 0.0,
    };
    
    self.queue.write_buffer(
        &self.material_buffer,
        0,
        bytemuck::cast_slice(&[material_uniform])
    );
    
    let output = surface.get_current_texture()?;
    let view = output.texture.create_view(&TextureViewDescriptor::default());
    
    let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });
    
    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.5, // Более светлый синий, чтобы видеть изменения
                        g: 0.5,
                        b: 0.8,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.model_bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_bind_group, &[]);
        
        // Устанавливаем буферы вершин и индексов
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        
        // Рисуем тестовый куб
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    
    Ok(())
}

    // Создание менеджера ресурсов для рендеринга
    pub fn create_resource_manager() -> RenderResourceManager {
        RenderResourceManager::new()
    }

    // Метод для обновления камеры
    fn update_camera(&mut self, camera: &CameraComponent) {
        // Обновление матрицы вида и проекции
        let view_proj = camera.build_view_projection_matrix();
        
        // Обновление буфера униформ для камеры
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            view_position: [camera.position.x, camera.position.y, camera.position.z],
            _padding: 0.0,
        };
        
        // Обновляем буфер камеры
        if let Some(surface) = &self.surface {
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform])
            );
        }
    }
}

impl<'window> System for RenderSystem<'window> {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        self.render(world, _delta_time);
    }
}

/// Компонент камеры
pub struct CameraComponent {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl CameraComponent {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            view_position: [0.0, 5.0, -10.0],
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
}

impl ModelUniform {
    fn new() -> Self {
        Self {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
    _padding: f32,
}

impl MaterialUniform {
    fn new() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    _padding1: f32,
    color: [f32; 3],
    _padding2: f32,
    intensity: f32,
    _padding3: [f32; 3],
}

impl LightUniform {
    fn new() -> Self {
        Self {
            position: [0.0, 5.0, -5.0],
            _padding1: 0.0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
            intensity: 1.0,
            _padding3: [0.0, 0.0, 0.0],
        }
    }
} 