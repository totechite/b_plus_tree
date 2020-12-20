#![feature(is_sorted)]
extern crate b_plus_tree;
use b_plus_tree::BPlusTree;
use rand::Rng;

use  std::collections::BTreeMap;

const VOLUME: usize = 1000;


fn main() {
    let mut b_plus_tree = BPlusTree::new();
    let mut b_tree = BTreeMap::new();
    let mut test_data = gen_test_items();

    let mut none_counter = 0;
    for key in &test_data {
        let data = format!("data: {:?}", key);
        b_tree.insert(key, data.clone());
        if let Some(ret) = b_plus_tree.insert(key, data){
            println!("{:?}", ret);
            none_counter += 1;
        }
    }
    println!("{:?}", none_counter);

    println!("{:?}", b_plus_tree.keys().is_sorted());
    let iter = b_plus_tree.iter();
    let mut counter = 0;
    for d in iter{
        counter+=1;
        // println!("{:?}", d);
    }
    println!("{:?}",b_plus_tree.keys().len());
    println!("{:?}",b_tree.len());
    println!("{:?}",b_plus_tree.len());
    test_data.sort();
    println!("{:?}", test_data.last());
    println!("{:?}",counter);
}

fn gen_test_items() -> Vec<u16> {
    let mut rng = rand::thread_rng();
    let mut insert_items = Vec::with_capacity(VOLUME);
    for _ in 0..VOLUME {
        let key = rng.gen::<u16>();
        insert_items.push(key);
    }
    return insert_items;
}
