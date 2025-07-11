use std::{any::{Any, TypeId}, cell::{Ref, RefCell, RefMut}, collections::HashMap, ops::{Deref, DerefMut}};

use uuid::Uuid;



pub trait Resource {
    fn id() -> TypeId;

    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any;
    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any;
    
}

pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Any >>>,
}



// macro_rules! impl_get_disjoint_mut_x {
//     ($name:ident, $($type:ident, $var_name:ident),+) => {
//         pub fn $name<$($type: Resource + 'static),+>(&mut self) -> Option<($(&mut $type,)+)> {
//             match self.resources.get_disjoint_mut([$(&<$type>::id()),+]) {
//                 [$(Some($var_name)),+] => Some(($($var_name.downcast_mut().unwrap(),)+)),
//                 _ => None,
//             }
//         }
//     };
// }

pub struct Res<'r, T: Resource> {
    inner: Ref<'r,Box< dyn Any>>,
    _marker: std::marker::PhantomData<&'r T>,
}

impl<'r, T: Resource> Res<'r, T> {
    fn new(inner: Ref<'r, Box<dyn Any>>) -> Self {
        Res {
            inner,
            _marker: std::marker::PhantomData,
            
        }
    }
}

impl<T: Resource + 'static> Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref().unwrap()   
    }
}




pub struct ResMut<'r, T: Resource> {
    inner: RefMut<'r, Box< dyn Any>>,
    _marker: std::marker::PhantomData<&'r T>,
}

impl<'r, T: Resource> ResMut<'r, T> {
    fn new(inner: RefMut<'r, Box<dyn Any>>) -> Self {   
        ResMut {
            inner,
            _marker: std::marker::PhantomData,
            
        }
    }
    

    

}
impl<'r, T: Resource + 'static> Deref for ResMut<'r, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref().unwrap()       
    }     
}  

impl<'r, T: Resource + 'static> DerefMut for ResMut<'r, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.downcast_mut().unwrap()       
    }     
}  



pub struct ResOwned<T: Resource + Clone> {
    inner: T
}

impl<T: Resource + Clone> ResOwned<T> {
    pub fn new(inner: T) -> Self {   
        ResOwned {
            inner,
        }
    }
}


impl Resources {
    pub fn new() -> Self {
        Resources {
            resources: HashMap::new(),
        }
    }

    pub fn add<T: Resource + 'static>(&mut self, resource: T) {
        self.resources.insert(T::id(), RefCell::new(Box::new(resource)));
    }
    pub fn add_if_not_present<T: Resource + 'static>(&mut self, resource: T) {
        if !self.resources.contains_key(&T::id()) { 
            self.add(resource);
        }
    }

    pub fn remove<T: Resource + 'static>(&mut self) -> Option<T>{
        let some = self.resources.remove(&T::id());
        let r = some.and_then(|any| ((any).into_inner().downcast::<T>().ok())).and_then(|t| Some(*t));
        r
    }



    pub fn add_entry(&mut self, entry: (TypeId, RefCell<Box<dyn Any>>)) {
        self.resources.insert(entry.0, entry.1);
    }

    pub fn get<T: Resource + 'static>(&self) -> Option<Res<T>> {
        self.resources
            .get(&T::id())
            .map(|cell| cell.borrow())
            .map(|c| Res::new(c))
    }

    pub fn get_mut<T: Resource + 'static>(&self) -> Option<ResMut<T>> {
        self.resources
            .get(&T::id())
            .map(|cell| cell.borrow_mut())
            .map(|c| ResMut::new(c))
    }

    // pub fn observe<T: Clone + 'static>(&mut self) -> Receiver<T> {
    //     self.get_mut::<ChannelResource<T>>()
    //         .map(|channel| channel.subscribe())
    //         .unwrap_or_else(|| {
    //             let channel = ChannelResource::<T>::new(100);
    //             self.add(channel);
    //             self.get_mut::<ChannelResource<T>>().unwrap().subscribe()
    //         })
    // }

    // pub fn send<T: Clone + 'static>(
    //     &mut self,
    //     value: T,
    // ) -> Result<usize, tokio::sync::broadcast::error::SendError<T>> {
    //     self.get::<ChannelResource<T>>()
    //         .map(|channel| channel.send(value.clone()))
    //         .unwrap_or_else(|| {
    //             let channel = ChannelResource::<T>::new(100);
    //             self.add(channel);
    //             self.get::<ChannelResource<T>>().unwrap().send(value)
    //         })
    // }

    // impl_get_disjoint_mut_x!(get_disjoint_mut_2, T1, t1, T2, t2);
    // impl_get_disjoint_mut_x!(get_disjoint_mut_3, T1, t1, T2, t2, T3, t3);
    // impl_get_disjoint_mut_x!(get_disjoint_mut_4, T1, t1, T2, t2, T3, t3, T4, t4);
    // impl_get_disjoint_mut_x!(get_disjoint_mut_5, T1, t1, T2, t2, T3, t3, T4, t4, T5, t5);
    // impl_get_disjoint_mut_x!(
    //     get_disjoint_mut_6,
    //     T1,
    //     t1,
    //     T2,
    //     t2,
    //     T3,
    //     t3,
    //     T4,
    //     t4,
    //     T5,
    //     t5,
    //     T6,
    //     t6
    // );
}




impl<T: Any> Resource for T {
    fn id() -> TypeId {
       TypeId::of::<T>()

    }

    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any {
        self
    }

    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any {
        self
    }
}
