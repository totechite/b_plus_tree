#![feature(test)]
extern crate b_plus_tree;
extern crate test;

use b_plus_tree::BPlusTreeMap;
use rand::Rng;
use test::{black_box, Bencher};

const VOLUME: usize = 1000000;

fn gen_b_plus_tree() -> BPlusTreeMap<u64, [u8; 256]> {
    let mut rng = rand::thread_rng();
    let mut b_plus_tree = BPlusTreeMap::new();
    for _ in 0..VOLUME {
        let key = rng.gen::<u64>();
        b_plus_tree.insert(key, [0u8; 256]);
    }
    b_plus_tree
}

fn gen_b_tree() -> std::collections::BTreeMap<u64, [u8; 256]> {
    let mut rng = rand::thread_rng();
    let mut b_tree = std::collections::BTreeMap::new();
    for _ in 0..VOLUME {
        let key = rng.gen::<u64>();
        b_tree.insert(key, [0u8; 256]);
    }
    b_tree
}

#[bench]
fn bench_b_plus_tree_traverse(b: &mut Bencher) {
    let b_plus_tree = black_box(gen_b_plus_tree());
    b.iter(|| {
        b_plus_tree.iter().count();
    });
}

#[bench]
fn bench_b_tree_traverse(b: &mut Bencher) {
    let b_tree = black_box(gen_b_tree());
    b.iter(|| {
        b_tree.iter().count();
    });
}

#[bench]
fn bench_b_plus_tree_remove(b: &mut Bencher) {
    let mut b_plus_tree = black_box(gen_b_plus_tree());
    let mut rng = rand::thread_rng();
    let key = rng.gen::<u64>();
    b_plus_tree.insert(key, [0u8; 256]);
    b.iter(|| {
        b_plus_tree.remove(&key);
    });
}

#[bench]
fn bench_b_tree_remove(b: &mut Bencher) {
    let mut b_tree = black_box(gen_b_tree());
    let mut rng = rand::thread_rng();
    let key = rng.gen::<u64>();
    b_tree.insert(key, [0u8; 256]);
    b.iter(|| {
        b_tree.remove(&key);
    });
}

#[bench]
fn bench_b_plus_tree_get(b: &mut Bencher) {
    let mut b_plus_tree = black_box(gen_b_plus_tree());
    let mut rng = rand::thread_rng();
    let key = rng.gen::<u64>();
    b_plus_tree.insert(key, [0u8; 256]);
    b.iter(|| {
        b_plus_tree.get(&key);
    });
}

#[bench]
fn bench_b_tree_get(b: &mut Bencher) {
    let mut b_tree = black_box(gen_b_tree());
    let mut rng = rand::thread_rng();
    let key = rng.gen::<u64>();
    b_tree.insert(key, [0u8; 256]);
    b.iter(|| {
        b_tree.get(&key);
    });
}