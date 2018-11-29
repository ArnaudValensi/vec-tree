/*!
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

## What? Why?

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

## Features

* Zero `unsafe`
* There is different iterators to traverse the tree
* Well tested

## Usage

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
 */

#![forbid(unsafe_code)]

extern crate generational_arena;
use generational_arena::Arena;
pub use generational_arena::Index;

use core::ops;
use std::{fmt, mem};

#[derive(Debug)]
pub struct VecTree<T> {
    nodes: Arena<Node<T>>,
    root_index: Option<Index>,
}

#[derive(Clone, Debug)]
struct Node<T> {
    parent: Option<Index>,
    previous_sibling: Option<Index>,
    next_sibling: Option<Index>,
    first_child: Option<Index>,
    last_child: Option<Index>,
    data: T,
}

const DEFAULT_CAPACITY: usize = 4;

impl<T> Default for VecTree<T> {
    fn default() -> Self {
        VecTree::with_capacity(DEFAULT_CAPACITY)
    }
}

impl<T> VecTree<T> {
    /// Constructs a new, empty `VecTree`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::<usize>::new();
    /// # let _ = tree;
    /// ```
    pub fn new() -> VecTree<T> {
        VecTree::with_capacity(DEFAULT_CAPACITY)
    }

    /// Constructs a new, empty `VecTree<T>` with the specified capacity.
    ///
    /// The `VecTree<T>` will be able to hold `n` elements without further allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::with_capacity(10);
    /// let root = tree.try_insert_root(0).unwrap();
    ///
    /// // These insertions will not require further allocation.
    /// for i in 1..10 {
    ///     assert!(tree.try_insert(i, root).is_ok());
    /// }
    ///
    /// // But now we are at capacity, and there is no more room.
    /// assert!(tree.try_insert(99, root).is_err());
    /// ```
    pub fn with_capacity(n: usize) -> VecTree<T> {
        VecTree {
            nodes: Arena::with_capacity(n),
            root_index: None,
        }
    }


