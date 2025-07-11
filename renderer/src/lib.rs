use std::{
    collections::HashMap,
    fmt::Display,
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use app_base::{
    event::WindowEvent, event_loop::ActiveEventLoop, math::{Vec3, Vec4}, runtime::Runtime, storage::{Res, ResMut}, system::{commands::{Command, Commands}, scheduler::Update, IntoSystem}, window::{Window, WindowId}, App, ApplicationEvent, Plugin
};
use log::info;
pub use wgpu::*;
pub use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgc::device::queue,
    wgt::{BufferDescriptor, CommandEncoderDescriptor},
};
use window::Windows;
pub struct RendererPlugin;

impl RendererPlugin {
    pub fn new() -> Self {
        Self
    }
}

pub struct RenderResources {
    instance: Instance,
    resources: HashMap<WindowId, RenderResource>,
    main_resource_id: Option<WindowId>,
}

pub struct RenderResource {
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window_id: WindowId,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct MeshId {
    id: u64,
}
struct StoredMesh {
    mesh: Option<Mesh>,
    has_buffer: bool,
}

pub struct Meshes {
    meshes: HashMap<MeshId, Mesh>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
}

static MESH_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

impl MeshId {
    pub fn new() -> Self {
        MeshId {
            id: MESH_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub struct ClearMeshes;

impl Command for ClearMeshes {
    fn execute(self, scheduler: &mut app_base::system::scheduler::Scheduler) {
        info!("clearing meshes");
        if let Some(mut meshes) = scheduler.get_resource_mut::<Meshes>() {
            meshes.clear_meshes();
        }
        if let Some(mut render_meshes) = scheduler.get_resource_mut::<RenderMeshes>() {
            render_meshes.clear();
        }
    }
}

pub struct CopyBufferToBufferCommand {
    pub source: Buffer,
    pub destination: Buffer,
    pub destination_offset: u64,
    pub source_offset: u64,
    pub window_id: Option<WindowId>,
    pub size: u64,
}

impl CopyBufferToBufferCommand {
    pub fn execute_on_resources(self, render_resource: &mut RenderResource) {
        let device = &render_resource.device;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("copy data to buffer"),
        });

        encoder.copy_buffer_to_buffer(
            &self.source,
            self.source_offset,
            &self.destination,
            self.destination_offset,
            self.size,
        );

        let index = render_resource.queue.submit([encoder.finish()]);

        render_resource
            .device
            .poll(PollType::WaitForSubmissionIndex(index))
            .unwrap();
    }
}

impl Command for CopyBufferToBufferCommand {
    fn execute(self, scheduler: &mut app_base::system::scheduler::Scheduler) {
        let windows = scheduler.get_resource::<Windows>().unwrap();
        let mut render_resources = scheduler.get_resource_mut::<RenderResources>().unwrap();
        let render_resource = render_resources
            .get_resource_mut(&self.window_id.or_else(|| windows.main_window).unwrap())
            .unwrap();
        self.execute_on_resources(render_resource);
    }
}

pub trait CommandExt {
    fn copy_buffer(
        &mut self,
        source: Buffer,
        destination: Buffer,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
        window_id: Option<WindowId>,
    ) -> &mut Self;

    fn clear_meshes(&mut self) -> &mut Self;
}

impl<'a> CommandExt for Commands<'a> {
    fn copy_buffer(
        &mut self,
        source: Buffer,
        destination: Buffer,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
        window_id: Option<WindowId>,
    ) -> &mut Self {
        self.add_command(CopyBufferToBufferCommand {
            source: source,
            destination: destination,
            source_offset: source_offset,
            destination_offset: destination_offset,
            size: size,
            window_id: window_id,
        });
        self
    }

    fn clear_meshes(&mut self) -> &mut Self {
        self.add_command(ClearMeshes);
        self
    }
}

pub struct RenderMeshes {
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub vertex_ranges: HashMap<MeshId, Range<u64>>,
    pub index_ranges: HashMap<MeshId, Range<u32>>,
}

impl RenderMeshes {
    pub fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            vertex_ranges: HashMap::new(),
            index_ranges: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertex_buffer = None;
        self.index_buffer = None;
        self.vertex_ranges.clear();
        self.index_ranges.clear();
    }

