use crate::bplus_tree::*;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Formatter, Result},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Bound::*, RangeBounds},
    ptr::NonNull,
};

fn make_noderef<'a, K, V>(box_leaf: Box<LeafNode<K, V>>) -> RefLeafNode<marker::Ref<'a>, K, V>
where
    K: Ord,
    V: ,
{
     RefLeafNode::<marker::Ref<'a>, K, V> {
        node: NonNull::from(Box::leak(box_leaf)),
        _metatype: PhantomData,
    }
    
}

impl<'a, K: Ord, V> BPlusTree<K, V> {
    pub fn iter(&self) -> Iter<'_, K, V> {
        let (f, b) = {
            let (f, b) = self.full_range();
            (make_noderef(f), make_noderef(b))
        };

        let back_cursor_position = unsafe { b.node.as_ref().length() - 1 };

        Iter {
            range: Range {
                front: Some(Handler::<'_, K, V> {
                    node: f,
                    cursor_position: 0,
                }),
                back: Some(Handler::<'_, K, V> {
                    node: b,
                    cursor_position: back_cursor_position,
                }),
            },
            length: self.len(),
        }
    }

    fn full_range(&self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let front = self.root.get_front_leaf();
        let back = self.root.get_back_leaf();
        (front, back)
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a BPlusTree<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<BorrowType, K: Ord, V> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
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

    pub(crate) fn get_range<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        match self.force() {
            ForceResult::Internal(node) => node.get_range(key),
            ForceResult::Leaf(node) => node.get_range(key),
        }
    }
}

impl<BorrowType, K: Ord, V> NodeRef<BorrowType, K, V, marker::Internal> {
    fn get_front_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        internal.get_front_leaf()
    }

    fn get_back_leaf(&self) -> Box<LeafNode<K, V>> {
        let internal = self.as_internal();
        internal.get_back_leaf()
    }

    fn get_range<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        let internal = self.as_internal();
        internal.get_range(key)
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Leaf> {
    fn get_ref_leaf(&self) -> Box<LeafNode<K, V>> {
        unsafe { Box::from_raw(self.node.as_ptr().as_ptr()) }
    }

    fn get_range<T>(&self, _: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        unsafe { Box::from_raw(self.node.as_ptr().as_ptr()) }
    }
}

impl<'a, K: 'a + Ord, V: 'a> InternalNode<K, V> {
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

    fn get_range<T>(&self, key: &T) -> Box<LeafNode<K, V>>
    where
        K: Borrow<T>,
        T: Ord + ?Sized,
    {
        for idx in 0..self.length() - 1 {
            let next = unsafe { self.keys[idx].assume_init_ref() };
            if key <= next.borrow() {
                return unsafe { self.children[idx].assume_init_ref().get_range(key) };
            }
        }

        let idx = self.length() - 1;
        unsafe { self.children[idx].assume_init_ref().get_range(key) }
    }
}

#[derive(Clone)]
pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<K: Debug, V> Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list()
            .entries(self.inner.clone().map(|tuple| tuple.0))
            .finish()
    }
}

impl<'a, K: Ord, V> BPlusTree<K, V> {
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }
}

#[derive(Clone)]
pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<K, V: Debug> Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list()
            .entries(self.inner.clone().map(|tuple| tuple.1))
            .finish()
    }
}

impl<'a, K: Ord, V> BPlusTree<K, V> {
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }
}

pub struct Iter<'a, K, V> {
    range: Range<'a, K, V>,
    length: usize,
}

impl<K: Debug, V: Debug> Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Iter {
            range: self.range.clone(),
            length: self.length,
        }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            let ret = self.range.unchecked_next();
            self.length -= 1;
            Some(ret)
        }
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        if self.length == 0 {
            None
        } else {
            let ret = self.range.unchecked_next_back();
            self.length -= 1;
            Some(ret)
        }
    }
}

/// struct Range
///
/// BPlusTreeの要素の範囲サブセット
/// BPlusTree.range() -> Range
///
/// front: keyが小さい側のLeafNodeのポインタ
/// back: keyが大きい側のLeafNodeのポインタ
pub struct Range<'a, K, V> {
    front: Option<Handler<'a, K, V>>,
    back: Option<Handler<'a, K, V>>,
}

