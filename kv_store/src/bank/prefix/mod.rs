// use std::ops::Bound;

// pub mod immutable;
// pub mod mutable;
// pub mod range;

// /// Returns the KVStore Bound that would end an unbounded upper
// /// range query on a PrefixStore with the given prefix
// ///
// /// That is the smallest x such that, prefix + y < x for all y. If
// /// no such x exists (i.e. prefix = vec![255; N]; for some N) it returns Bound::Unbounded
// fn prefix_end_bound(mut prefix: Vec<u8>) -> Bound<Vec<u8>> {
//     loop {
//         let last = prefix.last_mut();

//         match last {
//             None => return Bound::Unbounded,
//             Some(last) => {
//                 if *last != 255 {
//                     *last += 1;
//                     return Bound::Excluded(prefix);
//                 }
//                 prefix.pop();
//             }
//         }
//     }
// }
