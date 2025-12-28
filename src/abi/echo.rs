use anyhow::Result;
use dashmap::{DashMap, DashSet};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time;
use tracing::debug;

static COUNTER: AtomicU64 = AtomicU64::new(1);
static TIMEOUT: Duration = Duration::from_secs(600);

lazy_static! {
    static ref ACTIVE_ECHOS: DashSet<u64> = DashSet::new();
    static ref RESPONSE_REGISTRY: DashMap<u64, oneshot::Sender<String>> = DashMap::new();
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Echo(u64);

impl Echo {
    pub fn new() -> Self {
        loop {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            if ACTIVE_ECHOS.insert(id) {
                return Self(id);
            }
        }
    }

    pub fn remove(val: Self) -> bool {
        ACTIVE_ECHOS.remove(&val.0).is_some()
    }
}

impl Serialize for Echo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Echo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EchoVisitor;

        impl<'v> de::Visitor<'v> for EchoVisitor {
            type Value = Echo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a u64 echo id")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<u64>()
                    .map(Echo)
                    .map_err(|_| de::Error::custom(format!("invalid echo format: {}", v)))
            }
        }

        deserializer.deserialize_str(EchoVisitor)
    }
}

pub fn echo_send_result(echo: &str, response: String) {
    if let Ok(echo_id) = echo.parse::<u64>()
        && let Some(entry) = RESPONSE_REGISTRY.remove(&echo_id)
    {
        let sender = entry.1;
        let _ = sender.send(response);
    }
}

pub struct EchoPending {
    echo: Echo,
    receiver: oneshot::Receiver<String>,
}

impl EchoPending {
    pub fn new(echo: Echo) -> Self {
        let (tx, rx) = oneshot::channel();
        RESPONSE_REGISTRY.insert(echo.0, tx);
        Self { echo, receiver: rx }
    }

    pub async fn wait(self) -> Result<String> {
        let ret = match time::timeout(TIMEOUT, self.receiver).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow::anyhow!("收到的响应通道已关闭")),
            Err(_) => Err(anyhow::anyhow!("等待 Echo 响应超时")),
        };

        debug!("清理 Echo: {:?}", self.echo);
        Echo::remove(self.echo);

        ret
    }
}
