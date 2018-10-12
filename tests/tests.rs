extern crate vec_tree;
use vec_tree::VecTree;

#[test]
fn can_get_live_value() {
    let mut tree = VecTree::with_capacity(1);
    let i = tree.try_insert(42).unwrap();
    assert_eq!(tree[i], 42);
}

#[test]
fn cannot_get_free_value() {
    let mut tree = VecTree::with_capacity(1);
    let i = tree.try_insert(42).unwrap();
    assert_eq!(tree.remove(i).unwrap(), 42);
    assert!(!tree.contains(i));
}

#[test]
fn cannot_get_other_generation_value() {
    let mut tree = VecTree::with_capacity(1);
    let i = tree.try_insert(42).unwrap();
    assert_eq!(tree.remove(i).unwrap(), 42);
    assert!(!tree.contains(i));
    let j = tree.try_insert(42).unwrap();
    assert!(!tree.contains(i));
    assert_eq!(tree[j], 42);
    assert!(i != j);
}

#[test]
fn try_insert_when_full() {
    let mut tree = VecTree::with_capacity(1);
    tree.try_insert(42).unwrap();
    assert_eq!(tree.try_insert(42).unwrap_err(), 42);
}

#[test]
fn insert_many_and_cause_doubling() {
    let mut tree = VecTree::new();
    let indices: Vec<_> = (0..1000).map(|i| tree.insert(i * i)).collect();
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
    let idx = tree.insert(5);
    tree[idx] += 1;
    assert_eq!(tree[idx], 6);
}

#[test]
#[should_panic]
fn index_deleted_item() {
    let mut tree = VecTree::new();
    let idx = tree.insert(42);
    tree.remove(idx);
    tree[idx];
}

#[test]
fn check_the_validity_of_the_tree_after_remove() {
    let mut tree: VecTree<usize> = VecTree::with_capacity(4);
    let root = tree.try_insert(0).unwrap();
    let child1 = tree.try_insert(1).unwrap();
    let child2 = tree.try_insert(2).unwrap();
    let child3 = tree.try_insert(3).unwrap();

    tree.append_child(root, child1).expect("valid");
    tree.append_child(root, child2).expect("valid");
    tree.append_child(root, child3).expect("valid");

    tree.remove(child3);
    let child4 = tree.try_insert(4).unwrap();
    tree.append_child(root, child4).expect("valid");

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1, 2, 4]
    );

    tree.remove(child2);
    let child5 = tree.try_insert(5).unwrap();
    tree.append_child(root, child5).expect("valid");

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1, 4, 5]
    );

    tree.remove(child1);
    let child6 = tree.try_insert(6).unwrap();
    tree.append_child(root, child6).expect("valid");

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
    let root = tree.try_insert(0).unwrap();

    let child1 = tree.try_insert(1).unwrap();
    tree.append_child(root, child1).expect("valid");
    tree.remove(child1);

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    let child2 = tree.try_insert(2).unwrap();
    tree.append_child(root, child2).expect("valid");

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
fn check_children_removed_on_remove() {}

#[test]
fn out_of_bounds_get_with_index_from_other_tree() {
    let mut tree1 = VecTree::with_capacity(1);
    let tree2 = VecTree::<usize>::with_capacity(1);
    tree1.insert(0);
    let idx = tree1.insert(42);
    assert!(tree2.get(idx).is_none());
}

#[test]
fn out_of_bounds_remove_with_index_from_other_tree() {
    let mut tree1 = VecTree::with_capacity(1);
    let mut tree2 = VecTree::<usize>::with_capacity(1);
    tree1.insert(0);
    let idx = tree1.insert(42);
    assert!(tree2.remove(idx).is_none());
}

