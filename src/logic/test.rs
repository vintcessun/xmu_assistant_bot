use super::BuildHelp;
use crate::{
    abi::{logic_import::*, message::from_str},
    api::xmu_service::{
        llm::ChooseCourse,
        lnt::{Distribute, Exams},
    },
    logic::helper::get_client_or_err,
};
use anyhow::anyhow;
use tracing::trace;

#[handler(msg_type=Message,command="test",echo_cmd=true,
help_msg=r#"用法:/test <描述>
<描述>:描述课程，后端使用LLM进行智能识别查询指定的课程的测试信息
功能: 查询指定课程的测试信息"#)]
pub async fn test(ctx: Context) -> Result<()> {
    let client = get_client_or_err(&ctx).await?;
    let msg_text = ctx.get_message_text();
    let course_id = {
        let course = ChooseCourse::get_from_client(&client, msg_text).await?;
        trace!("返回课程选择结果：");
        trace!(?course);
        course.course_id
    }
    .ok_or(anyhow!("未找到课程，请更加清晰的阐释课程的名称"))?;

    let exam_data = Exams::get_from_client(&client, *course_id).await?;

    for exam in exam_data.exams {
        trace!("测试信息：{:?}", exam);
        ctx.send_message_async(from_str(format!(
            r#"小测名称: {}
小测开始时间: {}
小测结束时间: {}
小测是否开始: {}
小测ID: {}"#,
            exam.title, exam.start_time, exam.end_time, exam.is_started, exam.id
        )));
    }

    Ok(())
}

#[handler(msg_type=Message,command="get_test",echo_cmd=true,
help_msg=r#"用法:/get_test <ID>
<ID>: 查询小测的ID，通过 /test 命令获取
功能: 查询指定小测的内容"#)]
pub async fn get_test(ctx: Context) -> Result<()> {
    let client = get_client_or_err(&ctx).await?;
    let id = ctx
        .get_message_text()
        .split_whitespace()
        .collect::<String>()
        .parse::<i64>()
        .map_err(|e| anyhow!("不是有效的ID: {}\n可以通过/test 获取ID", e))?;

    let distribute = Distribute::get_from_client(&client, id).await?;

    for subject in distribute.subjects {
        trace!("题目信息：{:?}", subject);
        //TODO:处理题目信息
        todo!("处理题目信息");
    }

    Ok(())
}
