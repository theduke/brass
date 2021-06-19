pub use bumpalo::collections::{String as BumpString, Vec as BumpVec};
use bumpalo::Bump;

#[derive(Debug)]
pub struct BumpMap<'bump, K, V> {
    items: BumpVec<'bump, (K, V)>,
}

impl<'bump, K, V> BumpMap<'bump, K, V>
where
    K: 'bump,
    V: 'bump,
{
    pub fn new_in(capacity: usize, bump: &'bump Bump) -> Self {
        let items = BumpVec::new_in(bump);
        Self { items }
    }

    pub fn with_capacity_in(capacity: usize, bump: &'bump Bump) -> Self {
        let items = BumpVec::with_capacity_in(capacity, bump);
        Self { items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.items.iter()
    }

    pub fn insert(&mut self, key: K, value: V) {
        // TODO: better to use binary search?
        // Probably not worthwhile because these maps will usually be small.
        for (old_key, old_value) in &mut self.items {
            if old_key == key {
                *old_value = value;
                return;
            }
        }
        self.items.push((key, value))
    }

    /// Insert an element without checking if it already exists.
    /// Can introduce duplicate keys!
    pub fn append_unchecked(&mut self, key: K, value: V) {
        self.items.push((key, value));
    }
}


impl<'bump, K, V> BumpMap<'bump, K, V>
where
    K: PartialOrd + Ord,
{
    pub fn sort_by_key(&mut self) {
        self.items.sort_by(|left, right| left.0.cmp(&right.0));
    }
}
