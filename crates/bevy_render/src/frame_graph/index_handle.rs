use core::marker::PhantomData;

pub struct IndexHandle<T> {
    index: usize,
    _marker: PhantomData<T>,
}

impl<T> IndexHandle<T> {
    pub const INVALID: Self = Self {
        index: usize::MAX,
        _marker: PhantomData,
    };

    pub fn new(index: usize) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_valid(&self) -> bool {
        self.index != usize::MAX
    }
}

impl<T> Clone for IndexHandle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IndexHandle<T> {
    fn default() -> Self {
        Self::INVALID.clone()
    }
}
