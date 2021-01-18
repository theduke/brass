/// A "HashMap" that internally uses a Vec for storage.
/// This is a good storage data structure for attribute lists, since it has
/// stable ordering and most dom elements have very few attributes.

#[derive(Clone, Debug)]
pub struct VecMap<T> {
    items: Vec<T>,
}

impl<T> Default for VecMap<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> VecMap<T>
where
    T: Eq + PartialEq,
{
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }
}
