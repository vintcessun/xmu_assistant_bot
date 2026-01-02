use ahash::RandomState;
use anyhow::Result;
use bytes::{BufMut, BytesMut};
use cookie::Cookie;
use dashmap::{DashMap, DashSet};
use fake_user_agent::get_chrome_rua;
use once_cell::sync::Lazy;
use reqwest::{
    Client, IntoUrl, Response,
    header::{COOKIE, HeaderValue, SET_COOKIE, USER_AGENT},
};
use smol_str::SmolStr;
use std::sync::Arc;
use url::Url;

const MAX_REDIRECTS: u8 = 20;

static GLOBAL_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none()) // 必须手动处理重定向，才能跨请求同步 Cookie
        .build()
        .unwrap()
});

pub struct SessionCookieStore {
    // Key: domain, Value: Map<cookie_name, cookie_value>
    raw_data: DashMap<SmolStr, DashMap<SmolStr, Arc<str>>, RandomState>,
    header_cache: DashMap<SmolStr, HeaderValue, RandomState>,
    dirty: DashSet<SmolStr, RandomState>,
}

impl Default for SessionCookieStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionCookieStore {
    pub fn new() -> Self {
        Self {
            raw_data: DashMap::with_hasher(RandomState::default()),
            header_cache: DashMap::with_hasher(RandomState::default()),
            dirty: DashSet::with_hasher(RandomState::default()),
        }
    }

    pub fn get_header(&self, host: &str) -> Option<HeaderValue> {
        // 1. 检查当前请求的 host 是否在脏集合中
        // 使用 remove 如果存在则返回 true，这在 DashSet 中是原子操作
        if self.dirty.remove(host).is_some() {
            // 只有当前 host 脏了才重构当前 host
            self.rebuild_cache_internal(host);
        }

        // 2. 直接返回缓存
        self.header_cache.get(host).map(|v| v.value().clone())
    }

    pub fn add_cookie_str(&self, host: &str, cookie_str: &str) {
        if let Ok(cookie) = Cookie::parse(cookie_str) {
            self.set(host, cookie.name(), cookie.value());
        }
    }

    pub fn set(&self, host: &str, key: &str, value: &str) {
        let host = SmolStr::new(host);

        let domain_map = self.raw_data.entry(host.clone()).or_default();
        domain_map.insert(SmolStr::new(key), Arc::from(value));

        self.dirty.insert(host);
    }

    fn rebuild_cache_internal(&self, host: &str) {
        // 只有当该域名确实有数据时才计算
        if let Some(domain_ref) = self.raw_data.get(host) {
            // 优化 A：直接从 DashMap 的 Entry 中克隆 Key (SmolStr)
            // SmolStr 的 Clone 是极其廉价的（引用计数增加或短字符串拷贝）
            let host_key = domain_ref.key().clone();
            let domain_map = domain_ref.value();

            // 优化：根据 Cookie 数量预估容量，平均每个 Cookie 假设 40 字节
            let estimated_size = domain_map.len() * 40;
            let mut buf = BytesMut::with_capacity(estimated_size); // 分配一次

            for item in domain_map.iter() {
                let (k, v) = (item.key(), item.value());

                if !buf.is_empty() {
                    buf.put_slice(b"; ");
                }
                buf.put_slice(k.as_bytes());
                buf.put_u8(b'=');
                buf.put_slice(v.as_bytes());
            }

            // 更新 HeaderValue 缓存
            if let Ok(hv) = HeaderValue::from_maybe_shared(buf.freeze()) {
                self.header_cache.insert(host_key, hv);
            }
        }
    }

    pub fn get(&self, host: &str, key: &str) -> Option<Arc<str>> {
        self.raw_data
            .get(host)
            .and_then(|domain_store| domain_store.get(key).map(|v| v.value().clone()))
    }
}

pub struct SessionClient {
    cookie_store: Arc<SessionCookieStore>,
    ua: HeaderValue,
}

impl Default for SessionClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionClient {
    pub fn new() -> Self {
        Self {
            cookie_store: Arc::new(SessionCookieStore::new()),
            ua: HeaderValue::from_static(get_chrome_rua()),
        }
    }

    /// 执行带 Cookie 隔离和自动重定向的请求
    async fn request_internal(
        &self,
        mut method: reqwest::Method,
        mut url: Url,
        mut body: Option<String>,
    ) -> Result<Response> {
        let mut redirect_count = 0;

        loop {
            if redirect_count > MAX_REDIRECTS {
                anyhow::bail!("重定向次数过多，可能存在循环重定向");
            }

            // 1. 构造本次请求的 Builder
            let mut builder = GLOBAL_CLIENT
                .request(method.clone(), url.clone())
                .header(USER_AGENT, &self.ua);

            // 2. 极致路径：直接从缓存取 HeaderValue (Arc clone)
            if let Some(c) = self
                .cookie_store
                .get_header(url.host_str().unwrap_or_default())
            {
                builder = builder.header(COOKIE, c);
            }

            // 3. 注入 Body (如果是 POST)
            if let Some(ref b) = body {
                builder = builder.header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                );
                builder = builder.body(b.clone());
            }

            let resp = builder.send().await?;

            // 4. 异步更新 Cookie (逻辑保持不变)
            for cookie in resp.headers().get_all(SET_COOKIE) {
                if let Ok(c_str) = cookie.to_str() {
                    self.cookie_store
                        .add_cookie_str(resp.url().host_str().unwrap_or_default(), c_str);
                }
            }

            // 5. 处理重定向 (解决第 4 点：状态码严谨性)
            if resp.status().is_redirection()
                && let Some(loc) = resp.headers().get(reqwest::header::LOCATION)
            {
                let next_url = resp.url().join(loc.to_str()?)?;
                let status = resp.status();

                match status {
                    // 301, 302, 303: 标准做法是转为 GET 并丢弃 Body
                    reqwest::StatusCode::MOVED_PERMANENTLY
                    | reqwest::StatusCode::FOUND
                    | reqwest::StatusCode::SEE_OTHER => {
                        method = reqwest::Method::GET;
                        body = None;
                    }
                    // 307, 308: 必须严格保持原有的 Method 和 Body
                    // 逻辑上这里不需要操作，保持当前的 method 和 body 变量即可
                    reqwest::StatusCode::TEMPORARY_REDIRECT
                    | reqwest::StatusCode::PERMANENT_REDIRECT => {}
                    // 其他不常见的重定向不自动处理
                    _ => return Ok(resp),
                }

                url = next_url;
                redirect_count += 1;
                continue;
            }
            return Ok(resp);
        }
    }

    pub async fn get<U: IntoUrl>(&self, url: U) -> Result<Response> {
        let url = url.into_url()?;
        self.request_internal(reqwest::Method::GET, url, None).await
    }

    pub async fn post<U: IntoUrl, T: serde::Serialize + ?Sized>(
        &self,
        url: U,
        data: &T,
    ) -> Result<Response> {
        let url = url.into_url()?;
        let body = serde_urlencoded::to_string(data)?;
        self.request_internal(reqwest::Method::POST, url, Some(body))
            .await
    }

    pub fn set_cookie(&self, key: &str, value: &str, url: url::Url) {
        self.cookie_store
            .set(url.host_str().unwrap_or_default(), key, value);
    }

    pub fn get_cookie(&self, key: &str, url: &url::Url) -> Option<Arc<str>> {
        self.cookie_store
            .get(url.host_str().unwrap_or_default(), key)
    }
}
