use crate::abi::utils::SmartJsonExt;
use helper::lnt_get_api;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SubjectType {
    SingleSelection,   //单选题
    MultipleSelection, //多选题
    TrueOrFalse,       //判断题
    FillInBlank,       //填空题
    ShortAnswer,       //简答题
    ParagraphDesc,     //段落说明
    Analysis,          //综合题
    Media,             //听力题
    Text,              //纯文本
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectOption {
    pub content: String,
    pub r#type: SubjectType,
    pub id: i64,
    //pub sort: IgnoredAny,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
    pub description: String,
    pub options: Vec<SubjectOption>,
    pub point: f64,
    pub sub_subjects: Vec<Subject>,
    pub r#type: SubjectType,
    pub id: i64,
    //pub answer_number: IgnoredAny,
    //pub data: IgnoredAny,
    //pub options: Vec<SubjectOption>,
    //pub point: f64,
    //pub sub_subjects: Vec<Box<Subject>>,
    //pub r#type: SubjectType,
    //pub difficulty_level: IgnoredAny,
    //pub last_updated_at: IgnoredAny,
    //pub note: IgnoredAny,
    //pub parent_id: IgnoredAny,
    //pub settings: IgnoredAny,
    //pub sort: IgnoredAny,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DistributeResponse {
    pub subjects: Vec<Subject>,
    //pub exam_paper_instance_id: IgnoredAny,
}

#[lnt_get_api(
    DistributeResponse,
    "https://lnt.xmu.edu.cn/api/exams/{exam_id:i64}/distribute"
)]
pub struct Distribute;

#[cfg(test)]
mod tests {
    use crate::api::xmu_service::login::castgc_get_session;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test() -> Result<()> {
        //TODO:目前只是REWRITE了尚未测试
        let castgc = "TGT-3827578-M2HQ5YkLD9VjNneiiEWeEXaizQy1X67ewOmxyCS4pfYHiMdQSYUwWP1HsHcrVM4A8WInull_main";
        let session = castgc_get_session(castgc).await?;
        let data = Distribute::get(&session, 71211).await?;
        println!("MyCourses: {:?}", data);
        Ok(())
    }
}
