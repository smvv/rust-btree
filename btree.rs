//! A B-tree is a tree data structure that keeps data sorted and allows
//! searches, sequential access, insertions, deletions in logarithmic time.
//! B-trees are different from binary search trees because a b-tree node can
//! have more than two children (also known as the *degree* of a b-tree).
//!
//! Basic example:
//!
//! ~~~ rust
//! use std::btree::BTree;
//!
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

#[feature(struct_variant)];
#[feature(globs)];

#[link(name="btree")];

extern mod extra;

use std::util;

#[cfg(test)]
use std::rand::{Rng, IsaacRng, SeedableRng};
#[cfg(test)]
use std::iter::range;
#[cfg(test)]
use extra::test::BenchHarness;

/// The number of keys a node can contain is between a lower and upper bound.
/// Every node other than the root must have at least `t - 1` keys and `t`
/// children. Every node can contain at most `2t - 1` keys and `2t` children.
/// The fixed integer `t` (where `t >= 2`) is called the *minimum degree* of
/// the B-tree.
pub static BTREE_MIN_DEGREE : uint = 20;
//pub static BTREE_MIN_DEGREE : uint = 2;
pub static BTREE_KEYS_LBOUND : uint = BTREE_MIN_DEGREE - 1;
pub static BTREE_KEYS_UBOUND : uint = 2 * BTREE_MIN_DEGREE - 1;

pub struct BTree<K, V> {
    priv used: uint,
    priv keys: [Option<K>, ..BTREE_KEYS_UBOUND],
    priv nodes: [Option<TreeItem<K, V>>, ..BTREE_KEYS_UBOUND + 1],
}

pub enum TreeItem<K, V> {
    TreeNode { value: ~BTree<K, V> },
    TreeLeaf { value: V },
}

impl<K: Num + Ord, V : Eq> BTree<K, V> {
    pub fn new() -> ~BTree<K, V> {
        // TODO: once https://github.com/mozilla/rust/issues/5244 is fixed,
        // use the following statement:
        //~BTree { used: 0, key: [None, ..BTREE_KEYS_UBOUND],
        //         nodes: [None, ..BTREE_KEYS_UBOUND + 1] }

        // NB for executing the commented tests below, use this statement:
        //~BTree { used: 0, keys: [None, None, None],
        //         nodes: [None, None, None, None] }

        ~BTree { used: 0, keys: [
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None,
            ],
            nodes: [
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None,
            ]
        }
    }

    /// Return the number of keys that can be stored in the b-tree node.
    #[inline]
    pub fn capacity(&self) -> uint { BTREE_KEYS_UBOUND }

    /// Return a reference to the value corresponding to the key.
    pub fn find<'a>(&'a self, key: K) -> Option<&'a V> {
        let mut current = self;

        loop {
            let pos = match current.nodes[0] {
                Some(TreeNode { value: _ }) |
                Some(TreeLeaf { value: _ }) => find_node_pos(current, &key),
                None => return None
            };

            match current.nodes[pos] {
                Some(TreeNode { value: ref tree }) => {
                    current = &'a **tree;
                }
                Some(TreeLeaf { value: ref value }) => {
                    // If the leaf's key equals the key to be found, return the
                    // value. If the leaf is the most right leaf, also return
                    // the value, because there is no corresponding key in the
                    // node (the key is stored in one of the parent nodes).
                    return if pos == current.used
                              || current.keys[pos].get_ref() == &key {
                        Some(value)
                    } else {
                        None
                    }
                }
                None => return None
            }
        }
    }

    /// Insert a key-value pair into the b-tree. Return true if the key did not
    /// already exist in the tree.
    ///
    /// TODO: return true if the key did not already exist. Determine if the
    /// key is new is not supported at the moment.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        if self.used == self.capacity() {
            let mut child = BTree::new();

            let mut i = 0;

            while i < BTREE_KEYS_UBOUND + 1 {
                util::swap(&mut self.nodes[i], &mut child.nodes[i]);
                i += 1;
            }

            i = 0;

            while i < BTREE_KEYS_UBOUND {
                util::swap(&mut self.keys[i], &mut child.keys[i]);
                i += 1;
            }

            util::replace(&mut self.nodes[0], Some(TreeNode { value: child }));

            self.used = 0;

            split_child(self, 0);
        }

        insert_non_full(self, key, value)
    }
}

