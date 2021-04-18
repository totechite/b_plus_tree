use crate::bplus_tree::*;
use std::{
    convert::TryFrom, convert::TryInto, iter::Iterator, marker::PhantomData, mem::MaybeUninit,
    ops::RangeBounds, ptr::NonNull, rc::Weak,
};

impl<K: Ord, V> BPlusTreeMap<K, V> {
    pub fn bulk_loading(mut source: Vec<(K, V)>) -> ManageDataNode<K, V> {
        fn safe_drain<T>(source: &mut Vec<T>, range: usize) -> Option<Vec<T>> {
            // prevents panic error.
            if source.len() == 0usize {
                return None;
            }
            if source.len() > range {
                Some(source.drain(0..range).collect())
            } else {
                Some(source.drain(0..source.len()).collect())
            }
        }
        let mut datanode = ManageDataNode::<K, V>::new();
        while let Some(chunked) = safe_drain(&mut source, CAPACITY) {
            datanode.add_node(chunked);
        }
        datanode
    }
}

#[derive(Debug)]
pub struct ManageDataNode<K, V> {
    pub current_node: Option<DataNode<K, V>>,
    pub front_ptr: NonNull<DataNode<K, V>>,
    pub back_ptr: NonNull<DataNode<K, V>>,
}

impl<K, V> Iterator for ManageDataNode<K, V> {
    type Item = LeafNode<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.current_node {
            let ret = Some(node.node);
            if let Some(node) = node.next {
                self.current_node = Some(*node.ptr.as_ptr());
            }
            return ret;
        }
        None
    }
}

impl<K, V> ManageDataNode<K, V> {
    fn new() -> Self {
        let content = Box::new(DataNode::<K, V>::new());
        let back_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(content)) };
        Self {
            current_node: Some(*back_ptr.as_ptr()),
            front_ptr: back_ptr,
            back_ptr: back_ptr,
        }
    }

    fn add_node(&mut self, data: Vec<(K, V)>) {
        let mut new_datanode = Box::new(DataNode::<K, V>::new());
        let length: u16 = data.len().try_into().expect("maybe CAPACITY");
        let (mut keys, mut vals) = (
            MaybeUninit::uninit_array::<CAPACITY>(),
            MaybeUninit::uninit_array::<CAPACITY>(),
        );
        for (idx, (k, v)) in data.into_iter().enumerate() {
            keys[idx] = MaybeUninit::new(k);
            vals[idx] = MaybeUninit::new(v);
        }

        new_datanode.node.keys = keys;
        new_datanode.node.vals = vals;
        new_datanode.node.length = length;

        unsafe {
            let new_datanoderef = NonNull::new_unchecked(Box::into_raw(new_datanode));

            let node_ref = DataNodeRef::<K, V> {
                ptr: new_datanoderef,
            };

            self.back_ptr.as_mut().next = Some(node_ref);

            self.back_ptr = new_datanoderef;
        }
    }
}

#[derive(Debug)]
pub struct DataNode<K, V> {
    node: LeafNode<K, V>,
    next: Option<DataNodeRef<K, V>>,
}

impl<K, V> DataNode<K, V> {
    fn new() -> Self {
        Self {
            node: LeafNode::new(),
            next: None,
        }
    }
}

#[derive(Debug)]
struct DataNodeRef<K, V> {
    pub ptr: NonNull<DataNode<K, V>>,
}

fn add_node<K, V>(datanode: &mut DataNode<K, V>, data: Vec<(K, V)>) {
    if datanode.next.is_some() {
        let next = datanode.next.as_mut().unwrap();
        unsafe {
            add_node(&mut next.ptr.as_mut(), data);
        }
    } else {
        let new_datanode = Box::new(DataNode::<K, V>::new());

        let length: u16 = data.len().try_into().expect("maybe CAPACITY");
        let (mut keys, mut vals) = (
            MaybeUninit::uninit_array::<CAPACITY>(),
            MaybeUninit::uninit_array::<CAPACITY>(),
        );
        for (idx, (k, v)) in data.into_iter().enumerate() {
            keys[idx] = MaybeUninit::new(k);
            vals[idx] = MaybeUninit::new(v);
        }

        datanode.node.keys = keys;
        datanode.node.vals = vals;
        datanode.node.length = length;
        unsafe {
            let noderef = DataNodeRef {
                ptr: NonNull::new_unchecked(Box::into_raw(new_datanode)),
            };
            datanode.next = Some(noderef);
        }
    };
}
