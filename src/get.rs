use crate::bplus_tree::*;
use std::{fmt::Debug, mem::MaybeUninit};

impl<'a, K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn get(&self, key: &'a K) -> Option<&V> {
        let v = self.root.get(key)?;
        Some(unsafe { &*v })
    }
}

impl<'a, BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        match self.force() {
            ForceResult::Leaf(node) => node.get(key),
            ForceResult::Internal(node) => node.get(key),
        }
    }
}

impl<'a, BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        let internal = self.as_internal();
        return internal.get(key);
    }
}
impl<'a, BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Leaf> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        return leaf.get(key);
    }
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        for idx in 0..self.length() - 1 {
            // 挿入位置を決定する。
            let next = unsafe { self.keys[idx].assume_init_read() };
            if key <= &next {
                return unsafe { self.children[idx].assume_init_ref().get(key) };
            }
        }

        // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
        let idx = self.length() - 1;
        return unsafe { self.children[idx].assume_init_ref().get(key) };
    }
}

impl<'a, K: Ord + Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<*const V> {
        let idx = {
            let matching_key = |x: &MaybeUninit<K>| unsafe { x.assume_init_ref() == key };
            self.keys[0..self.length()].iter().position(matching_key)?
        };
        return Some(self.vals[idx].as_ptr());
    }
}