fn find_node_pos<K: Num + Ord, V>(tree: &BTree<K, V>, key: &K) -> uint {
    // NB Find the position using binary search on the keys in this node. The
    // following code performs the binary search, but it results in slower
    // run-time. Binary search on the keys should be faster than linear search,
    // but perhaps cache misses explain why the binary search performs poor.
    /*
    if tree.used == 0 {
        return 0;
    }

    let mut l = 0;
    let mut u = tree.used - 1;

    loop {
        if l == u {
            if key <= tree.keys[l].get_ref() {
                return l;
            }
            else {
                return l + 1;
            }
        }

        let i = (l + u) / 2;

        let cmp_key = tree.keys[i].get_ref();

        if key < cmp_key {
            if i == 0 {
                return 0;
            }

            u = if i - 1 < l { l } else { i - 1 };
        } else if key > cmp_key {
            l = i + 1;
        } else {
            return i;
        }
    }
    */

    // Find the position using linear search on the keys in this node.
    for (i, k) in tree.keys.iter().enumerate() {
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

fn find_node<'r, K: Num + Ord, V>(tree: &'r BTree<K, V>,
                                  key: &K) -> &'r BTree<K, V> {
    // TODO make iterative if the borrow checker allows it
    //match tree.nodes[0] {
    //    Some(TreeNode { value: _ }) => {
    //        let pos = find_node_pos(tree, key);

    //        match tree.nodes[pos] {
    //            Some(TreeNode { value: ref mut tree }) => {
    //                return find_node(&mut **tree, key);
    //            }
    //            Some(TreeLeaf { value: _ }) |
    //            None => fail!("tree.nodes[pos] != TreeNode"),
    //        }
    //    }
    //    Some(TreeLeaf { value: _ }) |
    //    None => tree,
    //}
    let mut current = tree;

    loop {
        match current.nodes[0] {
            Some(TreeNode { value: _ }) => {
                let pos = find_node_pos(tree, key);

                match tree.nodes[pos] {
                    Some(TreeNode { value: ref tree }) => {
                        //return find_node(&mut **tree, key);
                        current = &**tree;
                    }
                    Some(TreeLeaf { value: _ }) |
                    None => fail!("tree.nodes[pos] != TreeNode"),
                }
            }
            Some(TreeLeaf { value: _ }) |
            None => return tree,
        }
    }
}

fn split_child<K: Num + Ord, V: Eq>(tree: &mut BTree<K, V>, pos: uint) {
    let t = BTREE_MIN_DEGREE;

    // Make a free slot in the parent node for the to-be-inserted key.
    // Move the median key from the left node to the parent node. The median
    // key separates the left and right node.
    let mut i = tree.used;

    while i > pos {
        tree.nodes.swap(i, i + 1);
        tree.keys.swap(i - 1, i);
        i -= 1;
    }

    let right = match tree.nodes[pos] {
        Some(TreeNode { value: ref mut left }) => {
            let mut right = BTree::new();

            let mut i = 0;

            // Move the larger `t - 1' keys from the left to the right node.
            while i < t - 1 {
                util::swap(&mut right.keys[i], &mut left.keys[i + t]);
                i += 1;
            }

            i = 0;

            // Move the larger `t' nodes from the left to the right node.
            while i < t {
                util::swap(&mut right.nodes[i], &mut left.nodes[i + t]);
                i += 1;
            }

            util::swap(&mut tree.keys[pos], &mut left.keys[t - 1]);

            left.used = t - 1;
            right.used = t - 1;

            right
        }
        _ => fail!("unreachable path: tree.nodes[pos] should be a TreeNode"),
    };

    // Insert the new right node into the parent
    tree.nodes[pos + 1] = Some(TreeNode { value: right });
    tree.used += 1;
}

fn is_leaf<K, V>(tree: &mut BTree<K, V>) -> bool {
    match tree.nodes[0] {
        Some(TreeLeaf { value: _ }) => true,
        Some(TreeNode { value: _ }) | None => false,
    }
}