#[test]
fn add_children_and_iterate_over_it() {
    let mut tree = VecTree::new();

    let root_node = tree.insert(1);
    let child_node_1 = tree.insert(2);
    let child_node_2 = tree.insert(3);
    let child_node_3 = tree.insert(4);
    let grandchild = tree.insert(5);

    tree.append_child(root_node, child_node_1).expect("valid");
    tree.append_child(root_node, child_node_2).expect("valid");
    tree.append_child(root_node, child_node_3).expect("valid");
    tree.append_child(child_node_3, grandchild).expect("valid");

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

    let root_node = tree.insert(1);
    let child_node_1 = tree.insert(2);
    let child_node_2 = tree.insert(3);
    let child_node_3 = tree.insert(4);

    tree.append_child(root_node, child_node_1).expect("valid");
    tree.append_child(root_node, child_node_2).expect("valid");
    tree.append_child(root_node, child_node_3).expect("valid");

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

    let root_node = tree.insert(1);
    let child_node_1 = tree.insert(2);
    let child_node_2 = tree.insert(3);
    let child_node_3 = tree.insert(4);

    tree.append_child(root_node, child_node_1).expect("valid");
    tree.append_child(root_node, child_node_2).expect("valid");
    tree.append_child(root_node, child_node_3).expect("valid");

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

    let root_node = tree.insert(1);
    let child_node_1 = tree.insert(2);
    let child_node_2 = tree.insert(3);
    let grandchild = tree.insert(5);

    tree.append_child(root_node, child_node_1).expect("valid");
    tree.append_child(root_node, child_node_2).expect("valid");
    tree.append_child(child_node_2, grandchild).expect("valid");

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
    let root_node = tree.insert(0);
    let node_1 = tree.insert(1);
    let node_2 = tree.insert(2);
    let node_3 = tree.insert(3);
    let node_4 = tree.insert(4);
    let node_5 = tree.insert(5);
    let node_6 = tree.insert(6);
    let node_7 = tree.insert(7);

    tree.append_child(root_node, node_1).expect("valid");
    tree.append_child(root_node, node_2).expect("valid");
    tree.append_child(root_node, node_3).expect("valid");
    tree.append_child(node_1, node_4).expect("valid");
    tree.append_child(node_1, node_5).expect("valid");
    tree.append_child(node_4, node_6).expect("valid");
    tree.append_child(node_2, node_7).expect("valid");

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
    let root_node = tree.insert(0);
    let node_1 = tree.insert(1);
    let node_2 = tree.insert(2);
    let node_3 = tree.insert(3);
    let node_4 = tree.insert(4);
    let node_5 = tree.insert(5);
    let node_6 = tree.insert(6);
    let node_7 = tree.insert(7);

    tree.append_child(root_node, node_1).expect("valid");
    tree.append_child(root_node, node_2).expect("valid");
    tree.append_child(root_node, node_3).expect("valid");
    tree.append_child(node_1, node_4).expect("valid");
    tree.append_child(node_1, node_5).expect("valid");
    tree.append_child(node_4, node_6).expect("valid");
    tree.append_child(node_2, node_7).expect("valid");

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
    let root_node = tree.try_insert(0).unwrap();
    let node_1 = tree.try_insert(1).unwrap();
    let node_2 = tree.try_insert(2).unwrap();
    let node_3 = tree.try_insert(3).unwrap();
    let node_4 = tree.try_insert(4).unwrap();

    tree.append_child(root_node, node_1).expect("valid");
    tree.append_child(node_1, node_2).expect("valid");
    tree.append_child(node_1, node_3).expect("valid");
    tree.append_child(node_3, node_4).expect("valid");

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 1, 2, 3, 4]);

    // 0
    tree.remove(node_1);

    // 0-5-7-8
    //   `-6
    let node_5 = tree.try_insert(5).unwrap();
    let node_6 = tree.try_insert(6).unwrap();
    let node_7 = tree.try_insert(7).unwrap();
    let node_8 = tree.try_insert(8).unwrap();

    tree.append_child(root_node, node_5).expect("valid");
    tree.append_child(node_5, node_6).expect("valid");
    tree.append_child(node_5, node_7).expect("valid");
    tree.append_child(node_7, node_8).expect("valid");

    let descendants = tree
        .descendants(root_node)
        .map(|node| tree[node])
        .collect::<Vec<i32>>();

    assert_eq!(descendants, [0, 5, 6, 7, 8]);
}
