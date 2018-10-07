extern crate generational_arena;
use generational_arena::{Arena, Index};

use core::ops;
use std::{fmt, mem};

#[derive(Debug)]
pub struct VecTree<T> {
    nodes: Arena<Node<T>>,
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

impl<T> VecTree<T> {
    pub fn new() -> VecTree<T> {
        VecTree::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(n: usize) -> VecTree<T> {
        VecTree {
            nodes: Arena::with_capacity(n),
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional_capacity: usize) {
        self.nodes.reserve(additional_capacity);
    }

    #[inline]
    pub fn try_insert(&mut self, data: T) -> Result<Index, T> {
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
    pub fn insert(&mut self, data: T) -> Index {
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

    #[inline]
    pub fn append_child(
        &mut self,
        node_id: Index,
        new_child_id: Index,
    ) -> Result<(), &'static str> {
        self.detach(new_child_id);

        let last_child_opt;
        {
            let (node_opt, new_child_node_opt) = self.nodes.get2_mut(node_id, new_child_id);

            if node_opt.is_none() {
                return Err("The node you are trying to append to is invalid");
            }

            if new_child_node_opt.is_none() {
                return Err("The node you are trying to append is invalid");
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

        Ok(())
    }

    // TODO: return error instead of panic
    #[inline]
    pub fn detach(&mut self, node_id: Index) {
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

    pub fn get(&self, node_id: Index) -> Option<&T> {
        match self.nodes.get(node_id) {
            Some(Node { ref data, .. }) => Some(data),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, node_id: Index) -> Option<&mut T> {
        match self.nodes.get_mut(node_id) {
            Some(Node { ref mut data, .. }) => Some(data),
            _ => None,
        }
    }

    pub fn capacity(&self) -> usize {
        println!("capacity");
        self.nodes.capacity()
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
