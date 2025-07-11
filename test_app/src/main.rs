use std::{borrow::Cow, num::NonZero};

use app_base::{
    event::WindowEvent, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, math::Vec4, storage::{Res, ResMut}, system::{commands::Commands, scheduler::Update}, App, ApplicationEvent, Quit
};
use log::info;
use renderer::{
    include_wgsl, util::{BufferInitDescriptor, DeviceExt}, vertex_attr_array, wgc::device, wgt::{TextureDescriptor, TextureViewDescriptor}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferAddress, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor, CommandExt, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FragmentState, Mesh, MeshId, Meshes, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PollType, PrimitiveState, RenderMeshes, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RenderResources, ShaderModule, ShaderModuleDescriptor, ShaderStages, Texture, TextureFormat, TextureUsages, VertexAttribute, VertexBufferLayout, VertexState
};
use window::{WindowConfigs, Windows, events};
fn main() -> Result<(), String> {
    let mut app = App::new();
    app.add_plugin(window::WindowPlugin::new("Test App", 800, 600));
    app.add_plugin(renderer::RendererPlugin::new());
    app.add_systems(Update, (entry, render, prepare_render_resources));

    app.add_window_event_system(recreate);
    app.add_resource(WasEmpty { empty: true });
    app.run().map_err(|e| e.to_string())
}

struct WasEmpty {
    empty: bool,
}

fn entry(mut commands: Commands, windows: ResMut<Windows>, mut was_empty: ResMut<WasEmpty>) {
    if windows.is_empty() && was_empty.empty == false {
        commands.insert_resource(Quit);
    } else if !windows.is_empty() {
        was_empty.empty = false;
    }
}

pub struct ShaderRef {
    path: String,
}

impl<T: Into<String>> From<T> for ShaderRef {
    fn from(value: T) -> Self {
        ShaderRef { path: value.into() }
    }
}

pub struct ComputeResources {
    pub pipeline: ComputePipeline,
    pub compute_pipeline_layout: PipelineLayout,
    pub compute_bind_group_layouts: Vec<BindGroupLayout>,
    pub compute_bind_groups: Vec<BindGroup>,
    pub target: Texture,
    pub render_pipeline: RenderPipeline,
    pub render_pipeline_layout: PipelineLayout,
    pub render_bind_group_layouts: Vec<BindGroupLayout>,
    pub render_bind_groups: Vec<BindGroup>,
    pub quad: MeshId,
    // pub triangle_vertex_buffer: Buffer,
    // pub triangle_index_buffer: Buffer,
}

fn recreate(event: ApplicationEvent, _event_loop: &ActiveEventLoop, mut commands: Commands) {
    match event {
        ApplicationEvent::WindowEvent { id: _, event } => match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyR) {
                    commands.remove_resource::<ComputeResources>();
                    commands.clear_meshes();
                }
            }
            WindowEvent::Resized(_) => {
                commands.remove_resource::<ComputeResources>();
                commands.clear_meshes();

            }
            _ => {}
        },
        _ => {}
    }
}

pub struct Shader {
    pub path: String,
}

impl Shader {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }

    pub fn load(&self, device: &Device) -> ShaderModule {
        let shader_code = std::fs::read_to_string(self.path.clone()).unwrap();
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: renderer::ShaderSource::Wgsl(Cow::Borrowed(&shader_code)),
        });
        shader_module
    }
}

