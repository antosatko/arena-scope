use arena::{Arena, Key};

pub type StackKey = Key<StackTag>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackTag;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Snapshot(pub StackKey);

#[derive(Debug)]
struct StackNode<T> {
    prev: Option<StackKey>,
    value: T,
}

#[derive(Debug)]
pub struct SnapshotStack<T> {
    arena: Arena<StackNode<T>, StackTag>,
    top: Option<StackKey>,
}

impl<T> Default for SnapshotStack<T> {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
            top: None,
        }
    }
}

impl<T> SnapshotStack<T> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.top.is_none()
    }

    #[inline]
    pub fn push(&mut self, value: T) -> StackKey {
        let key = self.arena.push(StackNode {
            prev: self.top,
            value,
        });

        self.top = Some(key);

        key
    }

    #[inline]
    pub fn pop(&mut self) -> T
    where
        T: Copy,
    {
        let top = unsafe { self.top.unwrap_unchecked() };

        let node = self.arena.get_unchecked(&top);

        self.top = node.prev;

        node.value
    }

    #[inline]
    pub fn current(&self) -> &T {
        let top = unsafe { self.top.unwrap_unchecked() };

        &self.arena.get_unchecked(&top).value
    }

    #[inline]
    pub fn current_mut(&mut self) -> &mut T {
        let top = unsafe { self.top.unwrap_unchecked() };

        &mut self.arena.get_mut_unchecked(&top).value
    }

    #[inline]
    pub fn snapshot(&self) -> Option<Snapshot> {
        self.top.map(Snapshot)
    }

    #[inline]
    pub fn restore(&mut self, snapshot: Snapshot) {
        self.top = Some(snapshot.0);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.top = None;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            arena: &self.arena,
            current: self.top,
        }
    }
}

pub struct Iter<'a, T> {
    arena: &'a Arena<StackNode<T>, StackTag>,
    current: Option<StackKey>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.current?;

        let node = self.arena.get_unchecked(&key);

        self.current = node.prev;

        Some(&node.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop() {
        let mut stack = SnapshotStack::default();

        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(*stack.current(), 3);

        assert_eq!(stack.pop(), 3);
        assert_eq!(*stack.current(), 2);

        assert_eq!(stack.pop(), 2);
        assert_eq!(*stack.current(), 1);

        assert_eq!(stack.pop(), 1);

        assert!(stack.is_empty());
    }

    #[test]
    fn snapshot_restore() {
        let mut stack = SnapshotStack::default();

        stack.push(1);
        stack.push(2);

        let snap = stack.snapshot().unwrap();

        stack.push(3);
        stack.push(4);

        assert_eq!(*stack.current(), 4);

        stack.restore(snap);

        assert_eq!(*stack.current(), 2);
    }

    #[test]
    fn nested_snapshots() {
        let mut stack = SnapshotStack::default();

        stack.push(1);

        let s1 = stack.snapshot().unwrap();

        stack.push(2);

        let s2 = stack.snapshot().unwrap();

        stack.push(3);

        assert_eq!(*stack.current(), 3);

        stack.restore(s2);
        assert_eq!(*stack.current(), 2);

        stack.restore(s1);
        assert_eq!(*stack.current(), 1);
    }

    #[test]
    fn snapshot_does_not_destroy_future_nodes() {
        let mut stack = SnapshotStack::default();

        stack.push(10);

        let s1 = stack.snapshot().unwrap();

        stack.push(20);

        let s2 = stack.snapshot().unwrap();

        stack.push(30);

        stack.restore(s1);
        assert_eq!(*stack.current(), 10);

        stack.restore(s2);
        assert_eq!(*stack.current(), 20);
    }

    #[test]
    fn iteration_order() {
        let mut stack = SnapshotStack::default();

        stack.push(1);
        stack.push(2);
        stack.push(3);

        let values: Vec<_> = stack.iter().copied().collect();

        assert_eq!(values, vec![3, 2, 1]);
    }
}
