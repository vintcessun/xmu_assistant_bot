const BASE: &str = "temp";

use super::BASE_DATA_DIR;
use crate::api::storage::file::{FileBackend, FileStorage};
use crate::config::ensure_dir;
use anyhow::Result;
use async_trait::async_trait;
use const_format::concatcp;
use dashmap::DashSet;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

pub static TEMP_DATA_DIR: LazyLock<&'static str> = LazyLock::new(|| {
    let path = concatcp!(BASE_DATA_DIR, "/", BASE);
    ensure_dir(path);
    path
});

static MANAGER: LazyLock<TempFileManager> = LazyLock::new(TempFileManager::new);

// --- 临时文件管理器 ---
pub struct TempFileManager {
    dir: PathBuf,
    cache: DashSet<String>,
    counter: AtomicUsize,
}

impl TempFileManager {
    pub fn new() -> Self {
        let dir = Path::new(*TEMP_DATA_DIR).to_path_buf();
        let cache = DashSet::new();
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

    pub fn alloc_path(&self, filename: &str) -> PathBuf {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("t");
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        if self.cache.insert(filename.to_string()) {
            return self.dir.join(filename);
        }
        loop {
            let id = self.counter.fetch_add(1, Ordering::Relaxed);
            let new_name = format!("{}_{}{}", stem, id, ext);
            if self.cache.insert(new_name.clone()) {
                return self.dir.join(new_name);
            }
        }
    }

    pub fn release(&self, path: PathBuf, remove_disk: bool) {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_string = name.to_string();
            self.cache.remove(&name_string);
            if remove_disk {
                // 异步删除物理文件
                tokio::task::spawn_blocking(move || {
                    let _ = fs::remove_file(path);
                });
            }
        }
    }
}

// --- TempFile 结构重构 ---
#[derive(Debug)]
pub struct TempFile {
    pub path: PathBuf,
    pub remove_on_drop: bool,
}

impl TempFile {
    /// 仅内部使用的物理占坑构造
    fn prepare_internal(filename: &str, remove: bool) -> Self {
        let path = MANAGER.alloc_path(filename);

        Self {
            path,
            remove_on_drop: remove,
        }
    }
}

/// RAII: 当作用域结束时，TempFile 会自动从管理器中释放并删除磁盘文件
impl Drop for TempFile {
    fn drop(&mut self) {
        MANAGER.release(self.path.clone(), self.remove_on_drop);
    }
}

// --- 实现协议，支持 from_url 自动路由 ---

#[async_trait]
impl FileStorage for TempFile {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }
    fn is_temp(&self) -> bool {
        true
    }
}

#[async_trait]
impl FileBackend for TempFile {
    /// 统一构造器：由 FileFromUrl 调用
    fn prepare(filename: &str) -> Self {
        // 默认 TempFile 在 from_url 场景下开启自动删除
        Self::prepare_internal(filename, true)
    }

    /// 下载完成钩子
    async fn on_complete(&mut self) -> Result<()> {
        Ok(())
    }
}