fn prepare_render_resources(
    mut commands: Commands,
    windows: Res<Windows>,
    render_resources: ResMut<RenderResources>,
    compute_resources: Option<Res<ComputeResources>>,
    mut meshes: ResMut<Meshes>,
) {
    if compute_resources.is_some() {
        return;
    }

    if let Some(main_window) = windows.try_get_main_window() {
        if let Some(resources) = render_resources.try_get_main_resource() {
            let mesh_id = meshes.add_mesh(Mesh {
                vertices: vec![
                    Vec4::new(-1., -1., 0., 0.),
                    Vec4::new(1., -1., 0., 0.),
                    Vec4::new(1., 1., 0., 0.),
                    Vec4::new(-1., 1., 0., 0.),
                ],
                indices: vec![0, 1, 2, 2 ,3 , 0],
            });
            let default_texture_format = resources.surface_config.format;
            info!("default texture format: {:?}", default_texture_format);
            // let output = resources.surface.get_current_texture().unwrap();
            let device = &resources.device;
            let shader = Shader::new("./test_app/shaders/compute.wgsl");

            let shader = shader.load(device);
            let target = device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: main_window.inner_size().width,
                    height: main_window.inner_size().height,
                    depth_or_array_layers: 1u32,
                },
                dimension: renderer::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
                view_formats: &[TextureFormat::Rgba8Unorm],
            });

            info!(
                "target texture size: {:?}, {:?}",
                target.size().width,
                target.size().height
            );
            let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: renderer::StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: renderer::TextureViewDimension::D2,
                    },
                    count: NonZero::new(1u32),
                }],
                label: Some("compute_bind_group_layout"),
            });
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: renderer::BindingResource::TextureView(
                        &target.create_view(&TextureViewDescriptor::default()),
                    ),
                }],
                label: None,
                layout: &layout,
            });

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&layout],
                label: None,
                push_constant_ranges: &[],
            });
            let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                cache: None,
                label: None,
                layout: Some(&pipeline_layout),
                compilation_options: PipelineCompilationOptions::default(),
                entry_point: Some("main"),
                module: &shader,
            });

            let render_bind_group_layout =
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        count: NonZero::new(1),
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: renderer::StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: renderer::TextureViewDimension::D2,
                        },
                    }],
                });

            let render_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &render_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: renderer::BindingResource::TextureView(&target.create_view(
                        &TextureViewDescriptor {
                            format: Some(TextureFormat::Rgba8Unorm),
                            ..Default::default()
                        },
                    )),
                }],
            });

            let render_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
                label: None,
                push_constant_ranges: &[],
            });

            let render_shader = Shader::new("./test_app/shaders/shader.wgsl");
            let vertex_module = render_shader.load(device);

            let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                cache: None,
                label: None,
                layout: Some(&render_layout),
                vertex: VertexState {
                    module: &vertex_module,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[VertexBufferLayout {
                        array_stride: size_of::<Vec4>() as BufferAddress,
                        step_mode: renderer::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![ 0 => Float32x4 ],
                    }],
                },
                fragment: Some(FragmentState {
                    compilation_options: PipelineCompilationOptions::default(),
                    module: &vertex_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: resources.surface_config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: renderer::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: renderer::FrontFace::Ccw,
                    cull_mode: Some(renderer::Face::Back),
                    polygon_mode: renderer::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },
                multiview: None,
            });

            commands.insert_resource(ComputeResources {
                pipeline,
                compute_pipeline_layout: pipeline_layout,
                compute_bind_group_layouts: vec![layout],
                compute_bind_groups: vec![bind_group],
                target: target,
                render_pipeline: render_pipeline,
                render_pipeline_layout: render_layout,
                render_bind_group_layouts: vec![render_bind_group_layout],
                render_bind_groups: vec![render_bind_group],
                quad: mesh_id,
            });
        }
    }
}

fn render(
    render_resources: ResMut<RenderResources>,
    windows: Res<Windows>,
    compute_resources: Option<Res<ComputeResources>>,
    render_meshes: Res<RenderMeshes>,
) {
    if let Some(compute_resources) = compute_resources {
        if let Some(main_window) = windows.try_get_main_window() {
            if let Some(resources) = render_resources.get_resource(&main_window.id()) {
                let output = resources.surface.get_current_texture().unwrap();

                let mut encoder = resources
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor::default());

                {
                    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(&compute_resources.pipeline);
                    pass.set_bind_group(0, Some(&compute_resources.compute_bind_groups[0]), &[]);
                    let workgroup_size = 16;
                    pass.dispatch_workgroups(
                        (compute_resources.target.width() as f32 / workgroup_size as f32).ceil() as u32 ,
                        (compute_resources.target.height() as f32 / workgroup_size as f32).ceil() as u32 ,
                        1,
                    );
                }

                {
                    let output_view = output.texture.create_view(&TextureViewDescriptor {
                        label: None,
                        format: Some(output.texture.format()),
                        array_layer_count: None,
                        aspect: renderer::TextureAspect::All,
                        usage: Some(TextureUsages::RENDER_ATTACHMENT),
                        ..Default::default()
                    });

                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &output_view,
                            ops: Operations {
                                load: renderer::LoadOp::Clear(Color::WHITE),
                                store: renderer::StoreOp::Store,
                            },
                            resolve_target: None,
                        })],
                        depth_stencil_attachment: None,
                        label: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    pass.set_pipeline(&compute_resources.render_pipeline);
                    pass.set_bind_group(0, Some(&compute_resources.render_bind_groups[0]), &[]);
                    render_meshes.draw_all(&mut pass);
                }
                // encoder.copy_texture_to_texture(
                //     compute_resources.target.as_image_copy(),
                //     output.texture.as_image_copy(),
                //     output.texture.size(),
                // );
                let buffer = encoder.finish();

                main_window.pre_present_notify();

                let index = resources.queue.submit([buffer]);
                let _ = resources
                    .device
                    .poll(PollType::WaitForSubmissionIndex(index));

                output.present();
            }
        }
    }
}
