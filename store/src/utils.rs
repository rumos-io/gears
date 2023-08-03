use std::iter::Peekable;

/// Favours a over b if keys are equal (so make a the cache)
pub struct MergedRange<'a, A, B>
where
    A: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
    B: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    a: Peekable<A>,
    b: Peekable<B>,
}

impl<'a, A, B> MergedRange<'a, A, B>
where
    A: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
    B: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    pub fn _merge(a: A, b: B) -> MergedRange<'a, A, B>
    where
        A: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
        B: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
    {
        MergedRange {
            a: a.peekable(),
            b: b.peekable(),
        }
    }
}

impl<'a, A, B> Iterator for MergedRange<'a, A, B>
where
    A: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
    B: Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    type Item = (&'a Vec<u8>, &'a Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let peek_a = self.a.peek();
        let peek_b = self.b.peek();

        match peek_a {
            Some(peek_a) => match peek_b {
                Some(peek_b) => {
                    // Both are valid.  Compare keys.
                    if peek_a.0 < peek_b.0 {
                        return self.a.next();
                    } else if peek_a.0 == peek_b.0 {
                        self.b.next(); // effectively skip this
                        return self.a.next();
                    } else {
                        return self.b.next();
                    }
                }
                None => return self.a.next(),
            },
            None => return self.b.next(),
        }
    }
}

// // TODO: are we assuming a and/or b are sorted? Does IAVL tree order items in range, BTreeMap does?
// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn merge_works() {
//         let a = vec![
//             (vec![1u8], vec![10u8]),
//             (vec![3], vec![11]),
//             (vec![5], vec![12]),
//         ]
//         .iter();
//         let b = vec![
//             (vec![2 as u8], vec![13 as u8]),
//             (vec![4], vec![14]),
//             (vec![5], vec![15]),
//         ]
//         .iter();

//         let got_pairs = MergedRange::merge(a, b).collect();

//         let expected_pairs = &vec![
//             (&vec![1u8], &vec![10u8]),
//             (&vec![2u8], &vec![13u8]),
//             (&vec![3u8], &vec![11u8]),
//             (&vec![4u8], &vec![14u8]),
//             (&vec![5u8], &vec![12u8]),
//         ];

//         assert_eq!(expected_pairs, got_pairs);
//     }

//     // This differs from the previous test in that iterator b reaches the duplicated value first
//     #[test]
//     fn merge_works_a_duplicates_b() {
//         let a = vec![
//             (vec![1], vec![10]),
//             (vec![3], vec![11]),
//             (vec![5], vec![12]),
//         ]
//         .into_iter();
//         let b = vec![(vec![2], vec![13]), (vec![5], vec![15])].into_iter();

//         let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = MergedRange::merge(a, b).collect();

//         let expected_pairs = vec![
//             (vec![1], vec![10]),
//             (vec![2], vec![13]),
//             (vec![3], vec![11]),
//             (vec![5], vec![12]),
//         ];

//         assert_eq!(expected_pairs, got_pairs);
//     }

//     // This differs from the previous test in that the duplicated value is in the middle of the range
//     #[test]
//     fn merge_works_mid_range_duplicate() {
//         let a = vec![
//             (vec![1], vec![10]),
//             (vec![3], vec![11]),
//             (vec![5], vec![12]),
//         ]
//         .into_iter();
//         let b = vec![
//             (vec![2], vec![13]),
//             (vec![3], vec![15]),
//             (vec![4], vec![14]),
//         ]
//         .into_iter();

//         let got_pairs: Vec<(Vec<u8>, Vec<u8>)> = MergedRange::merge(a, b).collect();

//         let expected_pairs = vec![
//             (vec![1], vec![10]),
//             (vec![2], vec![13]),
//             (vec![3], vec![11]),
//             (vec![4], vec![14]),
//             (vec![5], vec![12]),
//         ];

//         assert_eq!(expected_pairs, got_pairs);
//     }
// }