impl<K, V> Clone for Range<'_, K, V> {
    fn clone(&self) -> Self {
        Range {
            front: self.front.clone(),
            back: self.back.clone(),
        }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.unchecked_next())
        }
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Range<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        if self.is_empty() {
            None
        } else {
            Some(self.unchecked_next_back())
        }
    }
}

impl<'a, K: 'a, V: 'a> Range<'a, K, V> {
    fn unchecked_next(&mut self) -> (&'a K, &'a V) {
        let kv = self.front.as_mut().unwrap().next().unwrap();
        (&kv.0, &kv.1)
    }

    fn unchecked_next_back(&mut self) -> (&'a K, &'a V) {
        let kv = self.back.as_mut().unwrap().next_back().unwrap();
        (&kv.0, &kv.1)
    }
}

impl<'a, K, V> Range<'a, K, V> {
    fn is_empty(&self) -> bool {
        self.front == self.back
    }
}

impl<'a, K: 'a + Ord, V: 'a> FusedIterator for Range<'a, K, V> {}

/// struct Handler
///
/// LeafNodeをIteratorとして制御する為の構造体
///
/// cursor_position: LeafNode内部のkey-valueの現在位置を管理する
/// node: LeafNodeのポインタ
///

pub(crate) struct Handler<'a, K, V> {
    cursor_position: usize,
    node: RefLeafNode<marker::Ref<'a>, K, V>,
}

impl<'a, K, V> Handler<'a, K, V> {
    pub(crate) fn new(
        node_ptr: RefLeafNode<marker::Ref<'a>, K, V>,
        cursor_position: usize,
    ) -> Self {
        Self {
            cursor_position: cursor_position,
            node: node_ptr,
        }
    }

    fn cursor_position(&self) -> usize {
        self.cursor_position as usize
    }
}

impl<K: Debug, V: Debug> Debug for Handler<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Handler")
            .field("cursor_position", &self.cursor_position)
            .field("node", &self.node)
            .finish()
    }
}

impl<K, V> Clone for Handler<'_, K, V> {
    fn clone(&self) -> Self {
        Handler {
            cursor_position: self.cursor_position,
            node: self.node.clone(),
        }
    }
}

impl<K, V> PartialEq for Handler<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<K, V> Eq for Handler<'_, K, V> {}

impl<'a, K: 'a, V: 'a> Iterator for Handler<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let node: &'a LeafNode<K, V> = unsafe { self.node.node.as_ptr().as_ref()? };
        let count = self.cursor_position();

        match count.cmp(&(node.length() - 1)) {
            Ordering::Greater => {
                self.cursor_position = 0;
                self.node = RefLeafNode {
                    node: node.next_leaf?,
                    _metatype: PhantomData,
                };
                return self.next();
            }
            Ordering::Equal | Ordering::Less => {}
        }

        let key = unsafe { node.keys[count].assume_init_ref() };
        let data = unsafe { node.vals[count].assume_init_ref() };
        let ret = Some((key, data));

        self.cursor_position += 1;

        ret
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Handler<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        let node: &'a LeafNode<K, V> = unsafe { self.node.node.as_ptr().as_ref()? };
        let count = self.cursor_position();

        match count.cmp(&0) {
            Ordering::Greater | Ordering::Equal => {}
            Ordering::Less => unsafe {
                let prev_node = node.prev_leaf.unwrap();
                self.cursor_position = prev_node.as_ref().length() - 1;
                self.node = RefLeafNode {
                    node: prev_node,
                    _metatype: PhantomData,
                };
                return self.next_back();
            },
        };

        let key = unsafe { node.keys[count].assume_init_ref() };
        let data = unsafe { node.vals[count].assume_init_ref() };
        let ret = Some((key, data));

        self.cursor_position -= 1;

        ret
    }
}

pub(crate) struct RefLeafNode<BorrowType, K, V> {
    node: NonNull<LeafNode<K, V>>,
    _metatype: PhantomData<BorrowType>,
}

impl<BorrowType, K, V> PartialEq for RefLeafNode<BorrowType, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl<BorrowType, K, V> Eq for RefLeafNode<BorrowType, K, V> {}