    pub fn transfer_mesh(&mut self, render_resource: &mut RenderResource, id: MeshId, mesh: &Mesh) {
        if !self.vertex_ranges.contains_key(&id) {
            let vertex_data = unsafe {
                std::slice::from_raw_parts(
                    mesh.vertices.as_ptr() as *const _,
                    mesh.vertices.len() * 4 * 4,
                )
            };

            let as_f32 = unsafe {
                std::slice::from_raw_parts(vertex_data.as_ptr() as *const f32, vertex_data.len() / 4)
            };
            info!("vertex data {:?}", as_f32);
            let (b, r) = self.create_or_update_buffer(
                render_resource,
                vertex_data,
                &self.vertex_buffer,
                BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::MAP_READ | BufferUsages::MAP_WRITE,
            );
            self.vertex_buffer = Some(b);
            self.vertex_ranges.insert(id, r);
            info!("data {:?}", vertex_data);

            info!("mesh id {:?} vertex data len {}", id, vertex_data.len());

            let index_data = unsafe {
                std::slice::from_raw_parts(
                    mesh.indices.as_ptr() as *const _,
                    mesh.indices.len() * 4,
                )
            };

        
            info!("mesh id {:?} index data len {}", id, index_data.len());
            info!("data {:?}", index_data);
            let (b, r) = self.create_or_update_buffer(
                render_resource,
                index_data,
                &self.index_buffer,
                BufferUsages::INDEX | BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::MAP_READ | BufferUsages::MAP_WRITE,
            );
            self.index_buffer = Some(b);
            self.index_ranges.insert(id, r.start as u32..r.end as u32);
        }
    }

    pub fn draw_all(&self, render_pass: &mut RenderPass) {
        if self.vertex_buffer.is_none() {
            return;
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint32,
        );
        let count = self.index_buffer.as_ref().unwrap().size() as u32;
        render_pass.draw_indexed(0..count /4, 0, 0..1);
    }

    fn create_or_update_buffer(
        &self,
        render_resource: &mut RenderResource,
        contents: &[u8],
        old: &Option<Buffer>,
        buffer_usage: BufferUsages,
    ) -> (Buffer, Range<u64>) {
        let data_len = contents.len() as u64;
        if let Some(buffer) = old {
            let old_size = buffer.size();
            let new_size = old_size + data_len;
            let new_buffer = render_resource.device.create_buffer(&BufferDescriptor {
                label: None,
                size: new_size,
                usage: buffer_usage,
                mapped_at_creation: false,
            });

            let copy = CopyBufferToBufferCommand {
                source: buffer.clone(),
                destination: new_buffer.clone(),
                size: old_size,
                source_offset: 0,
                destination_offset: 0,
                window_id: Some(render_resource.window_id),
            };

            copy.execute_on_resources(render_resource);

            let new_data = render_resource
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    contents: contents,
                    label: None,
                    usage: buffer_usage,
                });

            let copy2 = CopyBufferToBufferCommand {
                source: new_data,
                destination: new_buffer.clone(),
                size: data_len,
                source_offset: 0,
                destination_offset: old_size,
                window_id: Some(render_resource.window_id),
            };
            copy2.execute_on_resources(render_resource);
            (new_buffer, old_size..new_size)
        } else {
            let new_data = render_resource
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    contents: contents,
                    label: None,
                    usage: buffer_usage,
                });
            (new_data, 0..data_len)
        }
    }
}

impl Meshes {
    pub fn new() -> Self {
        Meshes {
            meshes: HashMap::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        let id = MeshId::new();
        self.meshes.insert(id, mesh);
        id
    }

    pub fn get_mesh(&self, id: MeshId) -> Option<&Mesh> {
        self.meshes.get(&id)
    }

    pub fn remove_mesh(&mut self, id: MeshId) -> Option<Mesh> {
        self.meshes.remove(&id)
    }
    pub fn clear_meshes(&mut self) {
        self.meshes.clear();
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }
    // fn build_buffer(&self) {
    //     for mesh in self.meshes {}
    // }
}

pub struct Mesh {
    pub vertices: Vec<Vec4>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

}

impl RenderResources {
    pub fn new() -> Self {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        RenderResources {
            instance: instance,
            resources: HashMap::new(),
            main_resource_id: None,
        }
    }

