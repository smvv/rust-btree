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

use std::util;
use std::uint;

// The number of keys is chosen to vary between d and 2d, where d is the
// minimum number of keys and d+1 is the minimum degree (branching factor) of
// the tree. In this case, d = 2 which results in degree = 5.
static BTREE_DEGREE : uint = 5;

pub struct BTree<K, V> {
    priv used: uint,
    priv keys: [Option<K>, ..BTREE_DEGREE - 1],
    priv nodes: [Option<TreeItem<K, V>>, ..BTREE_DEGREE],
}

enum TreeItem<K, V> {
    TreeNode { value: ~BTree<K, V> },
    TreeLeaf { value: V },
}

impl<K: Eq + Ord + Copy, V> BTree<K, V> {
    pub fn new() -> ~BTree<K, V> {
        // TODO: once https://github.com/mozilla/rust/issues/5244 is fixed,
        // use the following statement:
        //~BTree { used: 0, key: [None, ..BTREE_DEGREE - 1],
        //         nodes: [None, ..BTREE_DEGREE] }
        ~BTree { used: 0, keys: [None, None, None, None],
                 nodes: [None, None, None, None, None] }
    }

    /// Return the number of nodes or values that can be stored in the b-tree
    /// node.
    pub fn capacity(&self) -> uint { BTREE_DEGREE - 1 }

    /// Return a reference to the value corresponding to the key.
    pub fn find<'a>(&'a self, key: K) -> Option<&'a V> {
        let mut current = self;

        loop {
            let pos = match current.nodes[0] {
                Some(TreeNode { value: _ }) => find_node_pos(current, &key),
                Some(TreeLeaf { value: _ }) => find_leaf_pos(current, &key),
                None => return None
            };

            match current.nodes[pos] {
                Some(TreeNode { value: ref tree }) => {
                    current = &'a **tree;
                }
                Some(TreeLeaf { value: ref value }) => {
                    return Some(value);
                }
                None => return None
            }
        }
    }

    /// Insert a key-value pair into the b-tree. Return true if the key did not
    /// already exist in the map.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        //debug!("insert key-value %?: %?", key, value);
        let (tree, new_key) = insert(self, key, value);
        new_key
    }
}

fn find_node_pos<K: Eq + Ord, V>(tree: &BTree<K, V>, key: &K) -> uint {
    for tree.keys.iter().enumerate().advance |(i, k)| {
        let k : &Option<K> = k;
        match *k {
            Some(ref k) => {
                if key <= k {
                    return i;
                }
            }
            None => return tree.used,
        };
    }

    tree.used
}

fn find_leaf_pos<K: Eq, V>(tree: &BTree<K, V>, key: &K) -> uint {
    for tree.keys.iter().enumerate().advance |(i, k)| {
        let k : &Option<K> = k;
        match *k {
            Some(ref k) => {
                if key == k {
                    return i;
                }
            }
            None => return tree.used,
        };
    }

    tree.used
}

fn find_node<'r, K: Eq + Ord, V>(tree: &'r mut BTree<K, V>,
                                 key: &K) -> &'r mut BTree<K, V> {
    // TODO make iterative if the borrow checker allows it
    match tree.nodes[0] {
        Some(TreeNode { value: _ }) => {
            let pos = find_node_pos(tree, key);

            match tree.nodes[pos] {
                Some(TreeNode { value: ref mut tree }) => {
                    return find_node(&mut **tree, key);
                }
                Some(TreeLeaf { value: _ }) |
                None => fail!("tree.nodes[pos] != TreeNode"),
            }
        }
        Some(TreeLeaf { value: _ }) |
        None => tree,
    }
}

fn insert<'r, K: Eq + Ord + Copy, V>(tree: &'r mut BTree<K, V>, key: K,
                                     value: V) -> (&'r mut BTree<K, V>, bool) {
    let tree = find_node(tree, &key);

    match tree.nodes[0] {
        Some(TreeLeaf { value: _ }) | None => {}
        Some(TreeNode { value: _ }) =>
            fail!("unreachable path: tree.nodes[0] == TreeNode"),
    }

    // If the node contains fewer than the maximum legal number of
    // elements, then there is room for the new element.
    if tree.used < tree.capacity() {
        // Determine the position for the new node based on the existing keys.
        // If None is found, use that position. If the key of the new node is
        // least than or equal to the tree node, use that position.
        let pos = tree.keys.iter().position(|x| {
            match *x {
                // TODO: If x == None, break
                None => true,
                Some(ref k) => key <= *k,
            }
        }).unwrap();

        let new_key = insert_key(tree, pos, Some(key));
        insert_node(tree, pos, Some(TreeLeaf { value: value }));
        tree.used += 1;

        //debug!("tree used: %?", tree.used);
        //debug!("tree keys: %?", tree.keys);
        //debug!("tree nodes: %?", tree.nodes);

        return (tree, new_key);
    }
    // Otherwise the node is full, evenly split it into two nodes.
    else {
        fail!("todo");
        // let capacity = tree.nodes.len();

        // // XXX: instead of calculating the median with the leaf's keys and the
        // // new element's key, the constant value `d' is used as median.

        // // TODO: instead of clearing `tree' and re-inserting the nodes, try to
        // // rebalance the tree using the algorithm described below. At this
        // // point, there is no `parent' visible to a tree instance.

        // // 1. A single median is chosen from among the leaf's elements and the
        // // new element. Insert new element into node list. Sort the list and
        // // divide the list over a left and right node.
        // assert!(BTREE_DEGREE % 2 == 1);
        // let median = (BTREE_DEGREE - 1) / 2;

        // //insert_node(tree, median, Some(TreeLeaf { key: key, value: value }));

        // // 2. Values less than the median are put in the new left node and
        // // values greater than the median are put in the new right node,
        // // with the median acting as a separation value.

        // let mut left = BTree::new();

        // for uint::range(0, median) |i| {
        //     util::swap(&mut left.nodes[i], &mut tree.nodes[i]);
        // }

        // let mut right = BTree::new();

        // for uint::range(0, median) |i| {
        //     util::swap(&mut right.nodes[i], &mut tree.nodes[median + 1 + i]);
        // }

        // left.used = median;
        // right.used = median;

        // // TODO left and right key
        // //let left_key = left.nodes[0];
        // //let right_key = right.nodes[0];

        // tree.clear();

        // insert_node(tree, 0, Some(TreeNode { value: left }));
        // insert_node(tree, 1, Some(TreeNode { value: right }));

        // // 3. The separation value is inserted in the node's parent, which
        // // may cause it to be split, and so on. If the node is the root,
        // // create a new root above this node.
        // let new_key = tree.insert(key, value);
        // return (tree, new_key);
    }
}

