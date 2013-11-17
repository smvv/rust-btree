extern mod btree;
use btree::BTree;

use std::rand::{Rng, IsaacRng, SeedableRng};
use std::iter::range;

fn main() {
    let iterations = 1_000_000;

    let mut t = BTree::new();
    let mut rng = IsaacRng::new();
    rng.reseed([42u32]);

    let mut random_keys = ~[];
    for k in range(0, iterations) { random_keys.push(k); }
    rng.shuffle_mut(random_keys);

    let mut i = 0;

    while i < iterations {
        let key = random_keys[i];

        t.insert(key, key);

        i += 1;
    }

    for &k in random_keys.iter() {
        assert_eq!(t.find(k).unwrap(), &k);
    }
}
