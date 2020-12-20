use crate::bplus_tree::*;
use std::{fmt::Debug, mem::MaybeUninit, ptr::NonNull};

#[derive(Debug)]
pub(crate) enum RemoveBehavior<K: Debug, V: Debug> {
    RaiseChild(NodeRef<marker::Owned, K, V, marker::LeafOrInternal>, Option<(usize, V)>),
    Success(Option<(usize, V)>),
}

impl<'a, K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let (len, value) = self.root.remove(key)?;
        self.length -= 1;
        if len == 1 {
            self.root.raise_node();
        };
        return Some(value);
    }
}

impl<'a, BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub(crate) fn remove(&mut self, key: &K) -> Option<(usize, V)> {
        let remove_behavior = match self.force() {
            ForceResult::Leaf(mut node) => node.remove(key),
            ForceResult::Internal(mut node) => node.remove(key),
        };
        match remove_behavior {
            RemoveBehavior::Success(ret) => return ret,
            RemoveBehavior::RaiseChild(node, ret) => {
                self.node = node.node;
                self.height = node.height;
                return ret;
            }
        }
    }

    pub(crate) fn raise_node(&mut self) {
        match self.force() {
            ForceResult::Leaf(_) => {}
            ForceResult::Internal(node) => {
                let raised_node = node.raise_node();
                self.node = raised_node.node;
                self.height = raised_node.height;
            }
        };
    }

    pub(crate) fn get_largest_key(&self) -> K {
        match self.force() {
            ForceResult::Leaf(node) => node.get_largest_key(),
            ForceResult::Internal(node) => node.get_largest_key(),
        }
    }

    pub(crate) fn devide(&mut self, node: &mut Self) -> bool {
        match (self.force(), node.force()) {
            (ForceResult::Leaf(mut devided), ForceResult::Leaf(mut supplied)) => {
                devided.devide(&mut supplied)
            }
            (ForceResult::Internal(mut devided), ForceResult::Internal(mut supplied)) => {
                devided.devide(&mut supplied)
            }
            _ => panic!(),
        }
    }

    pub(crate) fn marge(&mut self, node: &mut Self) {
        match (self.force(), node.force()) {
            (ForceResult::Leaf(mut marged), ForceResult::Leaf(mut marge_node)) => {
                marged.marge(&mut marge_node)
            }
            (ForceResult::Internal(mut marged), ForceResult::Internal(mut marge_node)) => {
                marged.marge(&mut marge_node)
            }
            _ => panic!(),
        }
    }
}

