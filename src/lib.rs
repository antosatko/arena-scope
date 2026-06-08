use arena::{Arena, Key};

use stack::{Snapshot, SnapshotStack};

pub mod stack;

pub type ScopeKey = Key<ArenaTag>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArenaTag;

#[derive(Debug)]
pub struct ScopeNode<K, V> {
    pub parent: Option<ScopeKey>,
    pub values: Vec<(K, V)>,
}

impl<K, V> ScopeNode<K, V>
where
    K: PartialEq,
{
    #[inline]
    pub fn new(parent: Option<ScopeKey>) -> Self {
        Self {
            parent,
            values: Vec::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.values.push((key, value));
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.values
            .iter()
            .rev()
            .find_map(|(k, v)| (k == key).then_some(v))
    }
}

#[derive(Debug)]
pub struct ScopeTree<K, V> {
    arena: Arena<ScopeNode<K, V>, ArenaTag>,
    stack: SnapshotStack<ScopeKey>,
}

impl<K, V> Default for ScopeTree<K, V> {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
            stack: SnapshotStack::default(),
        }
    }
}

impl<K, V> ScopeTree<K, V>
where
    K: PartialEq,
{
    #[inline]
    pub fn init(&mut self) -> ScopeKey {
        let root = self.arena.push(ScopeNode::new(None));

        self.stack.push(root);

        root
    }

    #[inline]
    pub fn current(&self) -> ScopeKey {
        *self.stack.current()
    }

    #[inline]
    pub fn push(&mut self) -> ScopeKey {
        let key = self.arena.push(ScopeNode::new(Some(*self.stack.current())));

        self.stack.push(key);

        key
    }

    #[inline]
    pub fn pop(&mut self) -> ScopeKey {
        self.stack.pop()
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        let current = *self.stack.current();

        self.arena.get_mut_unchecked(&current).insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut current = Some(*self.stack.current());

        while let Some(scope) = current {
            let node = self.arena.get_unchecked(&scope);

            if let Some(value) = node.get(key) {
                return Some(value);
            }

            current = node.parent;
        }

        None
    }

    #[inline]
    pub fn snapshot(&self) -> Option<Snapshot> {
        self.stack.snapshot()
    }

    #[inline]
    pub fn restore(&mut self, snapshot: Snapshot) {
        self.stack.restore(snapshot);
    }

    #[inline]
    pub fn arena(&self) -> &Arena<ScopeNode<K, V>, ArenaTag> {
        &self.arena
    }

    #[inline]
    pub fn node(&self, key: &ScopeKey) -> &ScopeNode<K, V> {
        self.arena.get_unchecked(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_in_current_scope() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("x", 42);

        assert_eq!(scopes.get(&"x"), Some(&42));
    }

    #[test]
    fn lookup_in_parent_scope() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("x", 1);

        scopes.push();

        assert_eq!(scopes.get(&"x"), Some(&1));
    }

    #[test]
    fn shadowing() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("x", 1);

        scopes.push();

        scopes.insert("x", 2);

        assert_eq!(scopes.get(&"x"), Some(&2));
    }

    #[test]
    fn pop_restores_parent_lookup() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("x", 1);

        scopes.push();
        scopes.insert("x", 2);

        assert_eq!(scopes.get(&"x"), Some(&2));

        scopes.pop();

        assert_eq!(scopes.get(&"x"), Some(&1));
    }

    #[test]
    fn snapshot_restore() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("a", 1);

        let snap = scopes.snapshot().unwrap();

        scopes.push();
        scopes.insert("b", 2);

        assert_eq!(scopes.get(&"b"), Some(&2));

        scopes.restore(snap);

        assert_eq!(scopes.get(&"a"), Some(&1));
        assert_eq!(scopes.get(&"b"), None);
    }

    #[test]
    fn deep_scope_lookup() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        scopes.insert("root", 0);

        scopes.push();
        scopes.insert("a", 1);

        scopes.push();
        scopes.insert("b", 2);

        scopes.push();
        scopes.insert("c", 3);

        assert_eq!(scopes.get(&"root"), Some(&0));
        assert_eq!(scopes.get(&"a"), Some(&1));
        assert_eq!(scopes.get(&"b"), Some(&2));
        assert_eq!(scopes.get(&"c"), Some(&3));
    }

    #[test]
    fn snapshot_backtracking() {
        let mut scopes = ScopeTree::<&str, i32>::default();

        scopes.init();

        let root = scopes.snapshot().unwrap();

        scopes.push();
        scopes.insert("x", 1);

        let branch = scopes.snapshot().unwrap();

        scopes.push();
        scopes.insert("y", 2);

        assert_eq!(scopes.get(&"x"), Some(&1));
        assert_eq!(scopes.get(&"y"), Some(&2));

        scopes.restore(branch);

        assert_eq!(scopes.get(&"x"), Some(&1));
        assert_eq!(scopes.get(&"y"), None);

        scopes.restore(root);

        assert_eq!(scopes.get(&"x"), None);
        assert_eq!(scopes.get(&"y"), None);
    }
}