fn insert_non_full<K: Num + Ord, V: Eq>(tree: &mut BTree<K, V>, key: K,
                                        value: V) -> bool {
    if tree.used == 0 || is_leaf(tree) {
        let pos = find_node_pos(tree, &key);

        let new_key = tree.keys[pos].is_none()
                      || tree.keys[pos].get_ref() != &key;

        if new_key {
            let mut i = tree.used;

            if i > 0 {

                while i > pos {
                    tree.keys.swap(i - 1, i);
                    i -= 1;
                }

                i = tree.used + 1;

                while i > pos {
                    tree.nodes.swap(i - 1, i);
                    i -= 1;
                }
            }

            tree.used += 1;
        }

        util::replace(&mut tree.keys[pos], Some(key));
        util::replace(&mut tree.nodes[pos], Some(TreeLeaf { value: value }));

        new_key
    } else {
        let mut pos = find_node_pos(tree, &key);
        let mut split = false;

        match tree.nodes[pos] {
            Some(TreeNode { value: ref mut t }) => {
                if t.used == t.capacity() {
                    split = true;
                }
            }
            Some(TreeLeaf { value: _ }) => {
                fail!("unreachable path: leaf has same depth as a node");
            }
            None => fail!("todo")
        }

        if split {
            split_child(tree, pos);

            match tree.keys[pos] {
                Some(ref k) => {
                    if key > *k {
                        pos += 1;
                    }
                }
                None => {}
            }
        }

        match tree.nodes[pos] {
            Some(TreeNode { value: ref mut t }) => {
                insert_non_full(&mut **t, key, value)
            }
            Some(TreeLeaf { value: _ }) |
            None => fail!("unreachable path: leaf has same depth as a node")
        }
    }
}

impl<K, V> Container for BTree<K, V> {
    /// Return the number of keys in use in the b-tree node.
    #[inline]
    fn len(&self) -> uint { self.used }

    /// Return true if the b-tree node contains no nodes or values.
    #[inline]
    fn is_empty(&self) -> bool { self.nodes.head().is_none() }
}

impl<K, V> Mutable for BTree<K, V> {
    /// Clear the b-tree, removing all nodes.
    fn clear(&mut self) {
        for key in self.keys.mut_iter() {
            *key = None;
        }

        for node in self.nodes.mut_iter() {
            *node = None;
        }

        self.used = 0;
    }
}

impl<K: ToStr, V> ToStr for BTree<K, V> {
    fn to_str(&self) -> ~str { to_str(self, 0) }
}

fn to_str<K: ToStr, V>(tree: &BTree<K, V>, indent: uint) -> ~str {
    let buf : ~[~str] = tree.nodes.iter().enumerate().map(|(i, x)| {
        if i < tree.used {
            let key = match tree.keys[i] {
                Some(ref key) => key,
                None => fail!("unreachable path"),
            };

            "\t".repeat(indent) + match *x {
                Some(TreeNode { value: ref tree }) => {
                    ~"Node(key=" + key.to_str() + ")\n"
                    + to_str::<K, V>(&**tree, indent + 1)
                }
                Some(TreeLeaf { value: _ }) => {
                    ~"Leaf(key=" + key.to_str() + ")"
                }
                None => ~"None",
            }
        } else {
            "\t".repeat(indent) + match *x {
                Some(TreeNode { value: ref tree }) => {
                    ~"Node(key=None)\n" + to_str::<K, V>(&**tree, indent + 1)
                }
                Some(TreeLeaf { value: _ }) => ~"Leaf(key=None)",
                None => ~"None",
            }
        }
    }).collect();

    buf.connect("\n")
}

impl<K: Num, V: Eq> Eq for BTree<K, V> {
    #[inline]
    fn eq(&self, other: &BTree<K, V>) -> bool {
        self.used == other.used
            && self.keys == other.keys
            && self.nodes == other.nodes
    }

    #[inline]
    fn ne(&self, other: &BTree<K, V>) -> bool { !(*self).eq(other) }
}

