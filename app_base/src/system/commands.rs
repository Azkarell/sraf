use std::{any::{Any, TypeId}, cell::RefCell, marker::PhantomData};

use crate::{storage::{ResMut, Resource, Resources}, system::{scheduler::{self, Scheduler}, SystemParam}};


pub trait BoxedCommand {
    fn execute_boxed(self: Box<Self>, scheduler: &mut Scheduler);
}

pub trait Command: BoxedCommand {
    fn execute(self, scheduler: &mut Scheduler);
}

impl<T> BoxedCommand for T where T: Command {
    fn execute_boxed(self: Box<Self>, scheduler: &mut Scheduler) {
        (*self).execute(scheduler);
    }
}

impl<T> Command for Box<T> where T: ?Sized + Command {
    fn execute(self, scheduler: &mut Scheduler) {
        self.execute_boxed(scheduler);
    }
}

pub struct AddResource {
    pub id: TypeId,
    pub resource: RefCell<Box<dyn Any>>,
}

impl Command for AddResource {
    fn execute(self, scheduler: &mut Scheduler) {
        scheduler.insert_entity((self.id, self.resource));
    }
}


pub struct RemoveResource<T> {
    pub marker: PhantomData<T>
}

impl<T: 'static> Command for RemoveResource<T> {
    fn execute(self, scheduler: &mut Scheduler) {
        scheduler.remove_resource::<T>();
    }
}

impl<T> RemoveResource<T> {
    pub fn new() -> Self {
        RemoveResource { marker: PhantomData }   
    }
}

pub struct CommandList {
    commands: Vec<Box<dyn Command>>,
}

impl CommandList {
    pub fn take(&mut self) -> Vec<Box<dyn Command>> {
        let commands = std::mem::replace(&mut self.commands, Vec::new());
        commands
    }
}

pub struct Commands<'a> {
    list: ResMut<'a, CommandList>,
}

impl Commands<'_> {
    pub fn from_resources(resources: &Resources) -> Commands {
        Commands {
            list: resources.get_mut().unwrap(),
        }
    }

    pub fn insert_resource<T: Resource + 'static>(&mut self, resource: T) {
        self.add_command(AddResource{
            id: T::id(),
            resource: RefCell::new(Box::new(resource)),
        });
    }

    pub fn remove_resource<T: Resource + 'static>(&mut self) {
        self.add_command(RemoveResource::<T>::new());
    }


    pub fn add_command<T: Command + 'static>(&mut self, command: T) {
        self.list.commands.push(Box::new(command));
    }

}




impl<'a> SystemParam for Commands<'a> {
    type Item<'new> = Commands<'new>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        Self::from_resources(resources)
    }

    fn prepare<'r>(_resources: &'r mut Resources) {
        _resources.add_if_not_present(CommandList { commands: vec![] });
    }
}