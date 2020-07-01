use crate::{
    resource_query::{FetchResource, ResourceQuery},
    Component, ComponentError, Ref, RefMut, TypeInfo,
};
use core::any::TypeId;
use std::collections::HashMap;
use hecs::Archetype;

#[derive(Default)]
pub struct Resources {
    resource_archetypes: HashMap<TypeId, Archetype>,
}

impl Resources {
    pub fn insert<T: Component>(&mut self, mut resource: T) {
        let type_id = TypeId::of::<T>();
        let archetype = self.resource_archetypes.entry(type_id).or_insert_with(|| {
            let mut types = Vec::new();
            types.push(TypeInfo::of::<T>());
            let mut archetype = Archetype::new(types);
            unsafe { archetype.allocate(0) };
            archetype
        });

        unsafe {
            let resource_ptr = (&mut resource as *mut T).cast::<u8>();
            archetype.put_dynamic(resource_ptr, type_id, core::mem::size_of::<T>(), 0);
        }
    }

    pub fn get<T: Component>(&self) -> Result<Ref<'_, T>, ComponentError> {
        self.resource_archetypes
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ComponentError::NoSuchEntity)
            .and_then(|archetype| unsafe {
                Ref::new(archetype, 0).map_err(|err| ComponentError::MissingComponent(err))
            })
    }

    pub fn get_mut<T: Component>(&self) -> Result<RefMut<'_, T>, ComponentError> {
        self.resource_archetypes
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ComponentError::NoSuchEntity)
            .and_then(|archetype| unsafe {
                RefMut::new(archetype, 0).map_err(|err| ComponentError::MissingComponent(err))
            })
    }

    pub fn query<Q: ResourceQuery>(&self) -> <Q::Fetch as FetchResource>::Item {
        unsafe { Q::Fetch::get(&self.resource_archetypes) }
    }
}

unsafe impl Send for Resources {}
unsafe impl Sync for Resources {}

#[cfg(test)]
mod tests {
    use crate::Resources;

    #[test]
    fn resource() {
        let mut resources = Resources::default();
        assert!(resources.get::<i32>().is_err());

        resources.insert(123);
        assert_eq!(*resources.get::<i32>().expect("resource exists"), 123);

        resources.insert(456.0);
        assert_eq!(*resources.get::<f64>().expect("resource exists"), 456.0);

        resources.insert(789.0);
        assert_eq!(*resources.get::<f64>().expect("resource exists"), 789.0);

        {
            let mut value = resources.get_mut::<f64>().expect("resource exists");
            assert_eq!(*value, 789.0);
            *value = -1.0;
        }

        assert_eq!(*resources.get::<f64>().expect("resource exists"), -1.0);
    }
}
