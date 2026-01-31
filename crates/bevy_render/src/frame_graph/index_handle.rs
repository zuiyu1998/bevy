use core::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub struct IndexHandle<T> {
    pub index: usize,
    _marker: PhantomData<T>,
}

impl<T: Any> Hash for IndexHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        let id = TypeId::of::<T>();
        id.hash(state);
    }
}

impl<T> Eq for IndexHandle<T> {}

impl<T> PartialEq for IndexHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Copy for IndexHandle<T> {}

impl<T> Clone for IndexHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> IndexHandle<T> {
    pub fn new(index: usize) -> Self {
        IndexHandle {
            index,
            _marker: PhantomData,
        }
    }
}
