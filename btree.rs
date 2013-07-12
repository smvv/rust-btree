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
//! let mut t = BTree::new();
//!
//! t.insert(1, "foo");
//! t.insert(42, "bar");
//!
//! assert_eq!(t.find(1).unwrap(), &foo);
//! assert_eq!(t.find(42).unwrap(), &bar);
//! ~~~

//#[link(name="btree", vers="0.1pre",
//       uuid="136fafb0-e4e0-11e2-a28f-0800200c9a66")];

use std::util;

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
    fn is_empty(&self) -> bool { self.nodes.head().is_none() }
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
                    self.used -= 1;
                }
            }
        }

        assert!(self.used == 0);
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
        let mut current = self;

        loop {
            let pos = current.nodes.iter().position(|x| {
                match *x {
                    // TODO: If x == None, break
                    None => false,
                    Some(TreeNode { key: ref k, value: _ }) => key <= *k,
                    Some(TreeLeaf { key: ref k, value: _ }) => key == *k,
                }
            });

            if pos.is_none() {
                return None;
            }

            match current.nodes[pos.unwrap()] {
                Some(TreeNode { key: _, value: ref tree }) => {
                    current = &'a **tree;
                }
                Some(TreeLeaf { key: _, value: ref value }) => {
                    return Some(value);
                }
                None => return None
            }
        }
    }

    /// Insert a key-value pair into the b-tree. Return true if the key did not
    /// already exist in the map.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        insert(self, key, value)
    }
}

fn insert<K: Eq + Ord, V>(tree: &mut BTree<K, V>, key: K, value: V) -> bool {
    let capacity = tree.nodes.len();

    // If the node contains fewer than the maximum legal number of
    // elements, then there is room for the new element.
    if tree.used < capacity {
        // Determine the position for the new node based on the existing keys.
        // If None is found, use that position. If the key of the new node is
        // least than or equal to the current node, use that position.
        let pos = tree.nodes.iter().position(|x| {
            match *x {
                None => true,
                Some(TreeNode { key: ref k, value: _ }) |
                Some(TreeLeaf { key: ref k, value: _ }) => key <= *k,
            }
        });

        let node = Some(TreeLeaf { key: key, value: value });

        match pos {
            // Since tree.used < capacity, there is at least one None in the
            // node list. Therefore, pos == None is not possible.
            None => fail!("unreachable path"),
            Some(pos) => {
                return insert_node(tree, pos, node);
            }
        }
    }
    // Otherwise the node is full, evenly split it into two nodes.
    else {
        fail!("node is full. Not implemented yet");
        // 1. A single median is chosen from among the leaf't elements and
        // the new element.

        // 2. Values less than the median are put in the new left node and
        // values greater than the median are put in the new right node,
        // with the median acting as a separation value.

        // 3. The separation value is inserted in the node't parent, which
        // may cause it to be split, and so on. If the node is the root,
        // create a new root above this node.
    }
}

fn insert_node<K: Eq + Ord, V>(tree: &mut BTree<K, V>, pos: uint,
                               node: Option<TreeItem<K, V>>) -> bool {
    debug!("insert node %? at pos: %u", node, pos);

    let mut j = tree.used;
    let new_key = match tree.nodes[pos] {
        None => true,
        Some(_) => {
            debug!("swap nodes: %u", j - pos);

            while j > pos {
                debug!("move %u (%?) to %u (%?) ", j - 1, tree.nodes[j - 1], j,
                    tree.nodes[j]);
                tree.nodes.swap(j, j - 1);
                j -= 1;
            }

            match node {
                Some(TreeLeaf { key: ref key, value: _ }) |
                Some(TreeNode { key: ref key, value: _ }) => {
                    match tree.nodes[pos] {
                        Some(TreeLeaf { key: ref k, value: _ }) |
                        Some(TreeNode { key: ref k, value: _ }) =>
                            *k != *key,
                        None => true,
                    }
                }
                None => fail!("unreachable path"),
            }
        }
    };

    util::replace(&mut tree.nodes[pos], node);

    tree.used += 1;

    debug!("tree nodes: %?", tree.nodes);

    new_key
}

#[cfg(test)]
mod test_btree {

    use super::*;

    #[test]
    fn test_basic_insert() {
        let foo = "foo";
        let bar = "bar";
        let baz = "baz";

        let mut t = BTree::new();
        assert!(t.is_empty());

        assert!(t.insert(42, bar));
        assert!(!t.is_empty());

        assert!(t.insert(3, baz));
        assert!(!t.is_empty());

        assert!(t.insert(1, foo));
        assert!(!t.is_empty());

        assert_eq!(t.find(1).unwrap(), &foo);
        assert_eq!(t.find(3).unwrap(), &baz);
        assert_eq!(t.find(42).unwrap(), &bar);
    }

    #[test]
    fn test_basic_len() {
        let foo = "foo";
        let bar = "bar";

        let mut t = BTree::new();
        assert_eq!(t.len(), 0);

        assert!(t.insert(1, foo));
        assert_eq!(t.len(), 1);

        assert!(t.insert(42, bar));
        assert_eq!(t.len(), 2);

        t.clear();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn test_clear() {
        let foo = "foo";
        let bar = "bar";

        let mut t = BTree::new();
        assert!(t.is_empty());

        assert!(t.insert(1, foo));
        assert!(!t.is_empty());

        assert!(t.insert(42, bar));
        assert!(!t.is_empty());

        t.clear();
        assert!(t.is_empty());

        assert_eq!(t.find(1), None);
        assert_eq!(t.find(42), None);
    }
}
