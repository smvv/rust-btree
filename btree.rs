//! A B-tree is a tree data structure that keeps data sorted and allows
//! searches, sequential access, insertions, deletions in logarithmic time.
//! B-trees are different from binary search trees because a b-tree node can
//! have more than two children (also known as the *degree* of a b-tree).
//!
//! Basic example:
//!
//! ~~~ rust
//! let foo = "foo";
//! let bar = "bar";
//!
//! let mut s = BTree::new();
//!
//! s.insert(1, "foo");
//! s.insert(42, "bar");
//!
//! assert_eq!(s.find(1).unwrap(), &foo);
//! assert_eq!(s.find(42).unwrap(), &bar);
//! ~~~

//#[link(name="btree", vers="0.1pre",
//       uuid="136fafb0-e4e0-11e2-a28f-0800200c9a66")];

//use std::vec::VecIterator;
use std::util::replace;
use std::io;

// The number of keys is chosen to vary between d and 2d, where d is the
// minimum number of keys and d+1 is the minimum degree (branching factor) of
// the tree. In this case, d = 2 which results in degree = 5.
static BTREE_DEGREE : uint = 5;

pub struct BTree<K, V> {
    priv used: uint,
    priv nodes: [Option<TreeItem<K, V>>, ..BTREE_DEGREE],
}

enum TreeItem<K, V> {
    TreeNode { key: K, value: ~BTree<K, V> },
    TreeLeaf { key: K, value: V },
}

impl<K: Eq + Ord, V> Container for BTree<K, V> {
    /// Return the number of nodes or values in use in the b-tree node.
    #[inline]
    fn len(&self) -> uint { self.used }

    /// Return true if the b-tree node contains no nodes or values.
    #[inline]
    fn is_empty(&self) -> bool {
        match *self.nodes.head() {
            None => { true }
            _ => { false }
        }
    }
}

impl<K: TotalOrd, V> Mutable for BTree<K, V> {
    /// Clear the b-tree, removing all nodes.
    fn clear(&mut self) {
        for self.nodes.mut_iter().advance |node| {
            match *node {
                None => {
                    break;
                }
                _ => {
                    *node = None;
                }
            }
        }
    }
}

impl<K: Eq + Ord, V> BTree<K, V> {
    pub fn new() -> BTree<K, V> {
        // TODO: once https://github.com/mozilla/rust/issues/5244 is fixed,
        // use the following statement:
        //BTree { used: 0, nodes: ~[None, ..BTREE_DEGREE] }
        BTree { used: 0, nodes: [None, None, None, None, None] }
    }

    /// Return the number of nodes or values that can be stored in the b-tree
    /// node.
    pub fn capacity(&self) -> uint { self.nodes.len() }

    /// Return a reference to the value corresponding to the key.
    pub fn find<'a> (&'a self, key: K) -> Option<&'a V> {
        let mut pos = 0;
        let mut current: &'a Option<TreeItem<K, V>> = &self.nodes[pos];

        loop {
            match *current {
                Some(TreeNode { key: ref k, value: ref tree }) => {
                    if key < *k {
                        pos += 1;
                    } else if key == *k {
                        fail!("TODO: key == *k");
                        //return Some(tree);
                    } else {
                        pos += 1;
                    }
                }
                Some(TreeLeaf { key: ref k, value: ref value }) => {
                    if key < *k {
                        pos += 1;
                    } else if key == *k {
                        return Some(value);
                    } else {
                        pos += 1;
                    }
                }
                None => return None
            }

            current = &self.nodes[pos];
        }
    }

    /// Insert a key-value pair into the b-tree.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        io::println(fmt!("insert key-value pair: %? -> %?", key, value));

        if self.is_empty() {
            insert_leaf(self, 0, key, value);
            return true;
        }

        let capacity = self.nodes.len();

        // If the node contains fewer than the maximum legal number of
        // elements, then there is room for the new element.
        if self.used < capacity {
            // Insert the new element in the node, keeping the node's elements
            // ordered.
            let mut pos = 0;

            while pos < capacity {
                io::println(fmt!("pos: %u", pos));

                match self.nodes[pos] {
                    Some(TreeNode { key: ref k, value: ref mut tree }) => {
                        if key < *k {
                            tree.insert(key, value);
                            break;
                        } else if key == *k {
                            fail!("TODO: key == *k");
                        }
                    }
                    Some(TreeLeaf { key: ref k, value: ref mut v }) => {
                        if key < *k {
                            fail!("TODO: key < *k");
                            break;
                        } else if key == *k {
                            replace(v, value);
                            break;
                        }
                    }
                    None => {
                        insert_leaf(self, pos, key, value);
                        break;
                    }
                }

                pos += 1;
            }

            // TODO: move to into node.
            assert!(pos < capacity);

            true
        }
        // Otherwise the node is full, evenly split it into two nodes.
        else {
            // 1. A single median is chosen from among the leaf's elements and
            // the new element.

            // 2. Values less than the median are put in the new left node and
            // values greater than the median are put in the new right node,
            // with the median acting as a separation value.

            // 3. The separation value is inserted in the node's parent, which
            // may cause it to be split, and so on. If the node is the root,
            // create a new root above this node.
            true
        }
    }
}

fn insert_leaf<K, V>(tree: &mut BTree<K, V>, pos: uint, key: K, value: V) {
    assert!(tree.used < tree.nodes.len());

    tree.used += 1;
    tree.nodes[pos] = Some(TreeLeaf { key: key, value: value });
}

fn main() {
    let foo = "foo";
    let bar = "bar";

    let mut s = BTree::new();
    assert!(s.is_empty());

    assert!(s.insert(1, foo));
    assert!(s.insert(42, bar));

    io::println(fmt!("find(1) -> %?", s.find(1)));
    io::println(fmt!("find(42) -> %?", s.find(42)));

    assert_eq!(s.find(1).unwrap(), &foo);
    assert_eq!(s.find(42).unwrap(), &bar);

    assert!(!s.is_empty());
    s.clear();
    assert!(s.is_empty());

    io::println("Done with btree!");
}
