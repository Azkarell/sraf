use std::path::Path;

use naga::{
    Arena, Constant, Expression, Handle, back::hlsl::Options, front::wgsl, valid::ValidationFlags,
};

pub fn test<P1: AsRef<Path>>(path: P1) {
    let parsed = std::fs::read_to_string(path).unwrap();

    let shader = wgsl::parse_str(&parsed).unwrap();

    let mut validator =
        naga::valid::Validator::new(ValidationFlags::all(), naga::valid::Capabilities::all());
    let info = validator.validate(&shader).unwrap();

    let opts = Options::default();
    let mut buffer = String::new();
    let mut writer = naga::back::hlsl::Writer::new(&mut buffer, &opts);
    println!("Shader: {:#?}", shader);
    let _ = writer.write(&shader, &info, None).unwrap();
    println!("hsls: {}", buffer);
}

fn build_add_expression(left: Handle<Expression>, right: Handle<Expression>) -> Expression {
    Expression::Binary {
        op: naga::BinaryOperator::Add,
        left,
        right,
    }
}

// fn test(module: Module) {

//     let entry_point_function= Function{
//         arguments: vec![
//             FunctionArgument{
//                 binding: Some(naga::Binding::Location { location: 0, interpolation: Some(naga::Interpolation::Perspective), sampling: Some(naga::Sampling::Centroid), blend_src: None }),
//                 name: Some("vertex".into()),
//                 ty:
//             }
//         ]
//     };
//     let entry_point = EntryPoint {
//         early_depth_test: None,
//         name: "vs_main".to_owned(),
//         stage: naga::ShaderStage::Vertex,
//         workgroup_size: [1, 1, 1],
//         workgroup_size_overrides: None,
//         function: entry_point_function
//     };
// }

// #[derive(Debug, Hash, Eq, PartialEq)]
// pub struct ShaderRef {
//     path: String,
// }

// pub struct Shader {
//     path: String,
//     module: Option<ShaderModule>,
//     window_id: Option<WindowId>,
// }

// impl<T: Into<String>> From<T> for ShaderRef {
//     fn from(value: T) -> Self {
//         ShaderRef::new(value)
//     }
// }

// impl ShaderRef {
//     pub fn new<T: Into<String>>(path: T) -> Self {
//         ShaderRef { path: path.into() }
//     }
// }

// impl Shader {
//     pub fn new<T: Into<String>>(path: T) -> Self {
//         Self {
//             module: None,
//             path: path.into(),
//             window_id: None,
//         }
//     }

//     pub fn new_with_window_id<T: Into<String>>(path: T, window_id: WindowId) -> Self {
//         Shader::new(path).with_window_id(window_id)
//     }

//     pub fn with_window_id(mut self, window_id: WindowId) -> Self {
//         self.window_id = Some(window_id);
//         self
//     }

//     pub fn with_module(mut self, module: ShaderModule) -> Self {
//         self.module = Some(module);
//         self
//     }

//     pub fn set_module(&mut self, module: ShaderModule) -> &mut Self {
//         self.module = Some(module);
//         self
//     }
//     pub fn create_ref(&self) -> ShaderRef {
//         ShaderRef::new(self.path.clone())
//     }
// }

// pub struct ShaderPlugin;

// impl Plugin for ShaderPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(Update, load_shaders);
//     }
// }

// pub struct Shaders {
//     storage: Vec<Shader>,
// }

// impl Shaders {
//     pub fn new() -> Self {
//         Shaders {
//             storage: Vec::new(),
//         }
//     }

//     pub fn add<T: Into<String>>(&mut self, path: T) -> ShaderRef {
//         let shader = Shader::new(path);
//         let sref = shader.create_ref();
//         self.storage.push(shader);
//         sref
//     }

//     pub fn remove(&mut self, sref: ShaderRef) {
//         self.storage.retain(|s| s.path != sref.path);
//     }
// }

// pub struct ReloadShaders;

// #[derive(Debug, Clone, Copy)]
// pub enum SlotType {
//     Vector,
//     Matrix,
//     Scalar,
//     Texture,
//     Shader,
// }

// pub struct Slot {
//     id: SlotId,
//     ty: SlotType,
//     name: String,
// }

// static SLOT_IDS: AtomicU32 = AtomicU32::new(0);

// impl Slot {
//     fn new(ty: SlotType, name: String) -> Self {
//         Slot {
//             id: SLOT_IDS.fetch_add(1, Ordering::Relaxed).into(),
//             ty,
//             name,
//         }
//     }

//     fn id(&self) -> SlotId {
//         self.id
//     }

//     fn ty(&self) -> SlotType {
//         self.ty
//     }

//     fn name(&self) -> &str {
//         &self.name
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct SlotId(u32);