    /// Allocate space for `additional_capacity` more elements in the tree.
    ///
    /// # Panics
    ///
    /// Panics if this causes the capacity to overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::with_capacity(10);
    /// tree.reserve(5);
    /// assert_eq!(tree.capacity(), 15);
    /// # let _: VecTree<usize> = tree;
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional_capacity: usize) {
        self.nodes.reserve(additional_capacity);
    }

    /// Attempts to insert `value` into the tree using existing capacity.
    ///
    /// This method will never allocate new capacity in the tree.
    ///
    /// If insertion succeeds, then the `value`'s index is returned. If
    /// insertion fails, then `Err(value)` is returned to give ownership of
    /// `value` back to the caller.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::new();
    /// let root = tree.insert_root(0);
    ///
    /// match tree.try_insert(42, root) {
    ///     Ok(idx) => {
    ///         // Insertion succeeded.
    ///         assert_eq!(tree[idx], 42);
    ///     }
    ///     Err(x) => {
    ///         // Insertion failed.
    ///         assert_eq!(x, 42);
    ///     }
    /// };
    /// ```
    #[inline]
    pub fn try_insert(&mut self, data: T, parent_id: Index) -> Result<Index, T> {
        let node_result = self.try_create_node(data);

        match node_result {
            Ok(node) => {
                self.append_child(parent_id, node);
                node_result
            }
            Err(err) => Err(err),
        }
    }

    /// Insert `value` into the tree, allocating more capacity if necessary.
    ///
    /// The `value`'s associated index in the tree is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::with_capacity(1);
    /// assert_eq!(tree.capacity(), 1);
    ///
    /// let root = tree.insert_root(0);
    ///
    /// let idx = tree.insert(42, root);
    /// assert_eq!(tree[idx], 42);
    /// assert_eq!(tree.capacity(), 2);
    /// ```
    #[inline]
    pub fn insert(&mut self, data: T, parent_id: Index) -> Index {
        let node = self.create_node(data);

        self.append_child(parent_id, node);

        node
    }

    #[inline]
    pub fn try_insert_root(&mut self, data: T) -> Result<Index, T> {
        if self.root_index.is_some() {
            panic!("A root node already exists");
        }

        match self.try_create_node(data) {
            Ok(node_id) => {
                self.root_index = Some(node_id);
                Ok(node_id)
            }
            Err(error) => Err(error),
        }
    }

    #[inline]
    pub fn insert_root(&mut self, data: T) -> Index {
        if self.root_index.is_some() {
            panic!("A root node already exists");
        }

        let node_id = self.create_node(data);
        self.root_index = Some(node_id);
        node_id
    }

    #[inline]
    fn try_create_node(&mut self, data: T) -> Result<Index, T> {
        let new_node = Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data,
        };

        match self.nodes.try_insert(new_node) {
            Ok(index) => Ok(index),
            Err(Node { data, .. }) => Err(data),
        }
    }

    #[inline]
    fn create_node(&mut self, data: T) -> Index {
        let new_node = Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data,
        };

        self.nodes.insert(new_node)
    }

    /// Remove the element at index `node_id` from the tree.
    ///
    /// If the element at index `node_id` is still in the tree, then it is
    /// returned. If it is not in the tree, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::new();
    /// let root = tree.insert_root(42);
    ///
    /// assert_eq!(tree.remove(root), Some(42));
    /// assert_eq!(tree.remove(root), None);
    /// ```
    pub fn remove(&mut self, node_id: Index) -> Option<T> {
        if !self.contains(node_id) {
            return None;
        }

        let descendants = self.descendants(node_id).skip(1).collect::<Vec<Index>>();
        let node = self.nodes.remove(node_id).unwrap();

        let previous_sibling_opt = node.previous_sibling;
        let next_sibling_opt = node.next_sibling;

        if let Some(previous_sibling_idx) = previous_sibling_opt {
            if let Some(next_sibling_idx) = next_sibling_opt {
                // If has previous and next.
                let (previous_sibling, next_sibling) =
                    self.nodes.get2_mut(previous_sibling_idx, next_sibling_idx);

                previous_sibling.unwrap().next_sibling = Some(next_sibling_idx);
                next_sibling.unwrap().previous_sibling = Some(previous_sibling_idx);
            } else if let Some(parent_idx) = node.parent {
                // If has previous but no next.
                let previous_sibling = &mut self.nodes[previous_sibling_idx];
                previous_sibling.next_sibling = None;

                let parent = &mut self.nodes[parent_idx];
                parent.last_child = Some(previous_sibling_idx);
            }
        } else if let Some(next_sibling_idx) = next_sibling_opt {
            // If has next but no previous.
            let next_sibling = &mut self.nodes[next_sibling_idx];
            next_sibling.previous_sibling = None;

            if let Some(parent_idx) = node.parent {
                let parent = &mut self.nodes[parent_idx];
                parent.first_child = Some(next_sibling_idx);
            }
        } else if let Some(parent_idx) = node.parent {
            // If it has no previous and no next.
            let parent = &mut self.nodes[parent_idx];
            parent.first_child = None;
            parent.last_child = None;
        }

        // Remove descendants from arena.
        for node_id in descendants {
            self.nodes.remove(node_id);
        }

        // Set root_index to None if needed
        if let Some(root_index) = self.root_index {
            if root_index == node_id {
                self.root_index = None;
            }
        }

        Some(node.data)
    }

    /// Is the element at index `node_id` in the tree?
    ///
    /// Returns `true` if the element at `node_id` is in the tree, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::new();
    /// let root = tree.insert_root(0);
    ///
    /// assert!(tree.contains(root));
    /// tree.remove(root);
    /// assert!(!tree.contains(root));
    /// ```
    pub fn contains(&self, node_id: Index) -> bool {
        self.nodes.get(node_id).is_some()
    }

    #[inline]
    pub fn append_child(&mut self, node_id: Index, new_child_id: Index) {
        self.detach(new_child_id);

        let last_child_opt;
        {
            let (node_opt, new_child_node_opt) = self.nodes.get2_mut(node_id, new_child_id);

            if node_opt.is_none() {
                panic!("The node you are trying to append to is invalid");
            }

            if new_child_node_opt.is_none() {
                panic!("The node you are trying to append is invalid");
            }

            let node = node_opt.unwrap();
            let new_child_node = new_child_node_opt.unwrap();

            new_child_node.parent = Some(node_id);

            last_child_opt = mem::replace(&mut node.last_child, Some(new_child_id));
            if let Some(last_child) = last_child_opt {
                new_child_node.previous_sibling = Some(last_child);
            } else {
                debug_assert!(node.first_child.is_none());
                node.first_child = Some(new_child_id);
            }
        }

        if let Some(last_child) = last_child_opt {
            debug_assert!(self.nodes[last_child].next_sibling.is_none());
            self.nodes[last_child].next_sibling = Some(new_child_id);
        }
    }

    #[inline]
    fn detach(&mut self, node_id: Index) {
        let (parent, previous_sibling, next_sibling) = {
            let node = &mut self.nodes[node_id];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
            )
        };

        if let Some(next_sibling) = next_sibling {
            self.nodes[next_sibling].previous_sibling = previous_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent].last_child = previous_sibling;
        }

        if let Some(previous_sibling) = previous_sibling {
            self.nodes[previous_sibling].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent].first_child = next_sibling;
        }
    }

    /// Get a shared reference to the element at index `node_id` if it is in the
    /// tree.
    ///
    /// If the element at index `node_id` is not in the tree, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::new();
    /// let root = tree.insert_root(42);
    ///
    /// assert_eq!(tree.get(root), Some(&42));
    /// tree.remove(root);
    /// assert!(tree.get(root).is_none());
    /// ```
    pub fn get(&self, node_id: Index) -> Option<&T> {
        match self.nodes.get(node_id) {
            Some(Node { ref data, .. }) => Some(data),
            _ => None,
        }
    }

    /// Get an exclusive reference to the element at index `node_id` if it is in the
    /// tree.
    ///
    /// If the element at index `node_id` is not in the tree, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::new();
    /// let root = tree.insert_root(42);
    ///
    /// *tree.get_mut(root).unwrap() += 1;
    /// assert_eq!(tree.remove(root), Some(43));
    /// assert!(tree.get_mut(root).is_none());
    /// ```
    pub fn get_mut(&mut self, node_id: Index) -> Option<&mut T> {
        match self.nodes.get_mut(node_id) {
            Some(Node { ref mut data, .. }) => Some(data),
            _ => None,
        }
    }

    pub fn get_root_index(&self) -> Option<Index> {
        self.root_index
    }

    /// Get the capacity of this tree.
    ///
    /// The capacity is the maximum number of elements the tree can hold
    /// without further allocation, including however many it currently
    /// contains.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::with_capacity(10);
    /// let root = tree.insert_root(0);
    ///
    /// // `try_insert` does not allocate new capacity.
    /// for i in 1..10 {
    ///     assert!(tree.try_insert(i, root).is_ok());
    ///     assert_eq!(tree.capacity(), 10);
    /// }
    ///
    /// // But `insert` will if the root is already at capacity.
    /// tree.insert(11, root);
    /// assert!(tree.capacity() > 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Clear all the items inside the tree, but keep its allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_tree::VecTree;
    ///
    /// let mut tree = VecTree::with_capacity(1);
    /// let root = tree.insert_root(42);
    /// tree.insert(43, root); // The capacity is doubled when reached.
    ///
    /// tree.clear();
    /// assert_eq!(tree.capacity(), 2);
    /// ```
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_index = None;
    }

    /// Return an iterator of references to this node’s parent.
    pub fn parent(&self, node_id: Index) -> Option<Index> {
        match self.nodes.get(node_id) {
            Some(node) => node.parent,
            _ => None,
        }
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(&self, node_id: Index) -> ChildrenIter<T> {
        ChildrenIter {
            tree: self,
            node_id: self.nodes[node_id].first_child,
        }
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(&self, node_id: Index) -> PrecedingSiblingsIter<T> {
        PrecedingSiblingsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&self, node_id: Index) -> FollowingSiblingsIter<T> {
        FollowingSiblingsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(&self, node_id: Index) -> AncestorsIter<T> {
        AncestorsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    fn traverse(&self, node_id: Index) -> TraverseIter<T> {
        TraverseIter {
            tree: self,
            root: node_id,
            next: Some(NodeEdge::Start(node_id)),
        }
    }

    /// Return an iterator of references to this node and its descendants, with deoth in the tree,
    /// in tree order.
    fn traverse_with_depth(&self, node_id: Index) -> TraverseWithDepthIter<T> {
        TraverseWithDepthIter {
            tree: self,
            root: node_id,
            next: Some(NodeEdgeWithDepth::Start(node_id, 0)),
        }
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn descendants(&self, node_id: Index) -> DescendantsIter<T> {
        DescendantsIter(self.traverse(node_id))
    }

    /// Return an iterator of references to this node and its descendants, with deoth in the tree,
    /// in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn descendants_with_depth(&self, node_id: Index) -> DescendantsWithDepthIter<T> {
        DescendantsWithDepthIter(self.traverse_with_depth(node_id))
    }
}

impl<T> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parent: {:?}, ", self.parent)?;
        write!(f, "Previous sibling: {:?}, ", self.previous_sibling)?;
        write!(f, "Next sibling: {:?}, ", self.next_sibling)?;
        write!(f, "First child: {:?}, ", self.first_child)?;
        write!(f, "Last child: {:?}", self.last_child)
    }
}

impl<T> ops::Index<Index> for VecTree<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> ops::IndexMut<Index> for VecTree<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

macro_rules! impl_node_iterator {
    ($name:ident, $next:expr) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = Index;

            fn next(&mut self) -> Option<Index> {
                match self.node_id.take() {
                    Some(node_id) => {
                        self.node_id = $next(&self.tree.nodes[node_id]);
                        Some(node_id)
                    }
                    None => None,
                }
            }
        }
    };
}