fn insert_key<K, V>(tree: &mut BTree<K, V>, pos: uint,
                    key: Option<K>) -> bool {
    let mut j = tree.used;

    let new_key = match tree.keys[pos] {
        Some(_) => {
            while j > pos {
                tree.keys.swap(j, j - 1);
                j -= 1;
            }

            true
        }
        None => true,
    };

    util::replace(&mut tree.keys[pos], key);

    new_key
}

fn insert_node<K, V>(tree: &mut BTree<K, V>, pos: uint,
                     node: Option<TreeItem<K, V>>) {
    let mut j = tree.used;

    while j > pos {
        tree.nodes.swap(j, j - 1);
        j -= 1;
    }

    util::replace(&mut tree.nodes[pos], node);
}

impl<K, V> Container for BTree<K, V> {
    /// Return the number of nodes or values in use in the b-tree node.
    #[inline]
    fn len(&self) -> uint { self.used }

    /// Return true if the b-tree node contains no nodes or values.
    #[inline]
    fn is_empty(&self) -> bool { self.nodes.head().is_none() }
}

impl<K, V> Mutable for BTree<K, V> {
    /// Clear the b-tree, removing all nodes.
    fn clear(&mut self) {
        for self.nodes.mut_iter().advance |node| {
            *node = None;
        }

        self.used = 0;
    }
}

impl<K: ToStr, V> ToStr for BTree<K, V> {
    fn to_str(&self) -> ~str { to_str(self, 0) }
}

fn to_str<K: ToStr, V>(tree: &BTree<K, V>, indent: uint) -> ~str {
    let buf : ~[~str] = tree.nodes.iter().enumerate().transform(|(i, x)| {
        if i < tree.used {
            let key = match tree.keys[i] {
                Some(ref key) => key,
                None => fail!("unreachable path"),
            };

            fmt!("%s%s", "\t".repeat(indent), match *x {
                Some(TreeNode { value: ref tree }) => {
                    ~"Node(key=" + key.to_str() + ")\n"
                    + to_str::<K, V>(&**tree, indent + 1)
                }
                Some(TreeLeaf { value: _ }) => {
                    ~"Leaf(key=" + key.to_str() + ")"
                }
                None => ~"None",
            })
        } else {
            fmt!("%s%s", "\t".repeat(indent), match *x {
                Some(TreeNode { value: ref tree }) => {
                    ~"Node(key=None)\n" + to_str::<K, V>(&**tree, indent + 1)
                }
                Some(TreeLeaf { value: _ }) => ~"Leaf(key=None)",
                None => ~"None",
            })
        }
    }).collect();

    buf.connect("\n")
}

#[cfg(test)]
mod test_btree {

    use super::*;
    use std::rand;
    use std::rand::RngUtil;

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

    //#[test]
    //fn test_insert_split_basic() {
    //    let mut t = BTree::new();
    //    let mut rng = rand::IsaacRng::new_seeded([42u8]);
    //    let mut keys = ~[];

    //    for 6.times {
    //        let i = rng.gen_uint_range(3, 42);
    //        keys.push(i);
    //        assert!(t.insert(i, i));
    //        assert_eq!(t.find(i).unwrap(), &i);
    //    }

    //    debug!("keys: %?", keys);
    //    debug!("tree: %?", t);

    //    println(fmt!("== tree: == \n%s", t.to_str()));

    //    for keys.iter().advance |&i| {
    //        assert_eq!(t.find(i).unwrap(), &i);
    //    }

    //    //assert_eq!(t.find(1).unwrap(), &foo);
    //    //assert_eq!(t.find(3).unwrap(), &baz);
    //    //assert_eq!(t.find(42).unwrap(), &bar);
    //}
}