// impl From<u32> for SlotId {
//     fn from(value: u32) -> Self {
//         SlotId(value)
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct SlotConnection {
//     from_slot: SlotId,
//     to_slot: SlotId,
// }

// static NODE_IDS: AtomicU32 = AtomicU32::new(0);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct NodeId {
//     id: u32,
// }

// impl From<u32> for NodeId {
//     fn from(value: u32) -> Self {
//         Self { id: value }
//     }
// }

// impl NodeId {
//     fn new() -> Self {
//         let id = NODE_IDS.fetch_add(1, Ordering::Relaxed);
//         Self { id }
//     }
// }

// static CONNECTION_IDS: AtomicU32 = AtomicU32::new(0);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct ConnectionId {
//     id: u32,
// }

// impl From<u32> for ConnectionId {
//     fn from(value: u32) -> Self {
//         Self { id: value }
//     }
// }

// impl ConnectionId {
//     fn new() -> Self {
//         let id = CONNECTION_IDS.fetch_add(1, Ordering::Relaxed);
//         Self { id }
//     }
// }

// pub struct ShaderContext {
//     pub nodes: FastHashMap<NodeId, Box<dyn ShaderNode>>,
//     pub connections: FastHashMap<ConnectionId, SlotConnection>,
//     pub module: Module,
//     pub output: Option<ShaderOutput>,
//     pub connection_node_lookup: FastHashMap<SlotId, NodeId>,
// }

// pub struct ShaderOutput {
//     node: NodeId,
// }

// pub trait ShaderNode {
//     fn id(&self) -> NodeId;
//     fn input_slots(&self) -> &[Slot];
//     fn output_slots(&self) -> &[Slot];

//     fn write(&mut self, context: &mut Module, inputs: &[Slot]) -> Result<(), String>;
// }

// pub struct GraphNode {
//     pub node_id: NodeId,
//     pub next: Vec<GraphNode>,
//     pub connections: HashSet<ConnectionId>,
// }

// impl GraphNode {
//     pub fn write(&mut self, context: &mut ShaderContext) -> Result<(), String> {
//         for next in &mut self.next {
//             next.write(context);
//         }
//         Ok(())
//     }

//     pub fn collect_inputs(&self, context: &mut ShaderContext) -> &[Slot] {
//         // for n in self.next {
//         //     n.
//         // }
//         // inputs
//         &[]
//     }
// }

// impl ShaderContext {
//     pub fn new() -> Self {
//         ShaderContext {
//             connections: FastHashMap::default(),
//             module: Module::default(),
//             nodes: FastHashMap::default(),
//             output: None,
//             connection_node_lookup: FastHashMap::default(),
//         }
//     }

//     pub fn set_output(&mut self, output: Option<NodeId>) {
//         self.output = output.map(|id| ShaderOutput { node: id }); // Set the output node ID
//     }

//     pub fn add_node<T: ShaderNode + 'static>(&mut self, node: T) -> NodeId {
//         let id = node.id();
//         for input_slot in node.input_slots() {
//             self.connection_node_lookup.insert(input_slot.id(), id);
//         }
//         for output_slot in node.output_slots() {
//             self.connection_node_lookup.insert(output_slot.id(), id);
//         }
//         self.nodes.insert(id, Box::new(node));
//         id
//     }

//     pub fn link_slots(&mut self, slot_in: SlotId, slot_out: SlotId) -> ConnectionId {
//         let conn_id = ConnectionId::new();
//         self.connections.insert(
//             conn_id,
//             SlotConnection {
//                 from_slot: slot_in,
//                 to_slot: slot_out,
//             },
//         );
//         conn_id
//     }

//     pub fn write_all(&mut self) -> Result<(), String> {
//         let mut graph = self.build_graph()?;
//         graph.write(self);
//         Ok(())
//     }

//     pub fn iter_slots(&self) -> impl Iterator<Item = &Slot> {
//         let mut queue = std::collections::VecDeque::new();
//         if let Some(o) = &self.output {
//             let node = self.nodes.get(&o.node).unwrap();
//             queue.push_back(node);
//         }
//         while let Some(node) = queue.pop_front() {
//             for slot in node.input_slots() {
//                 yield slot;
//             }
//         }

//     }

// }

// pub struct ResolvedShaderNode {
//     pub node_ref: NodeId,
//     pub inputs:
// }

// pub struct ShaderIter<'a> {
//     queue: VecDeque<&'a Box<dyn ShaderNode>>,
//     context: &'a ShaderContext,
// }

// impl<'a> Iterator for ShaderIter<'a> {
//     type Item =

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

// fn load_shaders(
//     reload: Option<Res<ReloadShaders>>,
//     render_resources: Res<RenderResources>,
//     mut shaders: ResMut<Shaders>,
// ) {
//     for shader in &mut shaders.storage {
//         if shader.module.is_none() || reload.is_some() {
//             if let Some(id) = shader.window_id {
//                 let resources = render_resources.get_resource(&id).unwrap();
//                 if let Ok(file) = std::fs::File::open(&shader.path) {
//                     if let Ok(contents) = std::io::read_to_string(file) {
//                         shader.module = Some(resources.device.create_shader_module(
//                             ShaderModuleDescriptor {
//                                 label: Some(&shader.path),
//                                 source: renderer::ShaderSource::Wgsl(Cow::Owned(contents)),
//                             },
//                         ));
//                     } else {
//                         error!("Failed to read shader file {}", &shader.path)
//                     }
//                 } else {
//                     error!("Failed to open shader file {} for reading", &shader.path)
//                 }
//             }
//         }
//     }
// }
