use anyhow::Result;
use async_trait::async_trait;
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use tokio::{runtime::Runtime, sync::mpsc};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use xmu_assistant_bot::abi::echo::Echo;
use xmu_assistant_bot::abi::logic_import::Message;
use xmu_assistant_bot::abi::message::event_message::{Group, SubTypeGroup};
use xmu_assistant_bot::abi::message::message_body::SegmentReceive;
use xmu_assistant_bot::abi::message::message_body::text::DataReceive;
use xmu_assistant_bot::abi::message::sender::Role;
use xmu_assistant_bot::abi::message::{MessageReceive, SenderGroup};
use xmu_assistant_bot::abi::message::{
    api::{Params as Request, data::ApiResponsePending}, // 导入 API 相关的类型和 Trait
};
use xmu_assistant_bot::abi::network::BotClient;
use xmu_assistant_bot::abi::router::context::Context;
use xmu_assistant_bot::abi::websocket::BotHandler;
use xmu_assistant_bot::logic::dispatch_all_handlers;

// 2. Mock 客户端 (T)
#[derive(Debug)]
struct MockClient;

#[async_trait]
impl BotClient for MockClient {
    // Mock call_api 行为，避免实际网络操作
    async fn call_api<R: Request + Send>(
        &self,
        _request: R,
        _echo: Echo,
    ) -> Result<ApiResponsePending<R::Response>> {
        // 返回一个 ApiResponsePending 实例
        Ok(ApiResponsePending::new(Echo::new()))
    }
}

#[async_trait]
impl BotHandler for MockClient {
    async fn on_connect(&self) {
        // do nothing
    }
    async fn on_disconnect(&self) {
        // do nothing
    }
    async fn init(
        &self,
        _event: mpsc::UnboundedSender<String>,
        _api: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        Ok(())
    }
    async fn handle_api(&self, _message: Utf8Bytes) {
        // This is a Mock, no-op
    }
    async fn handle_event(&self, _event: Utf8Bytes) {
        // This is a Mock, no-op
    }
}

// 3. 辅助函数
fn create_mock_context(text: &str) -> Context<MockClient, Message> {
    let client = Arc::new(MockClient);
    let message = Arc::new(Message::Group(Group {
        time: 1700000000,
        self_id: 1,
        sub_type: SubTypeGroup::Normal,
        message_id: 123456789,
        group_id: 111222333,
        user_id: 987654321,
        anonymous: None,
        raw_message: "".to_string() + text,
        font: 0,
        sender: SenderGroup {
            user_id: Some(987654321),
            nickname: Some("TestUser".to_string()),
            card: Some("TestCard".to_string()),
            sex: None,
            age: None,
            area: None,
            level: None,
            role: Role::Member,
            title: None,
        },
        message: MessageReceive::Array(vec![SegmentReceive::Text(DataReceive {
            text: text.to_string(),
        })]),
    }));
    Context::new(client, message)
}

// --- 基准测试 ---

fn bench_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // 1. 命中第一个 Handler (假设 echo::EchoHandler 匹配简单的文本)
    // 假设 "echo" 能匹配 EchoHandler (这是第一个注册的 Handler)
    c.bench_function("routing_hit_first", |b| {
        let ctx_template = create_mock_context("echo test");

        b.to_async(&rt).iter(|| {
            // 需要克隆上下文，因为 dispatch_all_handlers 消费了 Context
            let ctx = ctx_template.clone();
            async move {
                // 直接 await，确保本轮迭代完成时内存是安全的
                dispatch_all_handlers(ctx).await;
            }
        })
    });

    // 2. 遍历所有 Handler 但未命中
    c.bench_function("routing_miss_all", |b| {
        // 假设一个不会匹配任何 Handler 的长文本
        let ctx_template = create_mock_context("a long query text that wont match any handlers");

        b.to_async(&rt).iter(|| {
            let ctx = ctx_template.clone();
            async move {
                dispatch_all_handlers(ctx).await;
            }
        })
    });
}

criterion_group!(benches, bench_routing);
criterion_main!(benches);
