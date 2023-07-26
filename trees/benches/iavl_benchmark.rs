use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use database::RocksDB;
use pprof::criterion::{Output, PProfProfiler};
use rand::{distributions::Standard, Rng};
use trees::iavl::Tree;

const DB_DIR: &str = "db";

#[derive(Debug)]
struct Params {
    init_size: usize,
    //block_size: usize,
    key_length: usize,
    data_length: usize,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn iavl_benchmark(c: &mut Criterion) {
    // remove previous test DBs
    fs::remove_dir_all(DB_DIR).unwrap();
    fs::create_dir(DB_DIR).unwrap();

    let params = Params {
        init_size: 100_000,
        //block_size: 100,
        key_length: 16,
        data_length: 40,
    };

    let (tree, _) = prepare_tree(
        &PathBuf::from(format!("{}{}", DB_DIR, "/iavl-get")),
        &params,
    );
    c.bench_with_input(
        BenchmarkId::new("iavl-get-miss", &params),
        &params,
        |b, params| {
            b.iter(|| {
                let key: Vec<u8> = rand::thread_rng()
                    .sample_iter(Standard)
                    .take(params.key_length)
                    .collect();
                tree.get(black_box(&key));
            })
        },
    );

    let (mut tree, keys) = prepare_tree(
        &PathBuf::from(format!("{}{}", DB_DIR, "/iavl-update")),
        &params,
    );
    c.bench_with_input(
        BenchmarkId::new("iavl-update", &params),
        &params,
        |b, params| {
            b.iter(|| {
                let key: &Vec<u8> = keys
                    .get(rand::thread_rng().gen_range(0..params.init_size))
                    .unwrap();

                let mut data: Vec<u8> = rand::thread_rng()
                    .sample_iter(Standard)
                    .take(params.data_length)
                    .collect();

                rand::thread_rng().fill(&mut data[..]);
                tree.set(black_box(key.clone()), black_box(data.to_vec()));
            })
        },
    );

    // let (tree_range, _) = prepare_tree(
    //     &PathBuf::from(format!("{}{}", DB_DIR, "/iavl_range")),
    //     &params,
    // );
    // c.bench_function("iavl-range", |b| {
    //     b.iter(|| {
    //         let _range: Vec<(Vec<u8>, Vec<u8>)> = tree_range.range(..).collect();
    //     })
    // });
}

fn prepare_tree(path: &Path, params: &Params) -> (Tree<RocksDB>, Vec<Vec<u8>>) {
    let db = RocksDB::new(path).unwrap();
    let mut tree = Tree::new(db, None).unwrap();
    let mut keys = Vec::with_capacity(params.init_size);

    for _ in 0..params.init_size {
        let key: Vec<u8> = rand::thread_rng()
            .sample_iter(Standard)
            .take(params.key_length)
            .collect();

        let data: Vec<u8> = rand::thread_rng()
            .sample_iter(Standard)
            .take(params.data_length)
            .collect();

        tree.set(key.clone(), data);
        keys.push(key);
    }

    tree.save_version().unwrap();

    (tree, keys)
}

// fn prepare_tree_v2(
//     path: &Path,
//     params: &Params,
// ) -> (trees::iavl::tree_v2::Tree<RocksDB>, Vec<Vec<u8>>) {
//     let db = RocksDB::new(path).unwrap();
//     let mut tree = trees::iavl::tree_v2::Tree::new(db, None).unwrap();
//     let mut keys = Vec::with_capacity(params.init_size);

//     for _ in 0..params.init_size {
//         let key: Vec<u8> = rand::thread_rng()
//             .sample_iter(Standard)
//             .take(params.key_length)
//             .collect();

//         let data: Vec<u8> = rand::thread_rng()
//             .sample_iter(Standard)
//             .take(params.data_length)
//             .collect();

//         tree.set(key.clone(), data);
//         keys.push(key);
//     }

//     tree.save_version().unwrap();

//     (tree, keys)
// }

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = iavl_benchmark
}

criterion_main!(benches);