impl<'a, BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn remove(&mut self, key: &K) -> RemoveBehavior<K, V> {
        let internal = self.as_internal_mut();
        return internal.remove(key);
    }

    pub(crate) fn raise_node(&self) -> NodeRef<marker::Owned, K, V, marker::LeafOrInternal> {
        let internal = self.as_internal();
        return unsafe { internal.children[0].assume_init_read() };
    }

    pub(crate) fn devide(&mut self, node: &mut Self) -> bool {
        let (devided_node, supplied_node) = (self.as_internal_mut(), node.as_internal_mut());

        let length_sum = devided_node.length() + supplied_node.length();

        if (length_sum / 2) <= MIN_LEN {
            // return Failure
            return false;
        }

        let mut temp_keys: [MaybeUninit<_>; (CAPACITY * 2) + 1] = MaybeUninit::<K>::uninit_array();
        let mut temp_children: [MaybeUninit<_>; INTERNAL_CHILDREN_CAPACITY * 2] =
            MaybeUninit::<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>::uninit_array();

        let devided_node_length = devided_node.length();
        temp_keys[0..devided_node_length - 1]
            .swap_with_slice(&mut devided_node.keys[0..devided_node_length - 1]);
        temp_keys[devided_node_length - 1].write(devided_node.get_largest_key());
        temp_children[0..devided_node_length]
            .swap_with_slice(&mut devided_node.children[0..devided_node_length]);

        let supplied_node_length = supplied_node.length();
        temp_keys[devided_node_length..(devided_node_length + supplied_node_length - 1)]
            .swap_with_slice(&mut supplied_node.keys[0..supplied_node_length - 1]);
        temp_children[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.children[0..supplied_node_length]);

        devided_node.keys[0..(length_sum / 2) - 1]
            .swap_with_slice(&mut temp_keys[0..(length_sum / 2) - 1]);
        devided_node.children[0..(length_sum / 2)]
            .swap_with_slice(&mut temp_children[0..(length_sum / 2)]);

        supplied_node.keys[0..length_sum - (length_sum / 2)]
            .swap_with_slice(&mut temp_keys[(length_sum / 2)..length_sum]);
        supplied_node.children[0..length_sum - (length_sum / 2)]
            .swap_with_slice(&mut temp_children[(length_sum / 2)..length_sum]);

        // lengthの修正
        devided_node.length = (length_sum / 2) as u16;
        supplied_node.length = (length_sum - (length_sum / 2)) as u16;

        // return Success
        return true;
    }

    pub(crate) fn marge(&mut self, leaf: &mut Self) {
        let (marged_node, marge_node) = (self.as_internal_mut(), leaf.as_internal_mut());

        let key = marged_node.get_largest_key();
        marged_node.keys[marged_node.length() - 1].write(key);

        if marge_node.length() == 1 {
            unsafe {
                marged_node.children[marged_node.length()]
                    .write(marge_node.children[0].assume_init_read());
            }
            marged_node.length += 1;
        } else {
            for idx in 0..marge_node.length() {
                unsafe {
                    marged_node.keys[marged_node.length()]
                        .write(marge_node.keys[idx].assume_init_read());
                    marged_node.children[marged_node.length()]
                        .write(marge_node.children[idx].assume_init_read());
                }
                marged_node.length += 1;
            }
        }
    }

    pub(crate) fn get_largest_key(&self) -> K {
        let internal = self.as_internal();
        internal.get_largest_key()
    }
}

impl<'a,BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType,K, V, marker::Leaf> {
    pub(crate) fn remove(&mut self, key: &K) -> RemoveBehavior<K, V> {
        let leaf = unsafe { self.node.ptr.as_mut() };
        return leaf.remove(key);
    }

    pub(crate) fn marge(&mut self, leaf: &mut Self) {
        let (marged_node, marge_node) = unsafe { (self.node.ptr.as_mut(), leaf.node.ptr.as_mut()) };
        for idx in 0..marge_node.length() {
            unsafe {
                marged_node.keys[marged_node.length()]
                    .write(marge_node.keys[idx].assume_init_read());
                marged_node.vals[marged_node.length()]
                    .write(marge_node.vals[idx].assume_init_read());
            }
            marged_node.length += 1;
        }

        marged_node.next_leaf = marge_node.next_leaf.take();
    }

    pub(crate) fn devide(&mut self, leaf: &mut Self) -> bool {
        let (devided_node, supplied_node) =
            unsafe { (self.node.ptr.as_mut(), leaf.node.ptr.as_mut()) };

        let length_sum = devided_node.length() + supplied_node.length();

        if (length_sum / 2) <= MIN_LEN {
            // return Failure
            return false;
        }

        let mut temp_keys: [MaybeUninit<_>; CAPACITY * 2] = MaybeUninit::<K>::uninit_array();
        let mut temp_vals: [MaybeUninit<_>; CAPACITY * 2] = MaybeUninit::<V>::uninit_array();

        let devided_node_length = devided_node.length();
        temp_keys[0..devided_node_length]
            .swap_with_slice(&mut devided_node.keys[0..devided_node_length]);
        temp_vals[0..devided_node_length]
            .swap_with_slice(&mut devided_node.vals[0..devided_node_length]);
        let supplied_node_length = supplied_node.length();
        temp_keys[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.keys[0..supplied_node_length]);
        temp_vals[devided_node_length..(devided_node_length + supplied_node_length)]
            .swap_with_slice(&mut supplied_node.vals[0..supplied_node_length]);

        devided_node.keys[0..(length_sum / 2)].swap_with_slice(&mut temp_keys[0..(length_sum / 2)]);
        devided_node.vals[0..(length_sum / 2)].swap_with_slice(&mut temp_vals[0..(length_sum / 2)]);

        supplied_node.keys[0..(length_sum - (length_sum / 2))]
            .swap_with_slice(&mut temp_keys[(length_sum / 2)..length_sum]);
        supplied_node.vals[0..(length_sum - (length_sum / 2))]
            .swap_with_slice(&mut temp_vals[(length_sum / 2)..length_sum]);

        // lengthの修正
        devided_node.length = (length_sum / 2) as u16;
        supplied_node.length = (length_sum - (length_sum / 2)) as u16;

        // return Success
        return true;
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe { self.node.ptr.as_ref().get_largest_key() }
    }
}

