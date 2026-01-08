use crate::{
    api::storage::{FileStorage, HotTable},
    web::URL,
};
use anyhow::Result;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::SystemTime,
};

static DATA: LazyLock<HotTable<String, ExposeFileList>> = LazyLock::new(|| HotTable::new("file"));

pub fn query(id: &String) -> Option<Arc<ExposeFileList>> {
    DATA.get(id)
}

const EXPIRE_DURATION_SECS: u64 = 60 * 60 * 24; // 1 天

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub path: PathBuf,
    pub mime: String,
    pub is_temp: bool,
    pub expire_at: u64,
}

impl File {
    pub fn new<T: FileStorage>(file: &T) -> Self {
        let mime = mime_guess::from_path(file.get_path())
            .first_or_octet_stream()
            .to_string();
        Self {
            path: file.get_path().to_owned(),
            mime,
            is_temp: file.is_temp(),
            expire_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + EXPIRE_DURATION_SECS,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExposeFileList {
    pub files: Vec<File>,
    pub expire_at: u64,
}

impl ExposeFileList {
    pub fn new(files: Vec<File>) -> Self {
        Self {
            files,
            expire_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + EXPIRE_DURATION_SECS,
        }
    }
}

pub struct ExposeFileTask {
    pub id: String,
    pub list: Vec<File>,
    // 暂存所有待处理的 flush 任务
    pending_ios: Vec<BoxFuture<'static, Result<()>>>,
}

impl ExposeFileTask {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            list: Vec::new(),
            pending_ios: Vec::new(),
        }
    }

    /// O(1) 复杂度：只移动权，不等待 IO
    pub fn add<T: FileStorage + Send + Sync + 'static>(&mut self, file: Arc<T>) {
        // 1. 构造元数据并存入 list
        let f = File::new(file.as_ref());
        self.list.push(f);

        // 2. 将 wait_flush 转化为一个 Future 存起来，不在这里 await
        let fut = Box::pin(async move { file.wait_flush().await });
        self.pending_ios.push(fut);
    }

    /// 阻塞等待所有文件完成并写入 HotTable
    pub async fn finish(self) -> Result<()> {
        // 1. 等待所有文件写入完成
        futures::future::try_join_all(self.pending_ios).await?;

        // 2. 构造 ExposeFileList
        let expose_list = ExposeFileList::new(self.list.clone());

        // 3. 生成唯一 ID 并写入 HotTable
        DATA.insert(self.id, Arc::new(expose_list))?;

        Ok(())
    }

    pub fn get_url(&self) -> String {
        format!("{}/file/task/{}", URL, self.id)
    }
}
