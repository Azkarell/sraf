use std::marker::PhantomData;

use crate::{storage::Resources, system::{IntoSystem, System, SystemParam}};




pub struct FunctionSystem<F, In>
{
    f: F,
    marker: PhantomData<fn() -> In>
}


// impl<F: FnMut()> System for FunctionSystem<F, ()> {
//     fn run(&mut self, _resources: &Resources) {
//         (self.f)();
//     }
// }

// impl<F: FnMut(In1), In1: SystemParam> System for FunctionSystem<F, (In1)> {
//     fn run(&mut self, resources: &Resources, in1: In1) {
//         (self.f)(In1::param(resources));
//     }
// }



macro_rules! impl_system_function {
    
    ($($ty:ident),*) => {
          #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl<F: FnMut($($ty,)*), $($ty,)*> System for FunctionSystem<F, ($($ty),*)>
        where 
            $($ty : SystemParam,)*
            for<'a, 'b> &'a mut F: FnMut($($ty,)*) + FnMut($(<$ty as SystemParam>::Item<'b>,)*) { 
            fn run(&mut self, _resources: &mut Resources) {
               
                $({
                    $ty::prepare(_resources);
                })*
                fn call_inner<$($ty,)*>(mut f: impl FnMut($($ty,)*), $($ty: $ty,)*) {
                    f($($ty,)*);
                }
                call_inner(&mut self.f, $($ty::param(_resources),)*)
            }
        }

    };
}

impl_system_function!();
impl_system_function!(In1);
impl_system_function!(In1, In2);
impl_system_function!(In1, In2, In3);
impl_system_function!(In1, In2, In3, In4);
impl_system_function!(In1, In2, In3, In4, In5);




macro_rules! impl_into_system {
    
    ($($ty:ident),*) => {
          #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl<F: FnMut($($ty,)*), $($ty,)*> IntoSystem<($($ty,)*)> for F
        where 
            $($ty : SystemParam,)*
            for<'a, 'b> &'a mut F: FnMut($($ty,)*) + FnMut($(<$ty as SystemParam>::Item<'b>,)*) { 
            type System = FunctionSystem<F, ($($ty),*)>;
            fn into_system(self) -> Self::System {
                FunctionSystem{ f: self, marker: PhantomData }
            }
        }

    };
}

impl_into_system!();
impl_into_system!(In1);
impl_into_system!(In1, In2);
impl_into_system!(In1, In2, In3);
impl_into_system!(In1, In2, In3, In4);
impl_into_system!(In1, In2, In3, In4, In5);