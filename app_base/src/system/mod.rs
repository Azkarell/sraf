use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use log::info;
use winit::event_loop::ActiveEventLoop;

use crate::{
    ApplicationEvent, ChannelResource, EventReader, EventWriter,
    storage::{Res, ResMut, ResOwned, Resource, Resources},
    system::scheduler::{Scheduler, StoredSystem},
};

pub mod commands;
pub mod function_system;
pub mod scheduler;
pub mod window_event_system;

pub trait SystemParam {
    type Item<'new>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r>;
    fn prepare<'r>(_resources: &'r mut Resources) {}
}

pub struct StoredSystemsContainer {
    systems: Vec<StoredSystem>,
}

pub trait IntoContainer {
    fn into_container(&self) -> StoredSystemsContainer;
}

pub trait IntoStoredSystem {
    fn into_stored_system(self) -> StoredSystem;
}


pub trait IntoStoredSystems<I> {
    fn into_stored_systems(self) -> impl Iterator<Item = StoredSystem>;
}


macro_rules! impl_into_stored_systems {
    ($($ty:ident, $i:ident),*) => {
         #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl<$($ty, $i),*> IntoStoredSystems<($($i,)*)> for ($($ty,)*) 
        where $($ty: IntoSystem<$i>),*, $($ty::System : 'static),*{
            fn into_stored_systems(self) -> impl Iterator<Item = StoredSystem> {
                let ($($ty,)*) = self;
                vec![$($ty.into_system().into_stored_system(),)*].into_iter()
            }
        }
         
    };
}

impl_into_stored_systems!(T1, I1);
impl_into_stored_systems!(T1, I1, T2, I2, T3, I3);
impl_into_stored_systems!(T1, I1, T2, I2, T3, I3, T4, I4);
impl_into_stored_systems!(T1, I1, T2, I2, T3, I3, T4, I4, T5, I5);
impl_into_stored_systems!(T1, I1, T2, I2, T3, I3, T4, I4, T5, I5, T6, I6);


impl<T, I> IntoStoredSystems<I> for T  where T: IntoSystem<I>, T::System : 'static {
    fn into_stored_systems(self) -> impl Iterator<Item = StoredSystem> {
        vec![self.into_system().into_stored_system()].into_iter()
    }
}

impl<T1, I1, T2, I2> IntoStoredSystems<(I1, I2)> for (T1, T2)
where T1: IntoSystem<I1>, T2: IntoSystem<I2>, T1::System :'static, T2::System : 'static
{
    fn into_stored_systems(self) -> impl Iterator<Item = StoredSystem> {
        vec![self.0.into_system().into_stored_system(), self.1.into_system().into_stored_system()].into_iter()
    }
}


impl<S: System + 'static> IntoStoredSystem for S

{
    fn into_stored_system(self) -> StoredSystem {
        Box::new(self)
    }
}

// macro_rules! impl_into_stored_systems {
//     ($($ty:ident),*) => {
//          #[allow(
//             non_snake_case,
//             reason = "Certain variable names are provided by the caller, not by us."
//         )]
//         impl<$($ty : IntoStoredSystem),*> IntoStoredSystems for ($($ty),*) {
//             fn into_stored_systems(self) -> impl Iterator<Item = StoredSystem> {
//                 let ($($ty),*) = self;
//                 vec![$($ty.into_stored_system()),*].into_iter()

//             }
//         }
//     };
// }




// impl_into_stored_systems!(T1,);
// impl_into_stored_systems!(T1, T2);
// impl_into_stored_systems!(T1, T2, T3);
// impl_into_stored_systems!(T1, T2, T3, T4);
// impl_into_stored_systems!(T1, T2, T3, T4, T5);
// impl_into_stored_systems!(T1, T2, T3, T4, T5, T6);




pub trait IntoSystem<Input> {
    type System: System;
    fn into_system(self) -> Self::System;
}

pub trait IntoWindowEventSystem<Input> {
    type System<'new>: WindowEventSystem;
    fn into_system<'r>(self) -> Self::System<'r>;
}

pub trait WindowEventSystem {
    fn run(
        &mut self,
        window_event: ApplicationEvent,
        event_loop: &ActiveEventLoop,
        resources: &mut Resources,
    );
}

pub trait System {
    fn run(&mut self, resources: &mut Resources);
}

impl<T: Resource + 'static> SystemParam for Option<Res<'_, T>> {
    type Item<'new> = Option<Res<'new, T>>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        resources.get::<T>()
    }
}

impl<T: Resource + 'static> SystemParam for Option<ResMut<'_, T>> {
    type Item<'new> = Option<ResMut<'new, T>>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        resources.get_mut::<T>()
    }
}

impl<T: Resource + Clone + 'static> SystemParam for Option<ResOwned<T>> {
    type Item<'new> = Option<ResOwned<T>>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        resources.get::<T>().map(|val| ResOwned::new(val.clone()))
    }
}

impl<T: Resource + 'static> SystemParam for Res<'_, T> {
    type Item<'new> = Res<'new, T>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        Option::<Res<'r, T>>::param(resources).unwrap()
    }
}

impl<T: Resource + 'static> SystemParam for ResMut<'_, T> {
    type Item<'new> = ResMut<'new, T>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        Option::<ResMut<'r, T>>::param(resources).unwrap()
    }
}
impl<T: Resource + Clone + 'static> SystemParam for ResOwned<T> {
    type Item<'new> = ResOwned<T>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        Option::<ResOwned<T>>::param(resources).unwrap()
    }
}

// impl<T1: SystemParam, T2: SystemParam> SystemParam for (T1, T2) {
//     type Item<'new> = (T1::Item<'new>, T2::Item<'new>) ;

//     fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
//         (
//             T1::param(resources),
//             T2::param(resources),
//         )
//     }

// }

impl<T: Clone + 'static> SystemParam for EventReader<T> {
    type Item<'new> = EventReader<T>;

    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        let resource = resources.get::<ChannelResource<T>>().unwrap();

        return EventReader::new(resource.sender.clone());
    }

    fn prepare<'r>(resources: &'r mut Resources) {
        let res = resources.get::<ChannelResource<T>>().is_none();
        if res {
            resources.add(ChannelResource::<T>::new(100));
        }
    }
}

impl<T: Clone + 'static> SystemParam for EventWriter<T> {
    type Item<'new> = EventWriter<T>;
    fn param<'r>(resources: &'r Resources) -> Self::Item<'r> {
        let resource = resources.get::<ChannelResource<T>>().unwrap();
        return EventWriter::new(resource.sender.clone());
    }

    fn prepare<'r>(resources: &'r mut Resources) {
        let res = resources.get::<ChannelResource<T>>().is_none();
        if res {
            resources.add(ChannelResource::<T>::new(100));
        }
    }
}
