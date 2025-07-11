use std::time::Duration;

pub use base_derive::Resource;
use runtime::Runtime;
use storage::{Res, ResMut, Resource};
use tokio::sync::broadcast::{Receiver as TokioReceiver, Sender};
pub use uuid::Uuid;
pub use winit::*;
pub use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::EventLoopExtPumpEvents,
    window::WindowId,
};
pub mod runtime;
pub mod storage;
pub mod system;

pub mod math {
    pub use nalgebra::*;
    pub type Vec4 = Vector4<f32>;
    pub type Vec3 = Vector3<f32>;
    pub type Mat4 = Matrix4<f32>;
    pub type Mat3 = Matrix3<f32>;
    pub type Vec2 = Vector2<f32>;
    pub type UVec2 = Vector2<u32>;
}

use system::IntoSystem;

use crate::system::scheduler::{Label, StoredSystem};
use crate::system::{IntoStoredSystem, IntoStoredSystems};
use crate::system::{IntoWindowEventSystem, commands::CommandList, scheduler::Scheduler};

extern crate self as app_base;

pub struct App {
    plugins: Vec<PluginLifetime>,
    scheduler: Scheduler,
}

pub struct PluginLifetime {
    plugin: Box<dyn Plugin>,
    plugin_state: PluginState,
}

impl PluginLifetime {
    pub fn startup(plugin: Box<dyn Plugin>) -> Self {
        PluginLifetime {
            plugin,
            plugin_state: PluginState::Startup,
        }
    }

    pub fn state(&self) -> PluginState {
        self.plugin_state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Startup,
    Exiting,
    Running,
    Paused,
}

impl Plugin for PluginLifetime {
    fn build(&self, app: &mut App) {
        self.plugin.build(app);
    }
}

pub struct Quit;

pub struct EventReader<T>(tokio::sync::broadcast::Receiver<T>);

impl<T: Clone> Clone for EventReader<T> {
    fn clone(&self) -> Self {
        Self(self.0.resubscribe())
    }
}

#[derive(Clone)]
pub struct EventWriter<T>(tokio::sync::broadcast::Sender<T>);

impl<T: Clone> EventReader<T> {
    pub fn new(sender: tokio::sync::broadcast::Sender<T>) -> Self {
        EventReader(sender.subscribe())
    }
    pub async fn recv(&mut self) -> Result<T, tokio::sync::broadcast::error::RecvError> {
        self.0.recv().await
    }
    pub fn try_recv(&mut self) -> Result<T, tokio::sync::broadcast::error::TryRecvError> {
        self.0.try_recv()
    }
}

impl<T: Clone> EventWriter<T> {
    pub fn new(sender: tokio::sync::broadcast::Sender<T>) -> Self {
        EventWriter(sender)
    }
    pub fn send(&self, event: T) -> Result<usize, tokio::sync::broadcast::error::SendError<T>> {
        self.0.send(event)
    }
}

pub trait Plugin {
    fn build(&self, app: &mut App);
}

impl App {
    pub fn new() -> Self {
        App {
            plugins: Vec::new(),
            scheduler: Scheduler::new(),
        }
    }

    pub fn add_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        plugin.build(self);
        let wrapper = PluginLifetime::startup(Box::new(plugin));
        self.plugins.push(wrapper);
    }

    pub fn add_systems<S: IntoStoredSystems<I>, I>(&mut self, label: impl Label, systems: S) {
        self.scheduler.add_systems(label, systems);
    }

    pub fn add_window_event_system<S: IntoWindowEventSystem<I> + 'static, I: 'static>(
        &mut self,
        system: S,
    ) {
        self.scheduler.add_window_event_system(system);
    }

    pub fn run_event(&mut self, event: ApplicationEvent, event_loop: &ActiveEventLoop) {
        self.scheduler.run_events(event, event_loop);
    }

    fn handle_commands(&mut self) {
        let commands = if let Some(mut commands) = self.scheduler.get_resource_mut::<CommandList>()
        {
            commands.take()
        } else {
            Vec::new()
        };
        commands
            .into_iter()
            .for_each(|command| command.execute_boxed(&mut self.scheduler));
    }

    pub fn run(&mut self) -> Result<(), winit::error::EventLoopError> {
        dotenvy::dotenv().ok();
        env_logger::init();
        let rt = Runtime::new();
        let mut event_loop = EventLoop::new()?;
        log::debug!("Starting app with {} plugins", self.plugins.len());

        self.add_resource(rt);

        self.scheduler.startup();
        self.handle_commands();
        loop {
            event_loop.pump_app_events(
                Some(Duration::from_millis(16)),
                &mut AppHandler { app: self },
            );
            self.handle_commands();

            self.scheduler.run();
            self.handle_commands();

            if self.should_close() {
                break;
            }
        }

        Ok(())
        // let mut rt = tokio::runtime::Runtime::new().unwrap();
        // rt.block_on(async move {<|cursor|>});
        // rt.spawn(move || {
        //     while !r.try_recv().is_ok() {
        //         self.plugins.iter_mut().for_each(|plugin| {
        //             plugin.update(Duration::from_millis(16), &mut self.resources);
        //         });
        //         tokio::task::yield_now().await;
        //     }
        // });
    }

    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) {
        self.scheduler.add_resource(resource);
    }

    fn should_close(&self) -> bool {
        self.scheduler.get_resource::<Quit>().is_some()
    }

    // pub fn get_resource<T: Resource + 'static>(&self) -> Option<&T> {
    //     self.resources.get()
    // }
    // pub fn get_resource_mut<T: Resource + 'static>(&mut self) -> Option<&mut T> {
    //     self.resources.get_mut()
    // }
}

#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    Quit,
    WindowEvent { id: WindowId, event: WindowEvent },
    Resumed,
    Suspended,
}

struct ChannelResource<T: 'static> {
    sender: Sender<T>,
    receiver: TokioReceiver<T>,
}

pub struct Receiver<T: Clone> {
    inner: TokioReceiver<T>,
}

impl<T: Clone> From<TokioReceiver<T>> for Receiver<T> {
    fn from(inner: TokioReceiver<T>) -> Self {
        Receiver { inner }
    }
}

impl<T: Clone> Receiver<T> {
    pub async fn recv(&mut self) -> Result<T, tokio::sync::broadcast::error::RecvError> {
        self.inner.recv().await
    }

    pub fn try_recv(&mut self) -> Result<T, tokio::sync::broadcast::error::TryRecvError> {
        self.inner.try_recv()
    }
}

impl<T: Clone> ChannelResource<T> {
    pub fn new(cap: usize) -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(cap);
        ChannelResource { sender, receiver }
    }

    pub fn send(&self, value: T) -> Result<usize, tokio::sync::broadcast::error::SendError<T>> {
        self.sender.send(value)
    }

    pub fn subscribe(&mut self) -> Receiver<T> {
        Receiver {
            inner: self.receiver.resubscribe(),
        }
    }
}

struct AppHandler<'a> {
    app: &'a mut App,
}

impl ApplicationHandler for AppHandler<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app.run_event(ApplicationEvent::Resumed, event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.app.run_event(
            ApplicationEvent::WindowEvent {
                id: window_id,
                event,
            },
            event_loop,
        )
    }
}
