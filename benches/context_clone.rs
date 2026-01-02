use anyhow::Result;
use async_trait::async_trait;
use criterion::{Criterion, criterion_group, criterion_main};
use serde::Serialize;
use std::fmt;
use std::hint::black_box;
use std::sync::Arc;
use tokio::sync::mpsc;

use xmu_assistant_bot::abi::echo::Echo;
use xmu_assistant_bot::abi::message::{MessageSend, MessageType, Sender, Target, Type, api};
use xmu_assistant_bot::abi::network::BotClient;
use xmu_assistant_bot::abi::router::context::Context;
use xmu_assistant_bot::abi::websocket::BotHandler;

#[derive(Debug)]
struct MockClient;

#[async_trait]
impl BotClient for MockClient {
    async fn call_api<T: api::Params + Serialize + fmt::Debug>(
        &self,
        _params: T,
        _echo: Echo,
    ) -> Result<api::ApiResponsePending<T::Response>> {
        anyhow::bail!("mock client")
    }
}

#[async_trait]
impl BotHandler for MockClient {
    async fn init(
        &self,
        _event: mpsc::UnboundedSender<String>,
        _api: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        Ok(())
    }
    async fn handle_api(&self, _message: String) {}
    async fn handle_event(&self, _event: String) {}
    async fn on_connect(&self) {}
    async fn on_disconnect(&self) {}
}

#[derive(Debug)]
struct MockMessage;

impl MessageType for MockMessage {
    fn get_target(&self) -> Target {
        Target::Group(114514)
    }
    fn get_type(&self) -> Type {
        Type::Message
    }
    fn get_text(&self) -> String {
        "hello".to_string()
    }
    fn get_sender(&self) -> Sender {
        Sender {
            nickname: Some("nickname".to_string()),
            user_id: Some(114514),
            card: Some("card".to_string()),
            role: None,
        }
    }
}

fn bench_context_extreme(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let ctx = Context::new(Arc::new(MockClient), Arc::new(MockMessage));

    let mut group = c.benchmark_group("Context_Cloning_Pressure");

    // 场景 A：最轻量级（路由分发阶段）
    group.bench_function("clone_empty", |b| {
        b.iter(|| {
            let _ = black_box(ctx.clone());
        });
    });

    // 场景 B：重负载（消息处理中后期）
    let mut heavy_ctx = ctx.clone();
    for _ in 0..8 {
        heavy_ctx.send_message_async(MessageSend::new_message().text("性能压力测试文本").build());
    }

    group.bench_function("clone_heavy_8_msgs", |b| {
        b.iter(|| {
            let _ = black_box(heavy_ctx.clone());
        });
    });

    // 场景 C: 克隆并写入
    group.bench_function("clone_and_write", |b| {
        b.iter(|| {
            let mut local_ctx = black_box(ctx.clone()); // 41ns
            // 模拟 Handler 逻辑
            local_ctx.send_message_async(MessageSend::new_message().text("handler_msg1").build()); // 这里会发生真正的 malloc 和数据写入
            local_ctx.send_message_async(MessageSend::new_message().text("handler_msg2").build());
            local_ctx.send_message_async(MessageSend::new_message().text("handler_msg3").build());
            local_ctx.send_message_async(MessageSend::new_message().text("handler_msg4").build());
        });
    });

    group.bench_function("concurrent_handler_dispatch_x10", |b| {
        b.to_async(&rt).iter(|| {
            let base_ctx = ctx.clone();

            // 模拟 10 个 Handler 并发运行
            let mut tasks = vec![];
            for i in 0..10 {
                let mut local_ctx = base_ctx.clone(); // 24字节的高效克隆
                tasks.push(tokio::spawn(async move {
                    // 模拟 Handler 业务：写入不同类型的消息
                    // 这里的写入会触发 Box 分配
                    local_ctx.send_message_async(
                        MessageSend::new_message()
                            .text(format!("handler {} response", i))
                            .build(),
                    );
                    black_box(local_ctx);
                }));
            }

            // 等待所有 Handler 处理完成
            async move {
                for task in tasks {
                    let _ = task.await;
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_context_extreme);
criterion_main!(benches);