impl<K: Num, V: Eq> Eq for TreeItem<K, V> {
    #[inline]
    fn eq(&self, other: &TreeItem<K, V>) -> bool {
        match *self {
            TreeNode { value: ref v1 } => {
                match *other {
                    TreeNode { value: ref v2 } => v1 == v2,
                    TreeLeaf { value: _ } => false,
                }
            }
            TreeLeaf { value: ref v1 } => {
                match *other {
                    TreeLeaf { value: ref v2 } => v1 == v2,
                    TreeNode { value: _ } => false,
                }
            }
        }
    }

    #[inline]
    fn ne(&self, other: &TreeItem<K, V>) -> bool { !(*self).eq(other) }
}

#[bench]
fn bench_insert_random(bh: &mut BenchHarness) {
    let iterations = 1000;

    let mut rng = IsaacRng::new();
    rng.reseed([42u32]);

    let mut random_keys = ~[];
    for k in range(0, iterations) { random_keys.push(k); }
    rng.shuffle_mut(random_keys);

    do bh.iter {
        let mut t = BTree::new();

        for &key in random_keys.iter() {
            t.insert(key, key);
        }
    }
}

#[cfg(test)]
mod test_btree {

    use super::*;
    use std::rand::{Rng, IsaacRng, SeedableRng};
    use std::iter::range;

    fn tree<K, V>(keys: [Option<K>, ..BTREE_KEYS_UBOUND],
                  nodes: [Option<TreeItem<K, V>>, ..BTREE_KEYS_UBOUND + 1])
        -> ~BTree<K, V> {
        ~BTree { used: keys.iter().filter(|x| x.is_some()).len(),
            keys : keys, nodes: nodes }
    }

    fn node<K, V>(value: ~BTree<K, V>) -> Option<TreeItem<K, V>> {
        Some(TreeNode { value: value })
    }

    fn leaf<K, V>(value: V) -> Option<TreeItem<K, V>> {
        Some(TreeLeaf { value: value })
    }

    //macro_rules! check_values (
    //    ($list:expr, $values:expr) => {{
    fn check_values<T: Eq>(list: &[Option<T>], values: &[Option<T>]) {
            assert!(list.len() >= values.len());

            let mut i = 0;
            let len = values.len();

            while i < len {
                assert_eq!(&list[i], &values[i]);
                i += 1;
            }

            let len = list.len();

            while i < len {
                assert_eq!(&list[i], &None);
                i += 1;
            }
    }
    //    }}
    //)

    //macro_rules! check_used (
    //    ($list:expr, $used:expr) => {{
    fn check_used<T>(list: &[Option<T>], used: &[bool]) {
            assert!(list.len() >= used.len());

            let mut i = 0;
            let len = used.len();

            while i < len {
                if list[i].is_some() != used[i] {
                    fail!(format!("list[{}] = {:?} is not in use. used[{}] = {}",
                                  i, list[i], i, used[i]));
                }

                i += 1;
            }

            let len = list.len();

            while i < len {
                if list[i].is_some() {
                    fail!(format!("list[{}] = {:?} is used but should be unused",
                                  i, list[i]));
                }

                i += 1;
            }
    }
    //    }}
    //)