impl<'a, K: Ord + Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn remove(&mut self, key: &K) -> RemoveBehavior<K, V> {
        let (child_idx, child_length, val) = {
            let (child_idx, ret) = self.remove_aux(key);

            if let None = ret {
                return RemoveBehavior::Success(None);
            };

            let (child_length, val) = ret.unwrap();
            (child_idx, child_length, val)
        };

        // Check necessity balancing
        if child_length <= MIN_LEN {
            let mut devide_or_marge =
                |idx_of_balanced_node: usize, idx_of_delete_execed_node: usize| unsafe {
                    let mut delete_execed_node =
                        self.children[idx_of_delete_execed_node].assume_init_read();
                    let balanced_node = self.children[idx_of_balanced_node].assume_init_mut();

                    let is_success = balanced_node.devide(&mut delete_execed_node);
                    if is_success {
                        let balanced_node_key = self.children[idx_of_balanced_node]
                            .assume_init_ref()
                            .get_largest_key();
                        self.keys[idx_of_balanced_node].write(balanced_node_key);
                        self.children[idx_of_delete_execed_node].write(delete_execed_node);
                    } else {
                        // try marge()
                        balanced_node.marge(&mut delete_execed_node);
                        self.length -= 1;
                        for idx in idx_of_delete_execed_node..self.length() {
                            let key_idx = idx - 1;
                            self.keys.swap(key_idx, key_idx + 1);
                            self.children.swap(idx, idx + 1);
                        }
                    }
                };

            if child_idx == 0 {
                devide_or_marge(0, 1);
            } else {
                devide_or_marge(child_idx - 1, child_idx);
            }
        }

        RemoveBehavior::Success(Some((self.length(), val)))
    }

    pub(crate) fn remove_aux(&mut self, key: &K) -> (usize, Option<(usize, V)>) {
        for idx in 0..self.length() - 1 {
            // 挿入位置を決定する。
            let next = unsafe { self.keys[idx].assume_init_read() };
            if key <= &next {
                let ret = unsafe { self.children[idx].assume_init_mut().remove(key) };
                return (idx, ret);
            };
        }
        // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
        let idx = self.length() - 1;
        let ret = unsafe { self.children[idx].assume_init_mut().remove(key) };
        return (idx, ret);
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe {
            self.children[self.length() - 1]
                .assume_init_ref()
                .get_largest_key()
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn remove(&mut self, key: &K) -> RemoveBehavior<K, V> {
        // keyが存在するか確認
        let idx = {
            let matching_key = |x: &MaybeUninit<K>| unsafe { x.assume_init_ref() == key };
            let idx = self.keys[0..self.length()].iter().position(matching_key);
            if let None = idx {
                return RemoveBehavior::Success(None);
            }
            idx.unwrap()
        };
        let ret = unsafe { self.vals[idx].assume_init_read() };

        // 削除処理
        self.keys[idx] = MaybeUninit::uninit();
        self.vals[idx] = MaybeUninit::uninit();
        if idx + 1 != (self.length()) {
            for idx in idx..self.length() - 1 {
                self.keys.swap(idx, idx + 1);
                self.vals.swap(idx, idx + 1);
            }
        }
        self.length -= 1;
        RemoveBehavior::Success(Some((self.length(), ret)))
    }

    pub(crate) fn get_largest_key(&self) -> K {
        unsafe { self.keys[self.length() - 1].assume_init_read() }
    }
}
