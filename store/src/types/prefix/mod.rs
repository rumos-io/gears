use std::ops::Bound;

pub mod immutable;
pub mod mutable;
pub mod range;

/// Returns the KVStore Bound that would end an unbounded upper
/// range query on a PrefixStore with the given prefix
///
/// That is the smallest x such that, prefix + y < x for all y. If
/// no such x exists (i.e. prefix = vec![255; N]; for some N) it returns Bound::Unbounded
fn prefix_end_bound(mut prefix: Vec<u8>) -> Bound<Vec<u8>> {
    loop {
        let last = prefix.last_mut();

        match last {
            None => return Bound::Unbounded,
            Some(last) => {
                if *last != 255 {
                    *last += 1;
                    return Bound::Excluded(prefix);
                }
                prefix.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use trees::MemDB;

    use crate::{types::kv::KVStore, QueryableKVStore, TransactionalKVStore};

    use super::*;

    #[test]
    fn prefix_store_range_works() {
        let db = MemDB::new();
        let mut store = KVStore::new(db, None).unwrap();
        store.set(vec![0], vec![1]);
        store.set(vec![0, 1], vec![2]);
        store.set(vec![0, 2], vec![3]);
        store.set(vec![1], vec![4]);
        store.set(vec![1, 1], vec![5]);
        store.set(vec![1, 2], vec![6]);
        store.set(vec![1, 3], vec![7]);
        store.set(vec![1, 4], vec![8]);
        store.set(vec![1, 5], vec![9]);
        store.set(vec![2], vec![10]);
        store.set(vec![2, 1], vec![11]);
        store.set(vec![2, 2], vec![12]);
        store.set(vec![2, 3], vec![13]);
        store.commit(); //TODO: this won't be needed once the KVStore iterator correctly incorporates cached values

        let prefix_store = store.prefix_store(vec![1]);

        // unbounded
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store.range(..).collect();
        let expected_pairs = vec![
            (vec![], vec![4]),
            (vec![1], vec![5]),
            (vec![2], vec![6]),
            (vec![3], vec![7]),
            (vec![4], vec![8]),
            (vec![5], vec![9]),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));

        // [,]
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store.range(vec![1]..=vec![3]).collect();
        let expected_pairs = vec![(vec![1], vec![5]), (vec![2], vec![6]), (vec![3], vec![7])];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));

        // (,)
        let start = vec![1];
        let stop = vec![3];
        let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = prefix_store
            .range((Bound::Excluded(start), Bound::Excluded(stop)))
            .collect();
        let expected_pairs = vec![(vec![2], vec![6])];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (e.0.clone(), e.1.clone());
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn prefix_end_bound_works() {
        let prefix = vec![1, 2, 3];
        let expected = vec![1, 2, 4];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![1, 2, 255];
        let expected = vec![1, 3];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![1, 255, 255];
        let expected = vec![2];

        assert!(matches!(
            prefix_end_bound(prefix),
            Bound::Excluded(x) if x == expected));

        let prefix = vec![255, 255, 255];

        assert!(matches!(prefix_end_bound(prefix), Bound::Unbounded));
    }

    // TODO: uncomment this test once merged range works
    // /// Tests whether kv range works with cached and persisted values
    // #[test]
    // fn kv_store_merged_range_works() {
    //     let db = MemDB::new();
    //     let mut store = KVStore::new(db, None).unwrap();

    //     // values in this group will be in the persistent store
    //     store.set(vec![1], vec![1]);
    //     store.set(vec![7], vec![13]); // shadowed by value in tx cache
    //     store.set(vec![10], vec![2]); // shadowed by value in block cache
    //     store.set(vec![14], vec![234]); // shadowed by value in block cache and tx cache
    //     store.commit();

    //     // values in this group will be in the block cache
    //     store.set(vec![2], vec![3]);
    //     store.set(vec![9], vec![4]); // shadowed by value in tx cache
    //     store.set(vec![10], vec![7]); // shadows a persisted value
    //     store.set(vec![14], vec![212]); // shadows a persisted value AND shadowed by value in tx cache
    //     store.write_then_clear_tx_cache();

    //     // values in this group will be in the tx cache
    //     store.set(vec![3], vec![5]);
    //     store.set(vec![8], vec![6]);
    //     store.set(vec![7], vec![5]); // shadows a persisted value
    //     store.set(vec![9], vec![6]); // shadows a block cache value
    //     store.set(vec![14], vec![212]); // shadows a persisted value which shadows a persisted value

    //     let start = vec![0];
    //     let stop = vec![20];
    //     let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = store
    //         .range((Bound::Excluded(start), Bound::Excluded(stop)))
    //         .collect();
    //     let expected_pairs = vec![
    //         (vec![1], vec![1]),
    //         (vec![2], vec![3]),
    //         (vec![3], vec![5]),
    //         (vec![7], vec![5]),
    //         (vec![8], vec![6]),
    //         (vec![9], vec![6]),
    //         (vec![10], vec![7]),
    //         (vec![14], vec![212]),
    //     ];

    //     assert_eq!(expected_pairs, got_pairs);
    // }
}