    pub fn add_resource(&mut self, window_id: WindowId, resource: RenderResource) {
        self.resources.insert(window_id, resource);
    }

    pub fn get_resource(&self, window_id: &WindowId) -> Option<&RenderResource> {
        self.resources.get(window_id)
    }

    pub fn get_resource_mut(&mut self, window_id: &WindowId) -> Option<&mut RenderResource> {
        self.resources.get_mut(window_id)
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }

    pub fn remove(&mut self, window_id: WindowId) -> Option<RenderResource> {
        self.resources.remove(&window_id)
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn try_get_main_resource(&self) -> Option<&RenderResource> {
        self.main_resource_id
            .as_ref()
            .and_then(|id| self.resources.get(id))
    }

    pub fn main_resource_mut(&mut self) -> Option<&mut RenderResource> {
        self.main_resource_id
            .as_ref()
            .and_then(|id| self.resources.get_mut(id))
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_resource(RenderResources::new());
        app.add_resource(Meshes::new());
        app.add_resource(RenderMeshes::new());
        app.add_systems( Update, (update, transfer_meshes));
        app.add_window_event_system(on_event);
    }
}

fn update(
    mut render_resources: ResMut<RenderResources>,
    windows: Res<Windows>,
    runtime: Res<Runtime>,
) {
    render_resources.main_resource_id = windows.main_window;
    for window in &windows.windows {
        if let Some(_) = render_resources.get_resource(window.0) {
            continue;
        } else {
            create_render_resource(&mut render_resources, window.1.clone(), &runtime);
        }
    }
}

fn transfer_meshes(
    mut render_resources: ResMut<RenderResources>,
    mut meshes: ResMut<Meshes>,
    mut render_meshes: ResMut<RenderMeshes>,
) {
    if let Some(main_resource) = render_resources.main_resource_mut() {
        for (id, mesh) in &mut meshes.meshes {
            render_meshes.transfer_mesh(main_resource, *id, mesh);
            mesh.clear();
        }
    }
}

fn create_render_resource(
    render_resources: &mut RenderResources,
    window: Arc<Window>,
    runtime: &Runtime,
) {
    let surface = render_resources
        .instance
        .create_surface(window.clone())
        .expect("Failed to create surface");
    let adapter = runtime
        .block_on(
            render_resources
                .instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }),
        )
        .expect("Failed to request adapter");
    let (device, queue) = runtime
        .block_on(adapter.request_device(&DeviceDescriptor {
            required_limits: adapter.limits(),
            required_features: adapter.features(),
            ..Default::default()
        }))
        .expect("Failed to request device");

    let config = surface
        .get_default_config(
            &adapter,
            window.inner_size().width,
            window.inner_size().height,
        )
        .expect("Failed to get default config");
    surface.configure(&device, &config);

    render_resources.add_resource(
        window.id(),
        RenderResource {
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            surface_config: config,
            window_id: window.id(),
        },
    );
}

fn on_event(
    event: ApplicationEvent,
    _event_loop: &ActiveEventLoop,
    mut render_resources: ResMut<RenderResources>,
    mut meshes: ResMut<Meshes>,
    mut render_meshes: ResMut<RenderMeshes>
) {
    match event {
        ApplicationEvent::WindowEvent { id, ref event } => {
            if let WindowEvent::CloseRequested = *event {
                render_resources.remove(id);
            }

            if let WindowEvent::Resized(size) = *event {
                if let Some(render_resource) = render_resources.get_resource_mut(&id) {
                    render_resource.surface_config.width = size.width;
                    render_resource.surface_config.height = size.height;
                    render_resource
                        .surface
                        .configure(&render_resource.device, &render_resource.surface_config);
                }
            }
        }

        ApplicationEvent::Quit => {
            meshes.clear_meshes();
            render_meshes.clear();
            render_resources.clear();
        }
        _ => {}
    }
}
