use core::ops;
use std::{cmp, fmt, mem};

#[derive(Debug)]
pub struct VecTree<T> {
    nodes: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct Node<T> {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,

    pub data: T,
}

#[derive(Debug, Copy, Clone)]
pub struct NodeId {
    index: usize,
}

impl<T> VecTree<T> {
    pub fn new() -> VecTree<T> {
        VecTree { nodes: Vec::new() }
    }

    #[inline]
    pub fn insert(&mut self, data: T) -> NodeId {
        let index = self.nodes.len();

        self.nodes.push(Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data,
        });

        NodeId { index }
    }

    pub fn append_child(&mut self, node_id: NodeId, new_child_id: NodeId) {
        self.detach(new_child_id);

        let last_child_opt;
        {
            let (node, new_child_node) = self.nodes.get_pair_mut(
                node_id.index,
                new_child_id.index,
                "Can not append a node to itself",
            );

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
            debug_assert!(self.nodes[last_child.index].next_sibling.is_none());
            self.nodes[last_child.index].next_sibling = Some(new_child_id);
        }
    }

    pub fn detach(&mut self, node_id: NodeId) {
        let (parent, previous_sibling, next_sibling) = {
            let node = &mut self.nodes[node_id.index];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
            )
        };

        if let Some(next_sibling) = next_sibling {
            self.nodes[next_sibling.index].previous_sibling = previous_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent.index].last_child = previous_sibling;
        }

        if let Some(previous_sibling) = previous_sibling {
            self.nodes[previous_sibling.index].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent.index].first_child = next_sibling;
        }
    }

    pub fn get(&self, node_id: NodeId) -> Option<&T> {
        match self.nodes.get(node_id.index) {
            Some(Node { ref data, .. }) => Some(data),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Option<&mut T> {
        match self.nodes.get_mut(node_id.index) {
            Some(Node { ref mut data, .. }) => Some(data),
            _ => None,
        }
    }

    pub fn capacity(&self) -> usize {
        self.nodes.len()
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(&self, node_id: NodeId) -> ChildrenIter<T> {
        ChildrenIter {
            tree: self,
            node_id: self.nodes[node_id.index].first_child,
        }
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(&self, node_id: NodeId) -> PrecedingSiblingsIter<T> {
        PrecedingSiblingsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&self, node_id: NodeId) -> FollowingSiblingsIter<T> {
        FollowingSiblingsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(&self, node_id: NodeId) -> AncestorsIter<T> {
        AncestorsIter {
            tree: self,
            node_id: Some(node_id),
        }
    }
}

impl<T> Node<T> {
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.previous_sibling
    }

    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
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

trait GetPairMut<T> {
    /// Get mutable references to two distinct nodes. Panics if the two given IDs are the same.
    fn get_pair_mut(
        &mut self,
        a: usize,
        b: usize,
        same_index_error_message: &'static str,
    ) -> (&mut T, &mut T);
}

impl<T> GetPairMut<T> for Vec<T> {
    fn get_pair_mut(
        &mut self,
        a: usize,
        b: usize,
        same_index_error_message: &'static str,
    ) -> (&mut T, &mut T) {
        if a == b {
            panic!(same_index_error_message)
        }
        let (xs, ys) = self.split_at_mut(cmp::max(a, b));
        if a < b {
            (&mut xs[a], &mut ys[0])
        } else {
            (&mut ys[0], &mut xs[b])
        }
    }
}

impl<T> ops::Index<NodeId> for VecTree<T> {
    type Output = T;

    fn index(&self, index: NodeId) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> ops::IndexMut<NodeId> for VecTree<T> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

macro_rules! impl_node_iterator {
    ($name:ident, $next:expr) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<NodeId> {
                match self.node_id.take() {
                    Some(node_id) => {
                        self.node_id = $next(&self.tree.nodes[node_id.index]);
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
    node_id: Option<NodeId>,
}
impl_node_iterator!(ChildrenIter, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the siblings before a given node.
pub struct PrecedingSiblingsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<NodeId>,
}
impl_node_iterator!(PrecedingSiblingsIter, |node: &Node<T>| node
    .previous_sibling);

/// An iterator of references to the siblings after a given node.
pub struct FollowingSiblingsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<NodeId>,
}
impl_node_iterator!(FollowingSiblingsIter, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the ancestors a given node.
pub struct AncestorsIter<'a, T: 'a> {
    tree: &'a VecTree<T>,
    node_id: Option<NodeId>,
}
impl_node_iterator!(AncestorsIter, |node: &Node<T>| node.parent);
