use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub(crate) const B: usize = 3;
pub(crate) const MIN_LEN: usize = B - 1;
pub(crate) const CAPACITY: usize = 2 * B - 1;
pub(crate) const INTERNAL_CHILDREN_CAPACITY: usize = CAPACITY + 1;

pub(crate) mod marker {
    use core::marker::PhantomData;

    #[derive(Debug)]
    pub enum Leaf {}
    #[derive(Debug)]
    pub enum Internal {}
    #[derive(Debug)]
    pub enum LeafOrInternal {}

    #[derive(Debug)]
    pub struct LifetimeTag<'a>(PhantomData<&'a ()>);

    #[derive(Debug)]
    pub enum Owned {}

    #[derive(Debug)]
    pub struct Ref<'a>(PhantomData<&'a ()>);
}

#[derive(Debug)]
pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

#[derive(Debug)]
pub(crate) enum InsertBehavior<K: Debug, V: Debug> {
    Split(K, NodeRef<marker::Owned, K, V, marker::LeafOrInternal>),
    Fit,
}

#[derive(Debug)]
pub struct BPlusTree<K: Debug, V: Debug> {
    pub(crate) root: NodeRef<marker::Owned, K, V, marker::LeafOrInternal>,
    pub(crate) length: usize,
}

unsafe impl<K: Ord + Debug, V: Debug> Sync for BPlusTree<K, V> {}
unsafe impl<K: Ord + Debug, V: Debug> Send for BPlusTree<K, V> {}

impl<K: Debug, V: Debug> BPlusTree<K, V> {
    pub fn new() -> Self {
        let leaf = BoxedNode::from_leaf(Box::new(LeafNode::new()));
        let root = NodeRef::<marker::Owned, K, V, marker::Leaf>::from_boxed_node(leaf).up_cast();
        BPlusTree {
            root: root,
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BoxedNode<K: Debug, V: Debug> {
    pub(crate) ptr: NonNull<LeafNode<K, V>>,
}

#[derive(Debug)]
pub(crate) struct BoxedKey<'a, K: Debug> {
    content: &'a K,
}

#[derive(Debug)]
pub(crate) struct NodeRef<BorrowType, K: Debug, V: Debug, NodeType> {
    pub(crate) height: u16,
    pub(crate) node: BoxedNode<K, V>,
    pub(crate) _metatype: PhantomData<(BorrowType, NodeType)>,
}

unsafe impl<BorrowType, K: Ord + Debug, V: Debug, Type> Sync for NodeRef<BorrowType, K, V, Type> {}
unsafe impl<BorrowType, K: Ord + Debug, V: Debug, Type> Send for NodeRef<BorrowType, K, V, Type> {}

#[derive(Debug)]
pub(crate) struct InternalNode<K: Debug, V: Debug> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) children: [MaybeUninit<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>;
        INTERNAL_CHILDREN_CAPACITY],
}
unsafe impl<'a, K: Ord + Debug, V: Debug> Sync for InternalNode<K, V> {}
unsafe impl<'a, K: Ord + Debug, V: Debug> Send for InternalNode<K, V> {}

#[derive(Debug)]
pub(crate) struct LeafNode<K: Debug, V: Debug> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) vals: [MaybeUninit<V>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) prev_leaf: Option<NonNull<Self>>,
    pub(crate) next_leaf: Option<NonNull<Self>>,
}
unsafe impl<K: Ord + Debug, V: Debug> Sync for LeafNode<K, V> {}
unsafe impl<K: Ord + Debug, V: Debug> Send for LeafNode<K, V> {}

impl<K: Ord + Debug, V: Debug> BPlusTree<K, V> {
    pub fn values(&self) -> Vec<V> {
        let len = self.len();
        self.root.traverse_values(len)
    }

    pub fn keys(&self) -> Vec<K> {
        let len = self.len();
        self.root.traverse_keys(len)
    }
}

impl<BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub(crate) fn force(
        &self,
    ) -> ForceResult<
        NodeRef<BorrowType, K, V, marker::Leaf>,
        NodeRef<BorrowType, K, V, marker::Internal>,
    > {
        let boxed_node = BoxedNode::<K, V> {
            ptr: self.node.as_ptr(),
        };
        if self.height == 0 {
            ForceResult::Leaf(NodeRef {
                height: self.height,
                node: boxed_node,
                _metatype: PhantomData,
            })
        } else {
            ForceResult::Internal(NodeRef {
                height: self.height,
                node: boxed_node,
                _metatype: PhantomData,
            })
        }
    }
}

