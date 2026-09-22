//! Two-entry layout retention shared by the shell and panel adapters.
pub(crate) struct LayoutCache<K, V> {
    entries: [Option<(K, V)>; 2],
    next: usize,
}

impl<K, V> Default for LayoutCache<K, V> {
    fn default() -> Self {
        Self {
            entries: [None, None],
            next: 0,
        }
    }
}

impl<K: PartialEq, V: Clone> LayoutCache<K, V> {
    pub(crate) fn resolve(&mut self, key: K, solve: impl FnOnce() -> V) -> V {
        self.try_resolve(key, || Some(solve()))
            .expect("infallible layout solve")
    }

    pub(crate) fn try_resolve(&mut self, key: K, solve: impl FnOnce() -> Option<V>) -> Option<V> {
        for (previous, value) in self.entries.iter().flatten() {
            if *previous == key {
                return Some(value.clone());
            }
        }
        let value = solve()?;
        self.entries[self.next] = Some((key, value.clone()));
        self.next = (self.next + 1) % self.entries.len();
        Some(value)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.iter().flatten().count()
    }
}
