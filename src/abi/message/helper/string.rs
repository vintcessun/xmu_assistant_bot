use genai::chat::MessageContent;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;

/// 优雅的延迟字符串：利用 RawValue 的位置记录功能，配合 OnceLock 延迟解析
pub struct LazyString {
    // Box<RawValue> 记录了 JSON 原始片段的位置
    // 我们使用 'static 抹除生命周期，因为所有权由外部 ArcWith 保证
    inner: Box<RawValue>,
    // 缓存解析后的结果
    cache: OnceLock<String>,
}

impl Debug for LazyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyString")
            .field("value", &self.get())
            .finish()
    }
}

impl Clone for LazyString {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(), // Box<RawValue> 的 clone 很轻量
            // 只有当原对象已经解析了，才同步缓存；否则保持空锁
            cache: self
                .cache
                .get()
                .map(|s| {
                    let lock = OnceLock::new();
                    let _ = lock.set(s.clone());
                    lock
                })
                .unwrap_or_default(),
        }
    }
}

impl From<LazyString> for String {
    fn from(mut lazy: LazyString) -> Self {
        // 如果 OnceLock 已经物化了，直接通过 take 拿走 String 避免克隆
        // 注意：OnceLock::take 在 Rust 1.80+ 可用，若版本较低则用 self.get().to_string()
        lazy.cache.take().unwrap_or_else(|| {
            // 如果没物化，则进行解析
            serde_json::from_str(lazy.inner.get()).expect("LazyString: Failed to deserialize")
        })
    }
}

impl From<LazyString> for MessageContent {
    fn from(value: LazyString) -> Self {
        MessageContent::from_text(value.get())
    }
}

impl LazyString {
    /// 专门为 Option 转换准备的快捷方法
    pub fn into_opt_string(lazy: Option<Self>) -> Option<String> {
        lazy.map(String::from)
    }
}

impl LazyString {
    /// 触发解析：将原始 JSON 文本转义并物化为 String
    pub fn get(&self) -> &str {
        self.cache.get_or_init(|| {
            // 直接利用 serde_json 的标准解析流程
            // 这会自动处理转义符（如 \n, \t）
            let s: String = serde_json::from_str(self.inner.get())
                .expect("LazyString: Failed to deserialize segment");
            s
        })
    }
}

impl Deref for LazyString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

// --- 实现反序列化 ---
impl<'de> Deserialize<'de> for LazyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 1. 先反序列化为 RawValue 的 Box
        let raw: Box<RawValue> = Box::<RawValue>::deserialize(deserializer)?;

        // 2. 这里的“魔法”：我们将 Box<RawValue> 视为 'static。
        // 虽然 serde_json 给出的 raw 带有 'de 生命周期，但由于它被封装在 Box 中，
        // 它的内存布局其实就是 [ptr, len]。
        // 只要外部 Bytes 还在，这个 Box 指向的地址就是安全的。
        #[allow(clippy::useless_transmute)]
        let inner = unsafe { std::mem::transmute::<Box<RawValue>, Box<RawValue>>(raw) };

        Ok(Self {
            inner,
            cache: OnceLock::new(),
        })
    }
}