impl<BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Leaf> {
    pub(crate) fn from_boxed_node(boxednode: BoxedNode<K, V>) -> Self {
        Self {
            node: boxednode,
            height: 0,
            _metatype: PhantomData,
        }
    }

    pub(crate) fn up_cast(self) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _metatype: PhantomData,
        }
    }
}

impl<'a, BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn from_boxed_node(boxednode: BoxedNode<K, V>) -> Self {
        Self {
            node: boxednode,
            height: 0,
            _metatype: PhantomData,
        }
    }

    pub(crate) fn as_internal(&self) -> &'a InternalNode<K, V> {
        unsafe {
            &std::mem::transmute::<&LeafNode<K, V>, &InternalNode<K, V>>(&self.node.ptr.as_ref())
        }
    }
    pub(crate) fn as_internal_mut(&mut self) -> &'a mut InternalNode<K, V> {
        unsafe {
            std::mem::transmute::<&mut LeafNode<K, V>, &mut InternalNode<K, V>>(
                &mut self.node.ptr.as_mut(),
            )
        }
    }
    pub(crate) fn up_cast(self) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _metatype: PhantomData,
        }
    }
}

impl<K: Debug, V: Debug> BoxedNode<K, V> {
    pub(crate) fn from_leaf(node: Box<LeafNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)),
        }
    }

    pub(crate) fn from_internal(node: Box<InternalNode<K, V>>) -> Self {
        BoxedNode {
            ptr: NonNull::from(Box::leak(node)).cast(),
        }
    }

    pub(crate) fn as_ptr(&self) -> NonNull<LeafNode<K, V>> {
        NonNull::from(self.ptr)
    }
}

impl<'a, K: Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn new() -> Self {
        InternalNode {
            keys: MaybeUninit::uninit_array(),
            length: 0,
            children: MaybeUninit::uninit_array(),
        }
    }
}

impl<K: Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn new() -> Self {
        LeafNode {
            keys: MaybeUninit::uninit_array(),
            vals: MaybeUninit::uninit_array(),
            length: 0,
            prev_leaf: None,
            next_leaf: None,
        }
    }
}

impl<'a, BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    fn traverse_values(&self, len: usize) -> Vec<V> {
        match self.force() {
            ForceResult::Leaf(node) => node.traverse_values(len),
            ForceResult::Internal(node) => node.traverse_values(len),
        }
    }

    fn traverse_keys(&self, len: usize) -> Vec<K> {
        match self.force() {
            ForceResult::Leaf(node) => node.traverse_keys(len),
            ForceResult::Internal(node) => node.traverse_keys(len),
        }
    }
}

impl<'a, BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Leaf> {
    fn traverse_values(&self, len: usize) -> Vec<V> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        let mut buff = Vec::with_capacity(len);
        unsafe {
            leaf.traverse_values(&mut buff);
            return buff;
        };
    }

    fn traverse_keys(&self, len: usize) -> Vec<K> {
        let leaf = unsafe { self.node.ptr.as_ref() };
        let mut buff = Vec::with_capacity(len);
        unsafe {
            leaf.traverse_keys(&mut buff);
            return buff;
        };
    }
}

impl<'a, BorrowType, K: Debug, V: Debug> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn cut_right(&mut self) -> (K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().cut_right()
    }

    pub(crate) fn split(&mut self) -> (Box<InternalNode<K, V>>, K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().split()
    }

    fn traverse_values(&self, len: usize) -> Vec<V> {
        self.as_internal().traverse_values(len)
    }
    fn traverse_keys(&self, len: usize) -> Vec<K> {
        self.as_internal().traverse_keys(len)
    }

    pub(crate) unsafe fn join_node(
        &mut self,
        index: usize,
        key: K,
        node: NodeRef<marker::Owned, K, V, marker::LeafOrInternal>,
    ) {
        let mut self_as_internal = self.as_internal_mut();
        let mut key = MaybeUninit::new(key);
        let mut node = MaybeUninit::new(node);

        for idx in index..self_as_internal.length() {
            std::mem::swap(&mut self_as_internal.keys[idx], &mut key);
        }
        for idx in (index + 1)..self_as_internal.length() + 1 {
            std::mem::swap(&mut self_as_internal.children[idx], &mut node);
        }

        self_as_internal.length += 1;
    }
}

