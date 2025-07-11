use std::{collections::HashMap, sync::Arc, vec};

use app_base::{
    App, ApplicationEvent, Plugin,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    storage::ResMut,
    window::{Window, WindowAttributes, WindowId},
};
use log::info;

pub mod events;

pub struct WindowPlugin {
    windows: WindowConfigs,
}

impl WindowPlugin {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            windows: WindowConfigs {
                window_configs: vec![WindowConfig::new(title.to_string(), width, height)],
            },
        }
    }

    pub fn new_many(configs: Vec<WindowConfig>) -> Self {
        Self {
            windows: WindowConfigs {
                window_configs: configs,
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct Windows {
    pub windows: HashMap<WindowId, Arc<Window>>,
    pub main_window: Option<WindowId>,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            main_window: None,
        }
    }

    pub fn add_window(&mut self, window: Arc<Window>) {
        let id = window.id();
        self.windows.insert(id, window.clone());
        if self.main_window.is_none() {
            self.main_window = Some(id);
        }
    }

    pub fn get_window(&self, id: WindowId) -> Option<Arc<Window>> {
        self.windows.get(&id).cloned()
    }

    pub fn remove_window(&mut self, id: &WindowId) -> Option<Arc<Window>> {
        if let Some(window) = self.windows.remove(id) {
            if self.main_window == Some(*id) {
                self.main_window = None;
            }
            return Some(window);
        }
        None
    }

    pub fn main_window(&self) -> Arc<Window> {
        self.main_window
            .and_then(|id| self.windows.get(&id))
            .cloned()
            .unwrap()
    }
    pub fn try_get_main_window(&self) -> Option<Arc<Window>> {
        self.main_window
            .and_then(|id| self.windows.get(&id))
            .cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn clear(&mut self) {
        self.windows.clear();
        self.main_window = None;
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_resource(Windows::new());
        app.add_resource(self.windows.clone());
        app.add_window_event_system(on_window_event)
    }
}

#[derive(Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub window_id: Option<WindowId>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Window".to_string(),
            width: 800,
            height: 600,
            resizable: true,
            fullscreen: false,
            window_id: None,
        }
    }
}

impl WindowConfig {
    fn new(title: String, width: u32, height: u32) -> Self {
        Self {
            title,
            width,
            height,
            resizable: true,
            fullscreen: false,
            window_id: None,
        }
    }
}
#[derive(Clone)]
pub struct WindowConfigs {
    pub window_configs: Vec<WindowConfig>,
}
impl WindowConfigs {
    pub fn new() -> Self {
        Self {
            window_configs: vec![],
        }
    }

    pub fn add_window(mut self, title: String, width: u32, height: u32) -> Self {
        self.window_configs.push(WindowConfig::new(title, width, height));
        self
    }
}

fn on_window_event(
    event: ApplicationEvent,
    event_loop: &ActiveEventLoop,
    mut windows: ResMut<Windows>,
    mut window_configs: ResMut<WindowConfigs>,
) {
    match event {
        ApplicationEvent::WindowEvent { id, event } => {
            if let Some(_) = windows.get_window(id) {
                match event {
                    WindowEvent::CloseRequested => {
                        windows.remove_window(&id);
                    }
                    _ => {}
                }
            }
        }
        ApplicationEvent::Resumed => {
            info!("windows: {}", window_configs.window_configs.len());
            for config in &mut window_configs.window_configs {
                if config.window_id.is_none() {
                    let window = event_loop
                        .create_window(
                            WindowAttributes::default()
                                .with_title(config.title.clone())
                                .with_inner_size(LogicalSize::new(config.width, config.height)),
                        )
                        .expect("Failed to create window");
                    config.window_id = Some(window.id());
                    windows.add_window(Arc::new(window));
                }
            }
            let mut to_remove = vec![];
            for window in &windows.windows {
                if let None = window_configs
                    .window_configs
                    .iter()
                    .find(|w| w.window_id == Some(window.1.id()))
                {
                    to_remove.push(window.1.id());
                }
            }
            for id in to_remove {
                windows.remove_window(&id);
            }
        }
        ApplicationEvent::Quit => {
            windows.clear();
        }
        ApplicationEvent::Suspended => {}
    };
}
