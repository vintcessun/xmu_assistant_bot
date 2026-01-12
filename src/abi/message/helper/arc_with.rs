use std::ops::Deref;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Utf8Bytes;

/// 这是一个将对象 T 和它所依赖的原始数据 Bytes 捆绑在一起的包装器
#[derive(Debug)]
pub struct ArcWith<T> {
    inner: Arc<T>,
    source: Utf8Bytes,
}

impl<T> Clone for ArcWith<T> {
    fn clone(&self) -> Self {
        ArcWith {
            inner: self.inner.clone(),
            source: self.source.clone(),
        }
    }
}

impl<T> ArcWith<T> {
    pub fn new(data: T, source: Utf8Bytes) -> Self {
        Self {
            inner: Arc::new(data),
            source,
        }
    }
}

impl<T> Deref for ArcWith<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// 允许在线程间自由传递
unsafe impl<T: Send + Sync> Send for ArcWith<T> {}
unsafe impl<T: Send + Sync> Sync for ArcWith<T> {}
