use arena::{Arena, Key};

pub mod stack;

pub type ScopeKey = Key<ArenaTag>;
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
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
    current: Option<ScopeKey>,
}

impl<K, V> Default for ScopeTree<K, V> {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
            current: None,
        }
    }
}

impl<K, V> ScopeTree<K, V>
where
    K: PartialEq,
{
    #[inline]
    pub fn init(&mut self) {
        debug_assert!(self.current.is_none());

        let root = self.arena.push(ScopeNode::new(None));

        self.current = Some(root);
    }

    #[inline]
    pub fn current(&self) -> ScopeKey {
        unsafe { self.current.unwrap_unchecked() }
    }

    #[inline]
    pub fn push(&mut self) -> ScopeKey {
        let key = self.arena.push(ScopeNode::new(self.current));

        self.current = Some(key);

        key
    }

    #[inline]
    pub fn pop(&mut self) -> ScopeKey {
        let current = self.current();

        let parent = self.arena.get_unchecked(&current).parent;

        debug_assert!(parent.is_some());

        self.current = parent;

        current
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        let current = self.current();

        self.arena.get_mut_unchecked(&current).insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut current = self.current;

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
    pub fn snapshot(&self) -> Option<ScopeKey> {
        self.current
    }

    #[inline]
    pub fn restore(&mut self, snapshot: ScopeKey) {
        self.current = Some(snapshot);
    }

    #[inline]
    pub fn arena(&mut self) -> &Arena<ScopeNode<K, V>, ArenaTag> {
        &self.arena
    }

    #[inline]
    pub fn node(&mut self, key: &ScopeKey) -> &ScopeNode<K, V> {
        &self.arena.get_unchecked(key)
    }
}
