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
    let mut tree = VecTree::new();
    let root = tree.insert(0);
    let child1 = tree.insert(1);
    let child2 = tree.insert(2);
    let child3 = tree.insert(3);

    tree.append_child(root, child1).expect("valid");
    tree.append_child(root, child2).expect("valid");
    tree.append_child(root, child3).expect("valid");

    tree.remove(child3);

    let child4 = tree.insert(4);
    tree.append_child(root, child4).expect("valid");

    assert_eq!(
        tree.children(root)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [1, 2, 4]
    );
}

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