impl<BorrowType, K: Debug, V: Debug> Debug for RefLeafNode<BorrowType, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("RefLeafNode")
            .field("node", &self.node)
            .field("_metatype", &self._metatype)
            .finish()
    }
}
impl<BorrowType, K, V> Clone for RefLeafNode<BorrowType, K, V> {
    fn clone(&self) -> Self {
        RefLeafNode {
            node: self.node,
            _metatype: self._metatype,
        }
    }
}

impl<'a, K: 'a + Ord, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.inner.next()?.0;
        Some(key)
    }
}

impl<'a, K: 'a + Ord, V: 'a> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        let key = self.inner.next_back()?.0;
        Some(key)
    }
}

impl<'a, K: 'a + Ord, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?.1;
        Some(value)
    }
}

impl<'a, K: 'a + Ord, V: 'a> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        let value = self.inner.next_back()?.1;
        Some(value)
    }
}

impl<'a, K: 'a + Ord, V: 'a> FusedIterator for Iter<'a, K, V> {}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn range<T, R>(&self, range: R) -> Range<'_, K, V>
    where
        K: Borrow<T>,
        R: RangeBounds<T>,
        T: Ord + Debug + ?Sized,
    {
        let (front, start_key) = {
            match range.start_bound() {
                Included(start) => (self.root.get_range(start), range.start_bound()),
                Unbounded => (self.root.get_front_leaf(), range.start_bound()),
                Excluded(start) => (self.root.get_range(start), range.start_bound()),
            }
        };
        let (mut back, end_key) = {
            match range.end_bound() {
                Included(end) => (self.root.get_range(end), range.end_bound()),
                Unbounded => (self.root.get_back_leaf(), range.end_bound()),
                Excluded(end) => (self.root.get_range(end), range.end_bound()),
            }
        };
        let front_cursor_position = {
            let node = front.as_ref();
            let mut ret = None;

            match start_key {
                Included(key) => {
                    for idx in 0..node.length() {
                        let next_key = unsafe { node.keys[idx].assume_init_ref() };
                        if key <= next_key.borrow() {
                            ret = Some(idx);
                            break;
                        };
                    }
                }
                Unbounded => {}
                Excluded(key) => {
                    let mut node = front.as_ref();

                    let next_key = unsafe { node.keys[node.length() - 1].assume_init_ref() };
                    if key < next_key.borrow() {
                        if let Some(next_node) = node.next_leaf {
                            back = unsafe { Box::from_raw(next_node.as_ptr()) };
                            node = back.as_ref();
                        }
                    };

                    for idx in 0..node.length() {
                        let next_key = unsafe { node.keys[idx].assume_init_ref() };
                        if key < next_key.borrow() {
                            ret = Some(idx);
                            break;
                        };
                    }
                }
            }
            ret
        };
        let back_cursor_position = {
            let mut ret = None;

            match end_key {
                Included(key) => {
                    let node = back.as_ref();
                    for idx in 0..node.length() {
                        let next_back_key = unsafe { node.keys[idx].assume_init_ref() };
                        if next_back_key.borrow() <= key {
                            ret = Some(idx);
                            break;
                        };
                    }
                }
                Unbounded => {}
                Excluded(key) => {
                    let mut node = back.as_ref();

                    let next_back_key = unsafe { node.keys[0].assume_init_ref() };
                    if key < next_back_key.borrow() {
                        if let Some(prev_node) = node.prev_leaf {
                            back = unsafe { Box::from_raw(prev_node.as_ptr()) };
                            node = back.as_ref();
                        }
                    };

                    for idx in 1..=node.length() {
                        let idx = node.length() - idx;
                        let next_back_key = unsafe { node.keys[idx].assume_init_ref() };
                        if next_back_key.borrow() < key {
                            ret = Some(idx);
                            break;
                        };
                    }
                }
            }
            ret
        };

        let (front, back) =
            if let (Some(f), Some(b)) = (front_cursor_position, back_cursor_position) {
                let front = Handler::new(make_noderef(front), f);
                let back = Handler::new(make_noderef(back), b);
                (Some(front), Some(back))
            } else {
                let front = Handler::new(make_noderef(front), 0);
                let back = {
                    let init_position = back.length() - 1;
                    Handler::new(make_noderef(back), init_position)
                };
                (Some(front), Some(back))
            };

        Range::<'_, K, V> { front, back }
    }
}
