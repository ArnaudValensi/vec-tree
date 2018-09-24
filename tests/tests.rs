extern crate vec_tree;
use vec_tree::VecTree;

#[test]
fn create_tree() {
    let mut tree = VecTree::new();

    let root_node_id = tree.insert(1);
    let child_node_id_1 = tree.insert(2);
    let child_node_id_2 = tree.insert(3);

    tree.append_child(root_node_id, child_node_id_1);
    tree.append_child(root_node_id, child_node_id_2);

    assert!(tree.capacity() == 3, "it should have 3 nodes in the tree");
    assert!(
        *tree.get(root_node_id).unwrap() == 1,
        "it should have 1 as data"
    );

    assert!(tree[child_node_id_1] == 2, "it should have 1 as data");
}

#[test]
fn get_mut() {
    let mut tree = VecTree::new();
    let idx = tree.insert(5);
    tree[idx] += 1;
    assert_eq!(tree[idx], 6);
}

#[test]
fn iterate_over_children() {
    let mut tree = VecTree::new();

    let root_node = tree.insert(1);
    let child_node_1 = tree.insert(2);
    let child_node_2 = tree.insert(3);
    let child_node_3 = tree.insert(4);
    let grandchild = tree.insert(5);

    tree.append_child(root_node, child_node_1);
    tree.append_child(root_node, child_node_2);
    tree.append_child(root_node, child_node_3);
    tree.append_child(child_node_3, grandchild);

    assert_eq!(
        tree
            .children(root_node)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );

    assert_eq!(
        tree
            .children(child_node_1)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    assert_eq!(
        tree
            .children(child_node_2)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        []
    );

    assert_eq!(
        tree
            .children(child_node_3)
            .map(|node_id| tree[node_id])
            .collect::<Vec<_>>(),
        [5]
    );
}

// #[test]
// fn can_get_live_value() {
//     let mut arena = VecTree::with_capacity(1);
//     let i = arena.try_insert(42).unwrap();
//     assert_eq!(arena[i], 42);
// }