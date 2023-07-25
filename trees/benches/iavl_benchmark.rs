use criterion::{black_box, criterion_group, criterion_main, Criterion};
use database::MemDB;
use pprof::criterion::{Output, PProfProfiler};
use rand::{distributions::Standard, Rng};
use trees::iavl::Tree;

const KEY_LENGTH: usize = 16;
const DATA_LENGTH: usize = 40;
const INIT_SIZE: usize = 100_000;

pub fn criterion_benchmark(c: &mut Criterion) {
    let (tree_get, _) = prepare_tree(INIT_SIZE, KEY_LENGTH, DATA_LENGTH);
    let (mut tree_update, keys) = prepare_tree(INIT_SIZE, KEY_LENGTH, DATA_LENGTH);

    c.bench_function("iavl-get", |b| {
        b.iter(|| {
            let key: [u8; KEY_LENGTH] = rand::random();
            tree_get.get(black_box(&key));
        })
    });

    c.bench_function("iavl-update", |b| {
        b.iter(|| {
            let key: &Vec<u8> = keys
                .get(rand::thread_rng().gen_range(0..INIT_SIZE))
                .unwrap();
            let mut data = [0u8; DATA_LENGTH];
            rand::thread_rng().fill(&mut data[..]);
            tree_update.set(black_box(key.clone()), black_box(data.to_vec()));
        })
    });
}

fn prepare_tree(
    init_size: usize,
    key_length: usize,
    data_length: usize,
) -> (Tree<MemDB>, Vec<Vec<u8>>) {
    let db = MemDB::new();
    let mut tree = Tree::new(db, None).unwrap();
    let mut keys = Vec::with_capacity(init_size);

    for _ in 0..init_size {
        let key: Vec<u8> = rand::thread_rng()
            .sample_iter(Standard)
            .take(key_length)
            .collect();

        let data: Vec<u8> = rand::thread_rng()
            .sample_iter(Standard)
            .take(data_length)
            .collect();

        tree.set(key.clone(), data);
        keys.push(key);
    }

    tree.save_version().unwrap();

    (tree, keys)
}

// fn prepare_tree_v2(
//     init_size: usize,
//     key_length: usize,
//     data_length: usize,
// ) -> (trees::iavl::tree_v2::Tree<MemDB>, Vec<Vec<u8>>) {
//     let db = MemDB::new();
//     let mut tree = trees::iavl::tree_v2::Tree::new(db, None).unwrap();
//     let mut keys = Vec::with_capacity(init_size);

//     for _ in 0..init_size {
//         let key: Vec<u8> = rand::thread_rng()
//             .sample_iter(Standard)
//             .take(key_length)
//             .collect();

//         let data: Vec<u8> = rand::thread_rng()
//             .sample_iter(Standard)
//             .take(data_length)
//             .collect();

//         tree.set(key.clone(), data);
//         keys.push(key);
//     }

//     tree.save_version().unwrap();

//     (tree, keys)
// }

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

//criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
