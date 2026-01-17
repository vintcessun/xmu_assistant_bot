use crate::api::storage::ColdTable;
use anyhow::Result;
use dashmap::DashMap;
use hnsw_rs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// 假设你的 V 需要实现这个 trait 来提供向量
pub trait HasEmbedding {
    fn get_embedding(&self) -> &[f32];
}

pub struct VectorSearchEngine<V>
where
    V: Serialize + DeserializeOwned + Send + Sync + Clone + HasEmbedding + 'static,
{
    // 你原有的持久化表
    kv_table: ColdTable<Uuid, V>,
    // 内存向量索引
    index: Arc<Hnsw<'static, f32, DistCosine>>,
    // 内存 ID 映射：HNSW 内部 ID -> 业务 UUID
    id_map: Arc<DashMap<usize, Uuid>>,
}

impl<V> VectorSearchEngine<V>
where
    V: Serialize + DeserializeOwned + Send + Sync + Clone + HasEmbedding + 'static,
{
    /// 1. 加载并重建索引
    pub fn new(table_name: &'static str) -> Self {
        let kv_table: ColdTable<Uuid, V> = ColdTable::new(table_name);

        // 初始化 HNSW 参数
        // M=16, max_elements=100万, ef_construction=200, ef_search=20
        let index = Hnsw::new(16, 1000000, 200, 20, DistCosine {});
        let id_map = DashMap::new();

        let handle = tokio::runtime::Handle::try_current()
            .expect("VectorSearchEngine 必须在 Tokio 运行时内初始化");

        // 从 redb 读取所有数据
        let all_records = handle
            .block_on(async { kv_table.get_all().await })
            .expect("从持久化数据库加载数据失败");

        info!("正在重建索引，总计 {} 条记录...", all_records.len());

        for (i, (uuid, value)) in all_records.into_iter().enumerate() {
            let embedding = value.get_embedding();
            // 插入索引：(向量数据, 内部自增ID)
            index.insert((embedding, i));
            // 映射关系存入 DashMap
            id_map.insert(i, uuid);
        }

        Self {
            kv_table,
            index: Arc::new(index),
            id_map: Arc::new(id_map),
        }
    }

    /// 2. 插入新数据（同步写入磁盘和内存索引）
    pub async fn insert(&self, value: V) -> Result<()> {
        let uuid = Uuid::new_v4();
        let embedding = value.get_embedding().to_vec();

        // A. 写入持久化数据库 (ColdTable)
        self.kv_table.insert(uuid, value).await?;

        // B. 更新内存索引
        // 注意：这里需要确定一个新的 internal_id，通常可以用 id_map 的长度
        let internal_id = self.id_map.len();
        self.index.insert((&embedding, internal_id));
        self.id_map.insert(internal_id, uuid);

        Ok(())
    }

    /// 3. 向量搜索 (语义搜索)
    pub async fn search(&self, query_vec: Vec<f32>, top_k: usize) -> Result<Vec<V>> {
        let index = Arc::clone(&self.index);
        let id_map = Arc::clone(&self.id_map);

        // HNSW 的搜索是 CPU 密集型，建议在 spawn_blocking 中执行
        let neighbor_ids = tokio::task::spawn_blocking(move || {
            // search 参数：查询向量，返回数量，ef_search（搜索精度）
            index.search(&query_vec, top_k, 32)
        })
        .await?;

        let mut results = Vec::new();
        for neighbor in neighbor_ids {
            // 从 DashMap 获取 UUID
            if let Some(uuid) = id_map.get(&neighbor.d_id) {
                // 从 ColdTable 获取完整磁盘数据
                if let Some(data) = self.kv_table.get(*uuid).await? {
                    results.push(data);
                }
            }
        }

        Ok(results)
    }
}