impl<'a, K: Debug, V: Debug> InternalNode<K, V> {
    pub(crate) fn length(&'a self) -> usize {
        self.length as usize
    }

    fn set_length(&'a mut self, len: u16) {
        self.length = len;
    }

    pub(crate) fn cut_right(&'a mut self) -> (K, Box<InternalNode<K, V>>) {
        let mut right_internal_node: InternalNode<K, V> = InternalNode::new();

        let raised_key = unsafe { self.keys[B - 1].assume_init_read() };

        right_internal_node.keys[0..B - 1].swap_with_slice(&mut self.keys[B..CAPACITY]);

        right_internal_node.children[0..B].swap_with_slice(&mut self.children[B..CAPACITY + 1]);
        self.length = B as u16;
        right_internal_node.length = B as u16;

        (raised_key, Box::new(right_internal_node))
    }

    pub(crate) fn split(&'a mut self) -> (Box<InternalNode<K, V>>, K, Box<InternalNode<K, V>>) {
        let mut left_internal_node: InternalNode<K, V> = InternalNode::new();
        let mut right_internal_node: InternalNode<K, V> = InternalNode::new();

        let raised_key = unsafe { self.keys[B - 1].assume_init_read() };

        for idx in 0..B - 1 {
            std::mem::swap(&mut left_internal_node.keys[idx], &mut self.keys[idx]);
        }
        for idx in 0..B - 1 {
            std::mem::swap(&mut right_internal_node.keys[idx], &mut self.keys[B + idx]);
        }
        for idx in 0..B {
            std::mem::swap(
                &mut left_internal_node.children[idx],
                &mut self.children[idx],
            );
        }
        for idx in 0..B {
            std::mem::swap(
                &mut right_internal_node.children[idx],
                &mut self.children[B + idx],
            );
        }

        left_internal_node.length = B as u16;
        right_internal_node.length = B as u16;

        (
            Box::new(left_internal_node),
            raised_key,
            Box::new(right_internal_node),
        )
    }

    fn traverse_values(&self, len: usize) -> Vec<V> {
        unsafe { self.children[0].assume_init_ref().traverse_values(len) }
    }

    fn traverse_keys(&self, len: usize) -> Vec<K> {
        unsafe { self.children[0].assume_init_ref().traverse_keys(len) }
    }
}

impl<K: Debug, V: Debug> LeafNode<K, V> {
    pub(crate) fn length(&self) -> usize {
        self.length as usize
    }

    unsafe fn traverse_values(&self, buff: &mut Vec<V>) {
        let mut current_leaf_vals = self.vals[0..self.length()]
            .iter()
            .map(|x| x.assume_init_read())
            .collect::<Vec<V>>();
        buff.append(&mut current_leaf_vals);
        if let Some(next) = self.next_leaf {
            next.as_ref().traverse_values(buff);
        }
    }

    unsafe fn traverse_keys(&self, buff: &mut Vec<K>) {
        let mut current_leaf_keys = self.keys[0..self.length()]
            .iter()
            .map(|x| x.assume_init_read())
            .collect::<Vec<K>>();
        buff.append(&mut current_leaf_keys);
        if let Some(next) = self.next_leaf {
            next.as_ref().traverse_keys(buff);
        }
    }

    pub(crate) fn split(&mut self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let mut left_leafnode = LeafNode::new();
        let mut right_leafnode = LeafNode::new();

        for idx in 0..B {
            std::mem::swap(&mut right_leafnode.keys[idx], &mut self.keys[B - 1 + idx]);
            std::mem::swap(&mut right_leafnode.vals[idx], &mut self.vals[B - 1 + idx]);
        }
        right_leafnode.length = TryFrom::try_from(B).unwrap();

        for idx in 0..B - 1 {
            std::mem::swap(&mut left_leafnode.keys[idx], &mut self.keys[idx]);
            std::mem::swap(&mut left_leafnode.vals[idx], &mut self.vals[idx]);
        }
        left_leafnode.length = TryFrom::try_from(CAPACITY - B).unwrap();
        right_leafnode.prev_leaf = NonNull::new(&mut left_leafnode as *mut LeafNode<K, V>);
        right_leafnode.next_leaf = self.next_leaf.take();

        left_leafnode.prev_leaf = self.prev_leaf.take();
        left_leafnode.next_leaf = NonNull::new(&mut right_leafnode as *mut LeafNode<K, V>);
        (Box::new(left_leafnode), Box::new(right_leafnode))
    }
}
