use crate::bplus_tree::*;
use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

impl<K: Ord, V> BPlusTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let ret = self.insert_aux(key, value);
        if ret.is_none() {
            self.length += 1;
        };
         ret
    }

    fn insert_aux(&mut self, key: K, value: V) -> Option<V> {
        let mut new_root = Box::new(InternalNode::<K, V>::new());

        let root = self.root.force();
        let (behavior, ret, _) = self.root.insert(key, value);

        if let InsertBehavior::Split(key, inserted_node) = behavior {
            let node = match root {
                ForceResult::Leaf(node) => node.up_cast(),
                ForceResult::Internal(node) => node.up_cast(),
            };
            let left_child = node;
            let right_child = inserted_node;

            new_root.keys[0] = MaybeUninit::new(key);
            new_root.children[0] = MaybeUninit::new(left_child);
            new_root.children[1] = MaybeUninit::new(right_child);
            new_root.length = 2;
            self.root.node = BoxedNode::from_internal(new_root);
            self.root.height += 1;
        }
         ret
    }
}

impl<'a, BorrowType, K: Ord, V> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub(crate) fn insert(
        &'a mut self,
        key: K,
        value: V,
    ) -> (InsertBehavior<K, V>, Option<V>, usize) {
        match self.force() {
            ForceResult::Leaf(mut node) => {
                let (insertbehavior, option, idx) =
                    unsafe { node.node.ptr.as_mut().insert(key, value) };
                (insertbehavior, option, idx)
            }
            ForceResult::Internal(mut node) => {
                let length = node.as_internal().length();
                let (insertbehavior, option, idx) = node.insert(key, value);
                if let InsertBehavior::Split(key, inserted_node) = insertbehavior {
                    if CAPACITY < length {
                        let (mid_key, right_part) = node.cut_right();
                        let mut right_part = {
                            let boxed_node = BoxedNode::from_internal(right_part);
                            let mut node_ref =
                                NodeRef::<marker::Owned, K, V, marker::Internal>::from_boxed_node(
                                    boxed_node,
                                );
                            node_ref.height = node.height;
                            node_ref
                        };
                        if B <= idx {
                            let idx = idx - B;
                            unsafe { right_part.join_node(idx, key, inserted_node) };
                        } else {
                            unsafe { node.join_node(idx, key, inserted_node) };
                        }

                        return (
                            InsertBehavior::Split(mid_key, right_part.up_cast()),
                            option,
                            idx,
                        );
                    } else {
                        unsafe {
                            node.join_node(idx, key, inserted_node);
                        }
                    }
                }

                (InsertBehavior::Fit, option, idx)
            }
        }
    }
}

impl<'a, BorrowType, K: Ord, V> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>, usize) {
        let internal = self.as_internal_mut();
        internal.insert(key, value)
    }
}

impl<'a, BorrowType, K: Ord, V> NodeRef<BorrowType, K, V, marker::Leaf> {
    pub(crate) unsafe fn insert(
        &mut self,
        key: K,
        value: V,
    ) -> (InsertBehavior<K, V>, Option<V>, usize) {
        let leaf = self.node.ptr.as_mut();
        leaf.insert(key, value)
    }
}

impl<'a, K: Ord, V> InternalNode<K, V> {
    pub(crate) fn insert(
        &'a mut self,
        key: K,
        value: V,
    ) -> (InsertBehavior<K, V>, Option<V>, usize) {
        for idx in 0..self.length() - 1 {
            // 挿入位置を決定する。
            let next = unsafe { self.keys[idx].assume_init_read() };
            if key <= next {
                return {
                    let (insert_behavior, option, _) =
                        unsafe { self.children[idx].assume_init_mut().insert(key, value) };
                    (insert_behavior, option, idx)
                };
            }
        }

        // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
        let idx = self.length() - 1;
        let (insert_behavior, option, _) =
            unsafe { self.children[idx].assume_init_mut().insert(key, value) };
        (insert_behavior, option, idx)
    }
}

impl<K: Ord, V> LeafNode<K, V> {
    pub(crate) fn insert(&mut self, key: K, value: V) -> (InsertBehavior<K, V>, Option<V>, usize) {
        if self.length() < CAPACITY {
            // 空きがある場合

            if let Some(idx) = self.keys[0..self.length()]
                .iter()
                .position(|x| unsafe { x.assume_init_ref() == &key })
            {
                // 既存のkeyで挿入される場合、新しいvalueと古いvalueが交換され、古いvalueが戻り値となる。

                let mut swaped_val: MaybeUninit<V> = MaybeUninit::new(value);
                std::mem::swap(&mut self.vals[idx], &mut swaped_val);
                let ret: V = unsafe { swaped_val.assume_init() };
                (InsertBehavior::Fit, Some(ret), idx)
            } else {
                // 新規のkeyの場合、挿入位置を決定する。戻り値はNone。
                for idx in 0..self.length() {
                    let next = unsafe { self.keys[idx].assume_init_ref() };
                    if &key < next {
                        // idx番目から要素を詰める
                        let mut inserted_key = MaybeUninit::new(key);
                        let mut inserted_val = MaybeUninit::new(value);
                        for idx in idx..self.length() + 1 {
                            std::mem::swap(&mut self.keys[idx], &mut inserted_key);
                            std::mem::swap(&mut self.vals[idx], &mut inserted_val);
                        }
                        self.length += 1;
                        return (InsertBehavior::Fit, None, idx);
                    }
                }
                // ノードが保持するどのkeyよりも大きいkeyとして取り扱う。
                let inserted_key = MaybeUninit::new(key);
                let inserted_val = MaybeUninit::new(value);
                let idx = self.length();
                self.keys[idx] = inserted_key;
                self.vals[idx] = inserted_val;
                self.length += 1;

                (InsertBehavior::Fit, None, idx)
            }
        } else {
            //　空きがない場合

            let mut new_leafnode = LeafNode {
                keys: MaybeUninit::uninit_array(),
                vals: MaybeUninit::uninit_array(),
                length: TryFrom::try_from(B).unwrap(),
                prev_leaf: NonNull::new(self as *mut Self),
                next_leaf: self.next_leaf.take(),
            };

            for idx in 0..B {
                std::mem::swap(&mut new_leafnode.keys[idx], &mut self.keys[B - 1 + idx]);
                std::mem::swap(&mut new_leafnode.vals[idx], &mut self.vals[B - 1 + idx]);
            }

            self.length = TryFrom::try_from(CAPACITY - B).unwrap();

            let (_, option, _) = unsafe {
                if key <= self.keys[self.length() - 1].assume_init_read() {
                    self.insert(key, value)
                } else {
                    new_leafnode.insert(key, value)
                }
            };

            let new_boxedleafnode = Box::new(new_leafnode);

            let new_noderef = NodeRef {
                node: BoxedNode::from_leaf(new_boxedleafnode),
                height: 0,
                _metatype: PhantomData,
            };

            self.next_leaf = NonNull::new(new_noderef.node.as_ptr().as_ptr());

            unsafe {
                let shaft_key = self.keys[self.length() - 1].assume_init_read();
                (InsertBehavior::Split(shaft_key, new_noderef), option, 0)
            }
        }
    }
}
