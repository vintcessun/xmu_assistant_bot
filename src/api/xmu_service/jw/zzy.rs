use super::IDS_URL;
use super::JwAPI;
use crate::api::network::SessionClient;
use anyhow::Result;
use anyhow::bail;
use helper::jw_api;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[jw_api(
    url = "https://jw.xmu.edu.cn/jwapp/sys/zzygl/modules/xszzysq/cxxszzybmsq.do",
    app = "https://jw.xmu.edu.cn/appShow?appId=4939740894443498"
)]
pub struct Zzy {
    pub xznj_display: String,              // 学制年级显示
    pub sfzzlq: String,                    // 是否最终录取
    pub sqyx_display: String,              // 申请院系显示
    lqzt: Option<Box<RawValue>>,           // 录取状态
    yxdm_display: Option<Box<RawValue>>,   // 院系代码显示
    pclbdm_display: Option<Box<RawValue>>, // 批次类别代码显示
    lqzt_display: Option<Box<RawValue>>,   // 录取状态显示
    zyfxdm: Option<Box<RawValue>>,         // 专业方向代码
    zczy: Option<Box<RawValue>>,           // 专业主业
    zcnj: Option<Box<RawValue>>,           // 注册年级
    czsj: Option<Box<RawValue>>,           // 操作时间
    sqzy_display: Option<Box<RawValue>>,   // 申请专业显示
    sqsj: Option<Box<RawValue>>,           // 申请时间
    zbdm_display: Option<Box<RawValue>>,   // 招办代码显示
    kskssj: Option<Box<RawValue>>,         // 考试开始时间
    xznj: Option<Box<RawValue>>,           // 学制年级
    zbdm: Option<Box<RawValue>>,           // 招办代码
    zcbj: Option<Box<RawValue>>,           // 注册标记
    pcdm: Option<Box<RawValue>>,           // 批次代码
    ggkcj: Option<Box<RawValue>>,          // 公共课成绩
    msjssj: Option<Box<RawValue>>,         // 面试结束时间
    zcyx: Option<Box<RawValue>>,           // 专业意向
    zydm: Option<Box<RawValue>>,           // 专业代码
    pclbdm: Option<Box<RawValue>>,         // 批次类别代码
    zylcxbzt: Option<Box<RawValue>>,       // 专业录取选报状态
    by10: Option<Box<RawValue>>,           // 备用10
    czrxm: Option<Box<RawValue>>,          // 操作人姓名
    sfsx_display: Option<Box<RawValue>>,   // 是否生效显示
    bzsm: Option<Box<RawValue>>,           // 备注说明
    zyshzt: Option<Box<RawValue>>,         // 专业审核状态
    wid: Option<Box<RawValue>>,            // 唯一ID
    zyfxdm_display: Option<Box<RawValue>>, // 专业方向代码显示
    czr: Option<Box<RawValue>>,            // 操作人
    by2: Option<Box<RawValue>>,            // 备用2
    sqzy: Option<Box<RawValue>>,           // 申请专业
    by1: Option<Box<RawValue>>,            // 备用1
    ydxnxq: Option<Box<RawValue>>,         // 已读学年学期
    by4: Option<Box<RawValue>>,            // 备用4
    sqjg: Option<Box<RawValue>>,           // 申请结果
    by3: Option<Box<RawValue>>,            // 备用3
    sqzt: Option<Box<RawValue>>,           // 申请状态
    by6: Option<Box<RawValue>>,            // 备用6
    lqsj: Option<Box<RawValue>>,           // 录取时间
    by5: Option<Box<RawValue>>,            // 备用5
    zcj: Option<Box<RawValue>>,            // 总成绩
    by8: Option<Box<RawValue>>,            // 备用8
    yxdm: Option<Box<RawValue>>,           // 院系代码
    bjmc: Option<Box<RawValue>>,           // 班级名称
    czip: Option<Box<RawValue>>,           // 操作IP
    by7: Option<Box<RawValue>>,            // 备用7
    rzlbdm: Option<Box<RawValue>>,         // 入至类别代码
    by9: Option<Box<RawValue>>,            // 备用9
    fj: Option<Box<RawValue>>,             // 附加
    zyxh: Option<Box<RawValue>>,           // 专业学号
    xsbmkssj: Option<Box<RawValue>>,       // 学生报名开始时间
    ydlbdm: Option<Box<RawValue>>,         // 已读类别代码
    sqlx: Option<Box<RawValue>>,           // 申请类型
    sqly: Option<Box<RawValue>>,           // 申请理由
    zykcj: Option<Box<RawValue>>,          // 专业课成绩
    bjdm: Option<Box<RawValue>>,           // 班级代码
    ydxnxq_display: Option<Box<RawValue>>, // 已读学年学期显示
    tsxslx: Option<Box<RawValue>>,         // 特殊学生类型
    sqyx: Option<Box<RawValue>>,           // 申请院系
    sfdsq: Option<Box<RawValue>>,          // 是否待申请
    sfsx: Option<Box<RawValue>>,           // 是否生效
    zylb: Option<Box<RawValue>>,           // 专业类别
    zydm_display: Option<Box<RawValue>>,   // 专业代码显示
    ksjssj: Option<Box<RawValue>>,         // 考试结束时间
    pcdm_display: Option<Box<RawValue>>,   // 批次代码显示
    xsbmjssj: Option<Box<RawValue>>,       // 学生报名结束时间
    sqzyfx: Option<Box<RawValue>>,         // 申请专业方向
    xh: Option<Box<RawValue>>,             // 学号
    kch: Option<Box<RawValue>>,            // 课程号
    ydyy_dm: Option<Box<RawValue>>,        // 已读原有代码
    kcm: Option<Box<RawValue>>,            // 课程名
    orderfilter: Option<Box<RawValue>>,    // 排序过滤
    xm: Option<Box<RawValue>>,             // 姓名
    sfzzlq_display: Option<Box<RawValue>>, // 是否最终录取显示
    lxfs: Option<Box<RawValue>>,           // 联系方式
    pm: Option<Box<RawValue>>,             // 排名
    mskssj: Option<Box<RawValue>>,         // 面试开始时间
}

