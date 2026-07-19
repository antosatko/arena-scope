use arena::{Arena, Key};

pub type StackKey = Key<ArenaTag>;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub struct ArenaTag;

#[derive(Debug, Clone)]
pub struct StackNode<T> {
    pub parent: Option<StackKey>,
    pub value: T,
}

impl<T> StackNode<T> {
    #[inline]
    #[track_caller]
    pub fn new(parent: Option<StackKey>, value: T) -> Self {
        Self { parent, value }
    }
}

#[derive(Debug, Clone)]
pub struct Stack<T> {
    arena: Arena<StackNode<T>, ArenaTag>,
    top: Option<StackKey>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
            top: None,
        }
    }
}

impl<T> Stack<T> {
    /// Initialize stack with an optional initial value.
    #[inline]
    #[track_caller]
    pub fn init(&mut self, value: T) {
        debug_assert!(self.top.is_none());

        let root = self.arena.push(StackNode::new(None, value));
        self.top = Some(root);
    }

    #[inline]
    #[track_caller]
    pub fn is_empty(&self) -> bool {
        self.top.is_none()
    }

    #[inline]
    #[track_caller]
    pub fn push(&mut self, value: T) -> StackKey {
        let parent = self.top;

        let key = self.arena.push(StackNode::new(parent, value));
        self.top = Some(key);

        key
    }

    #[inline]
    #[track_caller]
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let current = self.top?;

        let node = self.arena.get_unchecked(&current);
        let value = node.value.clone();

        self.top = node.parent;

        Some(value)
    }

    #[inline]
    #[track_caller]
    pub fn get(&self) -> Option<&T> {
        let current = self.top?;
        Some(&self.arena.get_unchecked(&current).value)
    }

    #[inline]
    #[track_caller]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let current = self.top?;
        Some(&mut self.arena.get_mut_unchecked(&current).value)
    }

    #[inline]
    #[track_caller]
    pub fn get_unchecked(&self) -> &T {
        let current = self.top.unwrap();
        &self.arena.get_unchecked(&current).value
    }

    #[inline]
    #[track_caller]
    pub fn get_mut_unchecked(&mut self) -> &mut T {
        let current = self.top.unwrap();
        &mut self.arena.get_mut_unchecked(&current).value
    }

    #[inline]
    #[track_caller]
    pub fn snapshot(&self) -> Option<StackKey> {
        self.top
    }

    #[inline]
    #[track_caller]
    pub fn restore(&mut self, snapshot: StackKey) {
        self.top = Some(snapshot);
    }

    #[inline]
    #[track_caller]
    pub fn current_key(&self) -> Option<StackKey> {
        self.top
    }

    #[inline]
    #[track_caller]
    pub fn node(&self, key: &StackKey) -> &StackNode<T> {
        self.arena.get_unchecked(key)
    }

    #[inline]
    #[track_caller]
    pub fn arena(&self) -> &Arena<StackNode<T>, ArenaTag> {
        &self.arena
    }
}
