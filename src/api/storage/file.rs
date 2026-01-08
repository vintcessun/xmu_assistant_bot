const BASE: &str = "file";

use super::BASE_DATA_DIR;
use crate::{abi::message::file::FileUrl, config::ensure_dir};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use const_format::concatcp;
use dashmap::DashSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{
    fmt,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{self, UnboundedSender},
        oneshot, watch,
    },
};
use tracing::error;
use url::Url;

pub static DATA_DIR: LazyLock<&'static str> = LazyLock::new(|| {
    let path = concatcp!(BASE_DATA_DIR, "/", BASE);
    ensure_dir(path);
    path
});

static MANAGER: LazyLock<FileManager> = LazyLock::new(FileManager::new);

pub struct FileManager {
    dir: PathBuf,
    cache: DashSet<String>,
    counter: AtomicUsize,
}

impl FileManager {
    pub fn new() -> Self {
        let dir = Path::new(*DATA_DIR).to_path_buf();
        let cache = DashSet::new();

        // 预热：启动时扫描磁盘，确保内存与磁盘同步
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    cache.insert(name);
                }
            }
        }

        Self {
            dir,
            cache,
            counter: AtomicUsize::new(0),
        }
    }

    /// 极速分配路径：基于内存缓存和原子计数器，O(1) 复杂度
    pub fn alloc_path(&self, filename: &str) -> PathBuf {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("f");
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        // 尝试直接使用原名
        if self.cache.insert(filename.to_string()) {
            return self.dir.join(filename);
        }

        // 冲突处理：使用原子计数器快速寻找空位
        loop {
            let id = self.counter.fetch_add(1, Ordering::Relaxed);
            let new_name = format!("{}_{}{}", stem, id, ext);
            if self.cache.insert(new_name.clone()) {
                return self.dir.join(new_name);
            }
        }
    }
}
enum WriteOp {
    Write(Vec<u8>),
    Sync(oneshot::Sender<()>),
}

#[derive(Debug, Serialize)]
pub struct File {
    pub path: PathBuf,

    // 写入通道：仅在 create 模式下有效
    #[serde(skip)]
    write_tx: Option<UnboundedSender<WriteOp>>,

    // 读取状态同步：watch 允许无限个订阅者等待同一个读取任务完成
    #[serde(skip)]
    read_rx: Option<watch::Receiver<Option<Arc<Vec<u8>>>>>,
}

impl File {
    /// 【写模式】新建文件并持有写入权限
    pub fn create(filename: &str) -> Self {
        let path = MANAGER.alloc_path(filename);
        let (tx, mut rx) = mpsc::unbounded_channel::<WriteOp>();

        let p_clone = path.clone();
        tokio::spawn(async move {
            let p_init = p_clone.clone();
            // 物理创建占坑
            let _ = tokio::task::spawn_blocking(move || {
                if let Some(parent) = p_init.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::File::create(&p_init);
            })
            .await;

            while let Some(op) = rx.recv().await {
                match op {
                    WriteOp::Write(data) => {
                        let p_w = p_clone.clone();
                        let _ = tokio::task::spawn_blocking(move || {
                            if let Ok(mut f) = fs::File::create(p_w) {
                                let _ = f.write_all(&data);
                            }
                        })
                        .await;
                    }
                    WriteOp::Sync(reply) => {
                        let _ = reply.send(());
                    }
                }
            }
        });

        Self {
            path,
            write_tx: Some(tx),
            read_rx: None,
        }
    }

    /// 【反序列化恢复/手动加载】启动后台读取协程
    pub fn load(json: &str) -> Result<Self> {
        let mut file: Self = serde_json::from_str(json)?;
        file.start_read_task();
        Ok(file)
    }

    /// 内部逻辑：发起后台异步读取
    fn start_read_task(&mut self) {
        let (tx, rx) = watch::channel(None);
        let p = self.path.clone();

        tokio::spawn(async move {
            let data: Result<Arc<Vec<u8>>> = async {
                let mut f = tokio::fs::File::open(p).await?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).await?;
                Ok(Arc::new(buf))
            }
            .await;

            match data {
                Ok(d) => {
                    let _ = tx.send(Some(d));
                }
                Err(e) => {
                    error!("加载文件失败: {}", e);
                    let _ = tx.send(None);
                }
            }
        });

        self.read_rx = Some(rx);
    }

    /// 【异步读取屏障】等待直到读取任务完成
    pub async fn wait_for_data(&self) -> Result<Arc<Vec<u8>>> {
        let mut rx = self
            .read_rx
            .as_ref()
            .ok_or_else(|| anyhow!("读取任务未启动，请确保调用了 load()"))?
            .clone();

        // 如果数据还没准备好，则等待变更通知
        if rx.borrow().is_none() {
            rx.changed()
                .await
                .map_err(|_| anyhow!("读取协程意外关闭"))?;
        }

        rx.borrow()
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("文件读取失败或数据为空"))
    }

    /// 【异步写入】提交写入
    pub async fn write(&self, data: Vec<u8>) -> Result<()> {
        let tx = self
            .write_tx
            .as_ref()
            .ok_or_else(|| anyhow!("此句柄无写入权限 (ReadOnly)"))?;

        // 发起写入
        tx.send(WriteOp::Write(data))
            .map_err(|_| anyhow!("写入协程异常"))?;

        Ok(())
    }
}

// --- 手动实现反序列化，实现“恢复即加载” ---
impl<'de> Deserialize<'de> for File {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 定义内部辅助结构用于解析 JSON 字段
        #[derive(Deserialize)]
        struct FileFields {
            path: PathBuf,
        }

        let fields = FileFields::deserialize(deserializer)?;

        // 创建初始结构（此时没有读写权限）
        let mut file = Self {
            path: fields.path,
            write_tx: None,
            read_rx: None,
        };

        // 核心改动：在反序列化完成的一刻，自动开启后台读取协程
        file.start_read_task();

        Ok(file)
    }
}

#[async_trait]
pub trait FileStorage {
    fn get_path(&self) -> &PathBuf;
    async fn wait_flush(&self) -> Result<()>;
    const IS_TEMP: bool;
    fn is_temp(&self) -> bool {
        Self::IS_TEMP
    }
}

#[async_trait]
impl FileStorage for File {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }

    async fn wait_flush(&self) -> Result<()> {
        let tx = match self.write_tx.as_ref() {
            Some(t) => t,
            None => return Ok(()), //只读无需等写入完成
        };

        // 发起同步信号并等待回执
        let (res_tx, res_rx) = oneshot::channel();
        tx.send(WriteOp::Sync(res_tx))?;

        res_rx.await.map_err(|_| anyhow!("磁盘应用超时或失败"))
    }

    const IS_TEMP: bool = false;
}