impl Zzy {
    pub fn get_profile(self) -> Result<ZzyProfile> {
        let mut rows = self.datas.cxxszzybmsq.rows;

        if rows.is_empty() {
            bail!("未查询到转专业信息");
        }

        let first_row = rows.remove(0);
        let entry_year = first_row.xznj_display;

        let mut trans_dept: Vec<String> = rows
            .into_iter()
            .filter(|item| item.sfzzlq == "1")
            .map(|item| item.sqyx_display)
            .collect();

        if first_row.sfzzlq == "1" {
            trans_dept.insert(0, first_row.sqyx_display);
        }

        Ok(ZzyProfile {
            entry_year,
            trans_dept,
        })
    }

    pub async fn get(castgc: &str, student_id: &str) -> Result<Self> {
        Self::call(
            castgc,
            &ZzyRequest {
                batch_code: "01",
                student_id,
                tag: "-CZSJ,+ZYXH",
                page_size: 10,
                page_number: 1,
            },
        )
        .await
    }
}

#[derive(Debug)]
pub struct ZzyProfile {
    pub entry_year: String,
    pub trans_dept: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ZzyRequest<'a> {
    #[serde(rename = "PCLBDM")]
    batch_code: &'static str, // 批次类别代码
    #[serde(rename = "XH")]
    student_id: &'a str, // 学号
    #[serde(rename = "*order")]
    tag: &'static str, // 标签
    #[serde(rename = "pageSize")]
    page_size: usize, // 每页大小
    #[serde(rename = "pageNumber")]
    page_number: usize, // 页码
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_struct() -> Result<()> {
        let resp_body = Zzy {
            code: "0".to_string(),
            datas: ZzyDatas {
                cxxszzybmsq: ZzyDataApi {
                    rows: vec![ZzyResponse {
                        xznj_display: "四年制本科2024级".to_string(),
                        sfzzlq: "否".to_string(),
                        sqyx_display: "待审核".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            },
        };

        let data_json = serde_json::to_string(&resp_body)?;
        println!("Mock Response JSON: \n{}", data_json);
        Ok(())
    }

    #[tokio::test]
    async fn test_zzy_api() -> Result<()> {
        let castgc = "TGT-2269268-7Ubxct1-dn1-jqc4eeGdzVGNqjTwNvAjVPHcQnFU5kfpxljmJXxq2mmLL4xNskdALeMnull_main";
        let data = ZzyRequest {
            batch_code: "01",
            student_id: "34520242201240",
            tag: "-CZSJ,+ZYXH",
            page_size: 10,
            page_number: 1,
        };
        let zzy_api = Zzy::call(castgc, &data).await?;
        println!("Zzy API Response: {:?}", zzy_api);
        let profile = zzy_api.get_profile()?;
        println!("Zzy Profile: {:?}", profile);
        Ok(())
    }
}