/// An iterator of references to the children of a given node.
pub struct ChildrenIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<Index>,
}
impl_node_iterator!(ChildrenIter, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the siblings before a given node.
pub struct PrecedingSiblingsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<Index>,
}
impl_node_iterator!(PrecedingSiblingsIter, |node: &Node<T>| node
    .previous_sibling);

/// An iterator of references to the siblings after a given node.
pub struct FollowingSiblingsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<Index>,
}
impl_node_iterator!(FollowingSiblingsIter, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the ancestors a given node.
pub struct AncestorsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<Index>,
}
impl_node_iterator!(AncestorsIter, |node: &Node<T>| node.parent);

#[derive(Debug, Clone)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children. Yielded by `TraverseIter::next` before the
    /// node’s descendants.
    Start(T),

    /// Indicates that end of a node that has children. Yielded by `TraverseIter::next` after the
    /// node’s descendants.
    End(T),
}

/// An iterator of references to a given node and its descendants, in depth-first search pre-order
/// NLR traversal.
/// https://en.wikipedia.org/wiki/Tree_traversal#Pre-order_(NLR)
pub struct TraverseIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    root: Index,
    next: Option<NodeEdge<Index>>,
}

impl<'a, T> Iterator for TraverseIter<'a, T> {
    type Item = NodeEdge<Index>;

