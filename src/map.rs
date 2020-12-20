use crate::bplus_tree::*;
use std::{fmt::Debug, iter::Iterator, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

impl<'a, K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn iter(&self) -> Iter<'_, K, V> {
        let (f, b) = {
            let (f, b) = self.full_range();
            (Self::make_noderef(f), Self::make_noderef(b))
        };

        Iter {
            range: Range {
                front: Handler::<'_, K, V>::new(f),
                back: Handler::<'_, K, V>::new(b),
            },
            length: self.len(),
        }
    }

    fn full_range(&self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let front = self.root.get_front_leaf();
        let back = self.root.get_back_leaf();
        return (front, back);
    }

    fn make_noderef(box_leaf: Box<LeafNode<K, V>>) -> RefLaafNode<marker::Ref<'a>, K, V> {
        let ret = RefLaafNode::<marker::Ref<'a>, K, V> {
            node: NonNull::from(Box::leak(box_leaf)),
            _metatype: PhantomData,
        };
        return ret;
    }
}

impl<BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        match self.force() {
            ForceResult::Internal(node) => node.get_front_leaf(),
            ForceResult::Leaf(node) => node.get_ref_leaf(),
        }
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        match self.force() {
            ForceResult::Internal(node) => node.get_back_leaf(),
            ForceResult::Leaf(node) => node.get_ref_leaf(),
        }
    }
}

impl<BorrowType, K: Ord + Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Internal> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        return internal.get_front_leaf();
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        return internal.get_back_leaf();
    }
}

impl<BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Leaf> {
    fn get_ref_leaf(&self) -> Box<LeafNode<K, V>> {
        unsafe {
            let ret = Box::from_raw(self.node.as_ptr().as_ptr());
            return ret;
        }
    }
}

impl<'a, K: 'a + Ord + Debug, V: 'a + Debug> InternalNode<K, V> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        let idx = 0;
        let ret = unsafe { self.children[idx].assume_init_ref() }.get_front_leaf();
        return ret;
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        let idx = self.length();
        let ret = unsafe { self.children[idx - 1].assume_init_ref() }.get_back_leaf();
        return ret;
    }
}

#[derive(Debug)]
pub struct Keys<'a, K: Debug, V: Debug> {
    inner: Iter<'a, K, V>,
}

#[derive(Debug)]
pub struct Iter<'a, K: Debug, V: Debug> {
    range: Range<'a, K, V>,
    length: usize,
}

#[derive(Debug)]
struct Range<'a, K: Debug, V: Debug> {
    front: Handler<'a, K, V>,
    back: Handler<'a, K, V>,
}

#[derive(Debug)]
struct Handler<'a, K: Debug, V: Debug> {
    cursor_position: u8,
    node: RefLaafNode<marker::Ref<'a>, K, V>,
}

impl<'a, K: Debug, V: Debug> Handler<'a, K, V> {
    fn new(node_ptr: RefLaafNode<marker::Ref<'a>, K, V>) -> Self {
        Self {
            cursor_position: 0,
            node: node_ptr,
        }
    }

    fn cursor_position(&self) -> usize {
        self.cursor_position as usize
    }
}

#[derive(Debug)]
struct RefLaafNode<BorrowType, K: Debug, V: Debug> {
    node: NonNull<LeafNode<K, V>>,
    _metatype: PhantomData<BorrowType>,
}

impl<'a, K: 'a + Debug, V: 'a + Debug> Iterator for Handler<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let node: &'a LeafNode<K, V> = unsafe { self.node.node.as_ptr().as_ref()? };

        let count = self.cursor_position();
        if count < node.length() {
            unsafe {
                let key = node.keys[count].assume_init_ref();
                let data = node.vals[count].assume_init_ref();

                self.cursor_position += 1;
                return Some((key, data));
            }
        }
        self.cursor_position = 0;
        self.node = RefLaafNode {
            node: node.next_leaf?,
            _metatype: PhantomData,
        };
        return self.next();
    }
}

impl<'a, K: 'a + Debug, V: 'a + Debug> Iterator for Keys<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
impl<'a, K: 'a + Debug, V: 'a + Debug> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next()
    }
}

impl<'a, K: 'a + Debug, V: 'a + Debug> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.front.next()
    }
}
