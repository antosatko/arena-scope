# arena-scope

Arena-backed persistent scope tree with O(1) snapshots.

Useful for:

* compilers
* interpreters
* parsers
* transactional state
* backtracking systems

Snapshots are just arena indices, so cloning and restoring state is cheap.

## Example

```rust
use arena_scope::ScopeTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeTag;

type Env = ScopeTree<String, i32, ScopeTag>;

fn main() {
    let mut env = Env::default();

    env.init();

    env.insert("x".into(), 10);

    let snapshot = env.snapshot();

    env.push();

    env.insert("y".into(), 20);

    assert_eq!(env.get(&"x".to_string()), Some(&10));
    assert_eq!(env.get(&"y".to_string()), Some(&20));

    env.restore(snapshot.unwrap());

    assert_eq!(env.get(&"x".to_string()), Some(&10));
    assert_eq!(env.get(&"y".to_string()), None);
}
```

## Complexity

| Operation | Cost |
| --------- | ---- |
| push      | O(1) |
| pop       | O(1) |
| snapshot  | O(1) |
| restore   | O(1) |

## Notes

The arena is append-only. Restoring a snapshot does not free memory.

---

*This README was AI-generated and human reviewed in about two minutes.*
