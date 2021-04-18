use crate::bplus_tree::*;
use std::borrow::Borrow;

impl<K: Ord, V> BPlusTreeMap<K, V> {
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let leaf = self.root.lock().expect("pass").get_leaf(key.borrow());
        unsafe {
            for idx in 0..(leaf.length()) {
                if key == leaf.keys[idx].assume_init_ref().borrow() {
                    return leaf.vals[idx].as_ptr().as_ref();
                }
            }
        }
        None
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub(crate) fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        match self.force() {
            ForceResult::Internal(node) => node.get_front_leaf(),
            ForceResult::Leaf(node) => node.get_ref_leaf(),
        }
    }

    pub(crate) fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        match self.force() {
            ForceResult::Internal(node) => node.get_back_leaf(),
            ForceResult::Leaf(node) => node.get_ref_leaf(),
        }
    }

    pub(crate) fn get_leaf<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        match self.force() {
            ForceResult::Internal(node) => node.get_leaf(key),
            ForceResult::Leaf(node) => node.get_leaf(key),
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        internal.get_front_leaf()
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        internal.get_back_leaf()
    }

    fn get_leaf<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        let internal = self.as_internal();
        internal.get_leaf(key)
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Leaf> {
    fn get_ref_leaf(&self) -> Box<LeafNode<K, V>> {
        unsafe { Box::from_raw(self.node.as_ptr().as_ptr()) }
    }

    fn get_leaf<T>(&self, _: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        unsafe { Box::from_raw(self.node.as_ptr().as_ptr()) }
    }
}

impl<K, V> InternalNode<K, V> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        let idx = 0;
        let ret = unsafe { self.children[idx].assume_init_ref() }.get_front_leaf();
        ret
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        let idx = self.length();
        let ret = unsafe { self.children[idx - 1].assume_init_ref() }.get_back_leaf();
        ret
    }

    fn get_leaf<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        for idx in 0..self.length() - 1 {
            let next = unsafe { self.keys[idx].assume_init_ref() };
            if key <= next.borrow() {
                return unsafe { self.children[idx].assume_init_ref().get_leaf(key) };
            }
        }

        let idx = self.length() - 1;
        unsafe { self.children[idx].assume_init_ref().get_leaf(key) }
    }
}
