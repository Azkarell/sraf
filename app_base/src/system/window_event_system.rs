use std::marker::PhantomData;


use crate::{storage::Resources, system::{IntoWindowEventSystem, SystemParam, WindowEventSystem, ApplicationEvent}};


use winit::event_loop::ActiveEventLoop;







pub struct WindowEventSystemFunction<F, In>
{
    f: F,
    marker: PhantomData<fn() -> In>

}




macro_rules! impl_window_event_system_function {
    
    ($($ty:ident),*) => {
          #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl< F: FnMut(ApplicationEvent, & ActiveEventLoop, $($ty,)*), $($ty,)*> WindowEventSystem for WindowEventSystemFunction<F, (ApplicationEvent, & ActiveEventLoop, $($ty),*)>
        where 
            $($ty : SystemParam,)*
            for<'a, 'b> &'a mut F: FnMut(ApplicationEvent, & ActiveEventLoop, $($ty,)*) + FnMut(ApplicationEvent, & ActiveEventLoop, $(<$ty as SystemParam>::Item<'b>,)*) { 
            fn run(&mut self, _window_event: ApplicationEvent, _event_loop: & ActiveEventLoop, _resources: &mut Resources) {
               
                $({
                    $ty::prepare(_resources);

                })*
                
                fn call_inner< $($ty,)*>(mut f: impl FnMut(ApplicationEvent, & ActiveEventLoop, $($ty,)*), window_event: ApplicationEvent, event_loop: & ActiveEventLoop, $($ty: $ty,)*) {
                    f(window_event, event_loop, $($ty,)*);
                }
                call_inner(&mut self.f, _window_event, _event_loop, $($ty::param(_resources),)*)
            }
        }

    };
}

impl_window_event_system_function!();
impl_window_event_system_function!(In1);
impl_window_event_system_function!(In1, In2);
impl_window_event_system_function!(In1, In2, In3);
impl_window_event_system_function!(In1, In2, In3, In4);
impl_window_event_system_function!(In1, In2, In3, In4, In5);




macro_rules! impl_into_window_event_system {
    
    ($($ty:ident),*) => {
          #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl< F: FnMut(ApplicationEvent, & ActiveEventLoop, $($ty,)*), $($ty,)*> IntoWindowEventSystem<($($ty,)*)> for F
        where 
            $($ty : SystemParam,)*
            for<'a, 'b> &'a mut F: FnMut(ApplicationEvent, & ActiveEventLoop, $($ty,)*) + FnMut(ApplicationEvent, & ActiveEventLoop, $(<$ty as SystemParam>::Item<'b>,)*) { 
            type System<'new> = WindowEventSystemFunction<F, (ApplicationEvent, &'new ActiveEventLoop, $($ty),*)>;
            fn into_system<'r>(self) -> Self::System<'r> {
                WindowEventSystemFunction{ f: self, marker: PhantomData }
            }
        }
    };
}

impl_into_window_event_system!();
impl_into_window_event_system!(In1);
impl_into_window_event_system!(In1, In2);
impl_into_window_event_system!(In1, In2, In3);
impl_into_window_event_system!(In1, In2, In3, In4);
impl_into_window_event_system!(In1, In2, In3, In4, In5);