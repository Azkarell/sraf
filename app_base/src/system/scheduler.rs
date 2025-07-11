
use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, usize};

use log::{debug, info};
use uuid::Uuid;
use winit::event_loop::ActiveEventLoop;

use crate::{storage::{Res, ResMut, Resource, Resources}, system::{ IntoStoredSystems, IntoSystem, IntoWindowEventSystem, System, WindowEventSystem}, ApplicationEvent};


pub type StoredSystem = Box<dyn System>;
pub type StoredWindowEventSystem = Box<dyn WindowEventSystem>;

pub struct Scheduler {
    systems: HashMap<usize, Vec<StoredSystem>>,
    window_event_handler: Vec<StoredWindowEventSystem>,
    resources: Resources,
}

pub trait Label {
    fn label(&self) -> &str;
    fn order(&self) -> usize;
}

pub struct Startup;
impl Label for Startup {
    fn label(&self) -> &str {
        "Startup"
    }

    fn order(&self) -> usize {
       0
    }

}

pub struct Update;

impl Label for Update {
    fn label(&self) -> &str {
        "Update"
    }

    fn order(&self) -> usize {
        2
    }
}

impl Label for PostUpdate {
    fn label(&self) -> &str {
        "PostUpdate"
    }
    fn order(&self) -> usize {
        3
    }
}  

pub struct PostUpdate;
pub struct PreUpdate;

impl Label for PreUpdate {
    fn label(&self) -> &str {
        "PreUpdate"
    }

    fn order(&self) -> usize {
        1
    }
}




impl Scheduler {
    pub fn new() -> Self {
        Scheduler { 
            systems: HashMap::new(),
            resources: Resources::new(),
            window_event_handler: vec![],
        }
    }

    pub fn add_systems<T: IntoStoredSystems<I>, I>(&mut self, label: impl Label, systems: T){
        let priority= label.order();
        let stored =  if let Some(s) = self.systems.get_mut(&priority) {
            s
        } else {
            self.systems.insert(priority, Vec::new());
            self.systems.get_mut(&priority).unwrap()
        };
        for s in systems.into_stored_systems() {
            stored.push(s);
        }
    }


    pub fn add_window_event_system<T: IntoWindowEventSystem< I> + 'static, I: 'static>(&mut self, system: T)
   {
        self.window_event_handler.push(Box::new(system.into_system()));
    }

    pub fn insert_entity(&mut self, entity: (TypeId, RefCell<Box<dyn Any>>)) {
        self.resources.add_entry(entity)
    }

    pub fn startup(&mut self) {
        if let Some(startup) = self.systems.remove(&Startup.order()) {
            for mut s in startup {
                s.run(&mut self.resources);
            }
        }
    }

    pub fn run(&mut self) -> () {
        let mut sorted: Vec<_> = self.systems.iter_mut().collect();
        sorted.sort_by_key(|k| k.0);
        for (_, systems) in sorted {
            for s in systems {
                s.run(&mut self.resources);
            }
        }
    }

    pub fn run_events(&mut self, event: ApplicationEvent, event_loop: &ActiveEventLoop) {
        for system in &mut self.window_event_handler {
            system.run( event.clone(), event_loop, &mut self.resources);
        }
    }

    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) {
        self.resources.add(resource);
    }

    pub fn get_resource<T: Resource + 'static>(&self) -> Option<Res<T>> {
        self.resources.get::<T>()
    }
    pub fn get_resource_mut<T: Resource + 'static>(&self) -> Option<ResMut<T>> {
        self.resources.get_mut::<T>()
    }

    pub fn remove_resource<T: Resource + 'static>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }



}
