extern crate vec_tree;
use vec_tree::VecTree;

#[test]
fn try_insert_root() {
    let mut tree = VecTree::with_capacity(1);
    let root = tree.try_insert_root(42).unwrap();
    assert_eq!(tree[root], 42);
}

#[test]
fn insert_root() {
    let mut tree = VecTree::with_capacity(1);
    let root = tree.insert_root(42);
    assert_eq!(tree[root], 42);
}

#[test]
fn try_insert() {
    let mut tree = VecTree::with_capacity(3);
    let root = tree.try_insert_root(0).unwrap();
    let child_1 = tree.try_insert(1, root).unwrap();
    let child_2 = tree.try_insert(2, root).unwrap();
    assert_eq!(tree[root], 0);
    assert_eq!(tree[child_1], 1);
    assert_eq!(tree[child_2], 2);
}

#[test]
#[should_panic]
fn try_insert_root_twice() {
    let mut tree = VecTree::with_capacity(2);
    let _root = tree.try_insert_root(42).unwrap();
    let _root2 = tree.try_insert_root(43).unwrap();
}

#[test]
#[should_panic]
fn insert_root_twice() {
    let mut tree = VecTree::with_capacity(2);
    let _root = tree.insert_root(42);
    let _root2 = tree.insert_root(43);
}

#[test]
fn remove_a_root_node() {
    let mut tree = VecTree::with_capacity(1);
    let root_node1 = tree.try_insert_root(42).unwrap();
    tree.remove(root_node1);
    let root_node2 = tree.try_insert_root(43).unwrap();
    assert_eq!(tree[root_node2], 43);
}

#[test]
fn cannot_get_free_value() {
    let mut tree = VecTree::with_capacity(1);
    let i = tree.try_insert_root(42).unwrap();
    assert_eq!(tree.remove(i).unwrap(), 42);
    assert!(!tree.contains(i));
}

#[test]
fn cannot_get_other_generation_value() {
    let mut tree = VecTree::with_capacity(2);
    let root_node = tree.try_insert_root(42).unwrap();
    let i = tree.try_insert(42, root_node).unwrap();
    assert_eq!(tree.remove(i).unwrap(), 42);
    assert!(!tree.contains(i));
    let j = tree.try_insert(42, root_node).unwrap();
    assert!(!tree.contains(i));
    assert_eq!(tree[j], 42);
    assert!(i != j);
}

#[test]
fn try_insert_when_full() {
    let mut tree = VecTree::with_capacity(2);
    let root_node = tree.try_insert_root(42).unwrap();
    let _child = tree.try_insert(42, root_node).unwrap();
    assert_eq!(tree.try_insert(42, root_node).unwrap_err(), 42);
}

#[test]
fn insert_many_and_cause_doubling() {
    let mut tree = VecTree::new();

    let root = tree.try_insert_root(0).unwrap();

    let indices: Vec<_> = (0..1000).map(|i| tree.insert(i * i, root)).collect();
    for (i, idx) in indices.iter().cloned().enumerate() {
        assert_eq!(tree.remove(idx).unwrap(), i * i);
        assert!(!tree.contains(idx));
    }
}

#[test]
fn capacity_and_reserve() {
    let mut tree: VecTree<usize> = VecTree::with_capacity(42);
    assert_eq!(tree.capacity(), 42);
    tree.reserve(10);
    assert_eq!(tree.capacity(), 52);
}

#[test]
fn get_mut() {
    let mut tree = VecTree::new();
    let idx = tree.insert_root(5);
    tree[idx] += 1;
    assert_eq!(tree[idx], 6);
}

#[test]
#[should_panic]
fn index_deleted_item() {
    let mut tree = VecTree::new();
    let idx = tree.insert_root(42);
    tree.remove(idx);
    tree[idx];
}

#[test]
fn check_the_validity_of_the_tree_after_remove() {
    let mut tree: VecTree<usize> = VecTree::with_capacity(4);
    let root = tree.try_insert_root(0).unwrap();
    let child1 = tree.try_insert(1, root).unwrap();
    let child2 = tree.try_insert(2, root).unwrap();
    let child3 = tree.try_insert(3, root).unwrap();

    tree.remove(child3);
    tree.try_insert(4, root).unwrap();

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1, 2, 4]
    );

    tree.remove(child2);
    tree.try_insert(5, root).unwrap();

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1, 4, 5]
    );

    tree.remove(child1);
    tree.try_insert(6, root).unwrap();

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [4, 5, 6]
    );
}

#[test]
fn check_remove_with_one_child() {
    let mut tree: VecTree<usize> = VecTree::with_capacity(2);
    let root = tree.try_insert_root(0).unwrap();

    let child1 = tree.try_insert(1, root).unwrap();
    tree.remove(child1);

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    let child2 = tree.try_insert(2, root).unwrap();

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2]
    );

    tree.remove(child2);

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );
}

#[test]
fn out_of_bounds_get_with_index_from_other_tree() {
    let mut tree1 = VecTree::with_capacity(1);
    let mut tree2 = VecTree::with_capacity(1);
    let root_tree1 = tree1.insert_root(1);
    let _root_tree2 = tree2.insert_root(2);
    let child_tree1 = tree1.insert(2, root_tree1);
    assert!(tree2.get(child_tree1).is_none());
}

#[test]
fn out_of_bounds_remove_with_index_from_other_tree() {
    let mut tree1 = VecTree::with_capacity(1);
    let mut tree2 = VecTree::with_capacity(1);
    let root_tree1 = tree1.insert_root(1);
    let _root_tree2 = tree2.insert_root(2);
    let child_tree1 = tree1.insert(2, root_tree1);
    assert!(tree2.remove(child_tree1).is_none());
}

