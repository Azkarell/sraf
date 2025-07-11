use std::sync::Arc;

use app_base::window::Window;


#[derive(Clone, Debug)]
pub struct WindowCreatedEvent {
    pub window: Arc<Window>,
}



impl WindowCreatedEvent {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}

#[derive(Clone, Debug)]
pub struct WindowClosedEvent {
    pub window_id: app_base::window::WindowId,
}

impl WindowClosedEvent {
    pub fn new(window_id: app_base::window::WindowId) -> Self {
        Self { window_id }
    }
}