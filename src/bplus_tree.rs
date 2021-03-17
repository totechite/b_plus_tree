use std::{
    convert::TryFrom,
    fmt::{Debug, Formatter, Result},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

pub(crate) const B: usize = 12;
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

    #[derive(Debug, Clone)]
    pub struct Ref<'a>(PhantomData<&'a ()>);
}

#[derive(Debug)]
pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

pub(crate) enum InsertBehavior<K, V> {
    Split(K, NodeRef<marker::Owned, K, V, marker::LeafOrInternal>),
    Fit,
}

pub struct BPlusTreeMap<K, V> {
    pub(crate) root: Arc<Mutex<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>>,
    pub(crate) length: usize,
}

unsafe impl<K: Ord, V> Sync for BPlusTreeMap<K, V> {}

unsafe impl<K: Ord, V> Send for BPlusTreeMap<K, V> {}

impl<K: Ord + Debug, V: Debug> Debug for BPlusTreeMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            f.debug_struct("BPlusTreeMap")
                .field("length", &self.length)
                .field("root", &self.root)
                .finish()
        } else {
            let iter = self.iter();
            f.debug_map().entries(iter).finish()
        }
    }
}

impl<K, V> BPlusTreeMap<K, V> {
    pub fn new() -> Self {
        let leaf = BoxedNode::from_leaf(Box::new(LeafNode::new()));
        let root = NodeRef::<marker::Owned, K, V, marker::Leaf>::from_boxed_node(leaf).up_cast();
        BPlusTreeMap {
            root: Arc::from(Mutex::new(root)),
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone)]
pub(crate) struct BoxedNode<K, V> {
    pub(crate) ptr: NonNull<LeafNode<K, V>>,
}

impl<K: Debug, V: Debug> Debug for BoxedNode<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BoxedNode").field("ptr", &self.ptr).finish()
    }
}

pub(crate) struct NodeRef<BorrowType, K, V, NodeType> {
    pub(crate) height: u16,
    pub(crate) node: BoxedNode<K, V>,
    pub(crate) _metatype: PhantomData<(BorrowType, NodeType)>,
}

unsafe impl<BorrowType, K, V, Type> Sync for NodeRef<BorrowType, K, V, Type> {}

unsafe impl<BorrowType, K, V, Type> Send for NodeRef<BorrowType, K, V, Type> {}

impl<BorrowType, K: Debug, V: Debug> Debug for NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        unsafe {
            let node_ref = &self.node.as_ptr();
            if self.height == 0 {
                f.debug_struct("LeafNode")
                    .field("height", &self.height)
                    .field("length", &node_ref.as_ref().length())
                    .field("key-values", &node_ref.as_ref())
                    .finish()
            } else {
                f.debug_struct("InternalNode")
                    .field("height", &self.height)
                    .field(
                        "length",
                        &node_ref.cast::<InternalNode<K, V>>().as_ref().length(),
                    )
                    .field("contents", &node_ref.cast::<InternalNode<K, V>>().as_ref())
                    .finish()
            }
        }
    }
}

impl<BorrowType, K: Debug, V: Debug> Debug for NodeRef<BorrowType, K, V, marker::Internal> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        unsafe {
            let node = self.node.as_ptr().cast::<InternalNode<K, V>>();
            let length = node.as_ref().length();
            f.debug_struct("InternalNode")
                .field("height", &self.height)
                .field("length", &length)
                .field("content", &node.as_ref())
                .finish()
        }
    }
}

impl<BorrowType, K: Debug, V: Debug> Debug for NodeRef<BorrowType, K, V, marker::Leaf> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let node = &self.node.as_ptr();
        unsafe {
            let length = node.as_ref().length();
            f.debug_struct("LeafNode")
                .field("height", &self.height)
                .field("length", &length)
                .field("key-values", &node.as_ref())
                .finish()
        }
    }
}

pub(crate) struct InternalNode<K, V> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) children: [MaybeUninit<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>;
        INTERNAL_CHILDREN_CAPACITY],
}

