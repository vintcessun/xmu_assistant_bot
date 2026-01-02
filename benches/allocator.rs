use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

// 确保引用路径正确
use xmu_assistant_bot::abi::echo::Echo;
use xmu_assistant_bot::abi::message::MessageSend;
use xmu_assistant_bot::abi::message::api::data::SendMsgResponse;
use xmu_assistant_bot::abi::message::api::params::ApiSend;
use xmu_assistant_bot::abi::message::api::{SendGroupForwardMessageParams, SendMsgData, Status};

fn bench_allocator(c: &mut Criterion) {
    // --- 准备阶段：不计入耗时 ---
    let echo_instance = Echo::new();

    let mock_data = SendMsgData { message_id: 123456 };
    let mock_response = SendMsgResponse {
        status: Status::Ok,
        retcode: 0,
        message: None,
        data: Some(mock_data),
        echo: echo_instance,
        wording: None,
        stream: None,
    };

    // 预先序列化好响应字符串，测试时只测解析速度
    let response_json = serde_json::to_string(&mock_response).unwrap();

    c.bench_function("message_serialization_pipeline_sync", |b| {
        b.iter(|| {
            // --- 核心逻辑开始 (纯同步) ---

            // 1. 构造消息 (Builder 模式通常涉及多次小内存分配)
            let msg = SendGroupForwardMessageParams::new(
                0,
                vec![
                    MessageSend::new_message().text("测试消息内容").build(),
                    MessageSend::new_message().face("123").build(),
                ],
            );

            // 2. 模拟发送序列化 (Mimalloc 分配 String 缓冲区)
            let api_send = ApiSend {
                action: "send_msg",
                params: msg,
                echo: Echo::new(),
            };

            let json_str = serde_json::to_string(&api_send).unwrap();
            black_box(json_str);

            // 3. 模拟接收反序列化 (Mimalloc 分配结构体内存)
            // 注意：这里直接引用外部预生成的 response_json
            let res: SendMsgResponse =
                serde_json::from_str(black_box(response_json.as_str())).unwrap();

            black_box(res);

            // --- 核心逻辑结束 ---
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = bench_allocator
}
criterion_main!(benches);
