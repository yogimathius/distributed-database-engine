use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nextdb_storage::{LSMTree, StorageConfig};
use tempfile::TempDir;

async fn create_lsm_tree() -> LSMTree {
    let temp_dir = TempDir::new().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
    config.wal_dir = temp_dir.path().join("wal").to_string_lossy().to_string();
    
    LSMTree::open(config).await.unwrap()
}

fn bench_put_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("lsm_put", |b| {
        b.iter(|| {
            rt.block_on(async {
                let lsm = create_lsm_tree().await;
                let key = black_box(b"benchmark_key".to_vec());
                let value = black_box(b"benchmark_value".to_vec());
                lsm.put(key, value).await.unwrap();
            });
        });
    });
}

fn bench_get_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("lsm_get", |b| {
        let lsm = rt.block_on(create_lsm_tree());
        let key = b"benchmark_key".to_vec();
        let value = b"benchmark_value".to_vec();
        rt.block_on(lsm.put(key.clone(), value)).unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                let result = lsm.get(black_box(&key)).await.unwrap();
                black_box(result);
            });
        });
    });
}

criterion_group!(benches, bench_put_operations, bench_get_operations);
criterion_main!(benches);