    fn get_node<'r, K, V>(tree: &'r BTree<K, V>, pos: uint)
        -> &'r BTree<K, V> {
        match tree.nodes[pos] {
            Some(TreeNode { value: ref v }) => &**v,
            Some(TreeLeaf { value: _ }) |
            None  => fail!("unreachable path"),
        }
    }

    #[test]
    fn test_basic_insert() {
        let foo = "foo";
        let bar = "bar";
        let baz = "baz";

        let mut t = BTree::new();
        assert!(t.is_empty());
        assert_eq!(t.used, 0);

        assert!(t.insert(42, bar));
        assert!(!t.is_empty());
        assert_eq!(t.used, 1);

        assert!(t.insert(3, baz));
        assert!(!t.is_empty());
        assert_eq!(t.used, 2);

        assert!(t.insert(1, foo));
        assert!(!t.is_empty());
        assert_eq!(t.used, 3);

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

    /*
    // NB The following test will only work when BTREE_MIN_DEGREE = 2;
    #[test]
    fn test_insert_split_root() {
        assert_eq!(BTREE_MIN_DEGREE, 2);
        let mut t = tree([Some(4), Some(5), Some(6)],
                         [leaf(4), leaf(5), leaf(6), None]);

        assert!(t.insert(10, 10));

        assert_eq!(t.used, 1);

        check_values(t.keys, [Some(5)]);
        check_used(t.nodes, [true, true]);

        let l = get_node(&*t, 0);
        check_values(l.keys, [Some(4)]);
        check_values(l.nodes, [leaf(4), leaf(5)]);

        let r = get_node(&*t, 1);
        check_values(r.keys, [Some(6), Some(10)]);
        check_values(r.nodes, [leaf(6), leaf(10)]);
    }

    // NB The following test will only work when BTREE_MIN_DEGREE = 2;
    #[test]
    fn test_insert_split_right_leaf() {
        assert_eq!(BTREE_MIN_DEGREE, 2);

        let l = tree([Some(4), None, None],
                     [leaf(4), leaf(5), None, None]);
        let r = tree([Some(6), Some(10), Some(17)],
                     [leaf(6), leaf(10), leaf(17), None]);
        let mut t = tree([Some(5), None, None],
                         [node(l), node(r), None, None]);

        assert!(t.insert(21, 21));

        assert_eq!(t.used, 2);

        check_values(t.keys, [Some(5), Some(10)]);
        check_used(t.nodes, [true, true, true]);

        let l = get_node(&*t, 0);
        check_values(l.keys, [Some(4)]);
        check_values(l.nodes, [leaf(4), leaf(5)]);

        let m = get_node(&*t, 1);
        check_values(m.keys, [Some(6)]);
        check_values(m.nodes, [leaf(6), leaf(10)]);

        let r = get_node(&*t, 2);
        check_values(r.keys, [Some(17), Some(21)]);
        check_values(r.nodes, [leaf(17), leaf(21)]);
    }

    // NB The following test will only work when BTREE_MIN_DEGREE = 2;
    #[test]
    fn test_insert_split_middle_leaf() {
        assert_eq!(BTREE_MIN_DEGREE, 2);

        let l = tree([Some(4), None, None],
                     [leaf(4), leaf(5), None, None]);
        let m = tree([Some(6), Some(7), Some(8)],
                     [leaf(6), leaf(7), leaf(8), leaf(10)]);
        let r = tree([Some(17), Some(21), None],
                     [leaf(17), leaf(21), None, None]);
        let mut t = tree([Some(5), Some(10), None],
                         [node(l), node(m), node(r), None]);

        assert!(t.insert(9, 9));

        assert_eq!(t.used, 3);

        check_values(t.keys, [Some(5), Some(7), Some(10)]);
        check_used(t.nodes, [true, true, true, true]);

        let t0 = get_node(&*t, 0);
        check_values(t0.keys, [Some(4)]);
        check_values(t0.nodes, [leaf(4), leaf(5)]);

        let t1 = get_node(&*t, 1);
        check_values(t1.keys, [Some(6)]);
        check_values(t1.nodes, [leaf(6), leaf(7)]);

        let t2 = get_node(&*t, 2);
        check_values(t2.keys, [Some(8), Some(9)]);
        check_values(t2.nodes, [leaf(8), leaf(9), leaf(10)]);

        let t3 = get_node(&*t, 3);
        check_values(t3.keys, [Some(17), Some(21)]);
        check_values(t3.nodes, [leaf(17), leaf(21)]);
    }
    */

    #[test]
    fn test_insert_split_random() {
        let iterations = 100000;

        let mut t = BTree::new();
        let mut rng = IsaacRng::new();
        rng.reseed([42u32]);

        let mut random_keys = ~[];
        for k in range(0, iterations) { random_keys.push(k); }
        rng.shuffle_mut(random_keys);

        for &key in random_keys.iter() {
            t.insert(key, key);
        }

        for &k in random_keys.iter() {
            assert_eq!(t.find(k).unwrap(), &k);
        }
    }
}
