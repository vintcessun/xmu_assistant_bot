use anyhow::Result;
use async_trait::async_trait;
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use tokio::{runtime::Runtime, sync::mpsc};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use xmu_assistant_bot::abi::echo::Echo;
use xmu_assistant_bot::abi::message::{
    MessageType,
    Sender,
    Target,
    api::{Params as Request, data::ApiResponsePending}, // 导入 API 相关的类型和 Trait
};
use xmu_assistant_bot::abi::network::BotClient;
use xmu_assistant_bot::abi::router::context::Context;
use xmu_assistant_bot::abi::websocket::BotHandler;
use xmu_assistant_bot::logic::dispatch_all_handlers;

// --- Mock 类型定义 ---

// 1. Mock 消息类型 (M)
#[derive(Debug)]
struct MockMessage {
    text: String,
    sender: Sender,
    target: Target,
}

impl MessageType for MockMessage {
    fn get_type(&self) -> xmu_assistant_bot::abi::message::Type {
        xmu_assistant_bot::abi::message::Type::Message
    }
    fn get_target(&self) -> Target {
        self.target
    }
    fn get_sender(&self) -> Sender {
        self.sender.clone()
    }
    // 关键：返回路由需要匹配的文本
    fn get_text(&self) -> String {
        self.text.clone()
    }
}

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
fn create_mock_context(text: &str) -> Context<MockClient, MockMessage> {
    let client = Arc::new(MockClient);
    let message = Arc::new(MockMessage {
        text: text.to_string(),
        sender: Sender {
            user_id: Some(123),
            nickname: Some("bench_user".to_string()),
            card: None,
            role: None,
        },
        target: Target::Private(123),
    });
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
                tokio::spawn(dispatch_all_handlers(ctx));
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
                tokio::spawn(dispatch_all_handlers(ctx));
            }
        })
    });
}

criterion_group!(benches, bench_routing);
criterion_main!(benches);
