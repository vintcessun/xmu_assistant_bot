use anyhow::Result;

use crate::api::{
    llm::chat::{file::LlmFile, llm::get_single_file_embedding},
    storage::{HasEmbedding, VectorSearchEngine},
};
use std::sync::{Arc, LazyLock};

static FILE_EMBEDDING_DB: LazyLock<VectorSearchEngine<LlmFile>> =
    LazyLock::new(|| VectorSearchEngine::new("llm_chat_file_embedding"));

impl HasEmbedding for LlmFile {
    fn get_embedding(&self) -> &[f32] {
        self.embedding.as_ref().unwrap().as_slice()
    }
}

pub async fn embedding_llm_file(mut file: LlmFile) -> Result<Arc<LlmFile>> {
    let embedding = get_single_file_embedding(&file).await?;
    file.embedding = Some(embedding);
    let file = Arc::new(file);
    FILE_EMBEDDING_DB.insert(file.clone()).await?;
    Ok(file)
}

pub async fn search_llm_file(key: Vec<f32>, top_k: usize) -> Result<Vec<Arc<LlmFile>>> {
    FILE_EMBEDDING_DB.search(key, top_k).await
}
