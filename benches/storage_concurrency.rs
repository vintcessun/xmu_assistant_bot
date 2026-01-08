use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use tokio::runtime::Runtime;
use xmu_assistant_bot::api::storage::HotTable;

fn bench_storage_concurrency(c: &mut Criterion) {
    // 1. 创建异步运行时
    let rt = Runtime::new().unwrap();

    // 2. 关键修复：通过 rt.enter() 进入运行时上下文
    // 这会返回一个 guard，在 guard 存活期间，所有初始化代码都能找到 Tokio Reactor
    let _guard = rt.enter();

    // 3. 在上下文保护内初始化 HotTable
    // 假设你的 HotTable 实现了某些内部异步同步逻辑，现在它可以安全获取线程局部的 Runtime 句柄了
    let table = Arc::new(HotTable::<String, String>::new("bench_test"));

    c.bench_function("hottable_concurrent_read_write_x100", |b| {
        b.to_async(&rt).iter(|| {
            let table = table.clone();

            async move {
                let mut tasks = vec![];

                // 模拟 100 个并发协程同时操作存储
                for i in 0..100 {
                    let t = table.clone();
                    tasks.push(tokio::spawn(async move {
                        let key = format!("user_{}", i);
                        // 模拟读写混合：注意这里使用了 .into() 适配你定义的 Value 类型
                        t.insert(key.clone(), "session_data".to_string().into())
                            .expect("Insert failed during bench");
                        let _ = t.get(&key);
                    }));
                }

                // 等待所有并发任务结束
                for task in tasks {
                    let _ = task.await;
                }
            }
        });
    });
}

criterion_group!(benches, bench_storage_concurrency);
criterion_main!(benches);
