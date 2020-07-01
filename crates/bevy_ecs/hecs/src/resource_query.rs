use crate::{Archetype, Component};
use core::{any::TypeId, ptr::NonNull, ops::{DerefMut, Deref}, hash::{Hasher, Hash}, marker::PhantomData};
use hashbrown::HashMap;

#[doc(hidden)]
#[derive(Debug)]
pub struct Res<'a, T: 'a> {
    #[allow(dead_code)]
    // held for drop impl
    _marker: PhantomData<&'a ()>,
    value: *const T,
}

unsafe impl<'a, T: Component> Send for Res<'a, T> {}
unsafe impl<'a, T: Component> Sync for Res<'a, T> {}
impl<'a, T: 'a> Clone for Res<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self { Res::new(self.value) }
}

impl<'a, T: 'a> Res<'a, T> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(resource: *const T) -> Self {
        Self {
            value: resource,
            _marker: PhantomData::default(),
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn map<K: 'a, F: FnMut(&T) -> &K>(&self, mut f: F) -> Res<'a, K> { Res::new(f(&self)) }
}

impl<'a, T: 'a> Deref for Res<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, T: 'a> AsRef<T> for Res<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T: 'a> std::borrow::Borrow<T> for Res<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T> PartialEq for Res<'a, T>
where
    T: 'a + PartialEq,
{
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl<'a, T> Eq for Res<'a, T> where T: 'a + Eq {}

impl<'a, T> PartialOrd for Res<'a, T>
where
    T: 'a + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<'a, T> Ord for Res<'a, T>
where
    T: 'a + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.value.cmp(&other.value) }
}

impl<'a, T> Hash for Res<'a, T>
where
    T: 'a + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ResMut<'a, T: 'a> {
    // held for drop impl
    _marker: PhantomData<&'a mut ()>,
    value: *mut T,
}

unsafe impl<'a, T: Component> Send for ResMut<'a, T> {}
unsafe impl<'a, T: Component> Sync for ResMut<'a, T> {}
impl<'a, T: 'a> Clone for ResMut<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self { ResMut::new(self.value) }
}

impl<'a, T: 'a> ResMut<'a, T> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(resource: *mut T) -> Self {
        Self {
            value: resource,
            _marker: PhantomData::default(),
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn map_into<K: 'a, F: FnMut(&mut T) -> K>(mut self, mut f: F) -> ResMut<'a, K> {
        ResMut::new(&mut f(&mut self))
    }
}

impl<'a, T: 'a> Deref for ResMut<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, T: 'a> DerefMut for ResMut<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.value } }
}

impl<'a, T: 'a> AsRef<T> for ResMut<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T: 'a> std::borrow::Borrow<T> for ResMut<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T> PartialEq for ResMut<'a, T>
where
    T: 'a + PartialEq,
{
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl<'a, T> Eq for ResMut<'a, T> where T: 'a + Eq {}

impl<'a, T> PartialOrd for ResMut<'a, T>
where
    T: 'a + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<'a, T> Ord for ResMut<'a, T>
where
    T: 'a + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.value.cmp(&other.value) }
}

impl<'a, T> Hash for ResMut<'a, T>
where
    T: 'a + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}


/// A collection of component types to fetch from a `World`
pub trait ResourceQuery {
    #[doc(hidden)]
    type Fetch: for<'a> FetchResource<'a>;
}

/// Streaming iterators over contiguous homogeneous ranges of components
pub trait FetchResource<'a>: Sized {
    /// Type of value to be fetched
    type Item: Clone;

    fn borrow(resource_archetypes: &HashMap<TypeId, Archetype>);
    fn release(resource_archetypes: &HashMap<TypeId, Archetype>);

    /// Construct a `Fetch` for `archetype` if it should be traversed
    ///
    /// # Safety
    /// `offset` must be in bounds of `archetype`
    unsafe fn get(resource_archetypes: &HashMap<TypeId, Archetype>) -> Self::Item;
}

impl<'a, T: Component> ResourceQuery for Res<'a, T> {
    type Fetch = FetchResourceRead<T>;
}

#[doc(hidden)]
pub struct FetchResourceRead<T>(NonNull<T>);

impl<'a, T: Component> FetchResource<'a> for FetchResourceRead<T> {
    type Item = Res<'a, T>;
    unsafe fn get(resource_archetypes: &HashMap<TypeId, Archetype>) -> Self::Item {
        let archetype = resource_archetypes
            .get(&TypeId::of::<T>())
            .expect("Resource does not exist");
        let res = archetype
            .get::<T>()
            .expect("Resource does not exist")
            .as_ptr();
        Res::new(res)
    }

    fn borrow(resource_archetypes: &HashMap<TypeId, Archetype>) {
        if let Some(archetype) = resource_archetypes.get(&TypeId::of::<T>()) {
            archetype.borrow::<T>();
        }
    }
    fn release(resource_archetypes: &HashMap<TypeId, Archetype>) {
        if let Some(archetype) = resource_archetypes.get(&TypeId::of::<T>()) {
            archetype.release::<T>();
        }
    }
}

impl<'a, T: Component> ResourceQuery for ResMut<'a, T> {
    type Fetch = FetchResourceWrite<T>;
}

#[doc(hidden)]
pub struct FetchResourceWrite<T>(NonNull<T>);

impl<'a, T: Component> FetchResource<'a> for FetchResourceWrite<T> {
    type Item = ResMut<'a, T>;
    unsafe fn get(resource_archetypes: &HashMap<TypeId, Archetype>) -> Self::Item {
        let archetype = resource_archetypes
            .get(&TypeId::of::<T>())
            .expect("Resource does not exist");
        let res = archetype
            .get::<T>()
            .expect("Resource does not exist")
            .as_ptr();
        ResMut::new(res)
    }

    fn borrow(resource_archetypes: &HashMap<TypeId, Archetype>) {
        if let Some(archetype) = resource_archetypes.get(&TypeId::of::<T>()) {
            archetype.borrow_mut::<T>();
        }
    }
    fn release(resource_archetypes: &HashMap<TypeId, Archetype>) {
        if let Some(archetype) = resource_archetypes.get(&TypeId::of::<T>()) {
            archetype.release_mut::<T>();
        }
    }
}

macro_rules! tuple_impl {
    ($($name: ident),*) => {
        impl<'a, $($name: FetchResource<'a>),*> FetchResource<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            #[allow(unused_variables)]
            fn borrow(resource_archetypes: &HashMap<TypeId, Archetype>) {
                $($name::borrow(resource_archetypes);)*
            }

            #[allow(unused_variables)]
            fn release(resource_archetypes: &HashMap<TypeId, Archetype>) {
                $($name::release(resource_archetypes);)*
            }

            #[allow(unused_variables)]
            unsafe fn get(resource_archetypes: &HashMap<TypeId, Archetype>) -> Self::Item {
                ($($name::get(resource_archetypes),)*)
            }
        }

        impl<$($name: ResourceQuery),*> ResourceQuery for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
        }
    };
}

smaller_tuples_too!(tuple_impl, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
