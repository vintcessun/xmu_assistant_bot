use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

// 引入你的业务结构体
use xmu_assistant_bot::abi::echo::Echo;
use xmu_assistant_bot::abi::message::MessageSend;
use xmu_assistant_bot::abi::message::api::data::SendMsgResponse;
use xmu_assistant_bot::abi::message::api::params::ApiSend;
use xmu_assistant_bot::abi::message::api::{SendGroupForwardMessageParams, SendMsgData, Status};

fn bench_zero_alloc_pipeline(c: &mut Criterion) {
    // --- 【准备阶段】 ---
    // 1. 预分配一个足够大的缓冲区（1KB 足够放下大部分单条消息）
    // 这样在测试循环中，Vec 的 capacity 永远不需要增长，实现真正的“零分配”
    let mut buffer = Vec::with_capacity(1024);

    // 预准备反序列化用的数据
    let echo_instance = Echo::new();
    let mock_response = SendMsgResponse {
        status: Status::Ok,
        retcode: 0,
        message: None,
        data: Some(SendMsgData { message_id: 123456 }),
        echo: echo_instance,
        wording: None,
        stream: None,
    };
    let response_json = serde_json::to_string(&mock_response).unwrap();

    c.bench_function("zero_alloc_pipeline", |b| {
        b.iter(|| {
            // --- 【测试核心逻辑】 ---

            // A. 构造消息
            // 注意：builder 内部如果涉及 String::from，mimalloc 依然会处理分配
            // 但我们通过重用外部缓冲区，消除了最沉重的“序列化结果字符串”分配
            let msg = SendGroupForwardMessageParams::new(
                0,
                vec![
                    MessageSend::new_message().text("测试消息内容").build(),
                    MessageSend::new_message().face("123").build(),
                ],
            );

            let api_send = ApiSend {
                action: "send_msg",
                params: msg,
                echo: Echo::new(),
            };

            // B. 零分配序列化逻辑
            buffer.clear(); // 仅清空长度，保留 capacity (不释放堆内存)

            // 使用 to_writer 绕过中间 String 的分配，直接写入 Vec
            serde_json::to_writer(&mut buffer, &api_send).unwrap();

            // 确保结果被使用，防止编译器优化掉整个序列化过程
            black_box(&buffer);

            // C. 反序列化
            // 虽然 from_str 会创建结构体，但如果结构体字段包含 String，那部分仍有分配
            let res: SendMsgResponse =
                serde_json::from_str(black_box(response_json.as_str())).unwrap();

            black_box(res);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = bench_zero_alloc_pipeline
}
criterion_main!(benches);
