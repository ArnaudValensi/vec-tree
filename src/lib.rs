use std::{cmp, fmt, mem};

#[derive(Debug)]
pub struct VecTree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> VecTree<T> {
    pub fn new() -> VecTree<T> {
        VecTree { nodes: Vec::new() }
    }

    pub fn new_node(&mut self, data: T) -> NodeId {
        let index = self.nodes.len();

        self.nodes.push(Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data: Some(data),
        });

        NodeId { index }
    }

    pub fn append_child(&mut self, node_id: NodeId, new_child_id: NodeId) {
        self.detach(new_child_id);

        let last_child_opt;
        {
            let (self_borrow, new_child_borrow) = self.nodes.get_pair_mut(
                node_id.index,
                new_child_id.index,
                "Can not append a node to itself",
            );

            new_child_borrow.parent = Some(node_id);

            last_child_opt = mem::replace(&mut self_borrow.last_child, Some(new_child_id));
            if let Some(last_child) = last_child_opt {
                new_child_borrow.previous_sibling = Some(last_child);
            } else {
                debug_assert!(self_borrow.first_child.is_none());
                self_borrow.first_child = Some(new_child_id);
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

    pub fn borrow_data(&self, node_id: NodeId) -> &T {
        let node = &self.nodes[node_id.index];

        node.data.as_ref().unwrap()
    }

    pub fn borrow_data_mut(&mut self, node_id: NodeId) -> &mut T {
        let node = &mut self.nodes[node_id.index];

        node.data.as_mut().unwrap()
    }

    pub fn take_data(&mut self, node_id: NodeId) -> T {
        let node = &mut self.nodes[node_id.index];

        node.data.take().unwrap()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NodeId {
    index: usize,
}

impl NodeId {}

#[derive(Debug)]
pub struct Node<T> {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,

    pub data: Option<T>,
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

#[cfg(test)]
mod tests {
    use super::VecTree;

    #[test]
    fn create_tree() {
        let mut tree = VecTree::new();

        let root_node_id = tree.new_node(1);
        let child_node_id_1 = tree.new_node(2);
        let child_node_id_2 = tree.new_node(3);

        tree.append_child(root_node_id, child_node_id_1);
        tree.append_child(root_node_id, child_node_id_2);

        assert!(tree.nodes.len() == 3, "it should have 3 nodes in the tree");
        assert!(
            *tree.borrow_data(root_node_id) == 1,
            "it should have 1 as data"
        );

        assert!(
            tree.take_data(child_node_id_1) == 2,
            "it should have 1 as data"
        );

        let child_node_id_2_mut_borrow = tree.borrow_data_mut(child_node_id_2);
        *child_node_id_2_mut_borrow = 4;

        assert!(*child_node_id_2_mut_borrow == 4, "it should have 1 as data");
    }
}
