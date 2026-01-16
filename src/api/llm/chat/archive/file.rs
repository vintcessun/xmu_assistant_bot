use crate::api::{
    llm::chat::file::LlmFile,
    storage::{ColdTable, HasEmbedding, VectorSearchEngine},
};
use std::sync::{Arc, LazyLock};

/*
static FILE_EMBEDDING_DB: LazyLock<VectorSearchEngine<Arc<LlmFile>>> =
    LazyLock::new(|| VectorSearchEngine::new("llm_chat_file_embedding"));

impl HasEmbedding for LlmFile {
    fn get_embedding(&self) -> Vec<f32> {
        todo!("完成 LlmFile 的向量化逻辑")
    }
}
*/

pub async fn embedding_llm_file(file: Arc<LlmFile>) {
    //TODO:完成LLM进行文件嵌入
    todo!()
}
