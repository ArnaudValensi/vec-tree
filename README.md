# `vec-tree`

A safe tree using an arena allocator that allows deletion without suffering from
[the ABA problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational
indices.

It uses [generational-arena](https://github.com/fitzgen/generational-arena) under
the hood, made by [fitzgen](https://github.com/fitzgen), special thanks to him.

[generational-arena](https://github.com/fitzgen/generational-arena) is itself inspired
by [Catherine West's closing keynote at RustConf
2018](http://rustconf.com/program.html#closingkeynote), where these ideas
were presented in the context of an Entity-Component-System for games
programming.

### What? Why?

When you are working with a tree and you want to add and delete individual
nodes at a time, or you are writing a game and its world consists of many
inter-referencing objects with dynamic lifetimes that depend on user
input. These are situations where matching Rust's ownership and lifetime rules
can get tricky.

It doesn't make sense to use shared ownership with interior mutability (ie
`Rc<RefCell<T>>` or `Arc<Mutex<T>>`) nor borrowed references (ie `&'a T` or `&'a
mut T`) for structures. The cycles rule out reference counted types, and the
required shared mutability rules out borrows. Furthermore, lifetimes are dynamic
and don't follow the borrowed-data-outlives-the-borrower discipline.

In these situations, it is tempting to store objects in a `Vec<T>` and have them
reference each other via their indices. No more borrow checker or ownership
problems! Often, this solution is good enough.

However, now we can't delete individual items from that `Vec<T>` when we no
longer need them, because we end up either

* messing up the indices of every element that follows the deleted one, or

* suffering from the [ABA
  problem](https://en.wikipedia.org/wiki/ABA_problem). To elaborate further, if
  we tried to replace the `Vec<T>` with a `Vec<Option<T>>`, and delete an
  element by setting it to `None`, then we create the possibility for this buggy
  sequence:

    * `obj1` references `obj2` at index `i`

    * someone else deletes `obj2` from index `i`, setting that element to `None`

    * a third thing allocates `obj3`, which ends up at index `i`, because the
      element at that index is `None` and therefore available for allocation

    * `obj1` attempts to get `obj2` at index `i`, but incorrectly is given
      `obj3`, when instead the get should fail.

By introducing a monotonically increasing generation counter to the collection,
associating each element in the collection with the generation when it was
inserted, and getting elements from the collection with the *pair* of index and
the generation at the time when the element was inserted, then we can solve the
aforementioned ABA problem. When indexing into the collection, if the index
pair's generation does not match the generation of the element at that index,
then the operation fails.

### Features

* Zero `unsafe`
* There is different iterators to traverse the tree
* Well tested

### Usage

First, add `vec-tree` to your `Cargo.toml`:

```toml
[dependencies]
vec-tree = "0.1"
```

Then, import the crate and use the `vec-tree::Tree`

```rust
extern crate vec_tree;
use vec_tree::VecTree;

let mut tree = VecTree::new();

// Insert some elements into the tree.
let root_node = tree.insert_root(1);
let child_node_1 = tree.insert(10, root_node);
let child_node_2 = tree.insert(11, root_node);
let child_node_3 = tree.insert(12, root_node);
let grandchild = tree.insert(100, child_node_3);

// Inserted elements can be accessed infallibly via indexing (and missing
// entries will panic).
assert_eq!(tree[child_node_1], 10);

// Alternatively, the `get` and `get_mut` methods provide fallible lookup.
if let Some(node_value) = tree.get(child_node_2) {
    println!("The node value is: {}", node_value);
}
if let Some(node_value) = tree.get_mut(grandchild) {
    *node_value = 101;
}

// We can remove elements.
tree.remove(child_node_3);

// Insert a new one.
let child_node_4 = tree.insert(13, root_node);

// The tree does not contain `child_node_3` anymore, but it does contain
// `child_node_4`, even though they are almost certainly at the same index
// within the arena of the tree in practice. Ambiguities are resolved with
// an associated generation tag.
assert!(!tree.contains(child_node_3));
assert!(tree.contains(child_node_4));

// We can also move a node (and its descendants).
tree.append_child(child_node_1, child_node_4);

// Iterate over the children of a node.
for value in tree.children(child_node_1) {
    println!("value: {:?}", value);
}

// Or all the descendants in depth first search order.
let descendants = tree
    .descendants(root_node)
    .map(|node| tree[node])
    .collect::<Vec<i32>>();

assert_eq!(descendants, [1, 10, 13, 11]);
```
