use super::tree::{Range as TreeRange, Tree};
use crate::merkle::EMPTY_HASH;
use std::ops::RangeBounds;

/// A versioned IAVL tree
#[derive(Debug, Clone, PartialEq)]
pub struct IAVLTreeStore {
    tree: Option<Tree>,
    version: u32,
}

impl IAVLTreeStore {
    pub fn new() -> IAVLTreeStore {
        IAVLTreeStore {
            tree: None,
            version: 0,
        }
    }

    pub fn save_version(&mut self) -> ([u8; 32], u32) {
        self.version += 1;
        (self.root_hash(), self.version)
    }

    pub fn root_hash(&self) -> [u8; 32] {
        match &self.tree {
            Some(root) => root.hash(),
            None => EMPTY_HASH,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        match &self.tree {
            Some(root) => root.get(key),
            None => None,
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        match &mut self.tree {
            Some(root) => root.set(key, value, self.version + 1),
            None => self.tree = Some(Tree::new(key, value, self.version + 1)),
        };
    }

    pub fn range<R>(&self, range: R) -> Range<R>
    where
        R: RangeBounds<Vec<u8>>,
    {
        match &self.tree {
            Some(tree) => Range {
                tree: Some(tree.range(range)),
            },
            None => Range { tree: None },
        }
    }
}

pub struct Range<'a, R: RangeBounds<Vec<u8>>> {
    tree: Option<TreeRange<'a, R>>,
}

impl<'a, T: RangeBounds<Vec<u8>>> Iterator for Range<'a, T> {
    type Item = (&'a Vec<u8>, &'a Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.tree {
            Some(range) => range.next(),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::ops::Bound;

    use super::*;

    #[test]
    fn repeated_set_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        let expected = [
            202, 52, 159, 10, 210, 166, 72, 207, 248, 190, 60, 114, 172, 147, 84, 27, 120, 202,
            189, 127, 230, 108, 58, 127, 251, 149, 9, 33, 87, 249, 158, 138,
        ];

        assert_eq!(expected, tree.root_hash());
    }

    #[test]
    fn save_version_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        tree.save_version();
        tree.save_version();
        tree.set(b"qwerty".to_vec(), b"312".to_vec());
        tree.set(b"-32".to_vec(), b"gamma".to_vec());
        tree.save_version();
        tree.set(b"alice".to_vec(), b"123".to_vec());
        tree.save_version();

        let expected = [
            37, 155, 233, 229, 243, 173, 29, 241, 235, 234, 85, 10, 36, 129, 53, 79, 77, 11, 29,
            118, 201, 233, 133, 60, 78, 187, 37, 81, 42, 96, 105, 150,
        ];

        assert_eq!(expected, tree.root_hash());
    }

    #[test]
    fn get_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());

        assert_eq!(tree.get(b"alice"), Some(&String::from("abc").into()));
        assert_eq!(tree.get(b"bob"), Some(&String::from("123").into()));
        assert_eq!(tree.get(b"c"), Some(&String::from("1").into()));
        assert_eq!(tree.get(b"q"), Some(&String::from("1").into()));
        assert_eq!(tree.get(b"house"), None);
    }

    #[test]
    fn scenario_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(vec![0, 117, 97, 116, 111, 109], vec![51, 52]);
        tree.set(
            vec![
                2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
                251, 251, 222, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 51, 52],
        );

        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();
        tree.save_version();

        tree.set(
            vec![
                2, 20, 59, 214, 51, 187, 112, 177, 248, 133, 197, 68, 36, 228, 124, 164, 14, 68,
                72, 143, 236, 46, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 49, 48],
        );
        tree.set(
            vec![
                2, 20, 129, 58, 194, 42, 97, 73, 22, 85, 226, 120, 106, 224, 209, 39, 214, 153, 11,
                251, 251, 222, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 2, 50, 51],
        );
        tree.set(
            vec![
                2, 20, 241, 130, 150, 118, 219, 87, 118, 130, 233, 68, 252, 52, 147, 212, 81, 182,
                127, 243, 226, 159, 117, 97, 116, 111, 109,
            ],
            vec![10, 5, 117, 97, 116, 111, 109, 18, 1, 49],
        );

        let expected = [
            34, 215, 64, 141, 118, 237, 192, 198, 47, 22, 34, 81, 0, 146, 145, 66, 182, 59, 101,
            145, 99, 187, 82, 49, 149, 36, 196, 63, 37, 42, 171, 124,
        ];

        let (hash, version) = tree.save_version();

        assert_eq!((expected, 8), (hash, version));
    }

    #[test]
    fn bounded_range_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"1".to_vec(), b"abc1".to_vec());
        tree.set(b"2".to_vec(), b"abc2".to_vec());
        tree.set(b"3".to_vec(), b"abc3".to_vec());
        tree.set(b"4".to_vec(), b"abc4".to_vec());
        tree.set(b"5".to_vec(), b"abc5".to_vec());
        tree.set(b"6".to_vec(), b"abc6".to_vec());
        tree.set(b"7".to_vec(), b"abc7".to_vec());

        // [,)
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(start..stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));

        // [,]
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(start..=stop).collect();
        let expected_pairs = vec![
            (b"3".to_vec(), b"abc3".to_vec()),
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
            (b"6".to_vec(), b"abc6".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));

        // (,)
        let start = b"3".to_vec();
        let stop = b"6".to_vec();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree
            .range((Bound::Excluded(start), Bound::Excluded(stop)))
            .collect();
        let expected_pairs = vec![
            (b"4".to_vec(), b"abc4".to_vec()),
            (b"5".to_vec(), b"abc5".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_unique_keys_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"bob".to_vec(), b"123".to_vec());
        tree.set(b"c".to_vec(), b"1".to_vec());
        tree.set(b"q".to_vec(), b"1".to_vec());
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![
            (b"alice".to_vec(), b"abc".to_vec()),
            (b"c".to_vec(), b"1".to_vec()),
            (b"q".to_vec(), b"1".to_vec()),
            (b"bob".to_vec(), b"123".to_vec()),
        ];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn full_range_duplicate_keys_works() {
        let mut tree = IAVLTreeStore::new();
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        tree.set(b"alice".to_vec(), b"abc".to_vec());
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(..).collect();

        let expected_pairs = vec![(b"alice".to_vec(), b"abc".to_vec())];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }

    #[test]
    fn empty_tree_range_works() {
        let tree = IAVLTreeStore::new();
        let got_pairs: Vec<(&Vec<u8>, &Vec<u8>)> = tree.range(..).collect();

        let expected_pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![];

        assert_eq!(expected_pairs.len(), got_pairs.len());
        assert!(expected_pairs.iter().all(|e| {
            let cmp = (&e.0, &e.1);
            got_pairs.contains(&cmp)
        }));
    }
}
