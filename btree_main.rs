extern mod btree;
use btree::BTree;

fn main() {
    let mut s = BTree::new();
    assert!(s.is_empty());
    assert!(s.insert(1, "foo"));
    assert!(s.insert(42, "bar"));
    assert!(!s.is_empty());
    s.clear();
    assert!(s.is_empty());
}
