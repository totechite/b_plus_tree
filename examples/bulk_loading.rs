#![feature(is_sorted)]
extern crate b_plus_tree;

use b_plus_tree::BPlusTreeMap;
use rand::Rng;
const VOLUME: usize = 100000;

use std::panic::catch_unwind;
fn main() {
    let test_data = catch_unwind(gen_test_items); // Randamy Numeric List

    let ret = BPlusTreeMap::bulk_loading(test_data.unwrap());

    unsafe {
        println!("{:?}", ret.back_ptr.as_ref());
    }
}

// Generate Numeric List
fn gen_test_items() -> Vec<(u32, u32)> {
    use std::convert::TryFrom;
    let mut rng = rand::thread_rng();
    let mut insert_items = Vec::with_capacity(VOLUME);
    for idx in 0..VOLUME {
        let key = rng.gen::<u32>();
        insert_items.push((TryFrom::try_from(idx).unwrap(), key));
    }
    return insert_items;
}