#[test]
fn add_children_and_iterate_over_it() {
    let mut tree = VecTree::new();

    let root_node = tree.insert_root(1);
    let child_node_1 = tree.insert(2, root_node);
    let child_node_2 = tree.insert(3, root_node);
    let child_node_3 = tree.insert(4, root_node);
    let _grandchild = tree.insert(5, child_node_3);

    assert_eq!(
        tree.children(root_node)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );

    assert_eq!(
        tree.children(child_node_1)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    assert_eq!(
        tree.children(child_node_2)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    assert_eq!(
        tree.children(child_node_3)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [5]
    );
}

#[test]
fn iterate_over_preceding_siblings() {
    let mut tree = VecTree::new();

    let root_node = tree.insert_root(1);
    let child_node_1 = tree.insert(2, root_node);
    let child_node_2 = tree.insert(3, root_node);
    let child_node_3 = tree.insert(4, root_node);

    assert_eq!(
        tree.preceding_siblings(root_node)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1]
    );

    assert_eq!(
        tree.preceding_siblings(child_node_1)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2]
    );

    assert_eq!(
        tree.preceding_siblings(child_node_2)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [3, 2]
    );

    assert_eq!(
        tree.preceding_siblings(child_node_3)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [4, 3, 2]
    );
}

#[test]
fn iterate_over_following_siblings() {
    let mut tree = VecTree::new();

    let root_node = tree.insert_root(1);
    let child_node_1 = tree.insert(2, root_node);
    let child_node_2 = tree.insert(3, root_node);
    let child_node_3 = tree.insert(4, root_node);

    assert_eq!(
        tree.following_siblings(root_node)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1]
    );

    assert_eq!(
        tree.following_siblings(child_node_1)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );

    assert_eq!(
        tree.following_siblings(child_node_2)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [3, 4]
    );

    assert_eq!(
        tree.following_siblings(child_node_3)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [4]
    );
}

#[test]
fn iterate_over_ancestors() {
    let mut tree = VecTree::new();

    let root_node = tree.insert_root(1);
    let child_node_1 = tree.insert(2, root_node);
    let child_node_2 = tree.insert(3, root_node);
    let grandchild = tree.insert(5, child_node_2);

    assert_eq!(
        tree.ancestors(root_node)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1]
    );

    assert_eq!(
        tree.ancestors(child_node_1)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2, 1]
    );

    assert_eq!(
        tree.ancestors(child_node_2)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [3, 1]
    );

    assert_eq!(
        tree.ancestors(grandchild)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [5, 3, 1]
    );
}

#[test]
fn iterate_over_descendants() {
    let mut tree = VecTree::new();

    // 0-1-4-6
    // | `-5
    // `-2
    // `-3
    let root_node = tree.insert_root(0);
    let node_1 = tree.insert(1, root_node);
    let node_2 = tree.insert(2, root_node);
    let _node_3 = tree.insert(3, root_node);
    let node_4 = tree.insert(4, node_1);
    let _node_5 = tree.insert(5, node_1);
    let _node_6 = tree.insert(6, node_4);
    let _node_7 = tree.insert(7, node_2);

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    let expected_result = [0, 1, 4, 6, 5, 2, 7, 3];

    assert_eq!(descendants, expected_result);
}

#[test]
fn iterate_over_descendants_with_depth() {
    let mut tree = VecTree::new();

    // 0-1-4-6
    // | `-5
    // `-2
    // `-3
    let root_node = tree.insert_root(0);
    let node_1 = tree.insert(1, root_node);
    let node_2 = tree.insert(2, root_node);
    let _node_3 = tree.insert(3, root_node);
    let node_4 = tree.insert(4, node_1);
    let _node_5 = tree.insert(5, node_1);
    let _node_6 = tree.insert(6, node_4);
    let _node_7 = tree.insert(7, node_2);

    let descendants = tree
        .descendants_with_depth(root_node)
        .map(|(node, depth)| (tree[node], depth))
        .collect::<Vec<(i32, u32)>>();

    let expected_result = [
        (0, 0),
        (1, 1),
        (4, 2),
        (6, 3),
        (5, 2),
        (2, 1),
        (7, 2),
        (3, 1),
    ];

    assert_eq!(descendants, expected_result);
}

#[test]
// It would panic when adding node_5 if the nodes where not recursively removed.
fn check_descendants_are_removed() {
    let mut tree = VecTree::with_capacity(5);

    // 0-1-3-4
    //   `-2
    let root_node = tree.try_insert_root(0).unwrap();
    let node_1 = tree.try_insert(1, root_node).unwrap();
    let _node_2 = tree.try_insert(2, node_1).unwrap();
    let node_3 = tree.try_insert(3, node_1).unwrap();
    let _node_4 = tree.try_insert(4, node_3).unwrap();

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 1, 2, 3, 4]);

    // 0
    tree.remove(node_1);

    // 0-5-7-8
    //   `-6
    let node_5 = tree.try_insert(5, root_node).unwrap();
    let _node_6 = tree.try_insert(6, node_5).unwrap();
    let node_7 = tree.try_insert(7, node_5).unwrap();
    let _node_8 = tree.try_insert(8, node_7).unwrap();

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 5, 6, 7, 8]);
}

#[test]
fn move_a_node() {
    let mut tree = VecTree::with_capacity(3);
    let root_node = tree.try_insert_root(0).unwrap();
    let node_1 = tree.try_insert(1, root_node).unwrap();
    let _node_2 = tree.try_insert(2, root_node).unwrap();

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 1, 2]);

    tree.append_child(root_node, node_1);

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 2, 1]);
}
