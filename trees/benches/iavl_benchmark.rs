use std::{fmt, fs};

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

fn iavl_get_benchmark(c: &mut Criterion, all_params: &Vec<Params>) {
    let mut group = c.benchmark_group("iavl-get");
    for params in all_params {
        let (tree, _) = prepare_tree(&params);
        group.bench_with_input(
            BenchmarkId::from_parameter(&params),
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
    }
    group.finish();
}

fn iavl_update_benchmark(c: &mut Criterion, all_params: &Vec<Params>) {
    let mut group = c.benchmark_group("iavl-update");
    for params in all_params {
        let (mut tree, keys) = prepare_tree(&params);
        group.bench_with_input(
            BenchmarkId::from_parameter(&params),
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
    }
    group.finish();
}

fn iavl_range_benchmark(c: &mut Criterion, all_params: &Vec<Params>) {
    let mut group = c.benchmark_group("iavl-range");
    for params in all_params {
        let (tree, _) = prepare_tree(&params);
        group.bench_with_input(BenchmarkId::from_parameter(&params), &params, |b, _| {
            b.iter(|| {
                let _range: Vec<(Vec<u8>, Vec<u8>)> = tree.range(..).collect();
            })
        });
    }
    group.finish();
}

pub fn iavl_benchmark(c: &mut Criterion) {
    let all_params = vec![
        Params {
            init_size: 100_000,
            key_length: 16,
            data_length: 40,
        },
        Params {
            init_size: 1_000_000,
            key_length: 16,
            data_length: 40,
        },
    ];

    iavl_get_benchmark(c, &all_params);
    iavl_update_benchmark(c, &all_params);
    iavl_range_benchmark(c, &all_params);
}

fn prepare_tree(params: &Params) -> (Tree<RocksDB>, Vec<Vec<u8>>) {
    // remove previous test DBs
    fs::remove_dir_all(DB_DIR).unwrap();
    fs::create_dir(DB_DIR).unwrap();

    let db = RocksDB::new(DB_DIR).unwrap();
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

// fn prepare_tree_v2(params: &Params) -> (trees::iavl::tree_v2::Tree<RocksDB>, Vec<Vec<u8>>) {
//     // remove previous test DBs
//     fs::remove_dir_all(DB_DIR).unwrap();
//     fs::create_dir(DB_DIR).unwrap();

//     let db = RocksDB::new(DB_DIR).unwrap();
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
