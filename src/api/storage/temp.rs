const BASE: &str = "temp";

use super::BASE_DATA_DIR;
use crate::config::ensure_dir;
use anyhow::Result;
use const_format::concatcp;
use dashmap::DashSet;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

pub static TEMP_DATA_DIR: LazyLock<&'static str> = LazyLock::new(|| {
    let path = concatcp!(BASE_DATA_DIR, "/", BASE);
    ensure_dir(path);
    path
});

static MANAGER: LazyLock<TempFileManager> = LazyLock::new(TempFileManager::new);

pub struct TempFileManager {
    dir: PathBuf,
    cache: DashSet<String>,
    counter: AtomicUsize,
}

impl TempFileManager {
    pub fn new() -> Self {
        let dir = Path::new(*TEMP_DATA_DIR).to_path_buf();
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

    /// 释放资源：从内存移除并异步删除物理文件
    pub fn release(&self, path: PathBuf, remove_disk: bool) {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_string = name.to_string();

            if remove_disk {
                self.cache.remove(&name_string);
                // 异步磁盘删除，不阻塞主流程
                tokio::task::spawn_blocking(move || {
                    let _ = fs::remove_file(path);
                });
            }
        }
    }
}

enum WriteOp {
    Write(Vec<u8>),
    Append(Vec<u8>),
    Sync(oneshot::Sender<()>),
}

pub struct TempFile {
    path: PathBuf,
    remove_on_drop: bool,
    write_tx: UnboundedSender<WriteOp>,
}

impl TempFile {
    /// 创建一个新的临时文件
    /// - `filename`: 期望文件名（如果冲突会自动更名）
    /// - `remove`: 是否在离开作用域时删除
    pub fn create(filename: &str, remove: bool) -> Self {
        let path = MANAGER.alloc_path(filename);
        let (tx, mut rx) = mpsc::unbounded_channel::<WriteOp>();

        let path_clone = path.clone();

        // 后台写入守护任务
        tokio::spawn(async move {
            let p_init = path_clone.clone();

            // 1. 物理占坑：确保文件创建成功
            let init_res = tokio::task::spawn_blocking(move || {
                if let Some(parent) = p_init.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                fs::File::create(&p_init)
            })
            .await;

            if let Ok(Ok(_)) = init_res {
                // 2. 顺序处理写入请求
                while let Some(op) = rx.recv().await {
                    let p = path_clone.clone();
                    match op {
                        WriteOp::Write(data) => {
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(mut f) = fs::File::create(p) {
                                    let _ = f.write_all(&data);
                                }
                            })
                            .await;
                        }
                        WriteOp::Append(data) => {
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(mut f) = OpenOptions::new().append(true).open(p) {
                                    let _ = f.write_all(&data);
                                }
                            })
                            .await;
                        }
                        WriteOp::Sync(reply) => {
                            // 屏障逻辑：所有先前的 IO 已由 spawn_blocking 完成
                            let _ = reply.send(());
                        }
                    }
                }
            }
        });

        Self {
            path,
            remove_on_drop: remove,
            write_tx: tx,
        }
    }

    /// 极速覆盖写入（内存操作）
    pub fn write_all(&self, data: Vec<u8>) -> Result<()> {
        self.write_tx
            .send(WriteOp::Write(data))
            .map_err(|_| anyhow::anyhow!("TempFile worker channel closed"))
    }

    /// 极速追加写入（内存操作）
    pub fn append_all(&self, data: Vec<u8>) -> Result<()> {
        self.write_tx
            .send(WriteOp::Append(data))
            .map_err(|_| anyhow::anyhow!("TempFile worker channel closed"))
    }

    /// 异步等待所有写入操作完成并落盘
    pub async fn wait_flush(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.write_tx
            .send(WriteOp::Sync(tx))
            .map_err(|_| anyhow::anyhow!("TempFile worker channel closed"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Flush operation cancelled"))
    }

    /// 获取文件物理路径
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// RAII: 自动资源回收
impl Drop for TempFile {
    fn drop(&mut self) {
        // 传递路径的所有权给管理器进行后台清理
        MANAGER.release(self.path.clone(), self.remove_on_drop);
    }
}
