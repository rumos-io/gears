use std::collections::HashMap;

enum BTreeMap<K: Clone, V: Clone> {
    Leaf(Leaf<K, V>),
    Inner(Inner<K, V>),
}

impl<K: Clone, V: Clone> BTreeMap<K, V> {
    fn iter(&self) -> BTreeMapIterator<K, V> {
        BTreeMapIterator::new(&self)
    }
}

struct Inner<K: Clone, V: Clone> {
    left_subtree: Option<Box<BTreeMap<K, V>>>,
    right_subtree: Option<Box<BTreeMap<K, V>>>,
    left_hash: [u8; 32],
    right_hash: [u8; 32],
}

#[derive(Clone)]
struct CachedInner {
    left_hash: [u8; 32],
    right_hash: [u8; 32],
}

#[derive(Clone)]
enum CachedBTreeMap<K, V> {
    Leaf(Leaf<K, V>),
    Inner(CachedInner),
}

#[derive(Clone)]
struct Leaf<K, V> {
    key: K,
    value: V,
}

enum BorrowedOrCached<'a, K: Clone, V: Clone> {
    Cached(CachedBTreeMap<K, V>),
    Borrowed(&'a BTreeMap<K, V>),
}

struct BTreeMapIterator<'a, K: Clone, V: Clone> {
    right_subtrees: Vec<BorrowedOrCached<'a, K, V>>,
    node_store: HashMap<[u8; 32], CachedBTreeMap<K, V>>,
}

impl<'a, K: Clone, V: Clone> BTreeMapIterator<'a, K, V> {
    fn new(root: &'a BTreeMap<K, V>) -> BTreeMapIterator<'a, K, V> {
        BTreeMapIterator {
            right_subtrees: vec![BorrowedOrCached::Borrowed(root)],
            node_store: HashMap::new(),
        }
    }

    fn traverse(&mut self) -> Option<V> {
        let mut subtree = self.right_subtrees.pop()?;

        loop {
            match &subtree {
                BorrowedOrCached::Cached(cached_subtree) => match cached_subtree {
                    CachedBTreeMap::Leaf(leaf) => return Some(leaf.value.clone()),
                    CachedBTreeMap::Inner(inner) => {
                        let right_subtree = self
                            .node_store
                            .get(&inner.right_hash)
                            .expect("db will have all nodes")
                            .clone(); // clone to simulate non borrowing cache

                        self.right_subtrees
                            .push(BorrowedOrCached::Cached(right_subtree));

                        let left_subtree = self
                            .node_store
                            .get(&inner.left_hash)
                            .expect("db will have all nodes")
                            .clone(); // clone to simulate non borrowing cache

                        subtree = BorrowedOrCached::Cached(left_subtree)
                    }
                },

                BorrowedOrCached::Borrowed(borrowed_subtree) => match borrowed_subtree {
                    BTreeMap::Leaf(leaf) => return Some(leaf.value.clone()),
                    BTreeMap::Inner(inner) => {
                        match &inner.right_subtree {
                            Some(right_subtree) => {
                                self.right_subtrees
                                    .push(BorrowedOrCached::Borrowed(&right_subtree));
                            }
                            None => {
                                let right_subtree = self
                                    .node_store
                                    .get(&inner.right_hash)
                                    .expect("db will have all nodes")
                                    .clone(); // clone to simulate non borrowing cache

                                self.right_subtrees
                                    .push(BorrowedOrCached::Cached(right_subtree));
                            }
                        }

                        match &inner.left_subtree {
                            Some(left_subtree) => {
                                subtree = BorrowedOrCached::Borrowed(left_subtree)
                            }
                            None => {
                                let left_subtree = self
                                    .node_store
                                    .get(&inner.left_hash)
                                    .expect("db will have all nodes")
                                    .clone(); // clone to simulate non borrowing cache

                                subtree = BorrowedOrCached::Cached(left_subtree)
                            }
                        }
                    }
                },
            }
        }
    }
}

impl<'a, K: Clone, V: Clone> Iterator for BTreeMapIterator<'a, K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.traverse()
    }
}
