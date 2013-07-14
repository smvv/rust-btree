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
pub static BTREE_DEGREE : uint = 5;

pub struct BTree<K, V> {
    priv used: uint,
    priv keys: [Option<K>, ..BTREE_DEGREE - 1],
    priv nodes: [Option<TreeItem<K, V>>, ..BTREE_DEGREE],
}

pub enum TreeItem<K, V> {
    TreeNode { value: ~BTree<K, V> },
    TreeLeaf { value: V },
}

impl<K: Eq + Ord, V : Eq> BTree<K, V> {
    pub fn new() -> ~BTree<K, V> {
        // TODO: once https://github.com/mozilla/rust/issues/5244 is fixed,
        // use the following statement:
        //~BTree { used: 0, key: [None, ..BTREE_DEGREE - 1],
        //         nodes: [None, ..BTREE_DEGREE] }
        ~BTree { used: 0, keys: [None, None, None, None],
                 nodes: [None, None, None, None, None] }
    }

    /// Return the number of keys that can be stored in the b-tree node.
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
    /// already exist in the tree.
    pub fn insert(&mut self, key: K, value: V) -> bool {
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

fn insert<'r, K: Eq + Ord, V : Eq>(tree: &'r mut BTree<K, V>, key: K,
                                   value: V) -> (&'r mut BTree<K, V>, bool) {
    let tree = find_node(tree, &key);

    match tree.nodes[0] {
        Some(TreeLeaf { value: _ }) | None => {}
        Some(TreeNode { value: _ }) =>
            fail!("unreachable path: tree.nodes[0] == TreeNode"),
    }

    // Check if the key already exists
    let pos = tree.keys.iter().position(|x| {
        match *x {
            Some(ref k) => key == *k,
            None => false,
        }
    });

    // If the key already exists, replace the leaf.
    if pos.is_some() {
        util::swap(&mut tree.nodes[pos.unwrap()],
                   &mut Some(TreeLeaf { value: value }));
        (tree, false)
    }
    // If the node contains fewer than the maximum legal number of
    // elements, then there is room for the new element.
    else if tree.used < tree.capacity() {
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

        (tree, new_key)
    }
    // Otherwise the node is full, evenly split it into two nodes.
    else {
        //debug!("tree used: %?", tree.used);
        //debug!("tree keys: %?", tree.keys);
        //debug!("tree nodes: %?", tree.nodes);

        let mut left = BTree::new();
        let mut right = BTree::new();

        // 1. A single median is chosen from among the leaf's elements and the
        // new element.
        assert!(BTREE_DEGREE % 2 == 1);
        let median = (BTREE_DEGREE - 1) / 2;

        // Determine where the new element should be inserted.
        let pos = tree.keys.iter().position(|x| {
            match *x {
                Some(ref k) => key < *k,
                None => fail!("unreachable path"),
            }
        }).unwrap();

        // 2. Values less than the median are put in the new left node and
        // values greater than the median are put in the new right node,
        // with the median acting as a separation value.

        let mut l = 0;
        let mut r = 0;
        let mut i = 0;

        let len = tree.keys.len();

        while i < median {
            if i == pos {
                l += 1;
            }

            if i < median - 1 || pos > median {
                util::swap(&mut left.keys[l], &mut tree.keys[i]);
            }

            util::swap(&mut left.nodes[l], &mut tree.nodes[i]);

            l += 1;
            i += 1;
        }

        while i < len {
            if i == pos {
                r += 1;
            }

            if i > median + 1 || pos < median {
                util::swap(&mut right.keys[r], &mut tree.keys[i]);
            }

            util::swap(&mut right.nodes[r], &mut tree.nodes[i]);

            r += 1;
            i += 1;
        }

        if pos == median {
            fail!("todo");
        } else if pos < median {
            util::replace(&mut left.keys[pos], Some(key));
            util::replace(&mut left.nodes[pos],
                          Some(TreeLeaf { value: value }));
        } else {
            util::replace(&mut right.keys[r - median], Some(key));
            util::replace(&mut right.nodes[r - median],
                          Some(TreeLeaf { value: value }));
        }

        left.used = median;
        right.used = median;

        //debug!("======= after split =========");

        //debug!("left used: %?", left.used);
        //debug!("left keys: %?", left.keys);
        //debug!("left nodes: %?", left.nodes);

        //debug!("right used: %?", right.used);
        //debug!("right keys: %?", right.keys);
        //debug!("right nodes: %?", right.nodes);

        // 3. The separation value is inserted in the node's parent, which
        // may cause it to be split, and so on. If the node is the root,
        // create a new root above this node.

        tree.used = 1;
        tree.keys.swap(0, median - 1);

        tree.nodes[0] = Some(TreeNode { value: left });
        tree.nodes[1] = Some(TreeNode { value: right });

        //debug!("tree used: %?", tree.used);
        //debug!("tree keys: %?", tree.keys);
        //debug!("tree nodes: %?", tree.nodes);

        (tree, true)
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

impl<K: Eq, V: Eq> Eq for BTree<K, V> {
    #[inline]
    fn eq(&self, other: &BTree<K, V>) -> bool {
        self.used == other.used
            && self.keys == other.keys
            && self.nodes == other.nodes
    }

    #[inline]
    fn ne(&self, other: &BTree<K, V>) -> bool { !(*self).eq(other) }
}

impl<K: Eq, V: Eq> Eq for TreeItem<K, V> {
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

#[cfg(test)]
mod test_btree {

    use super::*;
    use std::util;
    use std::rand;
    use std::rand::RngUtil;

    fn tree<K, V>(keys: [Option<K>, ..BTREE_DEGREE - 1],
                  nodes: [Option<TreeItem<K, V>>, ..BTREE_DEGREE])
        -> ~BTree<K, V> {
        ~BTree { used: keys.iter().filter(|x| x.is_some()).len_(),
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
                    fail!(fmt!("list[%u] = %? is not in use as used[%u] = %?",
                               i, list[i], i, used[i]));
                }

                i += 1;
            }

            let len = list.len();

            while i < len {
                if list[i].is_some() {
                    fail!(fmt!("list[%u] = %? is used but it should be unused",
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

    /*
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

    #[test]
    fn test_insert_split_basic() {
        let mut t = tree([Some(6), Some(17), Some(21), Some(26)],
                         [leaf(6), leaf(17), leaf(21), leaf(26), None]);

        assert!(t.insert(9, 9));

        assert_eq!(t.used, 1);

        check_values(t.keys, [Some(17)]);
        check_used(t.nodes, [true, true]);

        let l = get_node(t, 0);
        check_values(l.keys, [Some(6), Some(9)]);
        check_values(l.nodes, [leaf(6), leaf(9), leaf(17)]);

        let r = get_node(t, 1);
        check_values(r.keys, [Some(21), Some(26)]);
        check_values(r.nodes, [leaf(21), leaf(26)]);
    }
    */

    #[test]
    fn test_insert_split_add_root() {
        let l = tree([Some(4), Some(5), Some(6), Some(9)],
                     [leaf(4), leaf(5), leaf(6), leaf(9), None]);

        let r = tree([Some(21), Some(26), Some(33), Some(36)],
                     [leaf(21), leaf(26), leaf(33), leaf(36), None]);


        let mut t = tree([Some(17), None, None, None],
                         [node(l), node(r), None, None, None]);

        println(fmt!("== tree: == \n%s", t.to_str()));

        assert!(t.insert(18, 18));

        println(fmt!("== tree: == \n%s", t.to_str()));

        assert_eq!(t.used, 2);

        check_values(t.keys, [Some(17), Some(26)]);
        check_used(t.nodes, [true, true, true]);

        let l = get_node(&*t, 0);
        check_values(l.keys, [Some(6), Some(9)]);
        check_values(l.nodes, [leaf(6), leaf(9), leaf(17)]);

        let m = get_node(&*t, 1);
        check_values(m.keys, [Some(21), Some(26)]);
        check_values(m.nodes, [leaf(21), leaf(26)]);

        let r = get_node(&*t, 2);
        check_values(r.keys, [Some(21), Some(26)]);
        check_values(r.nodes, [leaf(21), leaf(26)]);
    }

    //#[test]
    //fn test_insert_split_random() {
    //    let mut t = BTree::new();
    //    let mut rng = rand::IsaacRng::new_seeded([42u8]);
    //    let mut keys = ~[];

    //    for 1000.times {
    //        let i = rng.gen_uint_range(0, 1000);
    //        keys.push(i);
    //        debug!("keys: %?", keys);
    //        assert!(t.insert(i, i));
    //        println(fmt!("== tree: == \n%s", t.to_str()));
    //        assert_eq!(t.find(i).unwrap(), &i);
    //    }

    //    debug!("keys: %?", keys);

    //    println(fmt!("== tree: == \n%s", t.to_str()));

    //    for keys.iter().advance |&i| {
    //        assert_eq!(t.find(i).unwrap(), &i);
    //    }
    //}
}
