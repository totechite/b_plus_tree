#![feature(test)]

extern crate b_plus_tree;

#[cfg(test)]
mod tests {

    use b_plus_tree::BPlusTreeMap;
    use rand::Rng;
    const VOLUME: usize = 5000;

    fn gen_test_items() -> Vec<u64> {
        let mut rng = rand::thread_rng();
        let mut insert_items = Vec::with_capacity(VOLUME);
        for _ in 0..VOLUME {
            let key = rng.gen::<u64>();
            insert_items.push(key);
        }
        return insert_items;
    }

    #[test]
    fn insert() {
        let mut b_plus_tree = BPlusTreeMap::new();
        let test_data = gen_test_items();

        for key in test_data {
            let data = format!("data: {:?}", key);
            b_plus_tree.insert(key, data);
        }

        assert_eq!(VOLUME, b_plus_tree.keys().count())
    }

    #[test]
    fn remove() {
        let mut b_plus_tree = BPlusTreeMap::new();
        let test_data = gen_test_items();

        for key in &test_data {
            let data = format!("data: {:?}", key);
            b_plus_tree.insert(key, data);
        }

        for key in &test_data {
            b_plus_tree.remove(&key);
        }

        assert_eq!(0, b_plus_tree.keys().count())
    }
}
