#![feature(is_sorted)]
extern crate b_plus_tree;

use b_plus_tree::BPlusTree;
use rand::Rng;
const VOLUME: usize = 100000;

fn main() {
    let mut b_plus_tree = BPlusTree::new();
    let mut b_tree = std::collections::BTreeMap::new();
    let test_data = gen_test_items(); // Randamy Numeric List

    for key in test_data {
        let data = format!("data: {:?}", key);
        b_plus_tree.insert(key, data.clone());
        b_tree.insert(key, data);
    }

    // Ordered entries
    assert!(b_plus_tree.iter().is_sorted());

    // Same contents as Btree
    assert_eq!(b_tree.len(), b_plus_tree.len());
}

// Generate Numeric List
fn gen_test_items() -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut insert_items = Vec::with_capacity(VOLUME);
    for _ in 0..VOLUME {
        let key = rng.gen::<u32>();
        insert_items.push(key);
    }
    return insert_items;
}
