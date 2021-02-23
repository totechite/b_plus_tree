#![feature(is_sorted)]
extern crate b_plus_tree;

use b_plus_tree::BPlusTree;
use rand::Rng;

use std::collections::BTreeMap;

const VOLUME: usize = 100000;

fn main() {
    let mut b_plus_tree = BPlusTree::new();
    let mut b_tree = BTreeMap::new();
    let test_data = gen_test_items();

    let mut duplex = vec![];

    for key in test_data {
        let data = format!("data: {:?}", key);
        let v = b_plus_tree.insert(key, data.clone());
        if let Some(_) = v {
            duplex.push(key);
        }
        b_tree.insert(key, data);
    }
    println!("{:?}", duplex);
    for key in duplex {
        let v = b_plus_tree.get(&key);
        println!("{:?}", v);
    }
    println!("btree.len(): {:#?}", b_tree.len());
    println!("{:?}", b_tree.range(100000..1000000));

    println!("b_plus_tree.len(): {:#?}", b_plus_tree.len());
}

fn gen_test_items() -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut insert_items = Vec::with_capacity(VOLUME);
    for _ in 0..VOLUME {
        let key = rng.gen::<u32>();
        insert_items.push(key);
    }
    return insert_items;
}