unsafe impl<'a, K, V> Sync for InternalNode<K, V> {}

unsafe impl<'a, K, V> Send for InternalNode<K, V> {}

impl<K: Debug, V: Debug> Debug for InternalNode<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let keys = unsafe {
            let nonnull_range = 0..self.length() - 1;
            MaybeUninit::slice_assume_init_ref(&self.keys[nonnull_range])
        };
        let children = unsafe {
            let nonnull_range = 0..self.length();
            MaybeUninit::slice_assume_init_ref(&self.children[nonnull_range])
        };

        let mut debug_map = f.debug_map();

        for idx in 0..self.length() - 1 {
            debug_map.key(&"child").value(&children[idx]);
            debug_map.key(&"key").value(&keys[idx]);
        }
        debug_map.key(&"child").value(&children[self.length() - 1]);

        debug_map.finish()
    }
}

pub(crate) struct LeafNode<K, V> {
    pub(crate) keys: [MaybeUninit<K>; CAPACITY],
    pub(crate) vals: [MaybeUninit<V>; CAPACITY],
    pub(crate) length: u16,
    pub(crate) prev_leaf: Option<NonNull<Self>>,
    pub(crate) next_leaf: Option<NonNull<Self>>,
}

unsafe impl<K: Ord, V> Sync for LeafNode<K, V> {}

unsafe impl<K: Ord, V> Send for LeafNode<K, V> {}

impl<K: Debug, V: Debug> Debug for LeafNode<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (keys, vals) = unsafe {
            let nonnull_range = 0..self.length();
            (
                MaybeUninit::slice_assume_init_ref(&self.keys[nonnull_range.clone()]),
                MaybeUninit::slice_assume_init_ref(&self.vals[nonnull_range]),
            )
        };

        let mut debug_map = f.debug_map();
        for idx in 0..self.length() {
            debug_map.key(&keys[idx]).value(&vals[idx]);
        }
        debug_map.finish()
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    
    #[inline(always)]
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

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Leaf> {
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

impl<'a, BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn from_boxed_node(boxednode: BoxedNode<K, V>) -> Self {
        Self {
            node: boxednode,
            height: 0,
            _metatype: PhantomData,
        }
    }

    #[allow(clippy::transmute_ptr_to_ptr)] #[inline]
    pub(crate) fn as_internal(&self) -> &'a InternalNode<K, V> {
        unsafe {
            &std::mem::transmute::<&LeafNode<K, V>, &InternalNode<K, V>>(&self.node.ptr.as_ref())
        }
    }

    #[allow(clippy::transmute_ptr_to_ptr)] #[inline]
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

impl<K, V> BoxedNode<K, V> {
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
        self.ptr
    }
}

impl<'a, K, V> InternalNode<K, V> {
    pub(crate) fn new() -> Self {
        InternalNode {
            keys: MaybeUninit::uninit_array(),
            length: 0,
            children: MaybeUninit::uninit_array(),
        }
    }
}

impl<K, V> LeafNode<K, V> {
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

impl<'a, BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    pub(crate) fn cut_right(&mut self) -> (K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().cut_right()
    }

    pub(crate) fn split(&mut self) -> (Box<InternalNode<K, V>>, K, Box<InternalNode<K, V>>) {
        self.as_internal_mut().split()
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

impl<'a, K, V> InternalNode<K, V> {
    pub(crate) fn length(&'a self) -> usize {
        self.length as usize
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
}

impl<K, V> LeafNode<K, V> {
    pub(crate) fn length(&self) -> usize {
        self.length as usize
    }

    pub(crate) fn split(&mut self) -> (Box<LeafNode<K, V>>, Box<LeafNode<K, V>>) {
        let mut left_leafnode = LeafNode::new();
        let mut right_leafnode = LeafNode::new();

        for idx in 0..B {
            std::mem::swap(&mut right_leafnode.keys[idx], &mut self.keys[B + idx]);
            std::mem::swap(&mut right_leafnode.vals[idx], &mut self.vals[B + idx]);
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