    fn next(&mut self) -> Option<NodeEdge<Index>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::Start(node_id) => match self.tree.nodes[node_id].first_child {
                        Some(first_child) => Some(NodeEdge::Start(first_child)),
                        None => Some(NodeEdge::End(node_id)),
                    },
                    NodeEdge::End(node_id) => {
                        if node_id == self.root {
                            None
                        } else {
                            match self.tree.nodes[node_id].next_sibling {
                                Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                                None => {
                                    match self.tree.nodes[node_id].parent {
                                        Some(parent) => Some(NodeEdge::End(parent)),

                                        // `self.tree.nodes[node_id].parent` here can only be `None`
                                        // if the tree has been modified during iteration, but
                                        // silently stoping iteration seems a more sensible behavior
                                        // than panicking.
                                        None => None,
                                    }
                                }
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

/// An iterator of references to a given node and its descendants, in tree order.
pub struct DescendantsIter<'a, T: 'a>(pub TraverseIter<'a, T>);

impl<'a, T> Iterator for DescendantsIter<'a, T> {
    type Item = Index;

    fn next(&mut self) -> Option<Index> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node_id)) => return Some(node_id),
                Some(NodeEdge::End(_)) => {}
                None => return None,
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdgeWithDepth<T> {
    /// Indicates that start of a node that has children. Yielded by `TraverseIter::next` before the
    /// node’s descendants.
    Start(T, u32),

    /// Indicates that end of a node that has children. Yielded by `TraverseIter::next` after the
    /// node’s descendants.
    End(T, u32),
}

/// An iterator of references to a given node and its descendants, with depth, in depth-first
/// search pre-order NLR traversal.
/// https://en.wikipedia.org/wiki/Tree_traversal#Pre-order_(NLR)
pub struct TraverseWithDepthIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    root: Index,
    next: Option<NodeEdgeWithDepth<Index>>,
}

impl<'a, T> Iterator for TraverseWithDepthIter<'a, T> {
    type Item = NodeEdgeWithDepth<Index>;

    fn next(&mut self) -> Option<NodeEdgeWithDepth<Index>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdgeWithDepth::Start(node_id, depth) => {
                        match self.tree.nodes[node_id].first_child {
                            Some(first_child) => {
                                Some(NodeEdgeWithDepth::Start(first_child, depth + 1))
                            }
                            None => Some(NodeEdgeWithDepth::End(node_id, depth)),
                        }
                    }
                    NodeEdgeWithDepth::End(node_id, depth) => {
                        if node_id == self.root {
                            None
                        } else {
                            match self.tree.nodes[node_id].next_sibling {
                                Some(next_sibling) => {
                                    Some(NodeEdgeWithDepth::Start(next_sibling, depth))
                                }
                                None => {
                                    match self.tree.nodes[node_id].parent {
                                        Some(parent) => {
                                            Some(NodeEdgeWithDepth::End(parent, depth - 1))
                                        }

                                        // `self.tree.nodes[node_id].parent` here can only be `None`
                                        // if the tree has been modified during iteration, but
                                        // silently stoping iteration seems a more sensible behavior
                                        // than panicking.
                                        None => None,
                                    }
                                }
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

/// An iterator of references to a given node and its descendants, with depth, in tree order.
pub struct DescendantsWithDepthIter<'a, T: 'a>(pub TraverseWithDepthIter<'a, T>);

impl<'a, T> Iterator for DescendantsWithDepthIter<'a, T> {
    type Item = (Index, u32);

    fn next(&mut self) -> Option<(Index, u32)> {
        loop {
            match self.0.next() {
                Some(NodeEdgeWithDepth::Start(node_id, depth)) => return Some((node_id, depth)),
                Some(NodeEdgeWithDepth::End(_, _)) => {}
                None => return None,
            }
        }
    }
}
