#[macro_export]
macro_rules! define_widget_component_fn {
    ((has $ty:ty as $name:ident on $entity:ident)) => {
        paste::item! {
            pub fn $name<'a, E>(&self, world: &'a E
            ) -> amethyst_core::ecs::borrow::Ref<'a, $ty>
            where
                E: amethyst_core::ecs::world::EntityStore
            {
                world.get_component::<$ty>(self.$entity)
                    .expect("Component should exist on entity")
            }

            pub fn [<$name _mut>]<'a, E>(&self, world: &'a mut E
            ) -> amethyst_core::ecs::borrow::RefMut<'a, $ty>
            where
                E: amethyst_core::ecs::world::EntityStore
            {
                world.get_component_mut::<$ty>(self.$entity)
                    .expect("Component should exist on entity")
            }
        }
    }
}

#[macro_export]
macro_rules! define_widget {
    (
        $ty: ident =>
            entities: [$($field:tt),*]
            components: [$($component:tt),*]
    ) => {
        #[derive(Clone, Debug)]
        pub struct $ty {
            $(
                pub $field: crate::Entity,
            )*
        }

        impl $ty {
            pub fn new($($field: crate::Entity),*) -> Self {
                Self {
                    $($field),*
                }
            }

            $(crate::define_widget_component_fn!{ $component })*
        }
    };
}