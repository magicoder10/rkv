use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rand::Rng;
use tempfile::TempDir;
use kvs::{KvsEngine, KvStore, Sled};

fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.bench_function("kvs", |b| {
        b.iter_batched(
            || {
                let tmp_dir = TempDir::new().unwrap();
                (KvStore::open(tmp_dir.path()).unwrap(), tmp_dir)
            },
            |(mut store, _tmp_dir)| {
                for i in 1..(1 << 10) {
                    let key = format!("key-{}", i);
                    let value = format!("value-{}", i);
                    store.set(key, value).unwrap();
                }
            },
            BatchSize::SmallInput)
    });
    group.bench_function("sled", |b|{
        b.iter_batched(||{
            let tmp_dir = TempDir::new().unwrap();
            (Sled::open(tmp_dir.path()).unwrap(), tmp_dir)
        }, |(mut store, _tmp_dir)| {
            for i in 1..(1<<10) {
                let key = format!("key-{}", i);
                let value = format!("value-{}", i);
                store.set(key, value).unwrap();
            }
        },
        BatchSize::SmallInput)
    });
    group.finish()
}

fn get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_bench");
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("kvs_{}", i), i, |b ,i| {
            let tmp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(tmp_dir.path()).unwrap();
            for key in 1..1<<i {
                store.set(format!("key{}", key), "value".to_string()).unwrap()
            }

            let mut rng= rand::thread_rng();
            b.iter(||{
                store.get(format!("key{}", rng.gen_range(1..1<<i)))
                    .unwrap();
            });
        });
    }

    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("sled_{}", i), i, |b ,i| {
            let tmp_dir = TempDir::new().unwrap();
            let mut store = Sled::open(tmp_dir.path()).unwrap();
            for key in 1..1<<i {
                store.set(format!("key{}", key), "value".to_string()).unwrap()
            }

            let mut rng= rand::thread_rng();
            b.iter(||{
                store.get(format!("key{}", rng.gen_range(1..1<<i)))
                    .unwrap();
            });
        });
    }

    group.finish()
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);