use crate::abi::utils::SmartJsonExt;
use helper::lnt_get_api;
use serde::{Deserialize, Serialize};

use crate::api::xmu_service::lnt::distribute::SubjectType;

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct CorrectAnswer {
    pub answer_option_ids: Vec<i64>,
    pub subject_id: i64,
    //pub content: IgnoredAny,
    //pub point: IgnoredAny,
    //pub r#type: IgnoredAny,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CorrectAnswerData {
    pub correct_answers: Vec<CorrectAnswer>,
}
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectOption {
    pub content: String,
    pub id: i64,
    pub is_answer: bool,
    pub r#type: SubjectType,
    //pub sort: IgnoredAny,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
    pub answer_explanation: String,
    pub description: String,
    pub id: i64,
    pub options: Vec<SubjectOption>,
    pub r#type: SubjectType,
    pub wrong_explanation: String,
    //pub answer_number: IgnoredAny,
    //pub correct_answers: IgnoredAny,
    //pub data: IgnoredAny,
    //pub difficulty_level: IgnoredAny,
    //pub last_updated_at: IgnoredAny,
    //pub note: IgnoredAny,
    //pub parent_id: IgnoredAny,
    //pub point: IgnoredAny,
    //pub settings: IgnoredAny,
    //pub sort: IgnoredAny,
    //pub sub_subjects: IgnoredAny,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectData {
    pub subjects: Vec<Subject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmissionResponse {
    pub subjects_data: SubjectData,
    //pub auto_mark: IgnoredAny,
    //pub check_ip_consistency_passed: IgnoredAny,
    //pub correct_answers_data: IgnoredAny,
    //pub correct_data:IgnoredAny,
    //pub exam_type:IgnoredAny,
    //pub is_makeup:IgnoredAny,
    //pub is_simulated:IgnoredAny,
    //pub knowledge_node_data:IgnoredAny,
    //pub score: IgnoredAny,
    //pub submission_comment_data: IgnoredAny,
    //pub submission_data: IgnoredAny,
    //pub submission_score_data: IgnoredAny,
    //pub submit_method: IgnoredAny,
    //pub submit_method_text: IgnoredAny,
}

#[lnt_get_api(
    SubmissionResponse,
    "https://lnt.xmu.edu.cn/api/exams/{exam_id:i64}/submissions/{submission_id:i64}"
)]
pub struct SubmissionsId;

#[cfg(test)]
mod tests {
    use crate::api::xmu_service::login::castgc_get_session;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test() -> Result<()> {
        let castgc = "TGT-3852721-5F6eRNQT3hKL70kX3mDbLKQOpeUcKCYbCwJUZNW-btgCA45jHAWRs6iRLEeNzYP3-1cnull_main";
        let session = castgc_get_session(castgc).await?;
        let data = SubmissionsId::get(&session, 18543, 1007385).await?;
        println!("SubmissionsId: {:?}", data);
        Ok(())
    }
}
